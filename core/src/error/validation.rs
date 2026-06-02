use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
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
    #[error("line_limit must be > 0, got {0}")]
    InvalidLineLimit(i32),
    #[error("timeout_ms must be > 0, got {0}")]
    InvalidTimeout(u64),
}
