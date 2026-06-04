use super::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;

#[derive(Clone)]
pub struct MetricsState {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    started_at: Instant,
    requests_total: AtomicU64,
    responses_2xx_total: AtomicU64,
    responses_4xx_total: AtomicU64,
    responses_5xx_total: AtomicU64,
    responses_other_total: AtomicU64,
    submissions_total: AtomicU64,
    security_denials_total: AtomicU64,
}

impl MetricsState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner::new()),
        }
    }

    pub fn record_submission(&self) {
        self.inner.submissions_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_security_denial(&self) {
        self.inner
            .security_denials_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        let uptime_seconds = self.inner.started_at.elapsed().as_secs();
        format!(
            concat!(
                "acceptability_engine_uptime_seconds {}\n",
                "acceptability_http_requests_total {}\n",
                "acceptability_http_responses_total{{class=\"2xx\"}} {}\n",
                "acceptability_http_responses_total{{class=\"4xx\"}} {}\n",
                "acceptability_http_responses_total{{class=\"5xx\"}} {}\n",
                "acceptability_http_responses_total{{class=\"other\"}} {}\n",
                "acceptability_runs_submitted_total {}\n",
                "acceptability_security_denials_total {}\n"
            ),
            uptime_seconds,
            self.inner.requests_total.load(Ordering::Relaxed),
            self.inner.responses_2xx_total.load(Ordering::Relaxed),
            self.inner.responses_4xx_total.load(Ordering::Relaxed),
            self.inner.responses_5xx_total.load(Ordering::Relaxed),
            self.inner.responses_other_total.load(Ordering::Relaxed),
            self.inner.submissions_total.load(Ordering::Relaxed),
            self.inner.security_denials_total.load(Ordering::Relaxed),
        )
    }

    fn record_request(&self) {
        self.inner.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    fn record_response(&self, status: StatusCode) {
        response_counter(&self.inner, status).fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for MetricsState {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsInner {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            requests_total: AtomicU64::new(0),
            responses_2xx_total: AtomicU64::new(0),
            responses_4xx_total: AtomicU64::new(0),
            responses_5xx_total: AtomicU64::new(0),
            responses_other_total: AtomicU64::new(0),
            submissions_total: AtomicU64::new(0),
            security_denials_total: AtomicU64::new(0),
        }
    }
}

pub async fn record_http_request(
    State(metrics): State<MetricsState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let started_at = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    metrics.record_request();

    let response = next.run(request).await;
    let status = response.status();
    metrics.record_response(status);
    tracing::info!(
        method = %method,
        path = %path,
        status = status.as_u16(),
        duration_ms = started_at.elapsed().as_millis() as u64,
        "http request completed"
    );
    response
}

pub async fn metrics_handler(
    State(state): State<AppState>,
) -> ([(axum::http::HeaderName, &'static str); 1], String) {
    (
        [(CONTENT_TYPE, "text/plain; version=0.0.4")],
        state.telemetry.render_prometheus(),
    )
}

fn response_counter(inner: &MetricsInner, status: StatusCode) -> &AtomicU64 {
    if status.is_success() {
        return &inner.responses_2xx_total;
    }
    if status.is_client_error() {
        return &inner.responses_4xx_total;
    }
    if status.is_server_error() {
        return &inner.responses_5xx_total;
    }
    &inner.responses_other_total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_prometheus_metrics() {
        let metrics = MetricsState::new();
        metrics.record_submission();
        metrics.record_security_denial();

        let output = metrics.render_prometheus();

        assert!(output.contains("acceptability_runs_submitted_total 1"));
        assert!(output.contains("acceptability_security_denials_total 1"));
    }
}
