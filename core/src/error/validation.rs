use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("contract id is required but was empty")]
    MissingContractId,
    #[error("contract id must be a safe single path segment: '{0}'")]
    InvalidContractId(String),
    #[error("repo_url is required but was empty")]
    MissingRepoUrl,
    #[error("repo_url must be an https or ssh git URL: '{0}'")]
    InvalidRepoUrl(String),
    #[error("base_sha is required but was empty")]
    MissingBaseSha,
    #[error("base_sha must be 40 hex chars, got {len} chars: '{value}'")]
    InvalidBaseShaLength { len: usize, value: String },
    #[error("base_sha contains non-hex characters: '{0}'")]
    InvalidBaseShaChars(String),
    #[error("scopes cannot be empty")]
    EmptyScopes,
    #[error("scope at index {index} is empty")]
    EmptyScope { index: usize },
    #[error("scope at index {index} must be a normalized relative path: '{value}'")]
    InvalidScopePath { index: usize, value: String },
    #[error("workspace path escaped configured workspace root")]
    WorkspaceEscapesRoot,
    #[error("AH_WORKSPACE_MODE has unsupported value: '{0}'")]
    InvalidWorkspaceMode(String),
    #[error("AH_WORKSPACE_MODE selected a mode that is not available: {0}")]
    UnsupportedWorkspaceMode(String),
}
