use crate::error::StoreError;
use crate::gates::result::{ExecutionResult, GateOutput, GateResult, TestMetrics};
use rusqlite::Connection;

struct GateRecord<'a> {
    gate_num: u8,
    passed: bool,
    message: &'a str,
    exit_code: Option<i32>,
    duration_ms: Option<u64>,
    stdout: Option<&'a [u8]>,
    stderr: Option<&'a [u8]>,
    test_passed: Option<u32>,
    test_failed: Option<u32>,
    test_ignored: Option<u32>,
    parse_errors: Option<u32>,
}

pub fn record_gate_run(
    conn: &Connection,
    run_id: i64,
    output: &GateOutput,
) -> Result<(), StoreError> {
    insert_gate_record(conn, run_id, &gate_record_from_output(output))
}

fn gate_record_from_output(output: &GateOutput) -> GateRecord<'_> {
    match output {
        GateOutput::Simple(result) => gate_record_from_simple(result),
        GateOutput::Execution(result) => gate_record_from_execution(result),
    }
}

fn gate_record_from_simple(result: &GateResult) -> GateRecord<'_> {
    GateRecord {
        gate_num: result.gate_num,
        passed: result.passed,
        message: &result.message,
        exit_code: None,
        duration_ms: None,
        stdout: None,
        stderr: None,
        test_passed: None,
        test_failed: None,
        test_ignored: None,
        parse_errors: None,
    }
}

fn gate_record_from_execution(result: &ExecutionResult) -> GateRecord<'_> {
    let metrics = result.test_metrics.as_ref();
    GateRecord {
        gate_num: result.base.gate_num,
        passed: result.base.passed,
        message: &result.base.message,
        exit_code: Some(result.exit_code),
        duration_ms: Some(result.duration_ms),
        stdout: Some(&result.stdout),
        stderr: Some(&result.stderr),
        test_passed: test_passed(metrics),
        test_failed: test_failed(metrics),
        test_ignored: test_ignored(metrics),
        parse_errors: parse_errors(metrics),
    }
}

fn test_passed(metrics: Option<&TestMetrics>) -> Option<u32> {
    metrics.map(|value| value.passed)
}

fn test_failed(metrics: Option<&TestMetrics>) -> Option<u32> {
    metrics.map(|value| value.failed)
}

fn test_ignored(metrics: Option<&TestMetrics>) -> Option<u32> {
    metrics.map(|value| value.ignored)
}

fn parse_errors(metrics: Option<&TestMetrics>) -> Option<u32> {
    metrics.map(|value| value.parse_errors)
}

fn insert_gate_record(
    conn: &Connection,
    run_id: i64,
    record: &GateRecord<'_>,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO gate_runs (
            run_id, gate_num, passed, message, exit_code, duration_ms, 
            stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            run_id,
            record.gate_num,
            record.passed as i32,
            record.message,
            record.exit_code,
            record.duration_ms,
            record.stdout,
            record.stderr,
            record.test_passed,
            record.test_failed,
            record.test_ignored,
            record.parse_errors
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}
