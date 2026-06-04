use super::security::TrustControls;
use super::telemetry::MetricsState;
use super::worker::RunQueueSender;
use crate::progress::ProgressHub;
use crate::store::SharedConnection;
use crate::workspace_mode::WorkspaceMode;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub db: SharedConnection,
    pub run_queue: RunQueueSender,
    pub workspace_root: PathBuf,
    pub workspace_mode: WorkspaceMode,
    pub trust: TrustControls,
    pub telemetry: MetricsState,
    pub progress: ProgressHub,
}
