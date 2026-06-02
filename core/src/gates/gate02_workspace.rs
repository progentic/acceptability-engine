use crate::error::git::GitError;
use crate::error::GateError;
use crate::gates::result::GateResult;
use crate::orchestrator::state_machine::Run;
use std::fs;
use std::path::Path;

pub async fn run(run: &Run) -> Result<GateResult, GateError> {
    let workspace_path = run.workspace.clone();
    let base_sha = run.contract.base_sha.clone();

    tokio::task::spawn_blocking(move || {
        ensure_workspace_exists(&workspace_path)?;
        validate_base_sha(&base_sha)?;
        Ok(())
    })
    .await
    .map_err(|source| GateError::ExecutorJoinFailed { source })??;

    Ok(GateResult::pass(2, "Workspace target verification successful"))
}

fn ensure_workspace_exists(path: &Path) -> Result<(), GitError> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|source| GitError::WorkspaceCreationFailed {
        path: path.to_string_lossy().into_owned(),
        source,
    })
}

fn validate_base_sha(sha: &str) -> Result<(), GitError> {
    if sha.trim().is_empty() {
        return Err(GitError::EmptyBaseSha);
    }
    Ok(())
}
