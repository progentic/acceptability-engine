use super::clock::current_unix_seconds;
use super::types::{AttemptId, RunId};
use crate::contract::Contract;
use crate::error::StoreError;
use rusqlite::Connection;

struct StoredContract {
    repo_url: String,
    base_sha: String,
    candidate_sha: String,
    candidate_ref: Option<String>,
    scopes_json: String,
    requires_human_review: bool,
    policy_json: String,
}

pub fn create_run(conn: &Connection, contract: &Contract) -> Result<RunId, StoreError> {
    create_run_with_status(conn, contract, "RUNNING")
}

#[cfg(test)]
pub fn create_queued_run(conn: &Connection, contract: &Contract) -> Result<RunId, StoreError> {
    create_run_for_tenant_with_status(conn, contract, "local", "QUEUED")
}

pub fn create_queued_run_for_tenant(
    conn: &Connection,
    contract: &Contract,
    tenant_id: &str,
) -> Result<RunId, StoreError> {
    create_run_for_tenant_with_status(conn, contract, tenant_id, "QUEUED")
}

pub fn update_run_status(conn: &Connection, run_id: RunId, status: &str) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE runs SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, run_id.get()],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

pub fn create_attempt(conn: &Connection, run_id: RunId) -> Result<AttemptId, StoreError> {
    let attempt_number = next_attempt_number(conn, run_id)?;
    conn.execute(
        "INSERT INTO attempts (run_id, attempt_number, status, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            run_id.get(),
            attempt_number,
            "RUNNING",
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(AttemptId::new(conn.last_insert_rowid()))
}

pub fn update_attempt_status(
    conn: &Connection,
    attempt_id: AttemptId,
    status: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE attempts SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, attempt_id.get()],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

fn create_run_with_status(
    conn: &Connection,
    contract: &Contract,
    status: &str,
) -> Result<RunId, StoreError> {
    create_run_for_tenant_with_status(conn, contract, "local", status)
}

fn create_run_for_tenant_with_status(
    conn: &Connection,
    contract: &Contract,
    tenant_id: &str,
    status: &str,
) -> Result<RunId, StoreError> {
    insert_contract_if_missing(conn, contract)?;
    insert_run(conn, contract, tenant_id, status)?;
    Ok(RunId::new(conn.last_insert_rowid()))
}

fn insert_contract_if_missing(conn: &Connection, contract: &Contract) -> Result<(), StoreError> {
    let policy_json = serde_json::to_string(&contract.admission_policy)
        .map_err(|source| StoreError::SerializationFailed { source })?;
    let scopes_json = serde_json::to_string(&contract.scopes)
        .map_err(|source| StoreError::SerializationFailed { source })?;
    conn.execute(
        "INSERT OR IGNORE INTO contracts (
            id, repo_url, base_sha, candidate_sha, candidate_ref, scopes_json,
            requires_human_review, policy_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            contract.id,
            contract.repo_url,
            contract.base_sha,
            contract.candidate_sha,
            contract.candidate_ref,
            scopes_json,
            contract.requires_human_review as i32,
            policy_json
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    ensure_stored_contract_matches(conn, contract, &policy_json, &scopes_json)?;
    Ok(())
}

fn ensure_stored_contract_matches(
    conn: &Connection,
    contract: &Contract,
    policy_json: &str,
    scopes_json: &str,
) -> Result<(), StoreError> {
    let stored = fetch_stored_contract(conn, &contract.id)?;
    if stored_contract_matches(&stored, contract, policy_json, scopes_json) {
        return Ok(());
    }
    Err(StoreError::InvalidParameter(format!(
        "contract '{}' already exists with different authority data",
        contract.id
    )))
}

fn fetch_stored_contract(
    conn: &Connection,
    contract_id: &str,
) -> Result<StoredContract, StoreError> {
    conn.query_row(
        "SELECT repo_url, base_sha, candidate_sha, candidate_ref, scopes_json,
                requires_human_review, policy_json
         FROM contracts
         WHERE id = ?1",
        rusqlite::params![contract_id],
        |row| {
            Ok(StoredContract {
                repo_url: row.get(0)?,
                base_sha: row.get(1)?,
                candidate_sha: row.get(2)?,
                candidate_ref: row.get(3)?,
                scopes_json: row.get(4)?,
                requires_human_review: row.get::<_, i64>(5)? != 0,
                policy_json: row.get(6)?,
            })
        },
    )
    .map_err(|source| StoreError::QueryFailed { source })
}

fn stored_contract_matches(
    stored: &StoredContract,
    contract: &Contract,
    policy_json: &str,
    scopes_json: &str,
) -> bool {
    stored.repo_url == contract.repo_url
        && stored.base_sha == contract.base_sha
        && stored.candidate_sha == contract.candidate_sha
        && stored.candidate_ref == contract.candidate_ref
        && stored.scopes_json == scopes_json
        && stored.requires_human_review == contract.requires_human_review
        && stored.policy_json == policy_json
}

fn insert_run(
    conn: &Connection,
    contract: &Contract,
    tenant_id: &str,
    status: &str,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO runs (contract_id, tenant_id, status, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![contract.id, tenant_id, status, current_unix_seconds()?],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

fn next_attempt_number(conn: &Connection, run_id: RunId) -> Result<i64, StoreError> {
    let latest: Option<i64> = conn
        .query_row(
            "SELECT MAX(attempt_number) FROM attempts WHERE run_id = ?1",
            rusqlite::params![run_id.get()],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    Ok(latest.unwrap_or(0) + 1)
}
