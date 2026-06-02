use serde::{Deserialize, Serialize};
use super::GateResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestMetrics {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub measured: u32,
    pub parse_errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub base: GateResult,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub test_metrics: Option<TestMetrics>,
}

impl ExecutionResult {
    pub fn pass(
        gate_num: u8,
        message: &str,
        exit_code: i32,
        duration_ms: u64,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    ) -> Self {
        Self {
            base: GateResult::pass(gate_num, message),
            exit_code,
            duration_ms,
            stdout,
            stderr,
            test_metrics: None,
        }
    }

    pub fn fail(
        gate_num: u8,
        message: &str,
        exit_code: i32,
        duration_ms: u64,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    ) -> Self {
        Self {
            base: GateResult::fail(gate_num, message.to_string()),
            exit_code,
            duration_ms,
            stdout,
            stderr,
            test_metrics: None,
        }
    }
}
