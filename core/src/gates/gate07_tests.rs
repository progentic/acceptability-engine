use crate::error::GateError;
use crate::gates::process::execute_with_timeout;
use crate::gates::result::{ExecutionResult, TestMetrics};
use crate::orchestrator::state_machine::Run;
use serde::Deserialize;
use std::process::Command;
use std::time::Duration;

#[derive(Deserialize)]
struct CargoTestEvent {
    #[serde(rename = "type")]
    event_type: String,
    event: Option<String>,
}

pub async fn run(run: &Run) -> Result<ExecutionResult, GateError> {
    let workspace_path = run.workspace.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new("cargo");
        cmd.arg("test")
            .arg("--")
            .arg("-Z")
            .arg("unstable-options")
            .arg("--format")
            .arg("json")
            .current_dir(&workspace_path);

        let mut exec_result = execute_with_timeout(
            cmd,
            7,
            "All test suites verified successfully",
            "Test execution failure: unresolved runtime test failures detected",
            Duration::from_secs(1800),
        )?;

        let metrics = parse_test_metrics(&exec_result.stdout);
        exec_result.test_metrics = Some(metrics.clone());

        if metrics.failed > 0 {
            exec_result.base.passed = false;
            exec_result.base.message = format!(
                "Test suite execution failed: {} passed, {} failed, {} ignored",
                metrics.passed, metrics.failed, metrics.ignored
            );
        }

        Ok::<ExecutionResult, crate::error::ProcessError>(exec_result)
    })
    .await
    .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(result)
}

// External constraint: Cargo test JSON format is unstable. Non-JSON lines are
// emitted before test events. parse_errors counter tracks these.
fn parse_test_metrics(stdout: &[u8]) -> TestMetrics {
    let mut metrics = TestMetrics::default();
    let mut suite_failed = false;

    for line in stdout.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }

        let event: CargoTestEvent = match serde_json::from_slice(line) {
            Ok(parsed) => parsed,
            Err(_) => {
                metrics.parse_errors += 1;
                continue;
            }
        };

        match (event.event_type.as_str(), event.event.as_deref()) {
            ("test", Some("ok")) => metrics.passed += 1,
            ("test", Some("failed")) => metrics.failed += 1,
            ("test", Some("ignored")) => metrics.ignored += 1,
            ("suite", Some("failed")) => suite_failed = true,
            _ => {}
        }
    }

    if suite_failed && metrics.failed == 0 && metrics.passed == 0 {
        metrics.failed = 1;
    }

    metrics
}
