use super::types::{
    AttemptGateDetail, AttemptId, AttemptSummary, EvidenceBundleId, EvidenceBundleSummary,
    GateRunId, GateRunSummary, ReviewDecisionId, RunId, RunListItem, RunStatusSummary,
};
use crate::error::StoreError;
use rusqlite::Row;

const GATE_OUTPUT_TEXT_LIMIT: usize = 8 * 1024;

pub(super) fn run_summary_from_row(row: &Row<'_>) -> Result<RunStatusSummary, StoreError> {
    Ok(RunStatusSummary {
        run_id: read_run_id(row, 0)?,
        contract_id: read_column(row, 1)?,
        base_sha: read_column(row, 2)?,
        candidate_sha: read_column(row, 3)?,
        candidate_ref: read_column(row, 4)?,
        status: read_column(row, 5)?,
        created_at: read_column(row, 6)?,
        gates: Vec::new(),
    })
}

pub(super) fn gate_summary_from_row(row: &Row<'_>) -> Result<GateRunSummary, StoreError> {
    let passed_int: i32 = read_column(row, 1)?;
    Ok(GateRunSummary {
        gate_num: read_column(row, 0)?,
        passed: passed_int != 0,
        message: read_column(row, 2)?,
        exit_code: read_column(row, 3)?,
        duration_ms: read_column(row, 4)?,
    })
}

pub(super) fn run_list_item_from_row(row: &Row<'_>) -> Result<RunListItem, StoreError> {
    Ok(RunListItem {
        run_id: read_run_id(row, 0)?,
        contract_id: read_column(row, 1)?,
        base_sha: read_column(row, 2)?,
        candidate_sha: read_column(row, 3)?,
        candidate_ref: read_column(row, 4)?,
        status: read_column(row, 5)?,
        created_at: read_column(row, 6)?,
    })
}

pub(super) fn attempt_summary_from_row(row: &Row<'_>) -> Result<AttemptSummary, StoreError> {
    Ok(AttemptSummary {
        attempt_id: read_attempt_id(row, 0)?,
        run_id: read_run_id(row, 1)?,
        attempt_number: read_column(row, 2)?,
        status: read_column(row, 3)?,
        created_at: read_column(row, 4)?,
    })
}

pub(super) fn attempt_gate_detail_from_row(row: &Row<'_>) -> Result<AttemptGateDetail, StoreError> {
    let passed_int: i32 = read_column(row, 3)?;
    let stdout: Option<Vec<u8>> = read_column(row, 7)?;
    let stderr: Option<Vec<u8>> = read_column(row, 8)?;
    Ok(AttemptGateDetail {
        gate_run_id: read_gate_run_id(row, 0)?,
        attempt_id: read_attempt_id(row, 1)?,
        gate_num: read_column(row, 2)?,
        passed: passed_int != 0,
        message: read_column(row, 4)?,
        exit_code: read_column(row, 5)?,
        duration_ms: read_column(row, 6)?,
        stdout: output_text(stdout.as_deref()),
        stdout_truncated: output_truncated(stdout.as_deref()),
        stderr: output_text(stderr.as_deref()),
        stderr_truncated: output_truncated(stderr.as_deref()),
        test_passed: read_column(row, 9)?,
        test_failed: read_column(row, 10)?,
        test_ignored: read_column(row, 11)?,
        parse_errors: read_column(row, 12)?,
    })
}

pub(super) fn evidence_bundle_summary_from_row(
    row: &Row<'_>,
) -> Result<EvidenceBundleSummary, StoreError> {
    Ok(EvidenceBundleSummary {
        evidence_bundle_id: read_evidence_bundle_id(row, 0)?,
        run_id: read_run_id(row, 1)?,
        attempt_id: read_optional_attempt_id(row, 2)?,
        gate_run_id: read_optional_gate_run_id(row, 3)?,
        review_decision_id: read_optional_review_decision_id(row, 4)?,
        kind: read_column(row, 5)?,
        label: read_column(row, 6)?,
        storage_uri: read_column(row, 7)?,
        sha256: read_column(row, 8)?,
        byte_len: read_column(row, 9)?,
        content_type: read_column(row, 10)?,
        summary: read_column(row, 11)?,
        created_at: read_column(row, 12)?,
    })
}

fn read_column<T: rusqlite::types::FromSql>(row: &Row<'_>, index: usize) -> Result<T, StoreError> {
    row.get(index)
        .map_err(|source| StoreError::QueryFailed { source })
}

fn read_run_id(row: &Row<'_>, index: usize) -> Result<RunId, StoreError> {
    read_column(row, index).map(RunId::new)
}

fn read_attempt_id(row: &Row<'_>, index: usize) -> Result<AttemptId, StoreError> {
    read_column(row, index).map(AttemptId::new)
}

fn read_optional_attempt_id(row: &Row<'_>, index: usize) -> Result<Option<AttemptId>, StoreError> {
    read_column::<Option<i64>>(row, index).map(|value| value.map(AttemptId::new))
}

fn read_gate_run_id(row: &Row<'_>, index: usize) -> Result<GateRunId, StoreError> {
    read_column(row, index).map(GateRunId::new)
}

fn read_optional_gate_run_id(row: &Row<'_>, index: usize) -> Result<Option<GateRunId>, StoreError> {
    read_column::<Option<i64>>(row, index).map(|value| value.map(GateRunId::new))
}

fn read_optional_review_decision_id(
    row: &Row<'_>,
    index: usize,
) -> Result<Option<ReviewDecisionId>, StoreError> {
    read_column::<Option<i64>>(row, index).map(|value| value.map(ReviewDecisionId::new))
}

fn read_evidence_bundle_id(row: &Row<'_>, index: usize) -> Result<EvidenceBundleId, StoreError> {
    read_column(row, index).map(EvidenceBundleId::new)
}

fn output_text(bytes: Option<&[u8]>) -> Option<String> {
    bytes.map(capped_lossy_output)
}

fn capped_lossy_output(bytes: &[u8]) -> String {
    let preview = &bytes[..bytes.len().min(GATE_OUTPUT_TEXT_LIMIT)];
    String::from_utf8_lossy(preview).into_owned()
}

fn output_truncated(bytes: Option<&[u8]>) -> bool {
    bytes.is_some_and(|value| value.len() > GATE_OUTPUT_TEXT_LIMIT)
}
