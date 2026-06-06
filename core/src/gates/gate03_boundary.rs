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
    let allowed_scopes = run.contract.scopes.clone();

    let changed_files = tokio::task::spawn_blocking(move || {
        execute_git_diff(&workspace_path, &base_sha, &candidate_sha)
    })
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

fn execute_git_diff(
    repo_path: &Path,
    base_sha: &str,
    candidate_sha: &str,
) -> Result<Vec<String>, GitError> {
    let output = run_git_command(repo_path, base_sha, candidate_sha)?;
    parse_git_diff_output(&output)
}

fn run_git_command(
    repo_path: &Path,
    base_sha: &str,
    candidate_sha: &str,
) -> Result<Output, GitError> {
    let comparison = format!("{base_sha}..{candidate_sha}");
    Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("diff")
        .arg("--name-only")
        .arg(comparison)
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_scope_boundary() {
        let scopes = vec!["src/api".to_string(), "src/core/".to_string()];

        assert!(is_file_allowed("src/api/file.rs", &scopes));
        assert!(is_file_allowed("src/api", &scopes));
        assert!(is_file_allowed("src/core/mod.rs", &scopes));

        assert!(!is_file_allowed("src/api_backup/file.rs", &scopes));
        assert!(!is_file_allowed("src/core_engine/mod.rs", &scopes));
    }

    #[test]
    fn git_diff_uses_base_to_candidate_boundary() {
        let repo = create_test_repo("candidate-diff");
        let base_sha = git_current_head(&repo);
        commit_file(
            &repo,
            "src/allowed.rs",
            b"pub fn allowed() {}\n",
            "candidate",
        );
        let candidate_sha = git_current_head(&repo);

        let files = execute_git_diff(&repo, &base_sha, &candidate_sha).unwrap();

        assert_eq!(files, vec!["src/allowed.rs"]);

        let _ = fs::remove_dir_all(repo);
    }

    fn create_test_repo(label: &str) -> PathBuf {
        let repo = std::env::temp_dir().join(unique_test_name(label));
        fs::create_dir_all(repo.join("src")).unwrap();
        git_raw(["init", repo.to_string_lossy().as_ref()]);
        commit_file(&repo, "src/lib.rs", b"pub fn base() {}\n", "base");
        repo
    }

    fn commit_file(repo: &Path, path: &str, contents: &[u8], message: &str) {
        fs::write(repo.join(path), contents).unwrap();
        git(repo, ["add", "."]);
        git(
            repo,
            [
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                message,
            ],
        );
    }

    fn git_current_head(repo: &Path) -> String {
        String::from_utf8_lossy(&git(repo, ["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string()
    }

    fn git<const N: usize>(repo: &Path, args: [&str; N]) -> Output {
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap()
    }

    fn git_raw<const N: usize>(args: [&str; N]) -> Output {
        Command::new("git").args(args).output().unwrap()
    }

    fn unique_test_name(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("acceptability-engine-{label}-{nanos}")
    }
}
