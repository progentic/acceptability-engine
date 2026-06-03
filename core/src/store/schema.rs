use crate::error::StoreError;
use rusqlite::Connection;

pub(super) fn init_schema(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         
         CREATE TABLE IF NOT EXISTS contracts (
             id TEXT PRIMARY KEY,
             repo_url TEXT NOT NULL,
             base_sha TEXT NOT NULL,
             requires_human_review INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS runs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             contract_id TEXT NOT NULL,
             status TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(contract_id) REFERENCES contracts(id)
         );

         CREATE TABLE IF NOT EXISTS gate_runs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL,
             gate_num INTEGER NOT NULL,
             passed INTEGER NOT NULL,
             message TEXT NOT NULL,
             exit_code INTEGER,
             duration_ms INTEGER,
             stdout BLOB,
             stderr BLOB,
             test_passed INTEGER,
             test_failed INTEGER,
             test_ignored INTEGER,
             parse_errors INTEGER,
             FOREIGN KEY(run_id) REFERENCES runs(id)
         );",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}
