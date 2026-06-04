pub mod execution;

pub use execution::{ExecutionResult, TestMetrics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_num: u8,
    pub passed: bool,
    pub message: String,
}

impl GateResult {
    pub fn pass(gate_num: u8, message: &str) -> Self {
        Self {
            gate_num,
            passed: true,
            message: message.to_string(),
        }
    }

    pub fn fail(gate_num: u8, message: String) -> Self {
        Self {
            gate_num,
            passed: false,
            message,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum GateOutput {
    Simple(GateResult),
    Execution(ExecutionResult),
}

impl GateOutput {
    pub fn passed(&self) -> bool {
        match self {
            Self::Simple(result) => result.passed,
            Self::Execution(result) => result.base.passed,
        }
    }

    pub fn gate_num(&self) -> u8 {
        match self {
            Self::Simple(result) => result.gate_num,
            Self::Execution(result) => result.base.gate_num,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Simple(result) => &result.message,
            Self::Execution(result) => &result.base.message,
        }
    }
}
