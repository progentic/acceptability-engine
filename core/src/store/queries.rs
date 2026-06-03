use super::mappers::{gate_summary_from_row, run_list_item_from_row, run_summary_from_row};
use super::types::{GateRunSummary, RunListItem, RunStatusSummary};
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};

const MAX_RUN_LIST_LIMIT: u32 = 100;

pub fn fetch_run_summary(
    conn: &Connection,
    run_id: i64,
) -> Result<Option<RunStatusSummary>, StoreError> {
    let Some(mut summary) = fetch_run_header(conn, run_id)? else {
        return Ok(None);
    };
    summary.gates = fetch_gate_summaries(conn, run_id)?;
    Ok(Some(summary))
}

pub fn list_runs(
    conn: &Connection,
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    validate_run_list_limit(limit)?;
    query_run_list(conn, status_filter, limit, offset)
}

fn fetch_run_header(
    conn: &Connection,
    run_id: i64,
) -> Result<Option<RunStatusSummary>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id, contract_id, status, created_at FROM runs WHERE id = ?1")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = next_row(&mut rows)? else {
        return Ok(None);
    };
    Ok(Some(run_summary_from_row(row)?))
}

fn fetch_gate_summaries(conn: &Connection, run_id: i64) -> Result<Vec<GateRunSummary>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT gate_num, passed, message, exit_code, duration_ms FROM gate_runs WHERE run_id = ?1 ORDER BY gate_num ASC")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id])
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
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    match status_filter {
        Some(status) => query_run_list_by_status(conn, status, limit, offset),
        None => query_run_list_without_status(conn, limit, offset),
    }
}

fn query_run_list_by_status(
    conn: &Connection,
    status: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id, contract_id, status, created_at FROM runs WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![status, limit, offset])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_run_list_items(rows)
}

fn query_run_list_without_status(
    conn: &Connection,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id, contract_id, status, created_at FROM runs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![limit, offset])
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
