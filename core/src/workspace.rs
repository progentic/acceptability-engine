use crate::contract::Contract;
use crate::error::{GitError, ValidationError};
use crate::workspace_mode::WorkspaceMode;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkspaceMaterializationError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("git error: {0}")]
    Git(#[from] GitError),
    #[error("failed to prepare workspace path '{path}': {source}")]
    PrepareFailed {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to clean workspace path '{path}': {source}")]
    CleanupFailed {
        path: String,
        source: std::io::Error,
    },
    #[error("unsafe workspace path: {0}")]
    UnsafePath(String),
}

pub async fn materialize_workspace(
    workspace_root: PathBuf,
    workspace_mode: WorkspaceMode,
    contract: Contract,
) -> Result<PathBuf, WorkspaceMaterializationError> {
    tokio::task::spawn_blocking(move || {
        let request = WorkspaceRequest {
            workspace_root,
            workspace_mode,
            contract,
        };
        materialize_workspace_blocking(request)
    })
    .await
    .map_err(|source| WorkspaceMaterializationError::PrepareFailed {
        path: "workspace materialization task".to_string(),
        source: std::io::Error::other(source),
    })?
}

pub fn runtime_workspace_path(
    workspace_root: &Path,
    contract: &Contract,
) -> Result<PathBuf, ValidationError> {
    if !is_single_workspace_segment(&contract.id) {
        return Err(ValidationError::WorkspaceEscapesRoot);
    }
    let runtime_workspace = workspace_root.join(&contract.id);
    if !runtime_workspace.starts_with(workspace_root) {
        return Err(ValidationError::WorkspaceEscapesRoot);
    }
    Ok(runtime_workspace)
}

struct WorkspaceRequest {
    workspace_root: PathBuf,
    workspace_mode: WorkspaceMode,
    contract: Contract,
}

fn materialize_workspace_blocking(
    request: WorkspaceRequest,
) -> Result<PathBuf, WorkspaceMaterializationError> {
    match request.workspace_mode {
        WorkspaceMode::Local => materialize_local_workspace(&request),
        WorkspaceMode::Git => materialize_git_workspace(&request),
    }
}

fn materialize_local_workspace(
    request: &WorkspaceRequest,
) -> Result<PathBuf, WorkspaceMaterializationError> {
    runtime_workspace_path(&request.workspace_root, &request.contract)
        .map_err(WorkspaceMaterializationError::from)
}

fn materialize_git_workspace(
    request: &WorkspaceRequest,
) -> Result<PathBuf, WorkspaceMaterializationError> {
    let workspace_path = prepare_workspace_path(&request.workspace_root, &request.contract)?;
    clean_existing_workspace(&workspace_path)?;
    clone_repository(&request.contract.repo_url, &workspace_path)?;
    verify_origin(&workspace_path, &request.contract.repo_url)?;
    fetch_candidate_ref(&workspace_path, request.contract.candidate_ref.as_deref())?;
    verify_commit(&workspace_path, &request.contract.base_sha)?;
    verify_commit(&workspace_path, &request.contract.candidate_sha)?;
    verify_base_ancestor(
        &workspace_path,
        &request.contract.base_sha,
        &request.contract.candidate_sha,
    )?;
    detach_head(&workspace_path, &request.contract.candidate_sha)?;
    verify_detached_head(&workspace_path)?;
    verify_head_commit(&workspace_path, &request.contract.candidate_sha)?;
    Ok(workspace_path)
}

fn prepare_workspace_path(
    workspace_root: &Path,
    contract: &Contract,
) -> Result<PathBuf, WorkspaceMaterializationError> {
    validate_workspace_root(workspace_root)?;
    std::fs::create_dir_all(workspace_root).map_err(|source| {
        WorkspaceMaterializationError::PrepareFailed {
            path: display_path(workspace_root),
            source,
        }
    })?;
    let workspace_root = workspace_root.canonicalize().map_err(|source| {
        WorkspaceMaterializationError::PrepareFailed {
            path: display_path(workspace_root),
            source,
        }
    })?;
    let workspace_path = runtime_workspace_path(&workspace_root, contract)?;
    Ok(workspace_path)
}

fn clean_existing_workspace(path: &Path) -> Result<(), WorkspaceMaterializationError> {
    if !path.exists() {
        return Ok(());
    }
    reject_symlink_workspace(path)?;
    std::fs::remove_dir_all(path).map_err(|source| WorkspaceMaterializationError::CleanupFailed {
        path: display_path(path),
        source,
    })
}

fn clone_repository(repo_url: &str, workspace_path: &Path) -> Result<(), GitError> {
    let output = git_output([
        "clone",
        "--no-recurse-submodules",
        "--no-tags",
        repo_url,
        workspace_path.to_string_lossy().as_ref(),
    ])?;
    ensure_success(output)
}

fn verify_origin(workspace_path: &Path, expected_repo_url: &str) -> Result<(), GitError> {
    let output = git_in_workspace(workspace_path, ["remote", "get-url", "origin"])?;
    ensure_success_with_stdout(output, &normalize_repo_url(expected_repo_url))
}

fn fetch_candidate_ref(workspace_path: &Path, candidate_ref: Option<&str>) -> Result<(), GitError> {
    let Some(candidate_ref) = candidate_ref else {
        return Ok(());
    };
    let output = git_in_workspace(
        workspace_path,
        ["fetch", "--no-tags", "origin", candidate_ref],
    )?;
    ensure_success(output)
}

fn verify_commit(workspace_path: &Path, sha: &str) -> Result<(), GitError> {
    let commit_ref = format!("{sha}^{{commit}}");
    let output = git_in_workspace(workspace_path, ["cat-file", "-e", commit_ref.as_str()])?;
    if output.status.success() {
        return Ok(());
    }
    Err(GitError::CommitNotFound {
        path: display_path(workspace_path),
        sha: sha.to_string(),
    })
}

fn verify_base_ancestor(
    workspace_path: &Path,
    base_sha: &str,
    candidate_sha: &str,
) -> Result<(), GitError> {
    let output = git_in_workspace(
        workspace_path,
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

fn detach_head(workspace_path: &Path, candidate_sha: &str) -> Result<(), GitError> {
    let output = git_in_workspace(workspace_path, ["checkout", "--detach", candidate_sha])?;
    ensure_success(output)
}

fn verify_detached_head(workspace_path: &Path) -> Result<(), GitError> {
    let output = git_in_workspace(workspace_path, ["rev-parse", "--abbrev-ref", "HEAD"])?;
    ensure_success_with_stdout(output, "HEAD")
}

fn verify_head_commit(workspace_path: &Path, candidate_sha: &str) -> Result<(), GitError> {
    let output = git_in_workspace(workspace_path, ["rev-parse", "HEAD"])?;
    let head = normalize_repo_url(&output_stdout(&output));
    if output.status.success() && head == candidate_sha.to_ascii_lowercase() {
        return Ok(());
    }
    Err(GitError::HeadMismatch {
        head,
        candidate_sha: candidate_sha.to_ascii_lowercase(),
    })
}

fn validate_workspace_root(workspace_root: &Path) -> Result<(), WorkspaceMaterializationError> {
    if workspace_root.as_os_str().is_empty() {
        return Err(unsafe_path("workspace root must not be empty"));
    }
    if workspace_root == Path::new(".") {
        return Err(unsafe_path("workspace root must not be current directory"));
    }
    if workspace_root.parent().is_none() {
        return Err(unsafe_path("workspace root must not be filesystem root"));
    }
    reject_symlink_root(workspace_root)?;
    Ok(())
}

fn reject_symlink_root(path: &Path) -> Result<(), WorkspaceMaterializationError> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        return Err(unsafe_path("workspace root must not be a symlink"));
    }
    Ok(())
}

fn reject_symlink_workspace(path: &Path) -> Result<(), WorkspaceMaterializationError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|source| {
        WorkspaceMaterializationError::PrepareFailed {
            path: display_path(path),
            source,
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(unsafe_path("workspace path must not be a symlink"));
    }
    Ok(())
}

fn unsafe_path(message: &str) -> WorkspaceMaterializationError {
    WorkspaceMaterializationError::UnsafePath(message.to_string())
}

fn git_in_workspace<const N: usize>(path: &Path, args: [&str; N]) -> Result<Output, GitError> {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|source| GitError::ProcessExecutionFailed { source })
}

