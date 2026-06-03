use super::types::{
    AttemptGateDetail, AttemptSummary, EvidenceBundleSummary, GateRunSummary, RunListItem,
    RunStatusSummary,
};
use crate::error::StoreError;
use rusqlite::Row;

const GATE_OUTPUT_TEXT_LIMIT: usize = 8 * 1024;

pub(super) fn run_summary_from_row(row: &Row<'_>) -> Result<RunStatusSummary, StoreError> {
    Ok(RunStatusSummary {
        run_id: read_column(row, 0)?,
        contract_id: read_column(row, 1)?,
        status: read_column(row, 2)?,
        created_at: read_column(row, 3)?,
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
        run_id: read_column(row, 0)?,
        contract_id: read_column(row, 1)?,
        status: read_column(row, 2)?,
        created_at: read_column(row, 3)?,
    })
}

pub(super) fn attempt_summary_from_row(row: &Row<'_>) -> Result<AttemptSummary, StoreError> {
    Ok(AttemptSummary {
        attempt_id: read_column(row, 0)?,
        run_id: read_column(row, 1)?,
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
        gate_run_id: read_column(row, 0)?,
        attempt_id: read_column(row, 1)?,
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
        evidence_bundle_id: read_column(row, 0)?,
        run_id: read_column(row, 1)?,
        attempt_id: read_column(row, 2)?,
        gate_run_id: read_column(row, 3)?,
        summary: read_column(row, 4)?,
        created_at: read_column(row, 5)?,
    })
}

fn read_column<T: rusqlite::types::FromSql>(row: &Row<'_>, index: usize) -> Result<T, StoreError> {
    row.get(index)
        .map_err(|source| StoreError::QueryFailed { source })
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
