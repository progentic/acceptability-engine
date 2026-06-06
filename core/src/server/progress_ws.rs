use super::security::{SecurityIdentity, SecurityRejection};
use super::state::AppState;
use crate::error::StoreError;
use crate::progress::RunProgressEvent;
use crate::store::{
    fetch_run_summary_for_tenant, record_audit_event, with_connection, AuditEvent, RunId,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, Query, State,
    },
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use tokio::sync::broadcast;

const PROGRESS_READ_ACTION: &str = "runs.progress.read";
const RESOURCE_NOT_VISIBLE_REASON: &str = "resource not found or not visible";

#[derive(Deserialize)]
pub struct ProgressQuery {
    after: Option<u64>,
}

pub async fn run_progress_handler(
    AxumPath(run_id): AxumPath<i64>,
    Query(query): Query<ProgressQuery>,
    State(state): State<AppState>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    let run_id = RunId::new(run_id);
    let identity = authorize_progress_read(&state, &headers).await?;
    ensure_progress_run_exists(&state, &identity, run_id).await?;
    audit_progress_read(&state, &identity, run_id).await;

    let after = query.after.unwrap_or_default();
    let receiver = state.progress.subscribe();
    let replay = state.progress.replay(run_id, after);
    Ok(websocket
        .on_upgrade(move |socket| stream_run_progress(socket, receiver, replay, run_id, after)))
}

async fn stream_run_progress(
    mut socket: WebSocket,
    mut receiver: broadcast::Receiver<RunProgressEvent>,
    replay: Vec<RunProgressEvent>,
    run_id: RunId,
    after: u64,
) {
    let Some(last_sent_sequence) = send_replay(&mut socket, replay, after).await else {
        return;
    };
    send_live_events(&mut socket, &mut receiver, run_id, last_sent_sequence).await;
}

async fn send_replay(
    socket: &mut WebSocket,
    replay: Vec<RunProgressEvent>,
    after: u64,
) -> Option<u64> {
    let mut last_sent_sequence = after;
    for event in replay {
        last_sent_sequence = event.sequence;
        if !send_event(socket, &event).await {
            return None;
        }
    }
    Some(last_sent_sequence)
}

async fn send_live_events(
    socket: &mut WebSocket,
    receiver: &mut broadcast::Receiver<RunProgressEvent>,
    run_id: RunId,
    mut last_sent_sequence: u64,
) {
    loop {
        let event = match receiver.recv().await {
            Ok(event) => event,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => return,
        };
        if event.run_id != run_id {
            continue;
        }
        if event.sequence <= last_sent_sequence {
            continue;
        }
        if !send_event(socket, &event).await {
            return;
        }
        last_sent_sequence = event.sequence;
    }
}

async fn send_event(socket: &mut WebSocket, event: &RunProgressEvent) -> bool {
    let Ok(payload) = serde_json::to_string(event) else {
        return false;
    };
    socket.send(Message::Text(payload)).await.is_ok()
}

async fn authorize_progress_read(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<SecurityIdentity, (StatusCode, String)> {
    match state.trust.authorize_read(headers).await {
        Ok(identity) => Ok(identity),
        Err(rejection) => reject_progress_request(state, rejection).await,
    }
}

async fn ensure_progress_run_exists(
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
            audit_progress_not_visible(state, identity, run_id).await;
            Err((
                StatusCode::NOT_FOUND,
                format!("Run record not found for ID '{}'", run_id.get()),
            ))
        }
        Err(store_error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query progress run: {store_error}"),
        )),
    }
}

async fn reject_progress_request<T>(
    state: &AppState,
    rejection: SecurityRejection,
) -> Result<T, (StatusCode, String)> {
    state.telemetry.record_security_denial();
    write_progress_audit(
        state,
        AuditEvent {
            tenant_id: rejection.tenant_id,
            actor: rejection.actor,
            role: rejection.role,
            action: PROGRESS_READ_ACTION.to_string(),
            resource_type: "request".to_string(),
            resource_id: None,
            outcome: "DENIED".to_string(),
            reason: Some(rejection.reason.clone()),
        },
    )
    .await;
    Err((rejection.status, rejection.reason))
}

async fn audit_progress_read(state: &AppState, identity: &SecurityIdentity, run_id: RunId) {
    write_progress_audit(
        state,
        AuditEvent {
            tenant_id: identity.tenant_id.clone(),
            actor: identity.actor.clone(),
            role: identity.role.as_str().to_string(),
            action: PROGRESS_READ_ACTION.to_string(),
            resource_type: "run".to_string(),
            resource_id: Some(run_id.get().to_string()),
            outcome: "ALLOWED".to_string(),
            reason: None,
        },
    )
    .await;
}

async fn audit_progress_not_visible(state: &AppState, identity: &SecurityIdentity, run_id: RunId) {
    if !hidden_progress_run_exists(state, run_id).await {
        return;
    }
    write_progress_audit(
        state,
        AuditEvent {
            tenant_id: identity.tenant_id.clone(),
            actor: identity.actor.clone(),
            role: identity.role.as_str().to_string(),
            action: PROGRESS_READ_ACTION.to_string(),
            resource_type: "run".to_string(),
            resource_id: Some(run_id.get().to_string()),
            outcome: "DENIED".to_string(),
            reason: Some(RESOURCE_NOT_VISIBLE_REASON.to_string()),
        },
    )
    .await;
}

