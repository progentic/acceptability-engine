use super::auth::SecurityIdentity;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const REQUEST_WINDOW: Duration = Duration::from_secs(60);
const SUBMISSION_WINDOW: Duration = Duration::from_secs(60 * 60);

#[derive(Debug)]
pub struct LimitRejection {
    pub status: StatusCode,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct LimitConfig {
    pub requests_per_minute: u32,
    pub submissions_per_hour: u32,
}

pub struct LimitState {
    requests: Mutex<HashMap<String, WindowCounter>>,
    submissions: Mutex<HashMap<String, WindowCounter>>,
    config: LimitConfig,
}

impl LimitState {
    pub fn new(config: LimitConfig) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            submissions: Mutex::new(HashMap::new()),
            config,
        }
    }

    pub async fn check_request(&self, identity: &SecurityIdentity) -> Result<(), LimitRejection> {
        check_window(
            &self.requests,
            identity_key(identity),
            REQUEST_WINDOW,
            self.config.requests_per_minute,
            "request rate limit exceeded",
        )
        .await
    }

    pub async fn check_submission(
        &self,
        identity: &SecurityIdentity,
    ) -> Result<(), LimitRejection> {
        check_window(
            &self.submissions,
            identity_key(identity),
            SUBMISSION_WINDOW,
            self.config.submissions_per_hour,
            "run submission quota exceeded",
        )
        .await
    }
}

async fn check_window(
    counters: &Mutex<HashMap<String, WindowCounter>>,
    key: String,
    window: Duration,
    limit: u32,
    reason: &str,
) -> Result<(), LimitRejection> {
    let mut counters = counters.lock().await;
    let counter = counters.entry(key).or_insert_with(WindowCounter::new);
    counter.reset_if_expired(window);
    if counter.count >= limit {
        return Err(rejection(reason));
    }
    counter.count += 1;
    Ok(())
}

fn identity_key(identity: &SecurityIdentity) -> String {
    format!("{}:{}", identity.tenant_id, identity.actor)
}

fn rejection(reason: &str) -> LimitRejection {
    LimitRejection {
        status: StatusCode::TOO_MANY_REQUESTS,
        reason: reason.to_string(),
    }
}

struct WindowCounter {
    started_at: Instant,
    count: u32,
}

impl WindowCounter {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            count: 0,
        }
    }

    fn reset_if_expired(&mut self, window: Duration) {
        if self.started_at.elapsed() < window {
            return;
        }
        self.started_at = Instant::now();
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::security::auth::Role;

    #[tokio::test]
    async fn rejects_requests_above_limit() {
        let limits = LimitState::new(LimitConfig {
            requests_per_minute: 1,
            submissions_per_hour: 1,
        });
        let identity = SecurityIdentity {
            tenant_id: "tenant-a".to_string(),
            actor: "actor-a".to_string(),
            role: Role::Viewer,
            repo_prefixes: vec!["*".to_string()],
        };

        limits.check_request(&identity).await.unwrap();
        let error = limits.check_request(&identity).await.unwrap_err();

        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    }
}
