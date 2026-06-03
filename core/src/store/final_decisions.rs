use super::clock::current_unix_seconds;
use super::types::{FinalDecisionId, RunId};
use crate::error::StoreError;
use rusqlite::Connection;

pub fn record_final_decision(
    conn: &Connection,
    run_id: RunId,
    decision: &str,
    reason: Option<&str>,
) -> Result<FinalDecisionId, StoreError> {
    conn.execute(
        "INSERT INTO final_decisions (run_id, decision, reason, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![run_id.get(), decision, reason, current_unix_seconds()?],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(FinalDecisionId::new(conn.last_insert_rowid()))
}
