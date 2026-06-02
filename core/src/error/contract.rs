use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractLoadError {
    #[error("failed to read contract file from disk at target path '{path}': {source}")]
    ReadFailed { path: String, #[source] source: std::io::Error },
    #[error("failed to parse contract JSON data structures: {source}")]
    ParseFailed { #[source] source: serde_json::Error },
}
