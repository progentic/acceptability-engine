use crate::error::git::GitError;
use crate::error::GateError;
use crate::gates::result::GateResult;
use crate::orchestrator::state_machine::Run;
use std::path::Path;
use std::process::{Command, Output};

pub async fn run(run: &Run) -> Result<GateResult, GateError> {
    let workspace_path = run.workspace.clone();
    let base_sha = run.contract.base_sha.clone();
    let candidate_sha = run.contract.candidate_sha.clone();

    tokio::task::spawn_blocking(move || {
        validate_local_workspace(&workspace_path)?;
        validate_sha("base_sha", &base_sha)?;
        validate_sha("candidate_sha", &candidate_sha)?;
        validate_git_repository(&workspace_path)?;
        validate_commit(&workspace_path, &base_sha)?;
        validate_commit(&workspace_path, &candidate_sha)?;
        validate_base_ancestor(&workspace_path, &base_sha, &candidate_sha)?;
        validate_head_commit(&workspace_path, &candidate_sha)?;
        Ok::<(), GitError>(())
    })
    .await
    .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(GateResult::pass(
        2,
        "Local Git workspace verification successful",
    ))
}

fn validate_local_workspace(path: &Path) -> Result<(), GitError> {
    if !path.exists() {
        return Err(GitError::WorkspaceNotFound {
            path: display_path(path),
        });
    }
    if !path.is_dir() {
        return Err(GitError::WorkspaceNotDirectory {
            path: display_path(path),
        });
    }
    Ok(())
}

fn validate_sha(_field: &str, sha: &str) -> Result<(), GitError> {
    if sha.trim().is_empty() {
        return Err(GitError::EmptyBaseSha);
    }
    Ok(())
}

fn validate_git_repository(path: &Path) -> Result<(), GitError> {
    let output = run_git_command(path, ["rev-parse", "--is-inside-work-tree"])?;
    if command_stdout_equals(&output, "true") {
        return Ok(());
    }
    Err(GitError::RepoNotFound {
        path: display_path(path),
    })
}

fn validate_commit(path: &Path, sha: &str) -> Result<(), GitError> {
    let commit_ref = format!("{sha}^{{commit}}");
    let output = run_git_command(path, ["cat-file", "-e", commit_ref.as_str()])?;
    if output.status.success() {
        return Ok(());
    }
    Err(GitError::CommitNotFound {
        path: display_path(path),
        sha: sha.to_string(),
    })
}

fn validate_base_ancestor(
    path: &Path,
    base_sha: &str,
    candidate_sha: &str,
) -> Result<(), GitError> {
    let output = run_git_command(
        path,
        ["merge-base", "--is-ancestor", base_sha, candidate_sha],
    )?;
    if output.status.success() {
        return Ok(());
    }
    Err(GitError::BaseNotAncestor {
        base_sha: base_sha.to_string(),
        candidate_sha: candidate_sha.to_string(),
    })
}

fn validate_head_commit(path: &Path, candidate_sha: &str) -> Result<(), GitError> {
    let output = run_git_command(path, ["rev-parse", "HEAD"])?;
    let head = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() && head == candidate_sha.to_ascii_lowercase() {
        return Ok(());
    }
    Err(GitError::HeadMismatch {
        head,
        candidate_sha: candidate_sha.to_ascii_lowercase(),
    })
}

fn run_git_command<const N: usize>(path: &Path, args: [&str; N]) -> Result<Output, GitError> {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|source| GitError::ProcessExecutionFailed { source })
}

fn command_stdout_equals(output: &Output, expected: &str) -> bool {
    output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == expected
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_missing_local_workspace() {
        let missing = std::env::temp_dir().join(unique_test_name("missing"));

        assert!(matches!(
            validate_local_workspace(&missing),
            Err(GitError::WorkspaceNotFound { .. })
        ));
    }

    #[test]
    fn rejects_non_directory_workspace() {
        let file_path = create_temp_file("not-dir");

        assert!(matches!(
            validate_local_workspace(&file_path),
            Err(GitError::WorkspaceNotDirectory { .. })
        ));

        let _ = fs::remove_file(file_path);
    }

    fn create_temp_file(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(unique_test_name(label));
        fs::write(&path, b"test").unwrap();
        path
    }

    fn unique_test_name(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("acceptability-engine-{label}-{nanos}")
    }
}
