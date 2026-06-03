pub mod state_machine;

use crate::contract::Contract;
use crate::error::OrchestratorError;
use crate::gates::result::GateOutput;
use crate::gates::runner::run_gates_sequential;
use crate::store::{
    create_attempt, create_evidence_bundle, create_run, record_final_decision, record_gate_run,
    update_attempt_status, update_run_status, with_connection, with_transaction, AttemptId, RunId,
    SharedConnection,
};
use state_machine::{FinalDecision, Run};
use std::path::PathBuf;

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
    run_id: RunId,
    contract: Contract,
    workspace: PathBuf,
) -> Result<FinalDecision, OrchestratorError> {
    mark_run_running(&db, run_id).await?;
    let attempt_id = create_run_attempt(&db, run_id).await?;
    let requires_human_review = contract.requires_human_review;
    let run_context = build_run_context(contract, workspace);
    let gate_outputs = match run_gates_sequential(&run_context).await {
        Ok(outputs) => outputs,
        Err(error) => {
            finalize_internal_error(&db, run_id, attempt_id).await?;
            return Err(error.into());
        }
    };
    let final_decision = decide_from_gate_outputs(&gate_outputs, requires_human_review);

    finalize_run_record(&db, run_id, attempt_id, &gate_outputs, &final_decision).await?;

    Ok(final_decision)
}

async fn create_run_record(
    db: &SharedConnection,
    contract: &Contract,
) -> Result<RunId, OrchestratorError> {
    let contract = contract.clone();
    Ok(with_connection(db.clone(), move |conn| create_run(conn, &contract)).await?)
}

async fn mark_run_running(db: &SharedConnection, run_id: RunId) -> Result<(), OrchestratorError> {
    Ok(with_connection(db.clone(), move |conn| {
        update_run_status(conn, run_id, "RUNNING")
    })
    .await?)
}

async fn create_run_attempt(
    db: &SharedConnection,
    run_id: RunId,
) -> Result<AttemptId, OrchestratorError> {
    Ok(with_connection(db.clone(), move |conn| create_attempt(conn, run_id)).await?)
}

async fn finalize_internal_error(
    db: &SharedConnection,
    run_id: RunId,
    attempt_id: AttemptId,
) -> Result<(), OrchestratorError> {
    Ok(with_connection(db.clone(), move |conn| {
        with_transaction(conn, |transaction| {
            update_attempt_status(transaction, attempt_id, "ERROR")?;
            update_run_status(transaction, run_id, "FAILED_INTERNAL")?;
            create_evidence_bundle(
                transaction,
                run_id,
                Some(attempt_id),
                None,
                "engine error during gate execution",
            )?;
            Ok(())
        })
    })
    .await?)
}

fn build_run_context(contract: Contract, workspace: PathBuf) -> Run {
    Run {
        contract,
        workspace,
    }
}

fn decide_from_gate_outputs(
    gate_outputs: &[GateOutput],
    requires_human_review: bool,
) -> FinalDecision {
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
    if requires_human_review {
        return FinalDecision::PendingHumanReview;
    }
    FinalDecision::Approve
}

async fn finalize_run_record(
    db: &SharedConnection,
    run_id: RunId,
    attempt_id: AttemptId,
    gate_outputs: &[GateOutput],
    final_decision: &FinalDecision,
) -> Result<(), OrchestratorError> {
    let gate_outputs = gate_outputs.to_vec();
    let status = final_status(final_decision);
    let attempt_status = final_attempt_status(final_decision);
    let final_decision_record = persisted_final_decision(final_decision);
    Ok(with_connection(db.clone(), move |conn| {
        with_transaction(conn, |transaction| {
            record_gate_outputs(transaction, run_id, attempt_id, &gate_outputs)?;
            update_attempt_status(transaction, attempt_id, attempt_status)?;
            update_run_status(transaction, run_id, status)?;
            record_persisted_final_decision(transaction, run_id, final_decision_record)
        })
    })
    .await?)
}

fn final_status(final_decision: &FinalDecision) -> &'static str {
    match final_decision {
        FinalDecision::Approve => "APPROVED",
        FinalDecision::PendingHumanReview => "PENDING_HUMAN_REVIEW",
        FinalDecision::Reject { .. } => "REJECTED",
    }
}

fn final_attempt_status(final_decision: &FinalDecision) -> &'static str {
    match final_decision {
        FinalDecision::Approve | FinalDecision::PendingHumanReview => "PASSED",
        FinalDecision::Reject { .. } => "FAILED",
    }
}

