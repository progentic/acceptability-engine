use crate::contract::Contract;
use crate::orchestrator::{execute_existing_run, state_machine::FinalDecision};
use crate::store::{update_run_status, with_connection, SharedConnection};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub const RUN_QUEUE_CAPACITY: usize = 64;

#[derive(Debug)]
pub struct RunWork {
    pub run_id: i64,
    pub contract: Contract,
    pub workspace: PathBuf,
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

pub fn spawn_run_worker(db: SharedConnection, receiver: RunQueueReceiver) -> RunWorker {
    let handle = tokio::spawn(async move {
        process_run_queue(db, receiver).await;
    });
    RunWorker { handle }
}

async fn process_run_queue(db: SharedConnection, mut receiver: RunQueueReceiver) {
    while let Some(work) = receiver.recv().await {
        process_run_work(db.clone(), work).await;
    }
}

async fn process_run_work(db: SharedConnection, work: RunWork) {
    let run_id = work.run_id;
    let result = execute_existing_run(db.clone(), work.run_id, work.contract, work.workspace).await;
    if should_mark_internal_failure(&result) {
        mark_run_failed_internal(&db, run_id).await;
    }
}

fn should_mark_internal_failure(
    result: &Result<FinalDecision, crate::error::OrchestratorError>,
) -> bool {
    result.is_err()
}

pub async fn mark_run_failed_internal(db: &SharedConnection, run_id: i64) {
    let _ = with_connection(db.clone(), move |conn| {
        update_run_status(conn, run_id, "FAILED_INTERNAL")
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_bounded_run_queue() {
        let (sender, mut receiver) = run_queue();
        let work = RunWork {
            run_id: 1,
            contract: Contract {
                id: "run-001".to_string(),
                repo_url: "https://github.com/progentic/acceptability-engine.git".to_string(),
                base_sha: "a9993e364706816aba3e25717850c26c9cd0d89d".to_string(),
                scopes: vec!["core/src".to_string()],
                requires_human_review: false,
            },
            workspace: PathBuf::from("/tmp/run-001"),
        };

        sender.send(work).await.unwrap();

        let queued = receiver.recv().await.unwrap();
        assert_eq!(queued.run_id, 1);
    }

    #[test]
    fn pending_human_review_is_successful_worker_completion() {
        let result = Ok(FinalDecision::PendingHumanReview);

        assert!(!should_mark_internal_failure(&result));
    }
}
