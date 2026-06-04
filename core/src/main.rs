mod contract;
mod error;
mod gates;
mod orchestrator;
mod progress;
mod server;
mod store;
mod workspace;
mod workspace_mode;

use clap::Parser;
use contract::Contract;
use error::ContractLoadError;
use orchestrator::state_machine::FinalDecision;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "accessibility-engine")]
#[command(about = "7-gate contract validation engine", long_about = None)]
struct Args {
    /// Path to contract.json file (Required for single-shot execution mode)
    #[arg(short, long)]
    contract: Option<PathBuf>,

    /// Path to workspace directory root
    #[arg(short, long)]
    workspace: PathBuf,

    /// Path to SQLite evidence database
    #[arg(short, long, default_value = "evidence.db")]
    database: String,

    /// Path to filesystem evidence artifact root
    #[arg(long, default_value = "artifacts")]
    artifact_root: PathBuf,

    /// Bind and run as an HTTP web service on specified network port
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    if let Some(exit_code) = run_sandbox_runner() {
        std::process::exit(exit_code);
    }
    setup_tracing();

    let args = Args::parse();
    let workspace_mode = match workspace_mode::WorkspaceMode::from_env() {
        Ok(mode) => mode,
        Err(error) => {
            eprintln!("PANIC: Invalid workspace mode configuration: {}", error);
            std::process::exit(3);
        }
    };

    let shared_db = match store::pooled_connection(&args.database) {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("PANIC: Database initialization failed: {}", error);
            std::process::exit(3);
        }
    };

    if let Some(listening_port) = args.port {
        if let Err(server_error) = server::run_server(
            shared_db,
            args.workspace,
            args.artifact_root,
            workspace_mode,
            listening_port,
        )
        .await
        {
            tracing::error!(error = %server_error, "network runtime engine crashed");
            std::process::exit(3);
        }
        return;
    }

    let contract_path = match &args.contract {
        Some(path) => path,
        None => {
            eprintln!("PANIC: Execution target contract file parameter (--contract) is missing.");
            std::process::exit(3);
        }
    };

    let contract_payload = match read_contract(contract_path) {
        Ok(contract) => contract,
        Err(error) => {
            eprintln!("PANIC: Failed to securely load contract: {}", error);
            std::process::exit(3);
        }
    };
    if let Err(error) = contract_payload.validate() {
        eprintln!("PANIC: Contract structure validation failed: {}", error);
        std::process::exit(3);
    }

    println!(
        "Orchestrator driving contract validation sequence for: {} in {} workspace mode",
        contract_payload.id,
        workspace_mode.as_str()
    );

    let artifact_store = store::ArtifactStore::new(args.artifact_root);
    let workspace = match workspace::materialize_workspace(
        args.workspace,
        workspace_mode,
        contract_payload.clone(),
    )
    .await
    {
        Ok(path) => path,
        Err(error) => {
            eprintln!("PANIC: Workspace materialization failed: {}", error);
            std::process::exit(3);
        }
    };
    match orchestrator::run_contract(shared_db, artifact_store, contract_payload, workspace).await {
        Ok(FinalDecision::Approve) => {
            println!("SUCCESS: Contract evaluation approved.");
            std::process::exit(0);
        }
        Ok(FinalDecision::PendingHumanReview) => {
            println!("PENDING: Contract evaluation requires human review.");
            std::process::exit(0);
        }
        Ok(FinalDecision::Reject { reason }) => {
            println!(
                "REJECTED: Contract validation failed checks. Reason: {}",
                reason
            );
            std::process::exit(2);
        }
        Err(error) => {
            eprintln!("PANIC: Critical runtime execution failure: {}", error);
            std::process::exit(3);
        }
    }
}

fn setup_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("core=info,tower_http=info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn run_sandbox_runner() -> Option<i32> {
    let mut args = std::env::args_os();
    let _binary = args.next();
    if args.next().as_deref() != Some(std::ffi::OsStr::new(gates::sandbox_runner::RUNNER_FLAG)) {
        return None;
    }
    Some(gates::sandbox_runner::run_from_args(args))
}

fn read_contract(path: &Path) -> Result<Contract, ContractLoadError> {
    let file_contents =
        std::fs::read_to_string(path).map_err(|source| ContractLoadError::ReadFailed {
            path: path.to_string_lossy().into_owned(),
            source,
        })?;

    let contract = serde_json::from_str(&file_contents)
        .map_err(|source| ContractLoadError::ParseFailed { source })?;

    Ok(contract)
}
