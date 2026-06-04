use super::clock::current_unix_seconds;
use super::types::{AttemptId, RunId};
use crate::error::StoreError;
use crate::policy::PolicyEvaluation;
use rusqlite::Connection;

pub fn record_policy_evaluation(
    conn: &Connection,
    run_id: RunId,
    attempt_id: AttemptId,
    evaluation: &PolicyEvaluation,
) -> Result<i64, StoreError> {
    conn.execute(
        "INSERT INTO policy_evaluations (
            run_id, attempt_id, policy_id, policy_version, passed, reason, trace_json, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            run_id.get(),
            attempt_id.get(),
            evaluation.policy_id,
            evaluation.policy_version,
            evaluation.passed as i32,
            evaluation.reason,
            evaluation.trace_json,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(conn.last_insert_rowid())
}
