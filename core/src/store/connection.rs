use super::schema::init_schema;
use crate::error::StoreError;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Duration;
#[cfg(test)]
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

const DEFAULT_POOL_SIZE: usize = 8;
const BUSY_TIMEOUT_SECONDS: u64 = 30;

#[derive(Clone)]
pub struct SharedConnection {
    backend: Arc<ConnectionBackend>,
}

enum ConnectionBackend {
    #[cfg(test)]
    Single(Arc<Mutex<Connection>>),
    Pooled(PooledConnection),
}

struct PooledConnection {
    database_url: Arc<str>,
    permits: Arc<Semaphore>,
}

#[cfg(test)]
pub fn shared_connection(connection: Connection) -> SharedConnection {
    SharedConnection {
        backend: Arc::new(ConnectionBackend::Single(Arc::new(Mutex::new(connection)))),
    }
}

pub fn pooled_connection(database_url: &str) -> Result<SharedConnection, StoreError> {
    validate_pooled_database_url(database_url)?;
    initialize_database(database_url)?;
    Ok(SharedConnection {
        backend: Arc::new(ConnectionBackend::Pooled(PooledConnection {
            database_url: Arc::from(database_url),
            permits: Arc::new(Semaphore::new(DEFAULT_POOL_SIZE)),
        })),
    })
}

fn validate_pooled_database_url(database_url: &str) -> Result<(), StoreError> {
    if database_url == ":memory:" {
        return Err(StoreError::InvalidParameter(
            "pooled store requires a file-backed SQLite database".to_string(),
        ));
    }
    Ok(())
}

pub async fn with_connection<T, F>(db: SharedConnection, operation: F) -> Result<T, StoreError>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, StoreError> + Send + 'static,
{
    match db.backend.as_ref() {
        #[cfg(test)]
        ConnectionBackend::Single(connection) => {
            run_single_connection_operation(connection.clone(), operation).await
        }
        ConnectionBackend::Pooled(pool) => {
            run_pooled_connection_operation(
                pool.database_url.clone(),
                pool.permits.clone(),
                operation,
            )
            .await
        }
    }
}

pub fn open(database_url: &str) -> Result<Connection, StoreError> {
    let conn = open_connection(database_url)?;
    init_schema(&conn)?;
    Ok(conn)
}

#[cfg(test)]
async fn run_single_connection_operation<T, F>(
    connection: Arc<Mutex<Connection>>,
    operation: F,
) -> Result<T, StoreError>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, StoreError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || run_locked_operation(connection, operation))
        .await
        .map_err(|source| StoreError::BlockingTaskFailed { source })?
}

async fn run_pooled_connection_operation<T, F>(
    database_url: Arc<str>,
    permits: Arc<Semaphore>,
    operation: F,
) -> Result<T, StoreError>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, StoreError> + Send + 'static,
{
    let permit = permits
        .acquire_owned()
        .await
        .map_err(|_| StoreError::PoolClosed)?;
    tokio::task::spawn_blocking(move || run_pooled_operation(database_url, permit, operation))
        .await
        .map_err(|source| StoreError::BlockingTaskFailed { source })?
}

#[cfg(test)]
fn run_locked_operation<T, F>(
    connection: Arc<Mutex<Connection>>,
    operation: F,
) -> Result<T, StoreError>
where
    F: FnOnce(&Connection) -> Result<T, StoreError>,
{
    let conn = connection.blocking_lock();
    operation(&conn)
}

fn run_pooled_operation<T, F>(
    database_url: Arc<str>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    operation: F,
) -> Result<T, StoreError>
where
    F: FnOnce(&Connection) -> Result<T, StoreError>,
{
    let conn = open_connection(&database_url)?;
    operation(&conn)
}

fn initialize_database(database_url: &str) -> Result<(), StoreError> {
    let conn = open(database_url)?;
    drop(conn);
    Ok(())
}

fn open_connection(database_url: &str) -> Result<Connection, StoreError> {
    let connection =
        Connection::open(database_url).map_err(|source| StoreError::ConnectionFailed { source })?;
    configure_connection(&connection)?;
    Ok(connection)
}

fn configure_connection(connection: &Connection) -> Result<(), StoreError> {
    connection
        .busy_timeout(Duration::from_secs(BUSY_TIMEOUT_SECONDS))
        .map_err(|source| StoreError::ConnectionFailed { source })?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|source| StoreError::ConnectionFailed { source })?;
    Ok(())
}
