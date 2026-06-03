use crate::error::GateError;
use crate::gates::process::execute_with_timeout;
use crate::gates::result::ExecutionResult;
use crate::orchestrator::state_machine::Run;
use std::process::Command;
use std::time::Duration;

pub async fn run(run: &Run) -> Result<ExecutionResult, GateError> {
    let workspace_path = run.workspace.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&workspace_path);

        execute_with_timeout(
            cmd,
            6,
            "Build compilation sequence completed successfully",
            "Build compilation validation failed: source compilation error detected",
            Duration::from_secs(600),
        )
    })
    .await
    .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(result)
}
