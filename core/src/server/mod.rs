pub mod handlers;
pub mod state;
pub mod worker;

use crate::store::SharedConnection;
use axum::{
    routing::{get, post},
    Router,
};
use state::AppState;
use std::net::SocketAddr;
use std::path::PathBuf;
use worker::{run_queue, spawn_run_worker};

pub async fn run_server(
    db: SharedConnection,
    workspace_root: PathBuf,
    port: u16,
) -> Result<(), std::io::Error> {
    let (sender, receiver) = run_queue();
    let worker = spawn_run_worker(db.clone(), receiver);
    let state = AppState {
        db,
        run_queue: sender,
        workspace_root,
    };

    let app = Router::new()
        .route(
            "/runs",
            post(handlers::submit_contract).get(handlers::list_runs_handler),
        )
        .route("/runs/:id", get(handlers::get_run_status))
        .with_state(state);

    let target_address = SocketAddr::from(([0, 0, 0, 0], port));
    let tcp_listener = tokio::net::TcpListener::bind(&target_address).await?;

    println!(
        "HTTP Network Control Plane online at http://{}",
        target_address
    );
    supervise_server(tcp_listener, app, worker).await
}

async fn supervise_server(
    tcp_listener: tokio::net::TcpListener,
    app: Router,
    mut worker: worker::RunWorker,
) -> Result<(), std::io::Error> {
    tokio::select! {
        server_result = axum::serve(tcp_listener, app) => {
            worker.abort();
            server_result
        }
        worker_result = worker.wait() => worker_exit_result(worker_result),
    }
}

fn worker_exit_result(result: Result<(), tokio::task::JoinError>) -> Result<(), std::io::Error> {
    match result {
        Ok(()) => Err(std::io::Error::other(
            "run worker stopped before server shutdown",
        )),
        Err(error) => Err(std::io::Error::other(format!(
            "run worker task failed: {error}"
        ))),
    }
}
