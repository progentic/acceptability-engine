use crate::error::StoreError;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn current_unix_seconds() -> Result<i64, StoreError> {
    let duration =
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| StoreError::MigrationFailed {
                source: rusqlite::Error::ExecuteReturnedResults,
            })?;
    Ok(duration.as_secs() as i64)
}
