use super::worker::RunQueueSender;
use crate::orchestrator::SharedConnection;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub db: SharedConnection,
    pub run_queue: RunQueueSender,
    pub workspace_root: PathBuf,
}
