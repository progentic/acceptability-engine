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
