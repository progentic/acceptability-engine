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
    #[error("workspace HEAD '{head}' does not match candidate_sha '{candidate_sha}'")]
    HeadMismatch { head: String, candidate_sha: String },
    #[error("base_sha '{base_sha}' is not an ancestor of candidate_sha '{candidate_sha}'")]
    BaseNotAncestor {
        base_sha: String,
        candidate_sha: String,
    },
    #[error("failed to execute git process: {source}")]
    ProcessExecutionFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("git command failed with exit code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },
}
