use crate::error::git::GitError;
use crate::error::GateError;
use crate::gates::result::GateResult;
use crate::orchestrator::state_machine::Run;
use std::path::Path;
use std::process::{Command, Output};

pub async fn run(run: &Run) -> Result<GateResult, GateError> {
    let workspace_path = run.workspace.clone();
    let base_sha = run.contract.base_sha.clone();
    let allowed_scopes = run.contract.scopes.clone();

    let changed_files =
        tokio::task::spawn_blocking(move || execute_git_diff(&workspace_path, &base_sha))
            .await
            .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    for file in changed_files {
        if !is_file_allowed(&file, &allowed_scopes) {
            return Ok(GateResult::fail(
                3,
                format!(
                    "Change boundary violation: file '{}' outside contract scopes",
                    file
                ),
            ));
        }
    }

    Ok(GateResult::pass(
        3,
        "All changed files fall within contract scopes",
    ))
}

fn execute_git_diff(repo_path: &Path, base_sha: &str) -> Result<Vec<String>, GitError> {
    let output = run_git_command(repo_path, base_sha)?;
    parse_git_diff_output(&output)
}

fn run_git_command(repo_path: &Path, base_sha: &str) -> Result<Output, GitError> {
    Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("diff")
        .arg("--name-only")
        .arg(base_sha)
        .arg("HEAD")
        .output()
        .map_err(|source| GitError::ProcessExecutionFailed { source })
}

fn parse_git_diff_output(output: &Output) -> Result<Vec<String>, GitError> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code().unwrap_or(-1);
        return Err(GitError::CommandFailed { code, stderr });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.push(trimmed.to_string());
        }
    }
    Ok(files)
}

fn is_file_allowed(file_path: &str, allowed_scopes: &[String]) -> bool {
    for scope in allowed_scopes {
        if scope.ends_with('/') {
            if file_path.starts_with(scope) {
                return true;
            }
        } else if file_path == scope || file_path.starts_with(&format!("{}/", scope)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_boundary() {
        let scopes = vec!["src/api".to_string(), "src/core/".to_string()];

        assert!(is_file_allowed("src/api/file.rs", &scopes));
        assert!(is_file_allowed("src/api", &scopes));
        assert!(is_file_allowed("src/core/mod.rs", &scopes));

        assert!(!is_file_allowed("src/api_backup/file.rs", &scopes));
        assert!(!is_file_allowed("src/core_engine/mod.rs", &scopes));
    }
}
