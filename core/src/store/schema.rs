use crate::error::StoreError;
use rusqlite::Connection;

pub(super) fn init_schema(conn: &Connection) -> Result<(), StoreError> {
    create_core_tables(conn)?;
    normalize_attempts_table(conn)?;
    migrate_gate_runs_table(conn)?;
    normalize_evidence_bundles_table(conn)?;
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
             attempt_number INTEGER NOT NULL,
             status TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id),
             UNIQUE(run_id, attempt_number)
         );

         CREATE TABLE IF NOT EXISTS final_decisions (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL UNIQUE,
             decision TEXT NOT NULL,
             reason TEXT,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id)
         );",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn normalize_attempts_table(conn: &Connection) -> Result<(), StoreError> {
    if table_has_column(conn, "attempts", "attempt_number")? {
        return Ok(());
    }
    rebuild_attempts_with_numbers(conn)
}

fn rebuild_attempts_with_numbers(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "ALTER TABLE attempts RENAME TO legacy_attempts;

         CREATE TABLE attempts (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL,
             attempt_number INTEGER NOT NULL,
             status TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id),
             UNIQUE(run_id, attempt_number)
         );

         INSERT INTO attempts (id, run_id, attempt_number, status, created_at)
         SELECT
             id,
             run_id,
             (
                 SELECT COUNT(*)
                 FROM legacy_attempts previous
                 WHERE previous.run_id = legacy_attempts.run_id
                   AND previous.id <= legacy_attempts.id
             ),
             status,
             created_at
         FROM legacy_attempts;

         DROP TABLE legacy_attempts;",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn migrate_gate_runs_table(conn: &Connection) -> Result<(), StoreError> {
    if !table_exists(conn, "gate_runs")? {
        return create_attempt_gate_runs_table(conn);
    }
    if table_has_column(conn, "gate_runs", "attempt_id")? {
        return Ok(());
    }
    migrate_legacy_gate_runs(conn)
}

fn migrate_legacy_gate_runs(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "ALTER TABLE gate_runs RENAME TO legacy_gate_runs;

         INSERT INTO attempts (run_id, attempt_number, status, created_at)
         SELECT id, 1, status, created_at
         FROM runs
         WHERE NOT EXISTS (
             SELECT 1
             FROM attempts
             WHERE attempts.run_id = runs.id
               AND attempts.attempt_number = 1
         );

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

         INSERT INTO gate_runs (
             attempt_id, gate_num, passed, message, exit_code, duration_ms,
             stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
         )
         SELECT
             attempts.id,
             legacy_gate_runs.gate_num,
             legacy_gate_runs.passed,
             legacy_gate_runs.message,
             legacy_gate_runs.exit_code,
             legacy_gate_runs.duration_ms,
             legacy_gate_runs.stdout,
             legacy_gate_runs.stderr,
             legacy_gate_runs.test_passed,
             legacy_gate_runs.test_failed,
             legacy_gate_runs.test_ignored,
             legacy_gate_runs.parse_errors
         FROM legacy_gate_runs
         JOIN attempts
           ON attempts.run_id = legacy_gate_runs.run_id
          AND attempts.attempt_number = 1;

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

fn normalize_evidence_bundles_table(conn: &Connection) -> Result<(), StoreError> {
    if !table_exists(conn, "evidence_bundles")? {
        return create_evidence_bundles_table(conn);
    }
    if table_has_column(conn, "evidence_bundles", "run_id")?
        && table_has_column(conn, "evidence_bundles", "gate_run_id")?
    {
        return Ok(());
    }
    rebuild_evidence_bundles(conn)
}

fn create_evidence_bundles_table(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE evidence_bundles (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL,
             attempt_id INTEGER,
             gate_run_id INTEGER,
             summary TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id),
             FOREIGN KEY(attempt_id) REFERENCES attempts(id),
             FOREIGN KEY(gate_run_id) REFERENCES gate_runs(id)
         );",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn rebuild_evidence_bundles(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "ALTER TABLE evidence_bundles RENAME TO legacy_evidence_bundles;

         CREATE TABLE evidence_bundles (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             run_id INTEGER NOT NULL,
             attempt_id INTEGER,
             gate_run_id INTEGER,
             summary TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             FOREIGN KEY(run_id) REFERENCES runs(id),
             FOREIGN KEY(attempt_id) REFERENCES attempts(id),
             FOREIGN KEY(gate_run_id) REFERENCES gate_runs(id)
         );

         INSERT INTO evidence_bundles (id, run_id, attempt_id, gate_run_id, summary, created_at)
         SELECT
             legacy_evidence_bundles.id,
             attempts.run_id,
             legacy_evidence_bundles.attempt_id,
             NULL,
             legacy_evidence_bundles.summary,
             legacy_evidence_bundles.created_at
         FROM legacy_evidence_bundles
         JOIN attempts ON attempts.id = legacy_evidence_bundles.attempt_id;

         DROP TABLE legacy_evidence_bundles;",
    )
    .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool, StoreError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            rusqlite::params![table_name],
            |row| row.get(0),
        )
        .map_err(|source| StoreError::QueryFailed { source })?;
    Ok(count > 0)
}

fn table_has_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, StoreError> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query([])
        .map_err(|source| StoreError::QueryFailed { source })?;
    while let Some(row) = rows
        .next()
        .map_err(|source| StoreError::QueryFailed { source })?
    {
        let current_column_name: String = row
            .get(1)
            .map_err(|source| StoreError::QueryFailed { source })?;
        if current_column_name == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}
