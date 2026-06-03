use super::types::{GateRunSummary, RunListItem, RunStatusSummary};
use crate::error::StoreError;
use rusqlite::Row;

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

fn read_column<T: rusqlite::types::FromSql>(row: &Row<'_>, index: usize) -> Result<T, StoreError> {
    row.get(index)
        .map_err(|source| StoreError::QueryFailed { source })
}
