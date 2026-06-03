use super::schema::init_schema;
use crate::error::StoreError;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedConnection = Arc<Mutex<Connection>>;

pub fn shared_connection(connection: Connection) -> SharedConnection {
    Arc::new(Mutex::new(connection))
}

pub async fn with_connection<T, F>(db: SharedConnection, operation: F) -> Result<T, StoreError>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, StoreError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || run_blocking_operation(db, operation))
        .await
        .map_err(|source| StoreError::BlockingTaskFailed { source })?
}

pub fn open(database_url: &str) -> Result<Connection, StoreError> {
    let conn = open_connection(database_url)?;
    init_schema(&conn)?;
    Ok(conn)
}

fn run_blocking_operation<T, F>(db: SharedConnection, operation: F) -> Result<T, StoreError>
where
    F: FnOnce(&Connection) -> Result<T, StoreError>,
{
    let conn = db.blocking_lock();
    operation(&conn)
}

fn open_connection(database_url: &str) -> Result<Connection, StoreError> {
    Connection::open(database_url).map_err(|source| StoreError::ConnectionFailed { source })
}
