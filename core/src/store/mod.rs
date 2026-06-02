pub use rusqlite::Connection;
use crate::error::StoreError;
use crate::gates::result::GateOutput;
use crate::contract::Contract;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct RunStatusSummary {
    pub run_id: i64,
    pub contract_id: String,
    pub status: String,
    pub created_at: i64,
    pub gates: Vec<GateRunSummary>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GateRunSummary {
    pub gate_num: u8,
    pub passed: bool,
    pub message: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RunListItem {
    pub run_id: i64,
    pub contract_id: String,
    pub status: String,
    pub created_at: i64,
}

// CONCURRENCY: Connection wrapped in Mutex. Long-running contracts hold lock
// for duration. Layer 8 will migrate to sqlx::SqlitePool for concurrent reads.
pub fn open(database_url: &str) -> Result<Connection, StoreError> {
    let conn = Connection::open(database_url)
        .map_err(|source| StoreError::ConnectionFailed { source })?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn create_run(conn: &Connection, contract: &Contract) -> Result<i64, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StoreError::MigrationFailed {
            source: rusqlite::Error::ExecuteReturnedResults,
        })?
        .as_secs() as i64;

    conn.execute(
        "INSERT OR IGNORE INTO contracts (id, repo_url, base_sha, requires_human_review) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![contract.id, contract.repo_url, contract.base_sha, contract.requires_human_review as i32],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;

    conn.execute(
        "INSERT INTO runs (contract_id, status, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![contract.id, "RUNNING", now],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;

    Ok(conn.last_insert_rowid())
}

pub fn update_run_status(conn: &Connection, run_id: i64, status: &str) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE runs SET status = ?1 WHERE id = ?2",
        rusqlite::params![status, run_id],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}

pub fn fetch_run_summary(conn: &Connection, run_id: i64) -> Result<Option<RunStatusSummary>, StoreError> {
    let mut run_statement = conn
        .prepare("SELECT id, contract_id, status, created_at FROM runs WHERE id = ?1")
        .map_err(|source| StoreError::QueryFailed { source })?;

    let mut run_rows = run_statement
        .query(rusqlite::params![run_id])
        .map_err(|source| StoreError::QueryFailed { source })?;

    let Some(row) = run_rows.next().map_err(|source| StoreError::QueryFailed { source })? else {
        return Ok(None);
    };

    let mut summary = RunStatusSummary {
        run_id: row.get(0).map_err(|source| StoreError::QueryFailed { source })?,
        contract_id: row.get(1).map_err(|source| StoreError::QueryFailed { source })?,
        status: row.get(2).map_err(|source| StoreError::QueryFailed { source })?,
        created_at: row.get(3).map_err(|source| StoreError::QueryFailed { source })?,
        gates: Vec::new(),
    };

    let mut gate_statement = conn
        .prepare("SELECT gate_num, passed, message, exit_code, duration_ms FROM gate_runs WHERE run_id = ?1 ORDER BY gate_num ASC")
        .map_err(|source| StoreError::QueryFailed { source })?;

    let mut gate_rows = gate_statement
        .query(rusqlite::params![run_id])
        .map_err(|source| StoreError::QueryFailed { source })?;

    while let Some(gate_row) = gate_rows.next().map_err(|source| StoreError::QueryFailed { source })? {
        let passed_int: i32 = gate_row.get(1).map_err(|source| StoreError::QueryFailed { source })?;
        summary.gates.push(GateRunSummary {
            gate_num: gate_row.get(0).map_err(|source| StoreError::QueryFailed { source })?,
            passed: passed_int != 0,
            message: gate_row.get(2).map_err(|source| StoreError::QueryFailed { source })?,
            exit_code: gate_row.get(3).map_err(|source| StoreError::QueryFailed { source })?,
            duration_ms: gate_row.get(4).map_err(|source| StoreError::QueryFailed { source })?,
        });
    }

    Ok(Some(summary))
}

pub fn list_runs(
    conn: &Connection,
    status_filter: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<RunListItem>, StoreError> {
    if limit == 0 || limit > 100 {
        return Err(StoreError::InvalidParameter(
            "limit must be between 1 and 100".to_string(),
        ));
    }

    let mut results = Vec::new();

    if let Some(status) = status_filter {
        let mut stmt = conn
            .prepare("SELECT id, contract_id, status, created_at FROM runs WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3")
            .map_err(|source| StoreError::QueryFailed { source })?;
        let mut rows = stmt
            .query(rusqlite::params![status, limit, offset])
            .map_err(|source| StoreError::QueryFailed { source })?;
        while let Some(row) = rows.next().map_err(|source| StoreError::QueryFailed { source })? {
            results.push(RunListItem {
                run_id: row.get(0).map_err(|source| StoreError::QueryFailed { source })?,
                contract_id: row.get(1).map_err(|source| StoreError::QueryFailed { source })?,
                status: row.get(2).map_err(|source| StoreError::QueryFailed { source })?,
                created_at: row.get(3).map_err(|source| StoreError::QueryFailed { source })?,
            });
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT id, contract_id, status, created_at FROM runs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
            .map_err(|source| StoreError::QueryFailed { source })?;
        let mut rows = stmt
            .query(rusqlite::params![limit, offset])
            .map_err(|source| StoreError::QueryFailed { source })?;
        while let Some(row) = rows.next().map_err(|source| StoreError::QueryFailed { source })? {
            results.push(RunListItem {
                run_id: row.get(0).map_err(|source| StoreError::QueryFailed { source })?,
                contract_id: row.get(1).map_err(|source| StoreError::QueryFailed { source })?,
                status: row.get(2).map_err(|source| StoreError::QueryFailed { source })?,
                created_at: row.get(3).map_err(|source| StoreError::QueryFailed { source })?,
            });
        }
    }

    Ok(results)
}

// PERFORMANCE: stdout/stderr cloned into DB. Limit gate timeouts to keep
// artifacts <10MB. Gate 8 will add LFS pointers for large logs.
pub fn record_gate_run(conn: &Connection, run_id: i64, output: &GateOutput) -> Result<(), StoreError> {
    let (gate_num, passed, message, exit_code, duration_ms, stdout, stderr,
         test_passed, test_failed, test_ignored, parse_errors) = match output {
        GateOutput::Simple(result) => (
            result.gate_num,
            result.passed,
            &result.message,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        GateOutput::Execution(result) => {
            let metrics = result.test_metrics.as_ref();
            (
                result.base.gate_num,
                result.base.passed,
                &result.base.message,
                Some(result.exit_code),
                Some(result.duration_ms),
                Some(&result.stdout),
                Some(&result.stderr),
                metrics.map(|m| m.passed),
                metrics.map(|m| m.failed),
                metrics.map(|m| m.ignored),
                metrics.map(|m| m.parse_errors),
            )
        }
    };

    conn.execute(
        "INSERT INTO gate_runs (
            run_id, gate_num, passed, message, exit_code, duration_ms, 
            stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            run_id,
            gate_num,
            passed as i32,
            message,
            exit_code,
            duration_ms,
            stdout,
            stderr,
            test_passed,
            test_failed,
            test_ignored,
            parse_errors
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;

    Ok(())
}

fn init_schema(conn: &Connection) -> Result<(), StoreError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;

    #[tokio::test]
    async fn test_fetch_run_not_found() {
        let conn = open(":memory:").unwrap();
        let result = fetch_run_summary(&conn, 999).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_runs_pagination() {
        let conn = open(":memory:").unwrap();

        let contract = Contract {
            id: "test-1".to_string(),
            repo_url: "x".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["src".to_string()],
            requires_human_review: false,
        };
        create_run(&conn, &contract).unwrap();
        create_run(&conn, &contract).unwrap();

        let page1 = list_runs(&conn, None, 1, 0).unwrap();
        assert_eq!(page1.len(), 1);
        let page2 = list_runs(&conn, None, 1, 1).unwrap();
        assert_eq!(page2.len(), 1);
        assert_ne!(page1[0].run_id, page2[0].run_id);

        let invalid = list_runs(&conn, None, 0, 0);
        assert!(matches!(invalid, Err(StoreError::InvalidParameter(_))));
    }
}
