use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("failed to spawn sub-process command: {source}")]
    SpawnFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("sub-process pipe was not available for {stream}")]
    MissingPipe { stream: &'static str },
    #[error("sub-process command timed out after execution threshold of {duration_ms}ms")]
    Timeout { duration_ms: u64 },
    #[error("sub-process {stream} exceeded output limit of {limit_bytes} bytes")]
    OutputLimitExceeded {
        stream: &'static str,
        limit_bytes: usize,
    },
    #[error("failed to launch sandbox runner: {source}")]
    RunnerLaunchFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("failed to wait on sub-process handle or collect concurrent I/O: {source}")]
    WaitFailed {
        #[source]
        source: std::io::Error,
    },
}
