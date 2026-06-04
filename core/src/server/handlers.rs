use super::security::{SecurityIdentity, SecurityRejection};
use super::state::AppState;
use super::worker::RunWork;
use crate::contract::Contract;
use crate::error::{StoreError, ValidationError};
use crate::store::{
    create_queued_run_for_tenant, fetch_run_summary_for_tenant, list_attempt_gates_for_tenant,
    list_run_attempts_for_tenant, list_run_evidence_for_tenant, list_runs_for_tenant,
    record_audit_event, update_run_status, with_connection, AttemptGateDetail, AttemptId,
    AttemptSummary, AuditEvent, EvidenceBundleSummary, RunId, RunListItem,
};
use crate::workspace_mode::WorkspaceMode;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub run_id: RunId,
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
    workspace_mode: WorkspaceMode,
    contract: &Contract,
) -> Result<PathBuf, ValidationError> {
    match workspace_mode {
        WorkspaceMode::Local => resolve_local_runtime_workspace(workspace_root, contract),
    }
}

fn resolve_local_runtime_workspace(
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
    headers: HeaderMap,
    Json(contract): Json<Contract>,
) -> Result<(StatusCode, Json<SubmitResponse>), (StatusCode, String)> {
    let identity = authorize_submit(&state, &headers, &contract).await?;

    if let Err(validation_error) = contract.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Contract structure validation failed: {}", validation_error),
        ));
    }

    let runtime_workspace =
        resolve_runtime_workspace(&state.workspace_root, state.workspace_mode, &contract).map_err(
            |error| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Runtime workspace validation failed: {}", error),
                )
            },
        )?;

    let run_id = create_queued_run_record(&state, &contract, &identity).await?;
    enqueue_contract_run(&state, run_id, contract, runtime_workspace).await?;
    audit_allowed(&state, &identity, "runs.submit", "run", Some(run_id)).await;

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
    identity: &SecurityIdentity,
) -> Result<RunId, (StatusCode, String)> {
    let contract = contract.clone();
    let tenant_id = identity.tenant_id.clone();
    with_connection(state.db.clone(), move |conn| {
        create_queued_run_for_tenant(conn, &contract, &tenant_id)
    })
    .await
    .map_err(internal_store_error("Failed to create queued run record"))
}

async fn enqueue_contract_run(
    state: &AppState,
    run_id: RunId,
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

async fn mark_queued_run_failed(state: &AppState, run_id: RunId) {
    let _ = with_connection(state.db.clone(), move |conn| {
        update_run_status(conn, run_id, "FAILED_INTERNAL")
    })
    .await;
}

pub async fn get_run_status(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::store::RunStatusSummary>, (StatusCode, String)> {
    let run_id = RunId::new(run_id);
    let identity = authorize_read(&state, &headers, "runs.read").await?;
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        fetch_run_summary_for_tenant(conn, run_id, &tenant_id)
    })
    .await
    {
        Ok(Some(summary)) => {
            audit_allowed(&state, &identity, "runs.read", "run", Some(run_id)).await;
            Ok(Json(summary))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!(
                "Run telemetry file record not found for ID '{}'",
                run_id.get()
            ),
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

pub async fn list_run_attempts_handler(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AttemptSummary>>, (StatusCode, String)> {
    let run_id = RunId::new(run_id);
    let identity = authorize_read(&state, &headers, "runs.attempts.read").await?;
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        list_run_attempts_for_tenant(conn, run_id, &tenant_id)
    })
    .await
    {
        Ok(Some(attempts)) => {
            audit_allowed(&state, &identity, "runs.attempts.read", "run", Some(run_id)).await;
            Ok(Json(attempts))
        }
        Ok(None) => missing_record("Run", run_id.get()),
        Err(store_error) => store_query_error("Failed to query run attempts", store_error),
    }
}

pub async fn list_attempt_gates_handler(
    AxumPath(attempt_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AttemptGateDetail>>, (StatusCode, String)> {
    let attempt_id = AttemptId::new(attempt_id);
    let identity = authorize_read(&state, &headers, "attempts.gates.read").await?;
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        list_attempt_gates_for_tenant(conn, attempt_id, &tenant_id)
    })
    .await
    {
        Ok(Some(gates)) => {
            audit_allowed(
                &state,
                &identity,
                "attempts.gates.read",
                "attempt",
                Some(attempt_id),
            )
            .await;
            Ok(Json(gates))
        }
        Ok(None) => missing_record("Attempt", attempt_id.get()),
        Err(store_error) => store_query_error("Failed to query attempt gates", store_error),
    }
}

pub async fn list_run_evidence_handler(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<EvidenceBundleSummary>>, (StatusCode, String)> {
    let run_id = RunId::new(run_id);
    let identity = authorize_read(&state, &headers, "runs.evidence.read").await?;
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        list_run_evidence_for_tenant(conn, run_id, &tenant_id)
    })
    .await
    {
        Ok(Some(evidence)) => {
            audit_allowed(&state, &identity, "runs.evidence.read", "run", Some(run_id)).await;
            Ok(Json(evidence))
        }
        Ok(None) => missing_record("Run", run_id.get()),
        Err(store_error) => store_query_error("Failed to query run evidence", store_error),
    }
}

pub async fn list_runs_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<Vec<RunListItem>>, (StatusCode, String)> {
    let status_filter = query.status;
    let identity = authorize_read(&state, &headers, "runs.list").await?;
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        list_runs_for_tenant(
            conn,
            &tenant_id,
            status_filter.as_deref(),
            query.limit,
            query.offset,
        )
    })
    .await
    {
        Ok(items) => {
            audit_allowed(&state, &identity, "runs.list", "run", None::<RunId>).await;
            Ok(Json(items))
        }
        Err(StoreError::InvalidParameter(msg)) => Err((StatusCode::BAD_REQUEST, msg)),
        Err(store_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query run list: {}", store_error),
        )),
    }
}

