use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("failed to create workspace directory at '{path}': {source}")]
    WorkspaceCreationFailed {
        path: String,
        source: std::io::Error,
    },
    #[error("base_sha is empty")]
    EmptyBaseSha,
    #[error("failed to execute git process: {source}")]
    ProcessExecutionFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("git command failed with exit code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },
}
