use crate::error::StoreError;
use rusqlite::Connection;

const CORE_TABLES_SQL: &str = include_str!("../../migrations/0001_core_tables.sql");
const GATE_RUNS_SQL: &str = include_str!("../../migrations/0002_gate_runs.sql");
const EVIDENCE_BUNDLES_SQL: &str = include_str!("../../migrations/0003_evidence_bundles.sql");
const QUERY_INDEXES_SQL: &str = include_str!("../../migrations/0004_query_indexes.sql");
const REBUILD_ATTEMPTS_SQL: &str =
    include_str!("../../migrations/0005_rebuild_attempts_with_numbers.sql");
const MIGRATE_LEGACY_GATE_RUNS_SQL: &str =
    include_str!("../../migrations/0006_migrate_legacy_gate_runs.sql");
const REBUILD_EVIDENCE_BUNDLES_SQL: &str =
    include_str!("../../migrations/0007_rebuild_evidence_bundles.sql");

pub(super) fn init_schema(conn: &Connection) -> Result<(), StoreError> {
    create_core_tables(conn)?;
    normalize_attempts_table(conn)?;
    migrate_gate_runs_table(conn)?;
    normalize_evidence_bundles_table(conn)?;
    create_query_indexes(conn)?;
    Ok(())
}

fn create_core_tables(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, CORE_TABLES_SQL)
}

fn normalize_attempts_table(conn: &Connection) -> Result<(), StoreError> {
    if table_has_column(conn, "attempts", "attempt_number")? {
        return Ok(());
    }
    rebuild_attempts_with_numbers(conn)
}

fn rebuild_attempts_with_numbers(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, REBUILD_ATTEMPTS_SQL)
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
    execute_migration(conn, MIGRATE_LEGACY_GATE_RUNS_SQL)
}

fn create_attempt_gate_runs_table(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, GATE_RUNS_SQL)
}

fn normalize_evidence_bundles_table(conn: &Connection) -> Result<(), StoreError> {
    if !table_exists(conn, "evidence_bundles")? {
        return create_evidence_bundles_table(conn);
    }
    if !has_current_evidence_link_columns(conn)? {
        return rebuild_evidence_bundles(conn);
    }
    add_missing_evidence_descriptor_columns(conn)
}

fn has_current_evidence_link_columns(conn: &Connection) -> Result<bool, StoreError> {
    Ok(table_has_column(conn, "evidence_bundles", "run_id")?
        && table_has_column(conn, "evidence_bundles", "gate_run_id")?)
}

fn create_evidence_bundles_table(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, EVIDENCE_BUNDLES_SQL)
}

fn rebuild_evidence_bundles(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, REBUILD_EVIDENCE_BUNDLES_SQL)
}

fn add_missing_evidence_descriptor_columns(conn: &Connection) -> Result<(), StoreError> {
    add_text_column_if_missing(
        conn,
        "evidence_bundles",
        "kind",
        "TEXT NOT NULL DEFAULT 'summary'",
    )?;
    add_text_column_if_missing(
        conn,
        "evidence_bundles",
        "label",
        "TEXT NOT NULL DEFAULT 'Evidence summary'",
    )?;
    add_text_column_if_missing(conn, "evidence_bundles", "storage_uri", "TEXT")?;
    add_text_column_if_missing(conn, "evidence_bundles", "sha256", "TEXT")?;
    add_integer_column_if_missing(conn, "evidence_bundles", "byte_len")?;
    add_text_column_if_missing(conn, "evidence_bundles", "content_type", "TEXT")
}

fn create_query_indexes(conn: &Connection) -> Result<(), StoreError> {
    execute_migration(conn, QUERY_INDEXES_SQL)
}

fn execute_migration(conn: &Connection, sql: &str) -> Result<(), StoreError> {
    conn.execute_batch(sql)
        .map_err(|source| StoreError::MigrationFailed { source })?;
    Ok(())
}

fn add_text_column_if_missing(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    definition: &str,
) -> Result<(), StoreError> {
    add_column_if_missing(conn, table_name, column_name, definition)
}

fn add_integer_column_if_missing(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<(), StoreError> {
    add_column_if_missing(conn, table_name, column_name, "INTEGER")
}

fn add_column_if_missing(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    definition: &str,
) -> Result<(), StoreError> {
    if table_has_column(conn, table_name, column_name)? {
        return Ok(());
    }
    conn.execute_batch(&format!(
        "ALTER TABLE {table_name} ADD COLUMN {column_name} {definition};"
    ))
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
