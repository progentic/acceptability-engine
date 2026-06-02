use super::{ValidationError, GateError, StoreError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("workspace materialization failed: {0}")]
    MaterializeFailed(String),
    #[error("gate execution tracking error: {0}")]
    GateExecution(#[from] GateError),
    #[error("storage sub-tier error: {0}")]
    Storage(#[from] StoreError),
}
