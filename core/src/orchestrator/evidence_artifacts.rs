use crate::error::StoreError;
use crate::gates::result::{ExecutionResult, GateOutput, GateResult};
use crate::store::{
    create_artifact_evidence_bundle, ArtifactInput, ArtifactStore, AttemptId, Connection,
    GateRunId, RunId, StoredArtifactDescriptor,
};

pub struct PendingGateArtifact {
    descriptor: StoredArtifactDescriptor,
}

pub fn prepare_gate_artifact(
    artifact_store: &ArtifactStore,
    run_id: RunId,
    attempt_id: AttemptId,
    output: &GateOutput,
) -> Result<PendingGateArtifact, StoreError> {
    let label = gate_artifact_label(output);
    let bytes = gate_artifact_bytes(output);
    let descriptor = artifact_store.write_artifact(ArtifactInput {
        run_id,
        attempt_id: Some(attempt_id),
        gate_run_id: None,
        kind: "gate-telemetry",
        label: &label,
        content_type: "application/json",
        summary: "gate telemetry artifact captured",
        bytes: &bytes,
    })?;
    Ok(PendingGateArtifact { descriptor })
}

pub fn record_gate_artifact_descriptor(
    conn: &Connection,
    run_id: RunId,
    attempt_id: AttemptId,
    gate_run_id: GateRunId,
    artifact: &PendingGateArtifact,
) -> Result<(), StoreError> {
    create_artifact_evidence_bundle(
        conn,
        run_id,
        Some(attempt_id),
        Some(gate_run_id),
        &artifact.descriptor,
    )?;
    Ok(())
}

fn gate_artifact_label(output: &GateOutput) -> String {
    format!("Gate {} telemetry", output.gate_num())
}

fn gate_artifact_bytes(output: &GateOutput) -> Vec<u8> {
    match output {
        GateOutput::Simple(result) => simple_gate_artifact(result),
        GateOutput::Execution(result) => execution_gate_artifact(result),
    }
}

fn simple_gate_artifact(result: &GateResult) -> Vec<u8> {
    serde_json::json!({
        "gate_num": result.gate_num,
        "passed": result.passed,
        "message": &result.message
    })
    .to_string()
    .into_bytes()
}

fn execution_gate_artifact(result: &ExecutionResult) -> Vec<u8> {
    serde_json::json!({
        "gate_num": result.base.gate_num,
        "passed": result.base.passed,
        "message": &result.base.message,
        "exit_code": result.exit_code,
        "duration_ms": result.duration_ms,
        "stdout_byte_len": result.stdout.len(),
        "stderr_byte_len": result.stderr.len(),
        "test_metrics": result.test_metrics
    })
    .to_string()
    .into_bytes()
}
