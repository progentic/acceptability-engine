mod contract;
mod error;
mod gates;
mod orchestrator;
mod policy;
mod progress;
mod sandbox_profile;
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

    /// Delete artifact files older than this many days and keep SQLite evidence descriptors
    #[arg(long)]
    retention_days: Option<u32>,

    /// Report retention candidates without deleting artifact files
    #[arg(long)]
    retention_dry_run: bool,

    /// Emit a deterministic JSON replay report for an existing run id
    #[arg(long)]
    replay_run_id: Option<i64>,

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
    let sandbox_profile = match sandbox_profile::SandboxProfile::from_env() {
        Ok(profile) => profile,
        Err(error) => {
            eprintln!("PANIC: Invalid sandbox profile configuration: {}", error);
            std::process::exit(3);
        }
    };
    let _sandbox_model = sandbox_profile.model();

    let shared_db = match store::pooled_connection(&args.database) {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("PANIC: Database initialization failed: {}", error);
            std::process::exit(3);
        }
    };

    if let Some(retention_days) = args.retention_days {
        let artifact_store = store::ArtifactStore::new(args.artifact_root);
        match run_artifact_retention(
            shared_db,
            artifact_store,
            retention_days,
            args.retention_dry_run,
        )
        .await
        {
            Ok(summary) => {
                println!(
                    "RETENTION: scanned={} eligible={} planned={} deleted={} missing={}",
                    summary.scanned,
                    summary.eligible,
                    summary.planned,
                    summary.deleted,
                    summary.missing
                );
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("PANIC: Artifact retention failed: {}", error);
                std::process::exit(3);
            }
        }
    }

    if let Some(run_id) = args.replay_run_id {
        let artifact_store = store::ArtifactStore::new(args.artifact_root);
        match replay_run_report(shared_db, artifact_store, store::RunId::new(run_id)).await {
            Ok(Some(report)) => {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                std::process::exit(0);
            }
            Ok(None) => {
                eprintln!("PANIC: Replay run record not found for ID '{}'", run_id);
                std::process::exit(2);
            }
            Err(error) => {
                eprintln!("PANIC: Replay failed: {}", error);
                std::process::exit(3);
            }
        }
    }

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
        "Orchestrator driving contract validation sequence for: {} in {} workspace mode with {} sandbox profile",
        contract_payload.id,
        workspace_mode.as_str(),
        sandbox_profile.as_str()
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

async fn replay_run_report(
    db: store::SharedConnection,
    artifact_store: store::ArtifactStore,
    run_id: store::RunId,
) -> Result<Option<store::ReplayReport>, error::StoreError> {
    store::with_connection(db, move |conn| {
        store::replay_run(conn, &artifact_store, run_id)
    })
    .await
}

async fn run_artifact_retention(
    db: store::SharedConnection,
    artifact_store: store::ArtifactStore,
    retention_days: u32,
    dry_run: bool,
) -> Result<store::RetentionSummary, error::StoreError> {
    let cutoff_unix_seconds = retention_cutoff(retention_days)?;
    store::with_connection(db, move |conn| {
        store::apply_artifact_retention(
            conn,
            &artifact_store,
            store::RetentionPolicy {
                cutoff_unix_seconds,
                dry_run,
            },
        )
    })
    .await
}

fn retention_cutoff(retention_days: u32) -> Result<i64, error::StoreError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| {
            error::StoreError::InvalidParameter("system clock is before UNIX epoch".to_string())
        })?;
    let retention_seconds = i64::from(retention_days) * 86_400;
    Ok(now.as_secs() as i64 - retention_seconds)
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
