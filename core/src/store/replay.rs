use super::artifacts::ArtifactStore;
use super::clock::current_unix_seconds;
use super::types::{AttemptId, GateRunId, ReviewDecisionId, RunId};
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};
use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayReport {
    pub replay: ReplayMetadata,
    pub contract: ReplayContract,
    pub run: ReplayRun,
    pub attempts: Vec<ReplayAttempt>,
    pub review_decision: Option<ReplayReviewDecision>,
    pub final_decision: Option<ReplayFinalDecision>,
    pub evidence: Vec<ReplayEvidence>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayMetadata {
    pub generated_at: i64,
    pub source_database_identity: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayContract {
    pub id: String,
    pub repo_url: String,
    pub base_sha: String,
    pub requires_human_review: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayRun {
    pub run_id: RunId,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayAttempt {
    pub attempt_id: AttemptId,
    pub attempt_number: i64,
    pub status: String,
    pub created_at: i64,
    pub gates: Vec<ReplayGate>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayGate {
    pub gate_run_id: GateRunId,
    pub gate_num: u8,
    pub passed: bool,
    pub message: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub test_passed: Option<u32>,
    pub test_failed: Option<u32>,
    pub test_ignored: Option<u32>,
    pub parse_errors: Option<u32>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayReviewDecision {
    pub review_decision_id: ReviewDecisionId,
    pub tenant_id: String,
    pub reviewer_actor: String,
    pub reviewer_role: String,
    pub decision: String,
    pub reason: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayFinalDecision {
    pub decision: String,
    pub reason: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplayEvidence {
    pub evidence_bundle_id: i64,
    pub run_id: RunId,
    pub attempt_id: Option<AttemptId>,
    pub gate_run_id: Option<GateRunId>,
    pub review_decision_id: Option<ReviewDecisionId>,
    pub kind: String,
    pub label: String,
    pub storage_uri: Option<String>,
    pub sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub content_type: Option<String>,
    pub summary: String,
    pub created_at: i64,
    pub artifact_present: Option<bool>,
}

struct ReplayHeader {
    contract: ReplayContract,
    run: ReplayRun,
}

pub fn replay_run(
    conn: &Connection,
    artifact_store: &ArtifactStore,
    run_id: RunId,
) -> Result<Option<ReplayReport>, StoreError> {
    let Some(header) = replay_header(conn, run_id)? else {
        return Ok(None);
    };
    let attempts = replay_attempts(conn, run_id)?;
    let review_decision = replay_review_decision(conn, run_id)?;
    let final_decision = replay_final_decision(conn, run_id)?;
    let evidence = replay_evidence(conn, artifact_store, run_id)?;
    Ok(Some(ReplayReport {
        replay: replay_metadata(conn)?,
        contract: header.contract,
        run: header.run,
        attempts,
        review_decision,
        final_decision,
        evidence,
    }))
}

fn replay_metadata(conn: &Connection) -> Result<ReplayMetadata, StoreError> {
    Ok(ReplayMetadata {
        generated_at: current_unix_seconds()?,
        source_database_identity: source_database_identity(conn)?,
    })
}

fn replay_header(conn: &Connection, run_id: RunId) -> Result<Option<ReplayHeader>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT contracts.id, contracts.repo_url, contracts.base_sha,
                    contracts.requires_human_review, runs.id, runs.status, runs.created_at
             FROM runs
             JOIN contracts ON contracts.id = runs.contract_id
             WHERE runs.id = ?1",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    Ok(Some(replay_header_from_row(row)?))
}

fn replay_attempts(conn: &Connection, run_id: RunId) -> Result<Vec<ReplayAttempt>, StoreError> {
    let mut attempts = query_attempts(conn, run_id)?;
    for attempt in &mut attempts {
        attempt.gates = replay_gates(conn, attempt.attempt_id)?;
    }
    Ok(attempts)
}

fn replay_gates(conn: &Connection, attempt_id: AttemptId) -> Result<Vec<ReplayGate>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, gate_num, passed, message, exit_code, duration_ms,
                    test_passed, test_failed, test_ignored, parse_errors
             FROM gate_runs
             WHERE attempt_id = ?1
             ORDER BY gate_num ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![attempt_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_gates(rows)
}

fn replay_review_decision(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<ReplayReviewDecision>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, tenant_id, reviewer_actor, reviewer_role, decision, reason, created_at
             FROM review_decisions
             WHERE run_id = ?1",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    Ok(Some(review_decision_from_row(row)?))
}

fn replay_final_decision(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<ReplayFinalDecision>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT decision, reason, created_at FROM final_decisions WHERE run_id = ?1")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    Ok(Some(final_decision_from_row(row)?))
}

fn replay_evidence(
    conn: &Connection,
    artifact_store: &ArtifactStore,
    run_id: RunId,
) -> Result<Vec<ReplayEvidence>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, attempt_id, gate_run_id, review_decision_id, kind,
                    label, storage_uri, sha256, byte_len, content_type, summary, created_at
             FROM evidence_bundles
             WHERE run_id = ?1
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_evidence(rows, artifact_store)
}

fn query_attempts(conn: &Connection, run_id: RunId) -> Result<Vec<ReplayAttempt>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, attempt_number, status, created_at
             FROM attempts
             WHERE run_id = ?1
             ORDER BY attempt_number ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_attempts(rows)
}

fn replay_header_from_row(row: &Row<'_>) -> Result<ReplayHeader, StoreError> {
    Ok(ReplayHeader {
        contract: ReplayContract {
            id: read(row, 0)?,
            repo_url: read(row, 1)?,
            base_sha: read(row, 2)?,
            requires_human_review: read_bool(row, 3)?,
        },
        run: ReplayRun {
            run_id: RunId::new(read(row, 4)?),
            status: read(row, 5)?,
            created_at: read(row, 6)?,
        },
    })
}

fn collect_attempts(mut rows: Rows<'_>) -> Result<Vec<ReplayAttempt>, StoreError> {
    let mut attempts = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        attempts.push(ReplayAttempt {
            attempt_id: AttemptId::new(read(row, 0)?),
            attempt_number: read(row, 1)?,
            status: read(row, 2)?,
            created_at: read(row, 3)?,
            gates: Vec::new(),
        });
    }
    Ok(attempts)
}

fn collect_gates(mut rows: Rows<'_>) -> Result<Vec<ReplayGate>, StoreError> {
    let mut gates = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        gates.push(ReplayGate {
            gate_run_id: GateRunId::new(read(row, 0)?),
            gate_num: read_i64_as_u8(row, 1)?,
            passed: read_bool(row, 2)?,
            message: read(row, 3)?,
            exit_code: read(row, 4)?,
            duration_ms: read_i64_as_u64(row, 5)?,
            test_passed: read_i64_as_u32(row, 6)?,
            test_failed: read_i64_as_u32(row, 7)?,
            test_ignored: read_i64_as_u32(row, 8)?,
            parse_errors: read_i64_as_u32(row, 9)?,
        });
    }
    Ok(gates)
}

fn collect_evidence(
    mut rows: Rows<'_>,
    artifact_store: &ArtifactStore,
) -> Result<Vec<ReplayEvidence>, StoreError> {
    let mut evidence = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        let mut item = evidence_from_row(row)?;
        item.artifact_present = artifact_presence(artifact_store, item.storage_uri.as_deref());
        evidence.push(item);
    }
    Ok(evidence)
}

fn evidence_from_row(row: &Row<'_>) -> Result<ReplayEvidence, StoreError> {
    Ok(ReplayEvidence {
        evidence_bundle_id: read(row, 0)?,
        run_id: RunId::new(read(row, 1)?),
        attempt_id: read_optional_id(row, 2)?.map(AttemptId::new),
        gate_run_id: read_optional_id(row, 3)?.map(GateRunId::new),
        review_decision_id: read_optional_id(row, 4)?.map(ReviewDecisionId::new),
        kind: read(row, 5)?,
        label: read(row, 6)?,
        storage_uri: read(row, 7)?,
        sha256: read(row, 8)?,
        byte_len: read(row, 9)?,
        content_type: read(row, 10)?,
        summary: read(row, 11)?,
        created_at: read(row, 12)?,
        artifact_present: None,
    })
}

fn review_decision_from_row(row: &Row<'_>) -> Result<ReplayReviewDecision, StoreError> {
    Ok(ReplayReviewDecision {
        review_decision_id: ReviewDecisionId::new(read(row, 0)?),
        tenant_id: read(row, 1)?,
        reviewer_actor: read(row, 2)?,
        reviewer_role: read(row, 3)?,
        decision: read(row, 4)?,
        reason: read(row, 5)?,
        created_at: read(row, 6)?,
    })
}

fn final_decision_from_row(row: &Row<'_>) -> Result<ReplayFinalDecision, StoreError> {
    Ok(ReplayFinalDecision {
        decision: read(row, 0)?,
        reason: read(row, 1)?,
        created_at: read(row, 2)?,
    })
}

fn artifact_presence(artifact_store: &ArtifactStore, storage_uri: Option<&str>) -> Option<bool> {
    storage_uri.map(|uri| artifact_store.artifact_exists(uri).unwrap_or(false))
}

fn source_database_identity(conn: &Connection) -> Result<Option<String>, StoreError> {
    let path: String = conn
        .query_row("PRAGMA database_list", [], |row| row.get(2))
        .map_err(|source| StoreError::QueryFailed { source })?;
    if path.is_empty() {
        return Ok(None);
    }
    Ok(Some(path))
}

fn next_row<'rows, 'stmt>(
    rows: &'rows mut Rows<'stmt>,
) -> Result<Option<&'rows Row<'stmt>>, StoreError> {
    rows.next()
        .map_err(|source| StoreError::QueryFailed { source })
}

fn read<T: rusqlite::types::FromSql>(row: &Row<'_>, index: usize) -> Result<T, StoreError> {
    row.get(index)
        .map_err(|source| StoreError::QueryFailed { source })
}

fn read_bool(row: &Row<'_>, index: usize) -> Result<bool, StoreError> {
    read::<i64>(row, index).map(|value| value != 0)
}

fn read_optional_id(row: &Row<'_>, index: usize) -> Result<Option<i64>, StoreError> {
    read(row, index)
}

fn read_i64_as_u8(row: &Row<'_>, index: usize) -> Result<u8, StoreError> {
    read::<i64>(row, index).map(|value| value as u8)
}

fn read_i64_as_u64(row: &Row<'_>, index: usize) -> Result<Option<u64>, StoreError> {
    read::<Option<i64>>(row, index).map(|value| value.map(|number| number as u64))
}

fn read_i64_as_u32(row: &Row<'_>, index: usize) -> Result<Option<u32>, StoreError> {
    read::<Option<i64>>(row, index).map(|value| value.map(|number| number as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::gates::result::{GateOutput, GateResult};
    use crate::store::{
        create_artifact_evidence_bundle, create_attempt, create_queued_run_for_tenant, open,
        record_final_decision, record_gate_run, ArtifactInput,
    };
    use std::path::PathBuf;

    #[test]
    fn replay_report_includes_run_history() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("history"));
        let run_id = seed_replay_run(&conn, &artifact_store, false);

        let report = replay_run(&conn, &artifact_store, run_id).unwrap().unwrap();

        assert_eq!(report.contract.id, "replay-run");
        assert_eq!(report.run.run_id, run_id);
        assert_eq!(report.attempts.len(), 1);
        assert_eq!(report.attempts[0].gates[0].gate_num, 1);
        assert_eq!(report.evidence.len(), 1);
        assert_eq!(report.evidence[0].artifact_present, Some(true));
        assert_eq!(
            report.review_decision.as_ref().unwrap().decision,
            "APPROVED"
        );
        assert_eq!(report.final_decision.as_ref().unwrap().decision, "APPROVED");
    }

    #[test]
    fn replay_report_marks_missing_artifacts() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("missing-artifact"));
        let run_id = seed_replay_run(&conn, &artifact_store, true);

        let report = replay_run(&conn, &artifact_store, run_id).unwrap().unwrap();

        assert_eq!(report.evidence[0].storage_uri.is_some(), true);
        assert_eq!(report.evidence[0].artifact_present, Some(false));
    }

    #[test]
    fn replay_is_deterministic_except_generated_at() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("deterministic"));
        let run_id = seed_replay_run(&conn, &artifact_store, false);

        let mut first = replay_run(&conn, &artifact_store, run_id).unwrap().unwrap();
        let mut second = replay_run(&conn, &artifact_store, run_id).unwrap().unwrap();
        first.replay.generated_at = 0;
        second.replay.generated_at = 0;

        assert_eq!(first, second);
    }

    #[test]
    fn replay_missing_run_returns_none() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("missing-run"));

        let report = replay_run(&conn, &artifact_store, RunId::new(999)).unwrap();

        assert!(report.is_none());
    }

    fn seed_replay_run(
        conn: &Connection,
        artifact_store: &ArtifactStore,
        delete_artifact: bool,
    ) -> RunId {
        let contract = test_contract();
        let run_id = create_queued_run_for_tenant(conn, &contract, "tenant-a").unwrap();
        let attempt_id = create_attempt(conn, run_id).unwrap();
        let gate_run_id = record_gate_run(
            conn,
            attempt_id,
            &GateOutput::Simple(GateResult::pass(1, "contract valid")),
        )
        .unwrap();
        let artifact = artifact_store
            .write_artifact(ArtifactInput {
                run_id,
                attempt_id: Some(attempt_id),
                gate_run_id: Some(gate_run_id),
                kind: "gate_telemetry",
                label: "Gate telemetry",
                content_type: "application/json",
                summary: "gate telemetry artifact captured",
                bytes: br#"{"passed":true}"#,
            })
            .unwrap();
        create_artifact_evidence_bundle(
            conn,
            run_id,
            Some(attempt_id),
            Some(gate_run_id),
            &artifact,
        )
        .unwrap();
        if delete_artifact {
            let _ = artifact_store
                .delete_artifact(&artifact.storage_uri)
                .unwrap();
        }
        seed_review_and_final_decision(conn, run_id);
        run_id
    }

    fn seed_review_and_final_decision(conn: &Connection, run_id: RunId) {
        conn.execute(
            "INSERT INTO review_decisions (
                run_id, tenant_id, reviewer_actor, reviewer_role, decision, reason, created_at
             ) VALUES (?1, 'tenant-a', 'reviewer-1', 'reviewer', 'APPROVED', 'approved', 10)",
            rusqlite::params![run_id.get()],
        )
        .unwrap();
        record_final_decision(conn, run_id, "APPROVED", Some("approved")).unwrap();
    }

    fn test_contract() -> Contract {
        Contract {
            id: "replay-run".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: true,
        }
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("acceptability-engine-replay-tests")
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
