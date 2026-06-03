use crate::contract::Contract;
use crate::error::StoreError;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_run(conn: &Connection, contract: &Contract) -> Result<i64, StoreError> {
    create_run_with_status(conn, contract, "RUNNING")
}

pub fn create_queued_run(conn: &Connection, contract: &Contract) -> Result<i64, StoreError> {
    create_run_with_status(conn, contract, "QUEUED")
}

pub fn update_run_status(conn: &Connection, run_id: i64, status: &str) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE runs SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, run_id],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

fn create_run_with_status(
    conn: &Connection,
    contract: &Contract,
    status: &str,
) -> Result<i64, StoreError> {
    insert_contract_if_missing(conn, contract)?;
    insert_run(conn, contract, status)?;
    Ok(conn.last_insert_rowid())
}

fn insert_contract_if_missing(conn: &Connection, contract: &Contract) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO contracts (id, repo_url, base_sha, requires_human_review) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![contract.id, contract.repo_url, contract.base_sha, contract.requires_human_review as i32],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

fn insert_run(conn: &Connection, contract: &Contract, status: &str) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO runs (contract_id, status, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![contract.id, status, current_unix_seconds()?],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

fn current_unix_seconds() -> Result<i64, StoreError> {
    let duration =
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StoreError::MigrationFailed {
                source: rusqlite::Error::ExecuteReturnedResults,
            })?;
    Ok(duration.as_secs() as i64)
}
