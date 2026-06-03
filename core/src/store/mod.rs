mod clock;
mod connection;
mod evidence;
mod final_decisions;
mod gate_records;
mod mappers;
mod queries;
mod runs;
mod schema;
mod transaction;
mod types;

pub use connection::{open, shared_connection, with_connection, SharedConnection};
pub use evidence::create_evidence_bundle;
pub use final_decisions::record_final_decision;
pub use gate_records::record_gate_run;
pub use queries::{fetch_run_summary, list_runs};
pub use runs::{
    create_attempt, create_queued_run, create_run, update_attempt_status, update_run_status,
};
pub use rusqlite::Connection;
pub use transaction::with_transaction;
pub use types::{RunListItem, RunStatusSummary};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::error::StoreError;

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

    #[tokio::test]
    async fn test_create_queued_run_status() {
        let conn = open(":memory:").unwrap();
        let contract = Contract {
            id: "test-queued".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["src".to_string()],
            requires_human_review: false,
        };

        let run_id = create_queued_run(&conn, &contract).unwrap();
        let summary = fetch_run_summary(&conn, run_id).unwrap().unwrap();

        assert_eq!(summary.status, "QUEUED");
    }

    #[test]
    fn migration_preserves_legacy_gate_rows_under_attempt() {
        let conn = Connection::open_in_memory().unwrap();
        create_legacy_schema(&conn);

        schema::init_schema(&conn).unwrap();

        assert_eq!(table_count(&conn, "attempts"), 2);
        assert_eq!(legacy_gate_attempt_number(&conn), 1);
    }

    #[test]
    fn latest_attempt_summary_excludes_older_attempt_gates() {
        let conn = open(":memory:").unwrap();
        let contract = Contract {
            id: "test-latest-attempt".to_string(),
            repo_url: "x".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["src".to_string()],
            requires_human_review: false,
        };

        let run_id = create_run(&conn, &contract).unwrap();
        let old_attempt_id = create_attempt(&conn, run_id).unwrap();
        let new_attempt_id = create_attempt(&conn, run_id).unwrap();
        record_gate_run(
            &conn,
            old_attempt_id,
            &crate::gates::result::GateOutput::Simple(crate::gates::result::GateResult::fail(
                1,
                "old failure".to_string(),
            )),
        )
        .unwrap();
        record_gate_run(
            &conn,
            new_attempt_id,
            &crate::gates::result::GateOutput::Simple(crate::gates::result::GateResult::pass(
                1, "new pass",
            )),
        )
        .unwrap();

        let summary = fetch_run_summary(&conn, run_id).unwrap().unwrap();

        assert_eq!(summary.gates.len(), 1);
        assert_eq!(summary.gates[0].message, "new pass");
    }

    #[test]
    fn final_decision_is_unique_per_run() {
        let conn = open(":memory:").unwrap();
        let contract = Contract {
            id: "test-final-unique".to_string(),
            repo_url: "x".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["src".to_string()],
            requires_human_review: false,
        };
        let run_id = create_run(&conn, &contract).unwrap();

        record_final_decision(&conn, run_id, "APPROVED", None).unwrap();
        let duplicate = record_final_decision(&conn, run_id, "REJECTED", Some("duplicate"));

        assert!(matches!(duplicate, Err(StoreError::InsertFailed { .. })));
    }

    fn create_legacy_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE contracts (
                 id TEXT PRIMARY KEY,
                 repo_url TEXT NOT NULL,
                 base_sha TEXT NOT NULL,
                 requires_human_review INTEGER NOT NULL
             );
             CREATE TABLE runs (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 contract_id TEXT NOT NULL,
                 status TEXT NOT NULL,
                 created_at INTEGER NOT NULL
             );
             CREATE TABLE gate_runs (
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
                 parse_errors INTEGER
             );
             INSERT INTO contracts (id, repo_url, base_sha, requires_human_review)
             VALUES ('legacy-1', 'x', 'a9993e364706816aba3e25717850c26c9cd0d89d', 0),
                    ('legacy-2', 'x', 'a9993e364706816aba3e25717850c26c9cd0d89d', 0);
             INSERT INTO runs (id, contract_id, status, created_at)
             VALUES (1, 'legacy-1', 'APPROVED', 10),
                    (2, 'legacy-2', 'QUEUED', 20);
             INSERT INTO gate_runs (run_id, gate_num, passed, message)
             VALUES (1, 1, 1, 'legacy gate');",
        )
        .unwrap();
    }

    fn table_count(conn: &Connection, table_name: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table_name}"), [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    fn legacy_gate_attempt_number(conn: &Connection) -> i64 {
        conn.query_row(
            "SELECT attempts.attempt_number
             FROM gate_runs
             JOIN attempts ON attempts.id = gate_runs.attempt_id
             WHERE gate_runs.message = 'legacy gate'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }
}
