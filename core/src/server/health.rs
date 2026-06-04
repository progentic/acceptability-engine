use super::state::AppState;
use crate::store::{check_store_ready, with_connection};
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub async fn live_handler() -> Json<HealthResponse> {
    Json(health_response("ok"))
}

pub async fn ready_handler(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    match with_connection(state.db.clone(), check_store_ready).await {
        Ok(()) => Ok(Json(health_response("ready"))),
        Err(error) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("datastore readiness check failed: {error}"),
        )),
    }
}

fn health_response(status: &'static str) -> HealthResponse {
    HealthResponse {
        status,
        service: "acceptability-engine",
    }
}
