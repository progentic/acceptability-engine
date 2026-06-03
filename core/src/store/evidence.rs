use super::clock::current_unix_seconds;
use crate::error::StoreError;
use rusqlite::Connection;

pub fn create_evidence_bundle(
    conn: &Connection,
    run_id: i64,
    attempt_id: Option<i64>,
    gate_run_id: Option<i64>,
    summary: &str,
) -> Result<i64, StoreError> {
    conn.execute(
        "INSERT INTO evidence_bundles (run_id, attempt_id, gate_run_id, summary, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            run_id,
            attempt_id,
            gate_run_id,
            summary,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(conn.last_insert_rowid())
}
