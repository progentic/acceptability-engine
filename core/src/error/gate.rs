use super::{GitError, ProcessError, StoreError, ValidationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GateError {
    #[error("executor join failed: {source}")]
    ExecutorJoinFailed {
        #[from]
        source: tokio::task::JoinError,
    },
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("git error: {0}")]
    Git(#[from] GitError),
    #[error("process command error: {0}")]
    Process(#[from] ProcessError),
    #[error("contract validation failure: {0}")]
    Validation(#[from] ValidationError),
}
