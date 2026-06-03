use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("database connection failed: {source}")]
    ConnectionFailed {
        #[source]
        source: rusqlite::Error,
    },
    #[error("failed to execute database migrations: {source}")]
    MigrationFailed {
        #[source]
        source: rusqlite::Error,
    },
    #[error("failed to insert record into store: {source}")]
    InsertFailed {
        #[source]
        source: rusqlite::Error,
    },
    #[error("failed to query record from data layer: {source}")]
    QueryFailed {
        #[source]
        source: rusqlite::Error,
    },
    #[error("invalid pagination parameter: {0}")]
    InvalidParameter(String),
    #[error("database blocking task failed: {source}")]
    BlockingTaskFailed {
        #[source]
        source: tokio::task::JoinError,
    },
    #[error("database connection pool closed")]
    PoolClosed,
    #[error("failed to write evidence artifact: {source}")]
    ArtifactWriteFailed {
        #[source]
        source: std::io::Error,
    },
}
