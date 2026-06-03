pub mod state_machine;

use crate::contract::Contract;
use crate::error::OrchestratorError;
use crate::gates::result::GateOutput;
use crate::gates::runner::run_gates_sequential;
use crate::store::{create_run, record_gate_run, update_run_status, Connection};
use state_machine::{FinalDecision, Run};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedConnection = Arc<Mutex<Connection>>;

pub async fn run_contract(
    db: SharedConnection,
    contract: Contract,
    workspace: PathBuf,
) -> Result<FinalDecision, OrchestratorError> {
    let run_id = create_run_record(&db, &contract).await?;
    execute_existing_run(db, run_id, contract, workspace).await
}

pub async fn execute_existing_run(
    db: SharedConnection,
    run_id: i64,
    contract: Contract,
    workspace: PathBuf,
) -> Result<FinalDecision, OrchestratorError> {
    mark_run_running(&db, run_id).await?;
    let run_context = build_run_context(contract, workspace);
    let gate_outputs = run_gates_sequential(&run_context).await?;
    let final_decision = decide_from_gate_outputs(&gate_outputs);

    finalize_run_record(&db, run_id, &gate_outputs, &final_decision).await?;

    Ok(final_decision)
}

async fn create_run_record(
    db: &SharedConnection,
    contract: &Contract,
) -> Result<i64, OrchestratorError> {
    let conn = db.lock().await;
    Ok(create_run(&conn, contract)?)
}

async fn mark_run_running(db: &SharedConnection, run_id: i64) -> Result<(), OrchestratorError> {
    let conn = db.lock().await;
    Ok(update_run_status(&conn, run_id, "RUNNING")?)
}

fn build_run_context(contract: Contract, workspace: PathBuf) -> Run {
    Run {
        contract,
        workspace,
    }
}

fn decide_from_gate_outputs(gate_outputs: &[GateOutput]) -> FinalDecision {
    for output in gate_outputs {
        if !output.passed() {
            return FinalDecision::Reject {
                reason: format!(
                    "Gate {} execution failed to clear verification checks.",
                    output.gate_num()
                ),
            };
        }
    }
    FinalDecision::Approve
}

async fn finalize_run_record(
    db: &SharedConnection,
    run_id: i64,
    gate_outputs: &[GateOutput],
    final_decision: &FinalDecision,
) -> Result<(), OrchestratorError> {
    let conn = db.lock().await;
    for output in gate_outputs {
        record_gate_run(&conn, run_id, output)?;
    }

    match final_decision {
        FinalDecision::Approve => update_run_status(&conn, run_id, "APPROVED")?,
        FinalDecision::Reject { .. } => update_run_status(&conn, run_id, "REJECTED")?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::result::{GateOutput, GateResult};
    use crate::store::{create_queued_run, fetch_run_summary, open};
    use std::path::PathBuf;

    fn valid_contract() -> Contract {
        Contract {
            id: "run-001".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
        }
    }

    #[test]
    fn approves_when_all_gate_outputs_pass() {
        let gate_outputs = vec![
            GateOutput::Simple(GateResult::pass(1, "contract ok")),
            GateOutput::Simple(GateResult::pass(2, "workspace ok")),
        ];

        assert!(matches!(
            decide_from_gate_outputs(&gate_outputs),
            FinalDecision::Approve
        ));
    }

    #[test]
    fn rejects_when_any_gate_output_fails() {
        let gate_outputs = vec![GateOutput::Simple(GateResult::fail(
            3,
            "boundary failed".to_string(),
        ))];

        assert!(matches!(
            decide_from_gate_outputs(&gate_outputs),
            FinalDecision::Reject { reason } if reason.contains("Gate 3")
        ));
    }

    #[test]
    fn builds_run_context_without_store_access() {
        let run_context = build_run_context(valid_contract(), PathBuf::from("/tmp/work/run-001"));

        assert_eq!(run_context.contract.id, "run-001");
        assert_eq!(run_context.workspace, PathBuf::from("/tmp/work/run-001"));
    }

    #[tokio::test]
    async fn records_gate_outputs_and_final_status_together() {
        let db = Arc::new(Mutex::new(open(":memory:").unwrap()));
        let contract = valid_contract();
        let run_id = create_run_record(&db, &contract).await.unwrap();
        let gate_outputs = vec![GateOutput::Simple(GateResult::fail(
            2,
            "workspace failed".to_string(),
        ))];
        let final_decision = decide_from_gate_outputs(&gate_outputs);

        finalize_run_record(&db, run_id, &gate_outputs, &final_decision)
            .await
            .unwrap();

        let conn = db.lock().await;
        let summary = fetch_run_summary(&conn, run_id).unwrap().unwrap();
        assert_eq!(summary.status, "REJECTED");
        assert_eq!(summary.gates.len(), 1);
    }

    #[tokio::test]
    async fn marks_queued_run_running_before_execution() {
        let db = Arc::new(Mutex::new(open(":memory:").unwrap()));
        let contract = valid_contract();
        let run_id = {
            let conn = db.lock().await;
            create_queued_run(&conn, &contract).unwrap()
        };

        mark_run_running(&db, run_id).await.unwrap();

        let conn = db.lock().await;
        let summary = fetch_run_summary(&conn, run_id).unwrap().unwrap();
        assert_eq!(summary.status, "RUNNING");
    }
}
