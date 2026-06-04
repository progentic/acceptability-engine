use super::security::{SecurityIdentity, SecurityRejection};
use super::state::AppState;
use super::worker::RunWork;
use crate::contract::Contract;
use crate::error::StoreError;
use crate::store::{
    create_queued_run_for_tenant, fetch_run_summary_for_tenant, finalize_human_review,
    list_attempt_gates_for_tenant, list_run_attempts_for_tenant, list_run_evidence_for_tenant,
    list_runs_for_tenant, record_audit_event, update_run_status, with_connection,
    AttemptGateDetail, AttemptId, AttemptSummary, AuditEvent, EvidenceBundleId,
    EvidenceBundleSummary, ReviewDecisionId, ReviewDecisionInput, ReviewDecisionKind, RunId,
    RunListItem,
};
use crate::workspace::runtime_workspace_path;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

const RESOURCE_NOT_VISIBLE_REASON: &str = "resource not found or not visible";

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub run_id: RunId,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewDecisionRequest {
    reason: String,
}

#[derive(Debug, Serialize)]
pub struct ReviewDecisionResponse {
    pub run_id: RunId,
    pub status: String,
    pub review_decision_id: ReviewDecisionId,
    pub evidence_bundle_id: EvidenceBundleId,
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

    validate_runtime_workspace(&state, &contract)?;

    let run_id = create_queued_run_record(&state, &contract, &identity).await?;
    enqueue_contract_run(&state, run_id, contract).await?;
    state.progress.publisher(run_id).queued();
    state.telemetry.record_submission();
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

fn validate_runtime_workspace(
    state: &AppState,
    contract: &Contract,
) -> Result<(), (StatusCode, String)> {
    runtime_workspace_path(&state.workspace_root, contract)
        .map(|_| ())
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Runtime workspace validation failed: {}", error),
            )
        })
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
) -> Result<(), (StatusCode, String)> {
    let work = RunWork {
        run_id,
        contract,
        workspace_root: state.workspace_root.clone(),
        workspace_mode: state.workspace_mode,
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
        Ok(None) => {
            audit_visibility_denied(&state, &identity, "runs.read", "run", run_id.get()).await;
            Err((
                StatusCode::NOT_FOUND,
                format!(
                    "Run telemetry file record not found for ID '{}'",
                    run_id.get()
                ),
            ))
        }
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
        Ok(None) => {
            audit_visibility_denied(&state, &identity, "runs.attempts.read", "run", run_id.get())
                .await;
            missing_record("Run", run_id.get())
        }
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
        Ok(None) => {
            audit_visibility_denied(
                &state,
                &identity,
                "attempts.gates.read",
                "attempt",
                attempt_id.get(),
            )
            .await;
            missing_record("Attempt", attempt_id.get())
        }
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
        Ok(None) => {
            audit_visibility_denied(&state, &identity, "runs.evidence.read", "run", run_id.get())
                .await;
            missing_record("Run", run_id.get())
        }
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

pub async fn approve_run_review_handler(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReviewDecisionRequest>,
) -> Result<Json<ReviewDecisionResponse>, (StatusCode, String)> {
    review_run(
        state,
        headers,
        RunId::new(run_id),
        request,
        ReviewDecisionKind::Approve,
        "runs.review.approve",
    )
    .await
}

pub async fn reject_run_review_handler(
    AxumPath(run_id): AxumPath<i64>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReviewDecisionRequest>,
) -> Result<Json<ReviewDecisionResponse>, (StatusCode, String)> {
    review_run(
        state,
        headers,
        RunId::new(run_id),
        request,
        ReviewDecisionKind::Reject,
        "runs.review.reject",
    )
    .await
}

async fn review_run(
    state: AppState,
    headers: HeaderMap,
    run_id: RunId,
    request: ReviewDecisionRequest,
    decision: ReviewDecisionKind,
    action: &'static str,
) -> Result<Json<ReviewDecisionResponse>, (StatusCode, String)> {
    let identity = authorize_review(&state, &headers, action).await?;
    validate_review_request(&request)?;
    ensure_review_run_exists(&state, &identity, run_id).await?;
    let recorded = finalize_review_record(&state, &identity, run_id, decision, &request).await?;
    audit_allowed(&state, &identity, action, "run", Some(run_id)).await;
    Ok(Json(ReviewDecisionResponse {
        run_id,
        status: decision.status().to_string(),
        review_decision_id: recorded.review_decision_id,
        evidence_bundle_id: recorded.evidence_bundle_id,
    }))
}

fn validate_review_request(request: &ReviewDecisionRequest) -> Result<(), (StatusCode, String)> {
    if !request.reason.trim().is_empty() {
        return Ok(());
    }
    Err((
        StatusCode::BAD_REQUEST,
        "review reason must not be empty".to_string(),
    ))
}

async fn ensure_review_run_exists(
    state: &AppState,
    identity: &SecurityIdentity,
    run_id: RunId,
) -> Result<(), (StatusCode, String)> {
    let tenant_id = identity.tenant_id.clone();
    match with_connection(state.db.clone(), move |conn| {
        fetch_run_summary_for_tenant(conn, run_id, &tenant_id)
    })
    .await
    {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            audit_visibility_denied(state, identity, "runs.review.read", "run", run_id.get()).await;
            Err((
                StatusCode::NOT_FOUND,
                format!("Run record not found for ID '{}'", run_id.get()),
            ))
        }
        Err(store_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query review run: {store_error}"),
        )),
    }
}

