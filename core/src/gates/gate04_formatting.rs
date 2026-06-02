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
        cmd.arg("fmt")
            .arg("--")
            .arg("--check")
            .current_dir(&workspace_path);

        execute_with_timeout(
            cmd,
            4,
            "Codebase formatting complies perfectly with standard style guidelines",
            "Codebase formatting validation failed: run 'cargo fmt' locally to clean workspace source files",
            Duration::from_secs(300),
        )
    })
    .await
    .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(result)
}
