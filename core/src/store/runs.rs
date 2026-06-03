use super::clock::current_unix_seconds;
use crate::contract::Contract;
use crate::error::StoreError;
use rusqlite::Connection;

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

pub fn create_attempt(conn: &Connection, run_id: i64) -> Result<i64, StoreError> {
    let attempt_number = next_attempt_number(conn, run_id)?;
    conn.execute(
        "INSERT INTO attempts (run_id, attempt_number, status, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![run_id, attempt_number, "RUNNING", current_unix_seconds()?],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(conn.last_insert_rowid())
}

pub fn update_attempt_status(
    conn: &Connection,
    attempt_id: i64,
    status: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE attempts SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, attempt_id],
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

fn next_attempt_number(conn: &Connection, run_id: i64) -> Result<i64, StoreError> {
    let latest: Option<i64> = conn
        .query_row(
            "SELECT MAX(attempt_number) FROM attempts WHERE run_id = ?1",
            rusqlite::params![run_id],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    Ok(latest.unwrap_or(0) + 1)
}
