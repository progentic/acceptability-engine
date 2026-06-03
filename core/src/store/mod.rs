mod connection;
mod gate_records;
mod mappers;
mod queries;
mod runs;
mod schema;
mod types;

pub use connection::{open, shared_connection, with_connection, SharedConnection};
pub use gate_records::record_gate_run;
pub use queries::{fetch_run_summary, list_runs};
pub use runs::{create_queued_run, create_run, update_run_status};
pub use rusqlite::Connection;
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
}
