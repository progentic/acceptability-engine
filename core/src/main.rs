mod contract;
mod error;
mod gates;
mod orchestrator;
mod server;
mod store;

use clap::Parser;
use contract::Contract;
use error::ContractLoadError;
use orchestrator::state_machine::FinalDecision;
use std::path::{Path, PathBuf};

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

    /// Bind and run as an HTTP web service on specified network port
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let raw_connection = match store::open(&args.database) {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("PANIC: Database initialization failed: {}", error);
            std::process::exit(3);
        }
    };

    if let Some(listening_port) = args.port {
        let shared_db = store::shared_connection(raw_connection);
        if let Err(server_error) =
            server::run_server(shared_db, args.workspace, listening_port).await
        {
            eprintln!(
                "PANIC: Network runtime engine bound crash: {}",
                server_error
            );
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

    println!(
        "Orchestrator driving contract validation sequence for: {}",
        contract_payload.id
    );

    let shared_db = store::shared_connection(raw_connection);
    match orchestrator::run_contract(shared_db, contract_payload, args.workspace).await {
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
