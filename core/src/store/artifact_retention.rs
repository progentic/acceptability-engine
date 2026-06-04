use super::artifacts::{ArtifactDeleteOutcome, ArtifactStore};
use super::audit::record_audit_event;
use super::types::AuditEvent;
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};

const RETENTION_ACTION: &str = "artifacts.retention";
const RETENTION_ACTOR: &str = "artifact-retention";
const RETENTION_ROLE: &str = "system";
const RETENTION_TENANT: &str = "system";

pub struct RetentionPolicy {
    pub cutoff_unix_seconds: i64,
    pub dry_run: bool,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct RetentionSummary {
    pub scanned: usize,
    pub eligible: usize,
    pub planned: usize,
    pub deleted: usize,
    pub missing: usize,
}

struct RetentionCandidate {
    evidence_bundle_id: i64,
    storage_uri: String,
}

pub fn apply_artifact_retention(
    conn: &Connection,
    artifact_store: &ArtifactStore,
    policy: RetentionPolicy,
) -> Result<RetentionSummary, StoreError> {
    let candidates = retention_candidates(conn, policy.cutoff_unix_seconds)?;
    process_retention_candidates(conn, artifact_store, candidates, policy.dry_run)
}

fn retention_candidates(
    conn: &Connection,
    older_than_seconds: i64,
) -> Result<Vec<RetentionCandidate>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, storage_uri
             FROM evidence_bundles
             WHERE storage_uri IS NOT NULL
               AND created_at < ?1
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![older_than_seconds])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_candidates(rows)
}

fn process_retention_candidates(
    conn: &Connection,
    artifact_store: &ArtifactStore,
    candidates: Vec<RetentionCandidate>,
    dry_run: bool,
) -> Result<RetentionSummary, StoreError> {
    let mut summary = RetentionSummary {
        scanned: candidates.len(),
        eligible: candidates.len(),
        ..RetentionSummary::default()
    };
    for candidate in candidates {
        apply_retention_candidate(conn, artifact_store, &candidate, dry_run, &mut summary)?;
    }
    Ok(summary)
}

fn apply_retention_candidate(
    conn: &Connection,
    artifact_store: &ArtifactStore,
    candidate: &RetentionCandidate,
    dry_run: bool,
    summary: &mut RetentionSummary,
) -> Result<(), StoreError> {
    artifact_store.validate_artifact_uri(&candidate.storage_uri)?;
    record_planned_retention(conn, candidate, dry_run)?;
    if dry_run {
        summary.planned += 1;
        return Ok(());
    }
    match artifact_store.delete_artifact(&candidate.storage_uri)? {
        ArtifactDeleteOutcome::Deleted => {
            summary.deleted += 1;
            record_retention_outcome(conn, candidate, "DELETED")
        }
        ArtifactDeleteOutcome::Missing => {
            summary.missing += 1;
            record_retention_outcome(conn, candidate, "MISSING")
        }
    }
}

fn record_planned_retention(
    conn: &Connection,
    candidate: &RetentionCandidate,
    dry_run: bool,
) -> Result<(), StoreError> {
    let outcome = if dry_run { "DRY_RUN" } else { "PLANNED" };
    record_retention_outcome(conn, candidate, outcome)
}

fn record_retention_outcome(
    conn: &Connection,
    candidate: &RetentionCandidate,
    outcome: &str,
) -> Result<(), StoreError> {
    record_audit_event(
        conn,
        &AuditEvent {
            tenant_id: RETENTION_TENANT.to_string(),
            actor: RETENTION_ACTOR.to_string(),
            role: RETENTION_ROLE.to_string(),
            action: RETENTION_ACTION.to_string(),
            resource_type: "evidence_bundle".to_string(),
            resource_id: Some(candidate.evidence_bundle_id.to_string()),
            outcome: outcome.to_string(),
            reason: Some(candidate.storage_uri.clone()),
        },
    )
}

fn collect_candidates(mut rows: Rows<'_>) -> Result<Vec<RetentionCandidate>, StoreError> {
    let mut candidates = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|source| StoreError::QueryFailed { source })?
    {
        candidates.push(candidate_from_row(row)?);
    }
    Ok(candidates)
}

