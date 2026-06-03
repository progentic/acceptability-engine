use super::clock::current_unix_seconds;
use crate::error::StoreError;
use rusqlite::Connection;

pub fn record_final_decision(
    conn: &Connection,
    run_id: i64,
    decision: &str,
    reason: Option<&str>,
) -> Result<i64, StoreError> {
    conn.execute(
        "INSERT INTO final_decisions (run_id, decision, reason, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![run_id, decision, reason, current_unix_seconds()?],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(conn.last_insert_rowid())
}
