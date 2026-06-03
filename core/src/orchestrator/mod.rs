pub mod state_machine;

use crate::contract::Contract;
use crate::error::OrchestratorError;
use crate::gates::runner::run_gates_sequential;
use crate::store::{create_run, record_gate_run, update_run_status, Connection};
use state_machine::{FinalDecision, Run};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_contract(
    db: Arc<Mutex<Connection>>,
    contract: Contract,
    workspace: PathBuf,
) -> Result<FinalDecision, OrchestratorError> {
    let run_id = {
        let conn = db.lock().await;
        create_run(&conn, &contract)?
    };

    let run_context = Run {
        contract,
        workspace,
    };

    let gate_outputs = run_gates_sequential(&run_context).await?;
    let mut final_decision = FinalDecision::Approve;

    for output in &gate_outputs {
        if !output.passed() {
            final_decision = FinalDecision::Reject {
                reason: format!(
                    "Gate {} execution failed to clear verification checks.",
                    output.gate_num()
                ),
            };
        }
    }

    {
        let conn = db.lock().await;
        for output in &gate_outputs {
            record_gate_run(&conn, run_id, output)?;
        }

        match &final_decision {
            FinalDecision::Approve => update_run_status(&conn, run_id, "APPROVED")?,
            FinalDecision::Reject { .. } => update_run_status(&conn, run_id, "REJECTED")?,
        }
    }

    Ok(final_decision)
}
