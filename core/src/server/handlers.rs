use super::state::AppState;
use crate::contract::Contract;
use crate::error::StoreError;
use crate::orchestrator::run_contract;
use crate::orchestrator::state_machine::FinalDecision;
use crate::store::{fetch_run_summary, list_runs, RunListItem};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SubmitResponse {
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

pub async fn submit_contract(
    State(state): State<AppState>,
    Json(contract): Json<Contract>,
) -> Result<Json<SubmitResponse>, (StatusCode, String)> {
    if let Err(validation_error) = contract.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Contract structure validation failed: {}", validation_error),
        ));
    }

    let mut runtime_workspace = state.workspace_root.clone();
    runtime_workspace.push(&contract.id);

    match run_contract(state.db.clone(), contract, runtime_workspace).await {
        Ok(FinalDecision::Approve) => Ok(Json(SubmitResponse {
            status: "APPROVED".to_string(),
            reason: None,
        })),
        Ok(FinalDecision::Reject { reason }) => Ok(Json(SubmitResponse {
            status: "REJECTED".to_string(),
            reason: Some(reason),
        })),
        Err(orchestration_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Internal engine pipeline execution failure: {}",
                orchestration_error
            ),
        )),
    }
}

pub async fn get_run_status(
    Path(run_id): Path<i64>,
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
