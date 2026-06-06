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
    pub policy_evaluations: Vec<ReplayPolicyEvaluation>,
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
    pub candidate_sha: String,
    pub candidate_ref: Option<String>,
    pub scopes: Vec<String>,
    pub requires_human_review: bool,
    pub policy_json: String,
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
pub struct ReplayPolicyEvaluation {
    pub policy_evaluation_id: i64,
    pub run_id: RunId,
    pub attempt_id: AttemptId,
    pub policy_id: String,
    pub policy_version: u32,
    pub passed: bool,
    pub reason: String,
    pub trace_json: String,
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
    let policy_evaluations = replay_policy_evaluations(conn, run_id)?;
    let review_decision = replay_review_decision(conn, run_id)?;
    let final_decision = replay_final_decision(conn, run_id)?;
    let evidence = replay_evidence(conn, artifact_store, run_id)?;
    Ok(Some(ReplayReport {
        replay: replay_metadata(conn)?,
        contract: header.contract,
        run: header.run,
        attempts,
        policy_evaluations,
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
                    contracts.candidate_sha, contracts.candidate_ref, contracts.scopes_json,
                    contracts.requires_human_review, contracts.policy_json,
                    runs.id, runs.status, runs.created_at
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

fn replay_policy_evaluations(
    conn: &Connection,
    run_id: RunId,
) -> Result<Vec<ReplayPolicyEvaluation>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, attempt_id, policy_id, policy_version, passed,
                    reason, trace_json, created_at
             FROM policy_evaluations
             WHERE run_id = ?1
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_policy_evaluations(rows)
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
            candidate_sha: read(row, 3)?,
            candidate_ref: read(row, 4)?,
            scopes: read_json(row, 5)?,
            requires_human_review: read_bool(row, 6)?,
            policy_json: read(row, 7)?,
        },
        run: ReplayRun {
            run_id: RunId::new(read(row, 8)?),
            status: read(row, 9)?,
            created_at: read(row, 10)?,
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

fn collect_policy_evaluations(
    mut rows: Rows<'_>,
) -> Result<Vec<ReplayPolicyEvaluation>, StoreError> {
    let mut evaluations = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        evaluations.push(policy_evaluation_from_row(row)?);
    }
    Ok(evaluations)
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

fn policy_evaluation_from_row(row: &Row<'_>) -> Result<ReplayPolicyEvaluation, StoreError> {
    Ok(ReplayPolicyEvaluation {
        policy_evaluation_id: read(row, 0)?,
        run_id: RunId::new(read(row, 1)?),
        attempt_id: AttemptId::new(read(row, 2)?),
        policy_id: read(row, 3)?,
        policy_version: read(row, 4)?,
        passed: read_bool(row, 5)?,
        reason: read(row, 6)?,
        trace_json: read(row, 7)?,
        created_at: read(row, 8)?,
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

fn read_json<T: serde::de::DeserializeOwned>(row: &Row<'_>, index: usize) -> Result<T, StoreError> {
    let json: String = read(row, index)?;
    serde_json::from_str(&json).map_err(|source| StoreError::SerializationFailed { source })
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
    use crate::policy::PolicyEvaluation;
    use crate::store::{
        create_artifact_evidence_bundle, create_attempt, create_queued_run_for_tenant, open,
        record_final_decision, record_gate_run, record_policy_evaluation, ArtifactInput,
    };
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn replay_report_includes_run_history() {
        let conn = open(":memory:").unwrap();
        let artifact_store = ArtifactStore::new(test_root("history"));
        let run_id = seed_replay_run(&conn, &artifact_store, false);

        let report = replay_run(&conn, &artifact_store, run_id).unwrap().unwrap();

        assert_eq!(report.contract.id, "replay-run");
        assert_eq!(report.contract.scopes, vec!["core/src".to_string()]);
        assert_eq!(report.run.run_id, run_id);
        assert_eq!(report.attempts.len(), 1);
        assert_eq!(report.attempts[0].gates[0].gate_num, 1);
        assert_eq!(report.policy_evaluations.len(), 1);
        assert_eq!(report.policy_evaluations[0].policy_id, "strict-v1");
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

    #[test]
    fn backup_validation_creates_reusable_recovery_fixture() {
        let fixture = recovery_fixture();
        let backup = create_recovery_backup(&fixture);

        assert!(fixture.backup_database.exists());
        assert!(fixture.backup_artifacts.exists());
        assert!(backup_replay_path(&fixture, backup.run_id).exists());
        assert!(fixture.backup_inventory.exists());
        assert!(artifact_file_count(&fixture.backup_artifacts) > 0);
        validate_backup_inventory(&fixture);
        assert_eq!(backup.before.contract.id, "replay-run");
    }

    #[test]
    fn disaster_recovery_restore_consumes_recovery_fixture() {
        let fixture = recovery_fixture();
        let backup = create_recovery_backup(&fixture);
        destroy_live_evidence_store(&fixture);
        restore_live_evidence_store(&fixture);
        let restored_artifact_store = ArtifactStore::new(fixture.artifacts);
        let after = normalized_replay(&fixture.database, &restored_artifact_store, backup.run_id);

        assert_eq!(backup.before, after);
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
        seed_policy_evaluation(conn, run_id, attempt_id);
        if delete_artifact {
            let _ = artifact_store
                .delete_artifact(&artifact.storage_uri)
                .unwrap();
        }
        seed_review_and_final_decision(conn, run_id);
        run_id
    }

    fn seed_file_backed_replay_run(database: &Path, artifact_store: &ArtifactStore) -> RunId {
        create_parent_directory(database);
        let database_url = database.to_string_lossy();
        let conn = open(database_url.as_ref()).unwrap();
        seed_replay_run(&conn, artifact_store, false)
    }

    fn normalized_replay(
        database: &Path,
        artifact_store: &ArtifactStore,
        run_id: RunId,
    ) -> ReplayReport {
        let database_url = database.to_string_lossy();
        let conn = open(database_url.as_ref()).unwrap();
        let mut report = replay_run(&conn, artifact_store, run_id).unwrap().unwrap();
        report.replay.generated_at = 0;
        report
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

    fn seed_policy_evaluation(conn: &Connection, run_id: RunId, attempt_id: AttemptId) {
        record_policy_evaluation(
            conn,
            run_id,
            attempt_id,
            &PolicyEvaluation {
                policy_id: "strict-v1".to_string(),
                policy_version: 1,
                passed: true,
                reason: "admission policy passed".to_string(),
                trace_json: "{}".to_string(),
            },
        )
        .unwrap();
    }

    fn test_contract() -> Contract {
        Contract {
            id: "replay-run".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_sha: "b9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_ref: None,
            scopes: vec!["core/src".to_string()],
            requires_human_review: true,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join("acceptability-engine-replay-tests")
            .join(name)
            .join(unique_suffix())
    }

    fn recovery_fixture() -> RecoveryFixture {
        let root = test_root("disaster-recovery");
        create_directory(&root);
        RecoveryFixture {
            database: root.join("live").join("evidence.db"),
            artifacts: root.join("live").join("artifacts"),
            backup_database: root.join("backup").join("evidence.db"),
            backup_artifacts: root.join("backup").join("artifacts"),
            backup_inventory: root.join("backup").join("inventory.txt"),
        }
    }

    struct RecoveryFixture {
        database: PathBuf,
        artifacts: PathBuf,
        backup_database: PathBuf,
        backup_artifacts: PathBuf,
        backup_inventory: PathBuf,
    }

    struct RecoveryBackup {
        run_id: RunId,
        before: ReplayReport,
    }

    fn create_recovery_backup(fixture: &RecoveryFixture) -> RecoveryBackup {
        let artifact_store = ArtifactStore::new(fixture.artifacts.clone());
        let run_id = seed_file_backed_replay_run(&fixture.database, &artifact_store);
        let before = normalized_replay(&fixture.database, &artifact_store, run_id);
        write_replay_baseline(fixture, run_id, &before);
        copy_file(&fixture.database, &fixture.backup_database);
        copy_directory(&fixture.artifacts, &fixture.backup_artifacts);
        write_backup_inventory(fixture, run_id);
        RecoveryBackup { run_id, before }
    }

    fn write_replay_baseline(fixture: &RecoveryFixture, run_id: RunId, report: &ReplayReport) {
        let replay = serde_json::to_string_pretty(report).unwrap();
        write_file(&backup_replay_path(fixture, run_id), &replay);
    }

    fn backup_replay_path(fixture: &RecoveryFixture, run_id: RunId) -> PathBuf {
        fixture
            .backup_database
            .parent()
            .unwrap()
            .join("replay")
            .join(format!("run-{}-pre-backup.json", run_id.get()))
    }

    fn write_backup_inventory(fixture: &RecoveryFixture, run_id: RunId) {
        let entries = backup_artifact_entries(fixture);
        let mut lines = backup_inventory_header(fixture, run_id, entries.len());
        lines.extend(backup_inventory_artifact_lines(&entries));
        write_file(&fixture.backup_inventory, &lines.join("\n"));
    }

    fn backup_inventory_header(
        fixture: &RecoveryFixture,
        run_id: RunId,
        artifact_count: usize,
    ) -> Vec<String> {
        let replay_path = backup_replay_path(fixture, run_id);
        vec![
            format!("created_at={}", current_unix_seconds().unwrap()),
            format!("source_database={}", fixture.database.display()),
            format!("source_artifact_root={}", fixture.artifacts.display()),
            "database_file_name=evidence.db".to_string(),
            format!("database_sha256={}", sha256_file(&fixture.backup_database)),
            format!(
                "replay_file_name={}",
                relative_backup_path(fixture, &replay_path)
            ),
            format!("replay_sha256={}", sha256_file(&replay_path)),
            format!("artifact_count={artifact_count}"),
        ]
    }

    fn backup_inventory_artifact_lines(entries: &[(String, String)]) -> Vec<String> {
        entries
            .iter()
            .enumerate()
            .flat_map(|(index, (path, sha256))| {
                [
                    format!("artifact.{index}.path={path}"),
                    format!("artifact.{index}.sha256={sha256}"),
                ]
            })
            .collect()
    }

    fn backup_artifact_entries(fixture: &RecoveryFixture) -> Vec<(String, String)> {
        let mut paths = relative_artifact_paths(&fixture.backup_artifacts);
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let sha256 = sha256_file(&fixture.backup_artifacts.join(&path));
                (path, sha256)
            })
            .collect()
    }

    fn relative_artifact_paths(root: &Path) -> Vec<String> {
        artifact_paths(root)
            .into_iter()
            .map(|path| relative_path(root, &path))
            .collect()
    }

    fn artifact_paths(root: &Path) -> Vec<PathBuf> {
        fs::read_dir(root)
            .unwrap()
            .flat_map(|entry| artifact_entry_paths(entry.unwrap().path()))
            .collect()
    }

    fn artifact_entry_paths(path: PathBuf) -> Vec<PathBuf> {
        if path.is_dir() {
            artifact_paths(&path)
        } else {
            vec![path]
        }
    }

    fn relative_path(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn relative_backup_path(fixture: &RecoveryFixture, path: &Path) -> String {
        let root = fixture.backup_database.parent().unwrap();
        relative_path(root, path)
    }

    fn validate_backup_inventory(fixture: &RecoveryFixture) {
        let inventory = read_file(&fixture.backup_inventory);
        assert_eq!(
            inventory_value(&inventory, "database_file_name"),
            "evidence.db"
        );
        assert_eq!(
            inventory_value(&inventory, "database_sha256"),
            sha256_file(&fixture.backup_database)
        );
        validate_inventory_replay(fixture, &inventory);
        validate_inventory_artifacts(fixture, &inventory);
    }

    fn validate_inventory_replay(fixture: &RecoveryFixture, inventory: &str) {
        let replay_file_name = inventory_value(inventory, "replay_file_name");
        let replay_path = fixture
            .backup_database
            .parent()
            .unwrap()
            .join(replay_file_name);
        assert_eq!(
            inventory_value(inventory, "replay_sha256"),
            sha256_file(&replay_path)
        );
    }

    fn validate_inventory_artifacts(fixture: &RecoveryFixture, inventory: &str) {
        let entries = backup_artifact_entries(fixture);
        assert_eq!(
            inventory_value(inventory, "artifact_count"),
            entries.len().to_string()
        );
        for (index, (path, sha256)) in entries.iter().enumerate() {
            assert_eq!(
                inventory_value(inventory, &format!("artifact.{index}.path")),
                *path
            );
            assert_eq!(
                inventory_value(inventory, &format!("artifact.{index}.sha256")),
                *sha256
            );
        }
    }

    fn inventory_value(inventory: &str, key: &str) -> String {
        let prefix = format!("{key}=");
        inventory
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap()
            .to_string()
    }

    fn destroy_live_evidence_store(fixture: &RecoveryFixture) {
        remove_file(&fixture.database);
        remove_directory(&fixture.artifacts);
    }

    fn restore_live_evidence_store(fixture: &RecoveryFixture) {
        copy_file(&fixture.backup_database, &fixture.database);
        copy_directory(&fixture.backup_artifacts, &fixture.artifacts);
    }

    fn copy_directory(source: &Path, destination: &Path) {
        create_directory(destination);
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            if source_path.is_dir() {
                copy_directory(&source_path, &destination_path);
            } else {
                copy_file(&source_path, &destination_path);
            }
        }
    }

    fn artifact_file_count(path: &Path) -> usize {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .map(|path| {
                if path.is_dir() {
                    artifact_file_count(&path)
                } else {
                    1
                }
            })
            .sum()
    }

    fn copy_file(source: &Path, destination: &Path) {
        create_parent_directory(destination);
        fs::copy(source, destination).unwrap();
    }

    fn write_file(path: &Path, contents: &str) {
        create_parent_directory(path);
        fs::write(path, contents).unwrap();
    }

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn sha256_file(path: &Path) -> String {
        hex_digest(fs::read(path).unwrap())
    }

    fn hex_digest(bytes: Vec<u8>) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn create_directory(path: &Path) {
        fs::create_dir_all(path).unwrap();
    }

    fn create_parent_directory(path: &Path) {
        create_directory(path.parent().unwrap());
    }

    fn remove_directory(path: &Path) {
        fs::remove_dir_all(path).unwrap();
    }

    fn remove_file(path: &Path) {
        fs::remove_file(path).unwrap();
    }

    fn unique_suffix() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    }
}
