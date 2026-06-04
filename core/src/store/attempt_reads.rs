use super::mappers::{attempt_gate_detail_from_row, attempt_summary_from_row};
use super::types::{AttemptGateDetail, AttemptId, AttemptSummary, RunId};
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};

#[cfg(test)]
pub fn list_run_attempts(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<Vec<AttemptSummary>>, StoreError> {
    list_run_attempts_for_tenant(conn, run_id, "local")
}

pub fn list_run_attempts_for_tenant(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<Option<Vec<AttemptSummary>>, StoreError> {
    if !run_exists_for_tenant(conn, run_id, tenant_id)? {
        return Ok(None);
    }
    Ok(Some(query_run_attempts(conn, run_id)?))
}

#[cfg(test)]
pub fn list_attempt_gates(
    conn: &Connection,
    attempt_id: AttemptId,
) -> Result<Option<Vec<AttemptGateDetail>>, StoreError> {
    list_attempt_gates_for_tenant(conn, attempt_id, "local")
}

pub fn list_attempt_gates_for_tenant(
    conn: &Connection,
    attempt_id: AttemptId,
    tenant_id: &str,
) -> Result<Option<Vec<AttemptGateDetail>>, StoreError> {
    if !attempt_exists_for_tenant(conn, attempt_id, tenant_id)? {
        return Ok(None);
    }
    Ok(Some(query_attempt_gates(conn, attempt_id)?))
}

fn run_exists_for_tenant(
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

fn attempt_exists_for_tenant(
    conn: &Connection,
    attempt_id: AttemptId,
    tenant_id: &str,
) -> Result<bool, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT 1
             FROM attempts
             JOIN runs ON runs.id = attempts.run_id
             WHERE attempts.id = ?1 AND runs.tenant_id = ?2
             LIMIT 1",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![attempt_id.get(), tenant_id])
        .map_err(|source| StoreError::QueryFailed { source })?;
    next_row(&mut rows).map(|row| row.is_some())
}

fn query_run_attempts(conn: &Connection, run_id: RunId) -> Result<Vec<AttemptSummary>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, attempt_number, status, created_at
             FROM attempts
             WHERE run_id = ?1
             ORDER BY attempt_number ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_attempt_summaries(rows)
}

fn query_attempt_gates(
    conn: &Connection,
    attempt_id: AttemptId,
) -> Result<Vec<AttemptGateDetail>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, attempt_id, gate_num, passed, message, exit_code, duration_ms,
                    stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
             FROM gate_runs
             WHERE attempt_id = ?1
             ORDER BY gate_num ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![attempt_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_attempt_gate_details(rows)
}

fn collect_attempt_summaries(mut rows: Rows<'_>) -> Result<Vec<AttemptSummary>, StoreError> {
    let mut results = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        results.push(attempt_summary_from_row(row)?);
    }
    Ok(results)
}

fn collect_attempt_gate_details(mut rows: Rows<'_>) -> Result<Vec<AttemptGateDetail>, StoreError> {
    let mut results = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        results.push(attempt_gate_detail_from_row(row)?);
    }
    Ok(results)
}

fn next_row<'rows, 'stmt>(
    rows: &'rows mut Rows<'stmt>,
) -> Result<Option<&'rows Row<'stmt>>, StoreError> {
    rows.next()
        .map_err(|source| StoreError::QueryFailed { source })
}