async fn finalize_review_record(
    state: &AppState,
    identity: &SecurityIdentity,
    run_id: RunId,
    decision: ReviewDecisionKind,
    request: &ReviewDecisionRequest,
) -> Result<crate::store::RecordedReviewDecision, (StatusCode, String)> {
    let tenant_id = identity.tenant_id.clone();
    let reviewer_actor = identity.actor.clone();
    let reviewer_role = identity.role.as_str().to_string();
    let reason = request.reason.clone();
    with_connection(state.db.clone(), move |conn| {
        finalize_human_review(
            conn,
            ReviewDecisionInput {
                run_id,
                tenant_id: &tenant_id,
                reviewer_actor: &reviewer_actor,
                reviewer_role: &reviewer_role,
                decision,
                reason: &reason,
            },
        )
    })
    .await
    .map_err(review_store_error)
}

fn review_store_error(store_error: StoreError) -> (StatusCode, String) {
    match store_error {
        StoreError::InvalidParameter(message) => (StatusCode::CONFLICT, message),
        error => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to finalize review decision: {error}"),
        ),
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

async fn authorize_review(
    state: &AppState,
    headers: &HeaderMap,
    action: &'static str,
) -> Result<SecurityIdentity, (StatusCode, String)> {
    match state.trust.authorize_review(headers).await {
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
    state.telemetry.record_security_denial();
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

async fn audit_visibility_denied(
    state: &AppState,
    identity: &SecurityIdentity,
    action: &'static str,
    resource_type: &'static str,
    resource_id: i64,
) {
    if !hidden_resource_exists(state, resource_type, resource_id).await {
        return;
    }
    audit_event(
        state,
        AuditEvent {
            tenant_id: identity.tenant_id.clone(),
            actor: identity.actor.clone(),
            role: identity.role.as_str().to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: Some(resource_id.to_string()),
            outcome: "DENIED".to_string(),
            reason: Some(RESOURCE_NOT_VISIBLE_REASON.to_string()),
        },
    )
    .await;
}

async fn hidden_resource_exists(
    state: &AppState,
    resource_type: &'static str,
    resource_id: i64,
) -> bool {
    with_connection(state.db.clone(), move |conn| match resource_type {
        "run" => run_exists(conn, resource_id),
        "attempt" => attempt_exists(conn, resource_id),
        _ => Ok(false),
    })
    .await
    .unwrap_or(false)
}

fn run_exists(conn: &rusqlite::Connection, run_id: i64) -> Result<bool, StoreError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM runs WHERE id = ?1)",
        rusqlite::params![run_id],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .map_err(|source| StoreError::QueryFailed { source })
}

fn attempt_exists(conn: &rusqlite::Connection, attempt_id: i64) -> Result<bool, StoreError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM attempts WHERE id = ?1)",
        rusqlite::params![attempt_id],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .map_err(|source| StoreError::QueryFailed { source })
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
        create_attempt, create_evidence_bundle, create_queued_run_for_tenant, create_run,
        list_run_attempts, open, record_gate_run, shared_connection, update_run_status,
    };
    use crate::workspace_mode::WorkspaceMode;
    use std::path::{Path, PathBuf};

    fn contract_with_id(id: &str) -> Contract {
        Contract {
            id: id.to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }

    #[test]
    fn resolves_runtime_workspace_under_root() {
        let root = Path::new("/tmp/acceptability-workspaces");
        let workspace = runtime_workspace_path(root, &contract_with_id("run-001")).unwrap();

        assert_eq!(workspace, root.join("run-001"));
    }

    #[test]
    fn rejects_runtime_workspace_that_escapes_root() {
        let root = Path::new("/tmp/acceptability-workspaces");
        let result = runtime_workspace_path(root, &contract_with_id("../escape"));

        assert!(result.is_err());
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
    async fn cross_tenant_run_status_is_hidden_and_audited() {
        let state = state_with_trust(
            "handler-tenant-run-status",
            super::super::security::TrustControls::api_key("secret|viewer|tenant-b|*"),
        );
        let (run_id, _) =
            create_attempt_test_data_for_tenant(&state, "tenant-run-status", "tenant-a").await;

        let error = get_run_status(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(audit_outcome_count(&state, "runs.read", "DENIED").await, 1);
    }

    #[tokio::test]
    async fn cross_tenant_attempt_gates_are_hidden_and_audited() {
        let state = state_with_trust(
            "handler-tenant-gates",
            super::super::security::TrustControls::api_key("secret|viewer|tenant-b|*"),
        );
        let (_, attempt_id) =
            create_attempt_test_data_for_tenant(&state, "tenant-gates", "tenant-a").await;

        let error = list_attempt_gates_handler(
            AxumPath(attempt_id.get()),
            State(state.clone()),
            auth_headers("secret"),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(
            audit_outcome_count(&state, "attempts.gates.read", "DENIED").await,
            1
        );
    }

    #[tokio::test]
    async fn cross_tenant_run_evidence_is_hidden_and_audited() {
        let state = state_with_trust(
            "handler-tenant-evidence",
            super::super::security::TrustControls::api_key("secret|viewer|tenant-b|*"),
        );
        let (run_id, _) =
            create_attempt_test_data_for_tenant(&state, "tenant-evidence", "tenant-a").await;

        let error = list_run_evidence_handler(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(
            audit_outcome_count(&state, "runs.evidence.read", "DENIED").await,
            1
        );
    }

    #[tokio::test]
    async fn cross_tenant_review_is_hidden_and_audited() {
        let state = state_with_trust(
            "handler-tenant-review",
            super::super::security::TrustControls::api_key("secret|reviewer|tenant-b|*"),
        );
        let run_id = create_pending_review_run(&state, "tenant-review", "tenant-a").await;

        let error = approve_run_review_handler(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
            Json(review_request("wrong tenant")),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(run_status(&state, run_id).await, "PENDING_HUMAN_REVIEW");
        assert_eq!(
            audit_outcome_count(&state, "runs.review.read", "DENIED").await,
            1
        );
    }

    #[tokio::test]
    async fn missing_authenticated_run_is_not_hidden_resource_denial() {
        let state = state_with_trust(
            "handler-missing-auth-run",
            super::super::security::TrustControls::api_key("secret|viewer|tenant-a|*"),
        );

        let error = get_run_status(AxumPath(999), State(state.clone()), auth_headers("secret"))
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(audit_outcome_count(&state, "runs.read", "DENIED").await, 0);
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

    #[tokio::test]
    async fn approves_pending_human_review_run() {
        let state = state_with_trust(
            "handler-review-approve",
            super::super::security::TrustControls::api_key("secret|reviewer|tenant-a|*"),
        );
        let run_id = create_pending_review_run(&state, "handler-review-approve", "tenant-a").await;

        let Json(response) = approve_run_review_handler(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
            Json(review_request("approved after inspection")),
        )
        .await
        .unwrap();
        let Json(evidence) = list_run_evidence_handler(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
        )
        .await
        .unwrap();

        assert_eq!(response.status, "APPROVED");
        assert_eq!(response.run_id, run_id);
        assert_eq!(audit_event_count(&state, "runs.review.approve").await, 1);
        assert!(evidence
            .iter()
            .any(|item| item.review_decision_id == Some(response.review_decision_id)));
    }

    #[tokio::test]
    async fn rejects_pending_human_review_run() {
        let state = state_with_trust(
            "handler-review-reject",
            super::super::security::TrustControls::api_key("secret|reviewer|tenant-a|*"),
        );
        let run_id = create_pending_review_run(&state, "handler-review-reject", "tenant-a").await;

        let Json(response) = reject_run_review_handler(
            AxumPath(run_id.get()),
            State(state),
            auth_headers("secret"),
            Json(review_request("evidence did not satisfy policy")),
        )
        .await
        .unwrap();

        assert_eq!(response.status, "REJECTED");
        assert_eq!(response.run_id, run_id);
    }

    #[tokio::test]
    async fn submitter_cannot_review_run() {
        let state = state_with_trust(
            "handler-review-auth",
            super::super::security::TrustControls::api_key("secret|submitter|tenant-a|*"),
        );
        let run_id = create_pending_review_run(&state, "handler-review-auth", "tenant-a").await;

        let error = approve_run_review_handler(
            AxumPath(run_id.get()),
            State(state.clone()),
            auth_headers("secret"),
            Json(review_request("not authorized")),
        )
        .await
        .unwrap_err();

        assert_eq!(error.0, StatusCode::FORBIDDEN);
        assert_eq!(audit_event_count(&state, "runs.review.approve").await, 1);
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
            telemetry: super::super::telemetry::MetricsState::new(),
            progress: crate::progress::ProgressHub::new(),
        }
    }

    fn headers() -> HeaderMap {
        HeaderMap::new()
    }

    fn auth_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", token.parse().unwrap());
        headers
    }

    fn review_request(reason: &str) -> ReviewDecisionRequest {
        ReviewDecisionRequest {
            reason: reason.to_string(),
        }
    }

    async fn create_pending_review_run(state: &AppState, id: &str, tenant_id: &str) -> RunId {
        let contract = contract_with_id(id);
        let tenant_id = tenant_id.to_string();
        with_connection(state.db.clone(), move |conn| {
            let run_id = create_queued_run_for_tenant(conn, &contract, &tenant_id)?;
            update_run_status(conn, run_id, "PENDING_HUMAN_REVIEW")?;
            Ok(run_id)
        })
        .await
        .unwrap()
    }

    async fn audit_event_count(state: &AppState, action: &str) -> i64 {
        let action = action.to_string();
        with_connection(state.db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM audit_events WHERE action = ?1",
                rusqlite::params![action],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn audit_outcome_count(state: &AppState, action: &str, outcome: &str) -> i64 {
        let action = action.to_string();
        let outcome = outcome.to_string();
        with_connection(state.db.clone(), move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM audit_events WHERE action = ?1 AND outcome = ?2",
                rusqlite::params![action, outcome],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::QueryFailed { source })
        })
        .await
        .unwrap()
    }

    async fn create_attempt_test_data(state: &AppState, id: &str) -> RunId {
        create_attempt_test_data_for_tenant(state, id, "local")
            .await
            .0
    }

    async fn create_attempt_test_data_for_tenant(
        state: &AppState,
        id: &str,
        tenant_id: &str,
    ) -> (RunId, AttemptId) {
        let contract = contract_with_id(id);
        let tenant_id = tenant_id.to_string();
        with_connection(state.db.clone(), move |conn| {
            let run_id = create_queued_run_for_tenant(conn, &contract, &tenant_id)?;
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
            Ok((run_id, attempt_id))
        })
        .await
        .unwrap()
    }

    async fn run_status(state: &AppState, run_id: RunId) -> String {
        with_connection(state.db.clone(), move |conn| {
            conn.query_row(
                "SELECT status FROM runs WHERE id = ?1",
                rusqlite::params![run_id.get()],
                |row| row.get(0),
            )
            .map_err(|source| StoreError::QueryFailed { source })
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
