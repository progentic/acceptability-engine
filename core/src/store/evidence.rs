use super::clock::current_unix_seconds;
use super::types::{AttemptId, EvidenceBundleId, GateRunId, RunId};
use crate::error::StoreError;
use rusqlite::Connection;

pub fn create_evidence_bundle(
    conn: &Connection,
    run_id: RunId,
    attempt_id: Option<AttemptId>,
    gate_run_id: Option<GateRunId>,
    summary: &str,
) -> Result<EvidenceBundleId, StoreError> {
    conn.execute(
        "INSERT INTO evidence_bundles (run_id, attempt_id, gate_run_id, summary, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            run_id.get(),
            attempt_id.map(AttemptId::get),
            gate_run_id.map(GateRunId::get),
            summary,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(EvidenceBundleId::new(conn.last_insert_rowid()))
}