async fn authorize_read(
    state: &AppState,
    headers: &HeaderMap,
    action: &'static str,
) -> Result<SecurityIdentity, (StatusCode, String)> {
    match state.trust.authorize_read(headers).await {
        Ok(identity) => Ok(identity),
        Err(rejection) => reject_request(state, rejection, action).await,
    }
}

async fn authorize_submit(
    state: &AppState,
    headers: &HeaderMap,
    contract: &Contract,
) -> Result<SecurityIdentity, (StatusCode, String)> {
    match state.trust.authorize_submit(headers, contract).await {
        Ok(identity) => Ok(identity),
        Err(rejection) => reject_request(state, rejection, "runs.submit").await,
    }
}

async fn reject_request<T>(
    state: &AppState,
    rejection: SecurityRejection,
    action: &'static str,
) -> Result<T, (StatusCode, String)> {
    audit_event(
        state,
        AuditEvent {
            tenant_id: rejection.tenant_id,
            actor: rejection.actor,
            role: rejection.role,
            action: action.to_string(),
            resource_type: "request".to_string(),
            resource_id: None,
            outcome: "DENIED".to_string(),
            reason: Some(rejection.reason.clone()),
        },
    )
    .await;
    Err((rejection.status, rejection.reason))
}

async fn audit_allowed<T>(
    state: &AppState,
    identity: &SecurityIdentity,
    action: &'static str,
    resource_type: &'static str,
    resource_id: Option<T>,
) where
    T: Into<i64>,
{
    audit_event(
        state,
        AuditEvent {
            tenant_id: identity.tenant_id.clone(),
            actor: identity.actor.clone(),
            role: identity.role.as_str().to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.map(|id| id.into().to_string()),
            outcome: "ALLOWED".to_string(),
            reason: None,
        },
    )
    .await;
}

async fn audit_event(state: &AppState, event: AuditEvent) {
    let _ = with_connection(state.db.clone(), move |conn| {
        record_audit_event(conn, &event)
    })
    .await;
}

fn internal_store_error(context: &'static str) -> impl FnOnce(StoreError) -> (StatusCode, String) {
    move |store_error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", context, store_error),
        )
    }
}

fn missing_record<T>(record_type: &str, id: i64) -> Result<Json<T>, (StatusCode, String)> {
    Err((
        StatusCode::NOT_FOUND,
        format!("{record_type} record not found for ID '{id}'"),
    ))
}

