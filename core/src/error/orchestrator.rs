use super::{GateError, StoreError, ValidationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("gate execution tracking error: {0}")]
    GateExecution(#[from] GateError),
    #[error("storage sub-tier error: {0}")]
    Storage(#[from] StoreError),
}