fn git_output<const N: usize>(args: [&str; N]) -> Result<Output, GitError> {
    Command::new("git")
        .args(args)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|source| GitError::ProcessExecutionFailed { source })
}

fn ensure_success(output: Output) -> Result<(), GitError> {
    if output.status.success() {
        return Ok(());
    }
    Err(command_failed(output))
}

fn ensure_success_with_stdout(output: Output, expected_stdout: &str) -> Result<(), GitError> {
    if output.status.success() && normalize_repo_url(&output_stdout(&output)) == expected_stdout {
        return Ok(());
    }
    Err(command_failed(output))
}

fn command_failed(output: Output) -> GitError {
    GitError::CommandFailed {
        code: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn output_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn normalize_repo_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

fn is_single_workspace_segment(id: &str) -> bool {
    let mut components = Path::new(id).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
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
    fn materializes_local_workspace_path() {
        let root = temp_path("local-root");
        let contract = contract_for_repo("https://github.com/progentic/acceptability-engine.git");

        let workspace = materialize_workspace_blocking(WorkspaceRequest {
            workspace_root: root.clone(),
            workspace_mode: WorkspaceMode::Local,
            contract,
        })
        .unwrap();

        assert_eq!(workspace, root.join("run-001"));
    }

    #[test]
    fn rejects_malicious_workspace_ids() {
        let root = temp_path("malicious-root");
        let mut contract =
            contract_for_repo("https://github.com/progentic/acceptability-engine.git");
        contract.id = "../escape".to_string();

        let result = runtime_workspace_path(&root, &contract);

        assert!(matches!(result, Err(ValidationError::WorkspaceEscapesRoot)));
    }

    #[test]
    fn rejects_empty_workspace_root() {
        let result = prepare_workspace_path(Path::new(""), &contract_for_repo("repo.git"));

        assert!(matches!(
            result,
            Err(WorkspaceMaterializationError::UnsafePath(_))
        ));
    }

    #[test]
    fn rejects_current_directory_workspace_root() {
        let result = prepare_workspace_path(Path::new("."), &contract_for_repo("repo.git"));

        assert!(matches!(
            result,
            Err(WorkspaceMaterializationError::UnsafePath(_))
        ));
    }

    #[test]
    fn rejects_filesystem_root_workspace_root() {
        let result = prepare_workspace_path(Path::new("/"), &contract_for_repo("repo.git"));

        assert!(matches!(
            result,
            Err(WorkspaceMaterializationError::UnsafePath(_))
        ));
    }

    #[test]
    fn malicious_workspace_ids_do_not_cleanup_existing_workspace() {
        let root = temp_path("malicious-cleanup-root");
        let protected = root.join("run-001");
        fs::create_dir_all(&protected).unwrap();
        fs::write(protected.join("keep.txt"), b"keep").unwrap();
        let mut contract = contract_for_repo("repo.git");
        contract.id = "../run-001".to_string();

        let result = materialize_git_workspace(&WorkspaceRequest {
            workspace_root: root.clone(),
            workspace_mode: WorkspaceMode::Git,
            contract,
        });

        assert!(result.is_err());
        assert!(protected.join("keep.txt").exists());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clones_checkout_and_verifies_origin() {
        let source = create_source_repo("source");
        let base_sha = git_current_head(&source);
        let root = temp_path("clone-root");
        let contract = contract_for_repo(&display_path(&source));

        let workspace = materialize_git_workspace(&WorkspaceRequest {
            workspace_root: root.clone(),
            workspace_mode: WorkspaceMode::Git,
            contract: Contract {
                base_sha: base_sha.clone(),
                candidate_sha: base_sha.clone(),
                ..contract
            },
        })
        .unwrap();

        assert!(workspace.join(".git").is_dir());
        assert_eq!(
            git_stdout(&workspace, ["rev-parse", "--abbrev-ref", "HEAD"]),
            "HEAD"
        );
        assert_eq!(git_stdout(&workspace, ["rev-parse", "HEAD"]), base_sha);
        assert_eq!(
            git_stdout(&workspace, ["remote", "get-url", "origin"]),
            display_path(&source)
        );

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[test]
    fn git_materialization_checks_out_candidate_sha() {
        let source = create_source_repo("candidate-source");
        let base_sha = git_current_head(&source);
        commit_source_change(&source, "pub fn candidate() {}\n", "candidate");
        let candidate_sha = git_current_head(&source);
        let root = temp_path("candidate-root");
        let contract = contract_for_repo(&display_path(&source));

        let workspace = materialize_git_workspace(&WorkspaceRequest {
            workspace_root: root.clone(),
            workspace_mode: WorkspaceMode::Git,
            contract: Contract {
                base_sha,
                candidate_sha: candidate_sha.clone(),
                ..contract
            },
        })
        .unwrap();

        assert_eq!(git_stdout(&workspace, ["rev-parse", "HEAD"]), candidate_sha);

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[test]
    fn cleans_existing_workspace_before_clone() {
        let source = create_source_repo("cleanup-source");
        let base_sha = git_current_head(&source);
        let root = temp_path("cleanup-root");
        let stale_workspace = root.join("run-001");
        fs::create_dir_all(&stale_workspace).unwrap();
        fs::write(stale_workspace.join("stale.txt"), b"stale").unwrap();

        let workspace = materialize_git_workspace(&WorkspaceRequest {
            workspace_root: root.clone(),
            workspace_mode: WorkspaceMode::Git,
            contract: Contract {
                base_sha: base_sha.clone(),
                candidate_sha: base_sha,
                ..contract_for_repo(&display_path(&source))
            },
        })
        .unwrap();

        assert!(!workspace.join("stale.txt").exists());
        assert!(workspace.join(".git").is_dir());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[test]
    fn verifies_normalized_origin_url() {
        let source = create_source_repo("normalized-origin-source");
        let root = temp_path("normalized-origin-root");
        let workspace = root.join("run-001");
        clone_repository(&display_path(&source), &workspace).unwrap();

        let result = verify_origin(&workspace, &format!("{}/", display_path(&source)));

        assert!(result.is_ok());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[test]
    fn rejects_wrong_origin_after_clone() {
        let source = create_source_repo("origin-source");
        let root = temp_path("origin-root");
        let workspace = root.join("run-001");
        clone_repository(&display_path(&source), &workspace).unwrap();

        let result = verify_origin(&workspace, "https://example.com/other.git");

        assert!(matches!(result, Err(GitError::CommandFailed { .. })));

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(source);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_workspace_root() {
        let target = temp_path("symlink-root-target");
        let link = temp_path("symlink-root-link");
        fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = prepare_workspace_path(&link, &contract_for_repo("repo.git"));

        assert!(matches!(
            result,
            Err(WorkspaceMaterializationError::UnsafePath(_))
        ));

        let _ = fs::remove_dir_all(target);
        let _ = fs::remove_file(link);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_workspace_before_cleanup() {
        let root = temp_path("symlink-workspace-root");
        let target = temp_path("symlink-workspace-target");
        let link = root.join("run-001");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("keep.txt"), b"keep").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = clean_existing_workspace(&link);

        assert!(matches!(
            result,
            Err(WorkspaceMaterializationError::UnsafePath(_))
        ));
        assert!(target.join("keep.txt").exists());

        let _ = fs::remove_file(link);
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(target);
    }

    fn contract_for_repo(repo_url: &str) -> Contract {
        Contract {
            id: "run-001".to_string(),
            repo_url: repo_url.to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_sha: "b9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_ref: None,
            scopes: vec!["src".to_string()],
            requires_human_review: false,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }

    fn create_source_repo(label: &str) -> PathBuf {
        let path = temp_path(label);
        fs::create_dir_all(path.join("src")).unwrap();
        git_raw(["init", path.to_string_lossy().as_ref()]);
        fs::write(path.join("src").join("lib.rs"), b"pub fn ok() {}\n").unwrap();
        git(&path, ["add", "."]);
        git(
            &path,
            [
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "initial",
            ],
        );
        path
    }

    fn commit_source_change(path: &Path, contents: &str, message: &str) {
        fs::write(path.join("src").join("candidate.rs"), contents).unwrap();
        git(path, ["add", "."]);
        git(
            path,
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

    fn git_current_head(path: &Path) -> String {
        git_stdout(path, ["rev-parse", "HEAD"])
    }

    fn git_stdout<const N: usize>(path: &Path, args: [&str; N]) -> String {
        let output = git(path, args);
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn git<const N: usize>(path: &Path, args: [&str; N]) -> Output {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn git_raw<const N: usize>(args: [&str; N]) {
        let output = Command::new("git").args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("acceptability-engine-{label}-{nanos}"))
    }
}
