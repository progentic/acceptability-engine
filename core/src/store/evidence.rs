use super::artifacts::StoredArtifactDescriptor;
use super::clock::current_unix_seconds;
use super::types::{AttemptId, EvidenceBundleId, GateRunId, RunId};
use crate::error::StoreError;
use rusqlite::Connection;

pub struct EvidenceDescriptor<'a> {
    pub run_id: RunId,
    pub attempt_id: Option<AttemptId>,
    pub gate_run_id: Option<GateRunId>,
    pub kind: &'a str,
    pub label: &'a str,
    pub storage_uri: Option<&'a str>,
    pub sha256: Option<&'a str>,
    pub byte_len: Option<i64>,
    pub content_type: Option<&'a str>,
    pub summary: &'a str,
}

pub fn create_evidence_bundle(
    conn: &Connection,
    run_id: RunId,
    attempt_id: Option<AttemptId>,
    gate_run_id: Option<GateRunId>,
    summary: &str,
) -> Result<EvidenceBundleId, StoreError> {
    create_evidence_bundle_record(
        conn,
        EvidenceDescriptor {
            run_id,
            attempt_id,
            gate_run_id,
            kind: "summary",
            label: summary,
            storage_uri: None,
            sha256: None,
            byte_len: None,
            content_type: None,
            summary,
        },
    )
}

pub fn create_artifact_evidence_bundle(
    conn: &Connection,
    run_id: RunId,
    attempt_id: Option<AttemptId>,
    gate_run_id: Option<GateRunId>,
    artifact: &StoredArtifactDescriptor,
) -> Result<EvidenceBundleId, StoreError> {
    create_evidence_bundle_record(
        conn,
        EvidenceDescriptor {
            run_id,
            attempt_id,
            gate_run_id,
            kind: &artifact.kind,
            label: &artifact.label,
            storage_uri: Some(&artifact.storage_uri),
            sha256: Some(&artifact.sha256),
            byte_len: Some(artifact.byte_len),
            content_type: Some(&artifact.content_type),
            summary: &artifact.summary,
        },
    )
}

pub fn create_evidence_bundle_record(
    conn: &Connection,
    descriptor: EvidenceDescriptor<'_>,
) -> Result<EvidenceBundleId, StoreError> {
    conn.execute(
        "INSERT INTO evidence_bundles (
            run_id, attempt_id, gate_run_id, kind, label, storage_uri, sha256,
            byte_len, content_type, summary, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            descriptor.run_id.get(),
            descriptor.attempt_id.map(AttemptId::get),
            descriptor.gate_run_id.map(GateRunId::get),
            descriptor.kind,
            descriptor.label,
            descriptor.storage_uri,
            descriptor.sha256,
            descriptor.byte_len,
            descriptor.content_type,
            descriptor.summary,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(EvidenceBundleId::new(conn.last_insert_rowid()))
}
