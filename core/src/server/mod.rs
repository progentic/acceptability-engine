pub mod state;
pub mod handlers;

use axum::{routing::{get, post}, Router};
use state::AppState;
use std::net::SocketAddr;

pub async fn run_server(state: AppState, port: u16) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/runs", post(handlers::submit_contract).get(handlers::list_runs_handler))
        .route("/runs/:id", get(handlers::get_run_status))
        .with_state(state);

    let target_address = SocketAddr::from(([0, 0, 0, 0], port));
    let tcp_listener = tokio::net::TcpListener::bind(&target_address).await?;

    println!("HTTP Network Control Plane online at http://{}", target_address);
    axum::serve(tcp_listener, app).await?;
    Ok(())
}
