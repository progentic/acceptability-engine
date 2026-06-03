use super::clock::current_unix_seconds;
use crate::error::StoreError;
use rusqlite::Connection;

pub fn create_evidence_bundle(conn: &Connection, attempt_id: i64) -> Result<i64, StoreError> {
    conn.execute(
        "INSERT INTO evidence_bundles (attempt_id, summary, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            attempt_id,
            "gate telemetry captured",
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(conn.last_insert_rowid())
}