async fn hidden_progress_run_exists(state: &AppState, run_id: RunId) -> bool {
    with_connection(state.db.clone(), move |conn| run_exists(conn, run_id))
        .await
        .unwrap_or(false)
}

fn run_exists(conn: &rusqlite::Connection, run_id: RunId) -> Result<bool, StoreError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM runs WHERE id = ?1)",
        rusqlite::params![run_id.get()],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .map_err(|source| StoreError::QueryFailed { source })
}

async fn write_progress_audit(state: &AppState, event: AuditEvent) {
    let _ = with_connection(state.db.clone(), move |conn| {
        record_audit_event(conn, &event)
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Contract;
    use crate::progress::{ProgressHub, RunProgressKind};
    use crate::server::security::TrustControls;
    use crate::server::telemetry::MetricsState;
    use crate::server::worker::run_queue;
    use crate::store::{create_queued_run_for_tenant, open, shared_connection};
    use crate::workspace_mode::WorkspaceMode;
    use axum::{routing::get, Router};
    use futures_util::StreamExt;
    use std::path::PathBuf;
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    async fn websocket_replays_and_streams_run_progress() {
        let state = test_state("progress-ws");
        let run_id = create_test_run(&state).await;
        let first = state.progress.publish(run_id, RunProgressKind::Queued);
        let replayed = state.progress.publish(run_id, RunProgressKind::Started);
        let app = progress_test_app(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!(
            "ws://{address}/runs/{}/progress?after={}",
            run_id.get(),
            first.sequence
        );
        let (mut websocket, _) = connect_async(url).await.unwrap();
        let replay_message = websocket.next().await.unwrap().unwrap();
        let replay_event = progress_value_from_message(replay_message);

        state.progress.publish(
            run_id,
            RunProgressKind::Finalized {
                status: "APPROVED".to_string(),
            },
        );
        let live_message = websocket.next().await.unwrap().unwrap();
        let live_event = progress_value_from_message(live_message);

        assert_eq!(event_sequence(&replay_event), replayed.sequence);
        assert!(event_sequence(&live_event) > event_sequence(&replay_event));
        assert_eq!(live_event["type"], "finalized");
        assert_eq!(live_event["status"], "APPROVED");

        server.abort();
    }

    #[tokio::test]
    async fn cross_tenant_progress_is_hidden_and_audited() {
        let state = test_state_with_trust(
            "progress-tenant",
            TrustControls::api_key("secret|viewer|tenant-b|*"),
        );
        let run_id = create_test_run_for_tenant(&state, "tenant-a").await;
        let identity = state
            .trust
            .authorize_read(&auth_headers("secret"))
            .await
            .unwrap();

        let error = ensure_progress_run_exists(&state, &identity, run_id)
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(
            audit_outcome_count(&state, PROGRESS_READ_ACTION, "DENIED").await,
            1
        );
    }

    #[tokio::test]
    async fn missing_progress_run_is_not_hidden_resource_denial() {
        let state = test_state_with_trust(
            "progress-missing",
            TrustControls::api_key("secret|viewer|tenant-a|*"),
        );
        let identity = state
            .trust
            .authorize_read(&auth_headers("secret"))
            .await
            .unwrap();

        let error = ensure_progress_run_exists(&state, &identity, RunId::new(999))
            .await
            .unwrap_err();

        assert_eq!(error.0, StatusCode::NOT_FOUND);
        assert_eq!(
            audit_outcome_count(&state, PROGRESS_READ_ACTION, "DENIED").await,
            0
        );
    }

    fn progress_value_from_message(
        message: tokio_tungstenite::tungstenite::Message,
    ) -> serde_json::Value {
        serde_json::from_str(message.to_text().unwrap()).unwrap()
    }

    fn event_sequence(event: &serde_json::Value) -> u64 {
        event["sequence"].as_u64().unwrap()
    }

    fn progress_test_app(state: AppState) -> Router {
        Router::new()
            .route("/runs/:id/progress", get(run_progress_handler))
            .with_state(state)
    }

    fn test_state(id: &str) -> AppState {
        test_state_with_trust(id, TrustControls::disabled())
    }

    fn test_state_with_trust(id: &str, trust: TrustControls) -> AppState {
        let conn = open(":memory:").unwrap();
        let db = shared_connection(conn);
        let (run_queue, _receiver) = run_queue();
        AppState {
            db,
            run_queue,
            workspace_root: PathBuf::from("/tmp/acceptability-workspaces").join(id),
            workspace_mode: WorkspaceMode::Local,
            trust,
            telemetry: MetricsState::new(),
            progress: ProgressHub::new(),
        }
    }

    async fn create_test_run(state: &AppState) -> RunId {
        create_test_run_for_tenant(state, "local").await
    }

    async fn create_test_run_for_tenant(state: &AppState, tenant_id: &str) -> RunId {
        let tenant_id = tenant_id.to_string();
        with_connection(state.db.clone(), move |conn| {
            create_queued_run_for_tenant(conn, &test_contract(), &tenant_id)
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

    fn auth_headers(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", token.parse().unwrap());
        headers
    }

    fn test_contract() -> Contract {
        Contract {
            id: "progress-run".to_string(),
            repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
            base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_sha: "b9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
            candidate_ref: None,
            scopes: vec!["core/src".to_string()],
            requires_human_review: false,
            admission_policy: crate::policy::AdmissionPolicy::default(),
        }
    }
}
