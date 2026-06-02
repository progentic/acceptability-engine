use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("failed to spawn sub-process command: {source}")]
    SpawnFailed { #[source] source: std::io::Error },
    #[error("sub-process command timed out after execution threshold of {duration_ms}ms")]
    Timeout { duration_ms: u64 },
    #[error("failed to wait on sub-process handle or collect concurrent I/O: {source}")]
    WaitFailed { #[source] source: std::io::Error },
}
