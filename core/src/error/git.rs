use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("workspace directory not found at '{path}'")]
    WorkspaceNotFound { path: String },
    #[error("workspace path is not a directory: '{path}'")]
    WorkspaceNotDirectory { path: String },
    #[error("base_sha is empty")]
    EmptyBaseSha,
    #[error("git repository not found at '{path}'")]
    RepoNotFound { path: String },
    #[error("commit '{sha}' not found in repository at '{path}'")]
    CommitNotFound { path: String, sha: String },
    #[error("failed to execute git process: {source}")]
    ProcessExecutionFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("git command failed with exit code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },
}
