use super::state::AppState;
use super::worker::RunWork;
use crate::contract::Contract;
use crate::error::{StoreError, ValidationError};
use crate::store::{
    create_queued_run, fetch_run_summary, list_runs, update_run_status, RunListItem,
};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Serialize)]
pub struct SubmitResponse {
    pub run_id: i64,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ListRunsQuery {
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_limit() -> u32 {
    50
}

fn resolve_runtime_workspace(
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

fn is_single_workspace_segment(id: &str) -> bool {
    let mut components = Path::new(id).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

pub async fn submit_contract(
    State(state): State<AppState>,
    Json(contract): Json<Contract>,
) -> Result<(StatusCode, Json<SubmitResponse>), (StatusCode, String)> {
    if let Err(validation_error) = contract.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Contract structure validation failed: {}", validation_error),
        ));
    }

    let runtime_workspace =
        resolve_runtime_workspace(&state.workspace_root, &contract).map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Runtime workspace validation failed: {}", error),
            )
        })?;

    let run_id = create_queued_run_record(&state, &contract).await?;
    enqueue_contract_run(&state, run_id, contract, runtime_workspace).await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(SubmitResponse {
            run_id,
            status: "QUEUED".to_string(),
            reason: None,
        }),
    ))
}

async fn create_queued_run_record(
    state: &AppState,
    contract: &Contract,
) -> Result<i64, (StatusCode, String)> {
    let database_guard = state.db.lock().await;
    create_queued_run(&database_guard, contract).map_err(|store_error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create queued run record: {}", store_error),
        )
    })
}

async fn enqueue_contract_run(
    state: &AppState,
    run_id: i64,
    contract: Contract,
    workspace: PathBuf,
) -> Result<(), (StatusCode, String)> {
    let work = RunWork {
        run_id,
        contract,
        workspace,
    };

    if state.run_queue.try_send(work).is_ok() {
        return Ok(());
    }

    mark_queued_run_failed(state, run_id).await;
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        "Run queue is unavailable".to_string(),
    ))
}

async fn mark_queued_run_failed(state: &AppState, run_id: i64) {
    let database_guard = state.db.lock().await;
    let _ = update_run_status(&database_guard, run_id, "FAILED_INTERNAL");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract_with_id(id: &str) -> Contract {
        Contract {
            id: id.to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
        }
    }

    #[test]
    fn resolves_runtime_workspace_under_root() {
        let root = Path::new("/tmp/acceptability-workspaces");
        let workspace = resolve_runtime_workspace(root, &contract_with_id("run-001")).unwrap();

        assert_eq!(workspace, root.join("run-001"));
    }

    #[test]
    fn rejects_runtime_workspace_that_escapes_root() {
        let root = Path::new("/tmp/acceptability-workspaces");
        let result = resolve_runtime_workspace(root, &contract_with_id("../escape"));

        assert!(matches!(result, Err(ValidationError::WorkspaceEscapesRoot)));
    }
}

pub async fn get_run_status(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
) -> Result<Json<crate::store::RunStatusSummary>, (StatusCode, String)> {
    let database_guard = state.db.lock().await;

    match fetch_run_summary(&database_guard, run_id) {
        Ok(Some(summary)) => Ok(Json(summary)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Run telemetry file record not found for ID '{}'", run_id),
        )),
        Err(store_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Failed to retrieve execution record from datastore: {}",
                store_error
            ),
        )),
    }
}

pub async fn list_runs_handler(
    State(state): State<AppState>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<Vec<RunListItem>>, (StatusCode, String)> {
    let database_guard = state.db.lock().await;

    match list_runs(
        &database_guard,
        query.status.as_deref(),
        query.limit,
        query.offset,
    ) {
        Ok(items) => Ok(Json(items)),
        Err(StoreError::InvalidParameter(msg)) => Err((StatusCode::BAD_REQUEST, msg)),
        Err(store_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query run list: {}", store_error),
        )),
    }
}
