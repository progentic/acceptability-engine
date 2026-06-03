use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct RunStatusSummary {
    pub run_id: i64,
    pub contract_id: String,
    pub status: String,
    pub created_at: i64,
    pub gates: Vec<GateRunSummary>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GateRunSummary {
    pub gate_num: u8,
    pub passed: bool,
    pub message: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RunListItem {
    pub run_id: i64,
    pub contract_id: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct AttemptSummary {
    pub attempt_id: i64,
    pub run_id: i64,
    pub attempt_number: i64,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct AttemptGateDetail {
    pub gate_run_id: i64,
    pub attempt_id: i64,
    pub gate_num: u8,
    pub passed: bool,
    pub message: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub stdout: Option<String>,
    pub stdout_truncated: bool,
    pub stderr: Option<String>,
    pub stderr_truncated: bool,
    pub test_passed: Option<u32>,
    pub test_failed: Option<u32>,
    pub test_ignored: Option<u32>,
    pub parse_errors: Option<u32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct EvidenceBundleSummary {
    pub evidence_bundle_id: i64,
    pub run_id: i64,
    pub attempt_id: Option<i64>,
    pub gate_run_id: Option<i64>,
    pub summary: String,
    pub created_at: i64,
}
