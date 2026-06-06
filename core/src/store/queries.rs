use super::mappers::{gate_summary_from_row, run_list_item_from_row, run_summary_from_row};
use super::types::{AttemptId, GateRunSummary, RunId, RunListItem, RunStatusSummary};
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};

const MAX_RUN_LIST_LIMIT: u32 = 100;

#[cfg(test)]
pub fn fetch_run_summary(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<RunStatusSummary>, StoreError> {
    fetch_run_summary_for_tenant(conn, run_id, "local")
}

pub fn fetch_run_summary_for_tenant(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<Option<RunStatusSummary>, StoreError> {
    if !run_belongs_to_tenant(conn, run_id, tenant_id)? {
        return Ok(None);
    }
    let Some(mut summary) = fetch_run_header(conn, run_id)? else {
        return Ok(None);
    };
    summary.gates = fetch_latest_attempt_gates(conn, run_id)?;
    Ok(Some(summary))
}

#[cfg(test)]
pub fn list_runs(
    conn: &Connection,
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    list_runs_for_tenant(conn, "local", status_filter, limit, offset)
}

pub fn list_runs_for_tenant(
    conn: &Connection,
    tenant_id: &str,
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    validate_run_list_limit(limit)?;
    query_run_list(conn, tenant_id, status_filter, limit, offset)
}

fn fetch_run_header(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<RunStatusSummary>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT runs.id, runs.contract_id, contracts.base_sha, contracts.candidate_sha,
                    contracts.candidate_ref, runs.status, runs.created_at
             FROM runs
             JOIN contracts ON contracts.id = runs.contract_id
             WHERE runs.id = ?1",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    Ok(Some(run_summary_from_row(row)?))
}

fn run_belongs_to_tenant(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<bool, StoreError> {
    let mut stmt = conn
        .prepare("SELECT 1 FROM runs WHERE id = ?1 AND tenant_id = ?2 LIMIT 1")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get(), tenant_id])
        .map_err(|source| StoreError::QueryFailed { source })?;
    next_row(&mut rows).map(|row| row.is_some())
}

fn fetch_latest_attempt_gates(
    conn: &Connection,
    run_id: RunId,
) -> Result<Vec<GateRunSummary>, StoreError> {
    let Some(attempt_id) = fetch_latest_attempt_id(conn, run_id)? else {
        return Ok(Vec::new());
    };
    fetch_gate_summaries(conn, attempt_id)
}

fn fetch_latest_attempt_id(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<AttemptId>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id FROM attempts WHERE run_id = ?1 ORDER BY attempt_number DESC, id DESC LIMIT 1",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    row.get(0)
        .map(|value| Some(AttemptId::new(value)))
        .map_err(|source| StoreError::QueryFailed { source })
}

fn fetch_gate_summaries(
    conn: &Connection,
    attempt_id: AttemptId,
) -> Result<Vec<GateRunSummary>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT gate_num, passed, message, exit_code, duration_ms FROM gate_runs WHERE attempt_id = ?1 ORDER BY gate_num ASC")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![attempt_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_gate_summaries(rows)
}

fn validate_run_list_limit(limit: u32) -> Result<(), StoreError> {
    if limit == 0 || limit > MAX_RUN_LIST_LIMIT {
        return Err(StoreError::InvalidParameter(
            "limit must be between 1 and 100".to_string(),
        ));
    }
    Ok(())
}

fn query_run_list(
    conn: &Connection,
    tenant_id: &str,
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    match status_filter {
        Some(status) => query_run_list_by_status(conn, tenant_id, status, limit, offset),
        None => query_run_list_without_status(conn, tenant_id, limit, offset),
    }
}

fn query_run_list_by_status(
    conn: &Connection,
    tenant_id: &str,
    status: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT runs.id, runs.contract_id, contracts.base_sha, contracts.candidate_sha,
                    contracts.candidate_ref, runs.status, runs.created_at
             FROM runs
             JOIN contracts ON contracts.id = runs.contract_id
             WHERE runs.tenant_id = ?1 AND runs.status = ?2
             ORDER BY runs.created_at DESC LIMIT ?3 OFFSET ?4",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![tenant_id, status, limit, offset])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_run_list_items(rows)
}

fn query_run_list_without_status(
    conn: &Connection,
    tenant_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT runs.id, runs.contract_id, contracts.base_sha, contracts.candidate_sha,
                    contracts.candidate_ref, runs.status, runs.created_at
             FROM runs
             JOIN contracts ON contracts.id = runs.contract_id
             WHERE runs.tenant_id = ?1
             ORDER BY runs.created_at DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![tenant_id, limit, offset])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_run_list_items(rows)
}

fn collect_gate_summaries(mut rows: Rows<'_>) -> Result<Vec<GateRunSummary>, StoreError> {
    let mut results = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        results.push(gate_summary_from_row(row)?);
    }
    Ok(results)
}

fn collect_run_list_items(mut rows: Rows<'_>) -> Result<Vec<RunListItem>, StoreError> {
    let mut results = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        results.push(run_list_item_from_row(row)?);
    }
    Ok(results)
}

fn next_row<'rows, 'stmt>(
    rows: &'rows mut Rows<'stmt>,
) -> Result<Option<&'rows Row<'stmt>>, StoreError> {
    rows.next()
        .map_err(|source| StoreError::QueryFailed { source })
}