fn store_query_error<T>(
    context: &'static str,
    store_error: StoreError,
) -> Result<Json<T>, (StatusCode, String)> {
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("{context}: {store_error}"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{
        create_attempt, create_evidence_bundle, create_run, list_run_attempts, open,
        record_gate_run, shared_connection,
    };

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
        let workspace =
            resolve_runtime_workspace(root, WorkspaceMode::Local, &contract_with_id("run-001"))
                .unwrap();

        assert_eq!(workspace, root.join("run-001"));
    }

    #[test]
    fn rejects_runtime_workspace_that_escapes_root() {
        let root = Path::new("/tmp/acceptability-workspaces");
        let result =
            resolve_runtime_workspace(root, WorkspaceMode::Local, &contract_with_id("../escape"));

        assert!(matches!(result, Err(ValidationError::WorkspaceEscapesRoot)));
    }

    #[tokio::test]
    async fn lists_run_attempts_for_http_clients() {
        let state = state_with_seeded_run("handler-attempts");
        let run_id = create_attempt_test_data(&state, "handler-attempts").await;

        let Json(attempts) =
            list_run_attempts_handler(AxumPath(run_id.get()), State(state), headers())
                .await
                .unwrap();

        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].run_id, run_id);
        assert_eq!(attempts[0].attempt_number, 1);
    }

    #[tokio::test]
    async fn lists_attempt_gates_for_http_clients() {
        let state = state_with_seeded_run("handler-gates");
        let run_id = create_attempt_test_data(&state, "handler-gates").await;
        let attempt_id = latest_attempt_id(&state, run_id).await;

        let Json(gates) =
            list_attempt_gates_handler(AxumPath(attempt_id.get()), State(state), headers())
                .await
                .unwrap();

        assert_eq!(gates.len(), 1);
        assert_eq!(gates[0].attempt_id, attempt_id);
        assert_eq!(gates[0].message, "contract valid");
    }

    #[tokio::test]
    async fn lists_run_evidence_for_http_clients() {
        let state = state_with_seeded_run("handler-evidence");
        let run_id = create_attempt_test_data(&state, "handler-evidence").await;

        let Json(evidence) =
            list_run_evidence_handler(AxumPath(run_id.get()), State(state), headers())
                .await
                .unwrap();

        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].summary, "gate evidence captured");
    }

    #[tokio::test]
    async fn missing_attempt_gates_return_not_found() {
        let state = state_with_seeded_run("handler-missing-attempt");

        let error = list_attempt_gates_handler(AxumPath(999), State(state), headers())
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn missing_run_attempts_return_not_found() {
        let state = state_with_seeded_run("handler-missing-run-attempts");

        let error = list_run_attempts_handler(AxumPath(999), State(state), headers())
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn missing_run_evidence_returns_not_found() {
        let state = state_with_seeded_run("handler-missing-run-evidence");

        let error = list_run_evidence_handler(AxumPath(999), State(state), headers())
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn existing_parent_without_children_returns_empty_lists() {
        let state = state_with_seeded_run("handler-empty-children");
        let empty_run_id = create_empty_run(&state, "handler-empty-run").await;
        let attempt_id = create_empty_attempt(&state, "handler-empty-attempt").await;

        let Json(attempts) = list_run_attempts_handler(
            AxumPath(empty_run_id.get()),
            State(state.clone()),
            headers(),
        )
        .await
        .unwrap();
        let Json(evidence) = list_run_evidence_handler(
            AxumPath(empty_run_id.get()),
            State(state.clone()),
            headers(),
        )
        .await
        .unwrap();
        let Json(gates) =
            list_attempt_gates_handler(AxumPath(attempt_id.get()), State(state), headers())
                .await
                .unwrap();

        assert!(attempts.is_empty());
        assert!(evidence.is_empty());
        assert!(gates.is_empty());
    }

    #[tokio::test]
    async fn submit_requires_api_key_when_security_is_enabled() {
        let state = state_with_trust(
            "handler-auth-required",
            super::super::security::TrustControls::api_key("secret|submitter|tenant-a|*"),
        );

        let error = submit_contract(
            State(state),
            headers(),
            Json(contract_with_id("secure-run")),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    fn state_with_seeded_run(id: &str) -> AppState {
        state_with_trust(id, super::super::security::TrustControls::disabled())
    }

    fn state_with_trust(id: &str, trust: super::super::security::TrustControls) -> AppState {
        let conn = open(":memory:").unwrap();
        let db = shared_connection(conn);
        let (run_queue, _receiver) = super::super::worker::run_queue();
        AppState {
            db,
            run_queue,
            workspace_root: PathBuf::from("/tmp/acceptability-workspaces").join(id),
            workspace_mode: WorkspaceMode::Local,
            trust,
        }
    }

    fn headers() -> HeaderMap {
        HeaderMap::new()
    }

    async fn create_attempt_test_data(state: &AppState, id: &str) -> RunId {
        let contract = contract_with_id(id);
        with_connection(state.db.clone(), move |conn| {
            let run_id = create_run(conn, &contract)?;
            let attempt_id = create_attempt(conn, run_id)?;
            let gate_run_id = record_gate_run(
                conn,
                attempt_id,
                &crate::gates::result::GateOutput::Simple(crate::gates::result::GateResult::pass(
                    1,
                    "contract valid",
                )),
            )?;
            create_evidence_bundle(
                conn,
                run_id,
                Some(attempt_id),
                Some(gate_run_id),
                "gate evidence captured",
            )?;
            Ok(run_id)
        })
        .await
        .unwrap()
    }

    async fn latest_attempt_id(state: &AppState, run_id: RunId) -> AttemptId {
        with_connection(state.db.clone(), move |conn| {
            Ok(list_run_attempts(conn, run_id)?.unwrap()[0].attempt_id)
        })
        .await
        .unwrap()
    }

    async fn create_empty_run(state: &AppState, id: &str) -> RunId {
        let contract = contract_with_id(id);
        with_connection(state.db.clone(), move |conn| create_run(conn, &contract))
            .await
            .unwrap()
    }

    async fn create_empty_attempt(state: &AppState, id: &str) -> AttemptId {
        let contract = contract_with_id(id);
        with_connection(state.db.clone(), move |conn| {
            let run_id = create_run(conn, &contract)?;
            create_attempt(conn, run_id)
        })
        .await
        .unwrap()
    }
}
