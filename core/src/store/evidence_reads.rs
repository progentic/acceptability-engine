use super::mappers::evidence_bundle_summary_from_row;
use super::types::{EvidenceBundleSummary, RunId};
use crate::error::StoreError;
use rusqlite::{Connection, Row, Rows};

#[cfg(test)]
pub fn list_run_evidence(
    conn: &Connection,
    run_id: RunId,
) -> Result<Option<Vec<EvidenceBundleSummary>>, StoreError> {
    list_run_evidence_for_tenant(conn, run_id, "local")
}

pub fn list_run_evidence_for_tenant(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<Option<Vec<EvidenceBundleSummary>>, StoreError> {
    if !run_exists_for_tenant(conn, run_id, tenant_id)? {
        return Ok(None);
    }
    Ok(Some(query_run_evidence(conn, run_id)?))
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

fn query_run_evidence(
    conn: &Connection,
    run_id: RunId,
) -> Result<Vec<EvidenceBundleSummary>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, run_id, attempt_id, gate_run_id, review_decision_id, kind,
                    label, storage_uri, sha256, byte_len, content_type, summary, created_at
             FROM evidence_bundles
             WHERE run_id = ?1
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    let rows = stmt
        .query(rusqlite::params![run_id.get()])
        .map_err(|source| StoreError::QueryFailed { source })?;
    collect_evidence_bundles(rows)
}

fn collect_evidence_bundles(mut rows: Rows<'_>) -> Result<Vec<EvidenceBundleSummary>, StoreError> {
    let mut results = Vec::new();
    while let Some(row) = next_row(&mut rows)? {
        results.push(evidence_bundle_summary_from_row(row)?);
    }
    Ok(results)
}

fn next_row<'rows, 'stmt>(
    rows: &'rows mut Rows<'stmt>,
) -> Result<Option<&'rows Row<'stmt>>, StoreError> {
    rows.next()
        .map_err(|source| StoreError::QueryFailed { source })
}
