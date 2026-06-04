use crate::gates::result::GateOutput;
use crate::store::{AttemptId, RunId};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 256;
const HISTORY_LIMIT: usize = 512;

#[derive(Clone)]
pub struct ProgressHub {
    inner: Arc<ProgressHubInner>,
}

struct ProgressHubInner {
    sender: broadcast::Sender<RunProgressEvent>,
    next_sequence: AtomicU64,
    history: Mutex<VecDeque<RunProgressEvent>>,
}

#[derive(Clone)]
pub struct ProgressPublisher {
    hub: Option<ProgressHub>,
    run_id: RunId,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunProgressEvent {
    pub sequence: u64,
    pub run_id: RunId,
    pub created_at: i64,
    #[serde(flatten)]
    pub event: RunProgressKind,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunProgressKind {
    Queued,
    Started,
    AttemptStarted {
        attempt_id: AttemptId,
    },
    GateStarted {
        gate_num: u8,
    },
    GateFinished {
        gate_num: u8,
        passed: bool,
        message: String,
    },
    Finalized {
        status: String,
    },
    FailedInternal {
        reason: String,
    },
}

impl ProgressHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            inner: Arc::new(ProgressHubInner {
                sender,
                next_sequence: AtomicU64::new(1),
                history: Mutex::new(VecDeque::with_capacity(HISTORY_LIMIT)),
            }),
        }
    }

    pub fn publisher(&self, run_id: RunId) -> ProgressPublisher {
        ProgressPublisher {
            hub: Some(self.clone()),
            run_id,
        }
    }

    pub fn publish(&self, run_id: RunId, event: RunProgressKind) -> RunProgressEvent {
        let progress_event = self.create_event(run_id, event);
        self.store_event(progress_event.clone());
        let _ = self.inner.sender.send(progress_event.clone());
        progress_event
    }

    pub fn replay(&self, run_id: RunId, after: u64) -> Vec<RunProgressEvent> {
        let history = self
            .inner
            .history
            .lock()
            .expect("progress history poisoned");
        history
            .iter()
            .filter(|event| event.run_id == run_id && event.sequence > after)
            .cloned()
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RunProgressEvent> {
        self.inner.sender.subscribe()
    }

    fn create_event(&self, run_id: RunId, event: RunProgressKind) -> RunProgressEvent {
        RunProgressEvent {
            sequence: self.inner.next_sequence.fetch_add(1, Ordering::Relaxed),
            run_id,
            created_at: current_unix_seconds(),
            event,
        }
    }

    fn store_event(&self, event: RunProgressEvent) {
        let mut history = self
            .inner
            .history
            .lock()
            .expect("progress history poisoned");
        if history.len() == HISTORY_LIMIT {
            history.pop_front();
        }
        history.push_back(event);
    }
}

impl Default for ProgressHub {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressPublisher {
    pub fn disabled() -> Self {
        Self {
            hub: None,
            run_id: RunId::new(0),
        }
    }

    pub fn queued(&self) {
        self.publish(RunProgressKind::Queued);
    }

    pub fn started(&self) {
        self.publish(RunProgressKind::Started);
    }

    pub fn attempt_started(&self, attempt_id: AttemptId) {
        self.publish(RunProgressKind::AttemptStarted { attempt_id });
    }

    pub fn gate_started(&self, gate_num: u8) {
        self.publish(RunProgressKind::GateStarted { gate_num });
    }

    pub fn gate_finished(&self, output: &GateOutput) {
        self.publish(RunProgressKind::GateFinished {
            gate_num: output.gate_num(),
            passed: output.passed(),
            message: output.message().to_string(),
        });
    }

    pub fn finalized(&self, status: &str) {
        self.publish(RunProgressKind::Finalized {
            status: status.to_string(),
        });
    }

    pub fn failed_internal(&self, reason: &str) {
        self.publish(RunProgressKind::FailedInternal {
            reason: reason.to_string(),
        });
    }

    fn publish(&self, event: RunProgressKind) {
        if let Some(hub) = &self.hub {
            hub.publish(self.run_id, event);
        }
    }
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_ordered_by_sequence() {
        let hub = ProgressHub::new();
        let run_id = RunId::new(7);

        let first = hub.publish(run_id, RunProgressKind::Queued);
        let second = hub.publish(run_id, RunProgressKind::Started);

        assert!(first.sequence < second.sequence);
    }

    #[test]
    fn replay_returns_events_after_sequence() {
        let hub = ProgressHub::new();
        let run_id = RunId::new(7);
        let other_run_id = RunId::new(8);

        let first = hub.publish(run_id, RunProgressKind::Queued);
        hub.publish(other_run_id, RunProgressKind::Queued);
        let second = hub.publish(run_id, RunProgressKind::Started);

        let replayed = hub.replay(run_id, first.sequence);

        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].sequence, second.sequence);
    }

    #[test]
    fn replay_is_bounded_to_recent_events() {
        let hub = ProgressHub::new();
        let run_id = RunId::new(7);

        for _ in 0..(HISTORY_LIMIT + 2) {
            hub.publish(run_id, RunProgressKind::Queued);
        }

        let replayed = hub.replay(run_id, 0);

        assert_eq!(replayed.len(), HISTORY_LIMIT);
        assert_eq!(replayed[0].sequence, 3);
    }
}
