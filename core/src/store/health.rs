use crate::error::StoreError;
use rusqlite::Connection;

pub fn check_store_ready(conn: &Connection) -> Result<(), StoreError> {
    conn.query_row("SELECT 1", [], |_| Ok(()))
        .map_err(|source| StoreError::QueryFailed { source })
}
