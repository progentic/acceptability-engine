use crate::error::GateError;
use crate::gates::process::execute_with_timeout;
use crate::gates::result::ExecutionResult;
use crate::orchestrator::state_machine::Run;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const GATE_NUM: u8 = 8;
const SUPPLY_CHAIN_TIMEOUT: Duration = Duration::from_secs(600);

pub async fn run(run: &Run) -> Result<ExecutionResult, GateError> {
    let workspace_path = run.workspace.clone();

    let result = tokio::task::spawn_blocking(move || run_supply_chain_checks(&workspace_path))
        .await
        .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(result)
}

fn run_supply_chain_checks(
    workspace_path: &Path,
) -> Result<ExecutionResult, crate::error::ProcessError> {
    let deny_result = execute_cargo_deny(workspace_path)?;
    if !deny_result.base.passed {
        return Ok(deny_result);
    }

    let audit_result = execute_cargo_audit(workspace_path)?;
    if !audit_result.base.passed {
        return Ok(audit_result);
    }

    Ok(merge_successful_results(deny_result, audit_result))
}

fn execute_cargo_deny(
    workspace_path: &Path,
) -> Result<ExecutionResult, crate::error::ProcessError> {
    execute_with_timeout(
        build_cargo_deny_command(workspace_path),
        GATE_NUM,
        "cargo deny policy check completed successfully",
        "Supply chain policy failure: cargo deny check reported violations",
        SUPPLY_CHAIN_TIMEOUT,
    )
}

fn execute_cargo_audit(
    workspace_path: &Path,
) -> Result<ExecutionResult, crate::error::ProcessError> {
    execute_with_timeout(
        build_cargo_audit_command(workspace_path),
        GATE_NUM,
        "cargo audit advisory scan completed successfully",
        "Supply chain advisory failure: cargo audit reported vulnerabilities",
        SUPPLY_CHAIN_TIMEOUT,
    )
}

fn build_cargo_deny_command(workspace_path: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.arg("deny").arg("check").current_dir(workspace_path);
    command
}

fn build_cargo_audit_command(workspace_path: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.arg("audit").current_dir(workspace_path);
    command
}

fn merge_successful_results(
    deny_result: ExecutionResult,
    audit_result: ExecutionResult,
) -> ExecutionResult {
    let mut stdout = deny_result.stdout;
    append_section_separator(&mut stdout);
    stdout.extend(audit_result.stdout);

    let mut stderr = deny_result.stderr;
    append_section_separator(&mut stderr);
    stderr.extend(audit_result.stderr);

    ExecutionResult::pass(
        GATE_NUM,
        "Supply chain checks passed: cargo deny and cargo audit completed successfully",
        0,
        deny_result.duration_ms + audit_result.duration_ms,
        stdout,
        stderr,
    )
}

fn append_section_separator(buffer: &mut Vec<u8>) {
    if !buffer.is_empty() {
        buffer.extend(b"\n---\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    #[test]
    fn builds_cargo_deny_command() {
        let workspace = PathBuf::from("/tmp/workspace");
        let command = build_cargo_deny_command(&workspace);

        assert_eq!(command.get_program(), OsStr::new("cargo"));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![OsStr::new("deny"), OsStr::new("check")]
        );
        assert_eq!(command.get_current_dir(), Some(workspace.as_path()));
    }

    #[test]
    fn builds_cargo_audit_command() {
        let workspace = PathBuf::from("/tmp/workspace");
        let command = build_cargo_audit_command(&workspace);

        assert_eq!(command.get_program(), OsStr::new("cargo"));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![OsStr::new("audit")]
        );
        assert_eq!(command.get_current_dir(), Some(workspace.as_path()));
    }

    #[test]
    fn merges_successful_supply_chain_results() {
        let deny_result = ExecutionResult::pass(8, "deny ok", 0, 10, b"deny".to_vec(), Vec::new());
        let audit_result =
            ExecutionResult::pass(8, "audit ok", 0, 20, b"audit".to_vec(), Vec::new());

        let merged = merge_successful_results(deny_result, audit_result);

        assert!(merged.base.passed);
        assert_eq!(merged.duration_ms, 30);
        assert_eq!(merged.stdout, b"deny\n---\naudit".to_vec());
    }
}
