use serde::Serialize;

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct RunId(i64);

impl RunId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<RunId> for i64 {
    fn from(value: RunId) -> Self {
        value.get()
    }
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct AttemptId(i64);

impl AttemptId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<AttemptId> for i64 {
    fn from(value: AttemptId) -> Self {
        value.get()
    }
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct GateRunId(i64);

impl GateRunId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<GateRunId> for i64 {
    fn from(value: GateRunId) -> Self {
        value.get()
    }
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct EvidenceBundleId(i64);

impl EvidenceBundleId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct FinalDecisionId(i64);

impl FinalDecisionId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct ReviewDecisionId(i64);

impl ReviewDecisionId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<ReviewDecisionId> for i64 {
    fn from(value: ReviewDecisionId) -> Self {
        value.get()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct RunStatusSummary {
    pub run_id: RunId,
    pub contract_id: String,
    pub base_sha: String,
    pub candidate_sha: String,
    pub candidate_ref: Option<String>,
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
    pub run_id: RunId,
    pub contract_id: String,
    pub base_sha: String,
    pub candidate_sha: String,
    pub candidate_ref: Option<String>,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct AttemptSummary {
    pub attempt_id: AttemptId,
    pub run_id: RunId,
    pub attempt_number: i64,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct AttemptGateDetail {
    pub gate_run_id: GateRunId,
    pub attempt_id: AttemptId,
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
    pub evidence_bundle_id: EvidenceBundleId,
    pub run_id: RunId,
    pub attempt_id: Option<AttemptId>,
    pub gate_run_id: Option<GateRunId>,
    pub review_decision_id: Option<ReviewDecisionId>,
    pub kind: String,
    pub label: String,
    pub storage_uri: Option<String>,
    pub sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub content_type: Option<String>,
    pub summary: String,
    pub created_at: i64,
}

#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub tenant_id: String,
    pub actor: String,
    pub role: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
}