fn persisted_final_decision(
    final_decision: &FinalDecision,
) -> Option<(&'static str, Option<String>)> {
    match final_decision {
        FinalDecision::Approve => Some(("APPROVED", None)),
        FinalDecision::PendingHumanReview => None,
        FinalDecision::Reject { reason } => Some(("REJECTED", Some(reason.clone()))),
    }
}

fn record_persisted_final_decision(
    conn: &crate::store::Connection,
    run_id: RunId,
    decision: Option<(&str, Option<String>)>,
) -> Result<(), crate::error::StoreError> {
    let Some((status, reason)) = decision else {
        return Ok(());
    };
    record_final_decision(conn, run_id, status, reason.as_deref())?;
    Ok(())
}

fn record_gate_outputs(
    conn: &crate::store::Connection,
    run_id: RunId,
    attempt_id: AttemptId,
    gate_outputs: &[GateOutput],
) -> Result<(), crate::error::StoreError> {
    for output in gate_outputs {
        let gate_run_id = record_gate_run(conn, attempt_id, output)?;
        create_evidence_bundle(
            conn,
            run_id,
            Some(attempt_id),
            Some(gate_run_id),
            "gate telemetry captured",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::result::{GateOutput, GateResult};
    use crate::store::{create_queued_run, fetch_run_summary, open, shared_connection};
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
            decide_from_gate_outputs(&gate_outputs, false),
            FinalDecision::Approve
        ));
    }

    #[test]
    fn requests_human_review_when_all_gate_outputs_pass() {
        let gate_outputs = vec![GateOutput::Simple(GateResult::pass(1, "contract ok"))];

        assert!(matches!(
            decide_from_gate_outputs(&gate_outputs, true),
            FinalDecision::PendingHumanReview
        ));
    }

    #[test]
    fn rejects_when_any_gate_output_fails() {
        let gate_outputs = vec![GateOutput::Simple(GateResult::fail(
            3,
            "boundary failed".to_string(),
        ))];

        assert!(matches!(
            decide_from_gate_outputs(&gate_outputs, true),
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
        let db = shared_connection(open(":memory:").unwrap());
        let contract = valid_contract();
        let run_id = create_run_record(&db, &contract).await.unwrap();
        let attempt_id = create_run_attempt(&db, run_id).await.unwrap();
        let gate_outputs = vec![GateOutput::Simple(GateResult::fail(
            2,
            "workspace failed".to_string(),
        ))];
        let final_decision = decide_from_gate_outputs(&gate_outputs, false);

        finalize_run_record(&db, run_id, attempt_id, &gate_outputs, &final_decision)
            .await
            .unwrap();

        let summary = with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(summary.status, "REJECTED");
        assert_eq!(summary.gates.len(), 1);
        assert_eq!(final_decision_count(&db, run_id).await, 1);
        assert_eq!(evidence_bundle_count(&db, attempt_id).await, 1);
        assert_eq!(gate_evidence_link_count(&db, attempt_id).await, 1);
    }

    #[tokio::test]
    async fn pending_human_review_skips_final_decision_record() {
        let db = shared_connection(open(":memory:").unwrap());
        let mut contract = valid_contract();
        contract.requires_human_review = true;

        let run_id = create_run_record(&db, &contract).await.unwrap();
        let attempt_id = create_run_attempt(&db, run_id).await.unwrap();
        let gate_outputs = vec![GateOutput::Simple(GateResult::pass(1, "contract ok"))];
        let final_decision = decide_from_gate_outputs(&gate_outputs, true);

        finalize_run_record(&db, run_id, attempt_id, &gate_outputs, &final_decision)
            .await
            .unwrap();

        let summary = with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(summary.status, "PENDING_HUMAN_REVIEW");
        assert_eq!(summary.gates.len(), 1);
        assert_eq!(final_decision_count(&db, run_id).await, 0);
        assert_eq!(evidence_bundle_count(&db, attempt_id).await, 1);
        assert_eq!(gate_evidence_link_count(&db, attempt_id).await, 1);
    }

    #[tokio::test]
    async fn passing_non_human_review_run_gets_approved_final_decision() {
        let db = shared_connection(open(":memory:").unwrap());
        let contract = valid_contract();
        let run_id = create_run_record(&db, &contract).await.unwrap();
        let attempt_id = create_run_attempt(&db, run_id).await.unwrap();
        let gate_outputs = vec![GateOutput::Simple(GateResult::pass(1, "contract ok"))];
        let final_decision = decide_from_gate_outputs(&gate_outputs, false);

        finalize_run_record(&db, run_id, attempt_id, &gate_outputs, &final_decision)
            .await
            .unwrap();

        let summary = with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(summary.status, "APPROVED");
        assert_eq!(attempt_status(&db, attempt_id).await, "PASSED");
        assert_eq!(final_decision_count(&db, run_id).await, 1);
    }

    #[tokio::test]
    async fn duplicate_final_decision_rolls_back_finalization() {
        let db = shared_connection(open(":memory:").unwrap());
        let contract = valid_contract();
        let run_id = create_run_record(&db, &contract).await.unwrap();
        let attempt_id = create_run_attempt(&db, run_id).await.unwrap();
        let gate_outputs = vec![GateOutput::Simple(GateResult::pass(1, "contract ok"))];
        let final_decision = decide_from_gate_outputs(&gate_outputs, false);
        insert_final_decision(&db, run_id).await;

        let result =
            finalize_run_record(&db, run_id, attempt_id, &gate_outputs, &final_decision).await;

        assert!(result.is_err());
        assert_eq!(attempt_status(&db, attempt_id).await, "RUNNING");
        assert_eq!(evidence_bundle_count(&db, attempt_id).await, 0);
        assert_eq!(latest_summary_gate_count(&db, run_id).await, 0);
    }

    #[tokio::test]
    async fn internal_error_finalizes_run_and_attempt() {
        let db = shared_connection(open(":memory:").unwrap());
        let contract = valid_contract();
        let run_id = create_run_record(&db, &contract).await.unwrap();
        let attempt_id = create_run_attempt(&db, run_id).await.unwrap();

        finalize_internal_error(&db, run_id, attempt_id)
            .await
            .unwrap();

        let summary = with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(summary.status, "FAILED_INTERNAL");
        assert_eq!(attempt_status(&db, attempt_id).await, "ERROR");
        assert_eq!(engine_error_evidence_count(&db, attempt_id).await, 1);
    }

    #[tokio::test]
    async fn marks_queued_run_running_before_execution() {
        let db = shared_connection(open(":memory:").unwrap());
        let contract = valid_contract();
        let run_id = with_connection(db.clone(), move |conn| create_queued_run(conn, &contract))
            .await
            .unwrap();

        mark_run_running(&db, run_id).await.unwrap();

        let summary = with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(summary.status, "RUNNING");
    }

    async fn final_decision_count(db: &SharedConnection, run_id: RunId) -> i64 {
        with_connection(db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM final_decisions WHERE run_id = ?1",
                rusqlite::params![run_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| crate::error::StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn evidence_bundle_count(db: &SharedConnection, attempt_id: AttemptId) -> i64 {
        with_connection(db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM evidence_bundles WHERE attempt_id = ?1",
                rusqlite::params![attempt_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| crate::error::StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn gate_evidence_link_count(db: &SharedConnection, attempt_id: AttemptId) -> i64 {
        with_connection(db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*)
                 FROM evidence_bundles
                 WHERE attempt_id = ?1
                   AND run_id IS NOT NULL
                   AND gate_run_id IS NOT NULL",
                rusqlite::params![attempt_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| crate::error::StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn attempt_status(db: &SharedConnection, attempt_id: AttemptId) -> String {
        with_connection(db.clone(), move |conn| {
            conn.query_row(
                "SELECT status FROM attempts WHERE id = ?1",
                rusqlite::params![attempt_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| crate::error::StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn insert_final_decision(db: &SharedConnection, run_id: RunId) {
        with_connection(db.clone(), move |conn| {
            crate::store::record_final_decision(conn, run_id, "APPROVED", None)?;
            Ok(())
        })
        .await
        .unwrap();
    }

    async fn latest_summary_gate_count(db: &SharedConnection, run_id: RunId) -> usize {
        with_connection(db.clone(), move |conn| fetch_run_summary(conn, run_id))
            .await
            .unwrap()
            .unwrap()
            .gates
            .len()
    }

    async fn engine_error_evidence_count(db: &SharedConnection, attempt_id: AttemptId) -> i64 {
        with_connection(db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*)
                 FROM evidence_bundles
                 WHERE attempt_id = ?1
                   AND gate_run_id IS NULL
                   AND summary = 'engine error during gate execution'",
                rusqlite::params![attempt_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| crate::error::StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }
}