fn candidate_from_row(row: &Row<'_>) -> Result<RetentionCandidate, StoreError> {
    Ok(RetentionCandidate {
        evidence_bundle_id: row
            .get(0)
            .map_err(|source| StoreError::QueryFailed { source })?,
        storage_uri: row
            .get(1)
            .map_err(|source| StoreError::QueryFailed { source })?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{
        create_artifact_evidence_bundle, create_queued_run_for_tenant, open, ArtifactInput,
    };
    use crate::{contract::Contract, store::AttemptId};
    use std::path::PathBuf;

    #[test]
    fn dry_run_records_plans_without_deleting_artifacts() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("dry-run"));
        let storage_uri = seed_artifact_evidence(&conn, &artifact_store, 10);

        let summary = apply_artifact_retention(
            &conn,
            &artifact_store,
            RetentionPolicy {
                cutoff_unix_seconds: 20,
                dry_run: true,
            },
        )
        .unwrap();

        assert_eq!(summary.planned, 1);
        assert_eq!(summary.deleted, 0);
        assert!(artifact_exists(&artifact_store, &storage_uri));
        assert_eq!(audit_count(&conn, "DRY_RUN"), 1);
    }

    #[test]
    fn cleanup_deletes_artifact_and_keeps_evidence_descriptor() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("delete"));
        let storage_uri = seed_artifact_evidence(&conn, &artifact_store, 10);
        let descriptor_before = evidence_descriptor(&conn);

        let summary = apply_artifact_retention(
            &conn,
            &artifact_store,
            RetentionPolicy {
                cutoff_unix_seconds: 20,
                dry_run: false,
            },
        )
        .unwrap();

        assert_eq!(summary.deleted, 1);
        assert!(!artifact_exists(&artifact_store, &storage_uri));
        assert_eq!(evidence_count(&conn), 1);
        assert_eq!(evidence_descriptor(&conn), descriptor_before);
        assert_eq!(audit_count(&conn, "PLANNED"), 1);
        assert_eq!(audit_count(&conn, "DELETED"), 1);
    }

    #[test]
    fn cleanup_records_missing_artifacts() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("missing"));
        let storage_uri = seed_artifact_evidence(&conn, &artifact_store, 10);
        let _ = artifact_store.delete_artifact(&storage_uri).unwrap();

        let summary = apply_artifact_retention(
            &conn,
            &artifact_store,
            RetentionPolicy {
                cutoff_unix_seconds: 20,
                dry_run: false,
            },
        )
        .unwrap();

        assert_eq!(summary.missing, 1);
        assert_eq!(audit_count(&conn, "MISSING"), 1);
    }

    #[test]
    fn retention_ignores_newer_artifacts() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("newer"));
        let storage_uri = seed_artifact_evidence(&conn, &artifact_store, 30);

        let summary = apply_artifact_retention(
            &conn,
            &artifact_store,
            RetentionPolicy {
                cutoff_unix_seconds: 20,
                dry_run: false,
            },
        )
        .unwrap();

        assert_eq!(summary.scanned, 0);
        assert!(artifact_exists(&artifact_store, &storage_uri));
    }

    #[cfg(unix)]
    #[test]
    fn dry_run_rejects_symlink_parent_before_audit_planning() {
        let conn = open(":memory:").unwrap();
        let root = test_root("dry-run-symlink-root");
        let target = test_root("dry-run-symlink-target");
        let artifact_store = ArtifactStore::new(root.clone());
        let run_id = create_queued_run_for_tenant(&conn, &test_contract(), "local").unwrap();
        let descriptor = crate::store::StoredArtifactDescriptor {
            kind: "retention".to_string(),
            label: "Retention test artifact".to_string(),
            storage_uri: "artifact://runs/7/artifact.bin".to_string(),
            sha256: "0".repeat(64),
            byte_len: 1,
            content_type: "text/plain".to_string(),
            summary: "retention test".to_string(),
        };
        std::fs::create_dir_all(root.join("runs")).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, root.join("runs").join("7")).unwrap();
        create_artifact_evidence_bundle(&conn, run_id, None, None, &descriptor).unwrap();
        set_evidence_created_at(&conn, 10);

        let result = apply_artifact_retention(
            &conn,
            &artifact_store,
            RetentionPolicy {
                cutoff_unix_seconds: 20,
                dry_run: true,
            },
        );

        assert!(matches!(result, Err(StoreError::InvalidArtifactUri(_))));
        assert_eq!(audit_count(&conn, "DRY_RUN"), 0);
        assert_eq!(audit_count(&conn, "PLANNED"), 0);
    }

    fn seed_artifact_evidence(
        conn: &rusqlite::Connection,
        artifact_store: &ArtifactStore,
        created_at: i64,
    ) -> String {
        let run_id = create_queued_run_for_tenant(conn, &test_contract(), "local").unwrap();
        let descriptor = artifact_store
            .write_artifact(ArtifactInput {
                run_id,
                attempt_id: Some(AttemptId::new(1)),
                gate_run_id: None,
                kind: "retention",
                label: "Retention test artifact",
                content_type: "text/plain",
                summary: "retention test",
                bytes: b"retained artifact",
            })
            .unwrap();
        let storage_uri = descriptor.storage_uri.clone();
        create_artifact_evidence_bundle(conn, run_id, None, None, &descriptor).unwrap();
        set_evidence_created_at(conn, created_at);
        storage_uri
    }

    fn set_evidence_created_at(conn: &rusqlite::Connection, created_at: i64) {
        conn.execute(
            "UPDATE evidence_bundles SET created_at = ?1",
            rusqlite::params![created_at],
        )
        .unwrap();
    }

    fn audit_count(conn: &rusqlite::Connection, outcome: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE action = ?1 AND outcome = ?2",
            rusqlite::params![RETENTION_ACTION, outcome],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn evidence_count(conn: &rusqlite::Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM evidence_bundles", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    fn evidence_descriptor(conn: &rusqlite::Connection) -> (String, String, i64) {
        conn.query_row(
            "SELECT storage_uri, sha256, byte_len FROM evidence_bundles LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
    }

    fn artifact_exists(artifact_store: &ArtifactStore, storage_uri: &str) -> bool {
        artifact_store
            .root_for_tests()
            .join(storage_uri.trim_start_matches("artifact://"))
            .exists()
    }

    fn test_contract() -> Contract {
        Contract {
            id: "retention-run".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("acceptability-engine-retention-tests")
            .join(name)
            .join(unique_suffix())
    }

    fn unique_suffix() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    }
}
