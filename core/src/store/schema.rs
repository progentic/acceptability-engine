use crate::error::StoreError;
use rusqlite::Connection;

pub(super) fn init_schema(conn: &Connection) -> Result<(), StoreError> {
    create_core_tables(conn)?;
    migrate_gate_runs_table(conn)?;
    Ok(())
}

fn create_core_tables(conn: &Connection) -> Result<(), StoreError> {
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

         CREATE TABLE IF NOT EXISTS attempts (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL,
             status TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id)
         );

         CREATE TABLE IF NOT EXISTS final_decisions (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL UNIQUE,
             decision TEXT NOT NULL,
             reason TEXT,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id)
         );

         CREATE TABLE IF NOT EXISTS evidence_bundles (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             attempt_id INTEGER NOT NULL UNIQUE,
             summary TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(attempt_id) REFERENCES attempts(id)
         );",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn migrate_gate_runs_table(conn: &Connection) -> Result<(), StoreError> {
    if !gate_runs_table_exists(conn)? {
        return create_attempt_gate_runs_table(conn);
    }
    if gate_runs_has_attempt_id(conn)? {
        return Ok(());
    }
    migrate_legacy_gate_runs(conn)
}

fn gate_runs_table_exists(conn: &Connection) -> Result<bool, StoreError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'gate_runs'",
            [],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    Ok(count > 0)
}

fn gate_runs_has_attempt_id(conn: &Connection) -> Result<bool, StoreError> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(gate_runs)")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query([])
        .map_err(|source| StoreError::QueryFailed { source })?;
    while let Some(row) = rows
        .next()
        .map_err(|source| StoreError::QueryFailed { source })?
    {
        let column_name: String = row
            .get(1)
            .map_err(|source| StoreError::QueryFailed { source })?;
        if column_name == "attempt_id" {
            return Ok(true);
        }
    }
    Ok(false)
}

fn migrate_legacy_gate_runs(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "ALTER TABLE gate_runs RENAME TO legacy_gate_runs;

         CREATE TABLE gate_runs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             attempt_id INTEGER NOT NULL,
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
             FOREIGN KEY(attempt_id) REFERENCES attempts(id)
         );

         INSERT INTO attempts (run_id, status, created_at)
         SELECT id, status, created_at
         FROM runs
         WHERE id IN (SELECT DISTINCT run_id FROM legacy_gate_runs)
           AND NOT EXISTS (
               SELECT 1 FROM attempts WHERE attempts.run_id = runs.id
           );

         INSERT INTO gate_runs (
             attempt_id, gate_num, passed, message, exit_code, duration_ms,
             stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
         )
         SELECT
             (
                 SELECT id
                 FROM attempts
                 WHERE attempts.run_id = legacy_gate_runs.run_id
                 ORDER BY id DESC
                 LIMIT 1
             ),
             gate_num, passed, message, exit_code, duration_ms,
             stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
         FROM legacy_gate_runs;

         DROP TABLE legacy_gate_runs;",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn create_attempt_gate_runs_table(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE gate_runs (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             attempt_id INTEGER NOT NULL,
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
             FOREIGN KEY(attempt_id) REFERENCES attempts(id)
         );",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}
