use crate::error::StoreError;
use rusqlite::Connection;

pub fn with_transaction<T, F>(conn: &Connection, operation: F) -> Result<T, StoreError>
where
    F: FnOnce(&Connection) -> Result<T, StoreError>,
{
    begin_transaction(conn)?;
    match operation(conn) {
        Ok(value) => commit_transaction(conn, value),
        Err(error) => rollback_transaction(conn, error),
    }
}

fn begin_transaction(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")
        .map_err(|source| StoreError::InsertFailed { source })
}

fn commit_transaction<T>(conn: &Connection, value: T) -> Result<T, StoreError> {
    conn.execute_batch("COMMIT")
        .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(value)
}

fn rollback_transaction<T>(conn: &Connection, error: StoreError) -> Result<T, StoreError> {
    let _ = conn.execute_batch("ROLLBACK");
    Err(error)
}
