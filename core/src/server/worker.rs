use crate::contract::Contract;
use crate::orchestrator::{execute_existing_run, state_machine::FinalDecision};
use crate::progress::ProgressHub;
use crate::store::{update_run_status, with_connection, ArtifactStore, RunId, SharedConnection};
use crate::workspace::materialize_workspace;
use crate::workspace_mode::WorkspaceMode;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub const RUN_QUEUE_CAPACITY: usize = 64;

#[derive(Debug)]
pub struct RunWork {
    pub run_id: RunId,
    pub contract: Contract,
    pub workspace_root: PathBuf,
    pub workspace_mode: WorkspaceMode,
}

pub type RunQueueSender = mpsc::Sender<RunWork>;
pub type RunQueueReceiver = mpsc::Receiver<RunWork>;

pub struct RunWorker {
    handle: JoinHandle<()>,
}

impl RunWorker {
    pub async fn wait(&mut self) -> Result<(), tokio::task::JoinError> {
        (&mut self.handle).await
    }

    pub fn abort(&self) {
        self.handle.abort();
    }
}

pub fn run_queue() -> (RunQueueSender, RunQueueReceiver) {
    mpsc::channel(RUN_QUEUE_CAPACITY)
}

pub fn spawn_run_worker(
    db: SharedConnection,
    artifact_store: ArtifactStore,
    progress: ProgressHub,
    receiver: RunQueueReceiver,
) -> RunWorker {
    let handle = tokio::spawn(async move {
        process_run_queue(db, artifact_store, progress, receiver).await;
    });
    RunWorker { handle }
}

async fn process_run_queue(
    db: SharedConnection,
    artifact_store: ArtifactStore,
    progress: ProgressHub,
    mut receiver: RunQueueReceiver,
) {
    while let Some(work) = receiver.recv().await {
        process_run_work(db.clone(), artifact_store.clone(), progress.clone(), work).await;
    }
}

async fn process_run_work(
    db: SharedConnection,
    artifact_store: ArtifactStore,
    progress: ProgressHub,
    work: RunWork,
) {
    let run_id = work.run_id;
    let workspace = match materialize_run_workspace(&work).await {
        Ok(workspace) => workspace,
        Err(error) => {
            publish_internal_failure(&progress, run_id);
            mark_run_failed_internal(&db, run_id).await;
            tracing::error!(run_id = run_id.get(), error = %error, "workspace materialization failed");
            return;
        }
    };
    let result = execute_existing_run(
        db.clone(),
        artifact_store,
        progress.publisher(run_id),
        work.run_id,
        work.contract,
        workspace,
    )
    .await;
    if should_mark_internal_failure(&result) {
        publish_internal_failure(&progress, run_id);
        mark_run_failed_internal(&db, run_id).await;
    }
}

async fn materialize_run_workspace(
    work: &RunWork,
) -> Result<PathBuf, crate::workspace::WorkspaceMaterializationError> {
    materialize_workspace(
        work.workspace_root.clone(),
        work.workspace_mode,
        work.contract.clone(),
    )
    .await
}

fn should_mark_internal_failure(
    result: &Result<FinalDecision, crate::error::OrchestratorError>,
) -> bool {
    result.is_err()
}

fn publish_internal_failure(progress: &ProgressHub, run_id: RunId) {
    progress
        .publisher(run_id)
        .failed_internal("engine error during run execution");
}

pub async fn mark_run_failed_internal(db: &SharedConnection, run_id: RunId) {
    let _ = with_connection(db.clone(), move |conn| {
        update_run_status(conn, run_id, "FAILED_INTERNAL")
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::{ProgressHub, RunProgressKind};

    #[tokio::test]
    async fn creates_bounded_run_queue() {
        let (sender, mut receiver) = run_queue();
        let work = RunWork {
            run_id: RunId::new(1),
            contract: Contract {
                id: "run-001".to_string(),
                repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
                base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
                scopes: vec!["core/src".to_string()],
                requires_human_review: false,
                admission_policy: crate::policy::AdmissionPolicy::default(),
            },
            workspace_root: PathBuf::from("/tmp"),
            workspace_mode: WorkspaceMode::Local,
        };

        sender.send(work).await.unwrap();

        let queued = receiver.recv().await.unwrap();
        assert_eq!(queued.run_id, RunId::new(1));
    }

    #[test]
    fn pending_human_review_is_successful_worker_completion() {
        let result = Ok(FinalDecision::PendingHumanReview);

        assert!(!should_mark_internal_failure(&result));
    }

    #[test]
    fn internal_failure_progress_is_published_once() {
        let progress = ProgressHub::new();
        let run_id = RunId::new(1);

        publish_internal_failure(&progress, run_id);

        let events = progress.replay(run_id, 0);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].event,
            RunProgressKind::FailedInternal { .. }
        ));
    }
}
