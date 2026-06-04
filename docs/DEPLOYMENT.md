# Deployment

## Runtime Endpoints

- `GET /health/live` reports process liveness.
- `GET /health/ready` checks SQLite readiness.
- `GET /metrics` exposes Prometheus text metrics.

## Container

Build the runtime image:

```bash
docker build -t acceptability-engine:local .
```

Run with Compose:

```bash
docker compose up --build
```

The container stores SQLite data in `/data`, evidence artifacts in `/artifacts`, and materialized workspaces in `/workspaces`.

Run artifact retention from the engine CLI against the same database and artifact root:

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90 --retention-dry-run
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90
```

Retention deletes only filesystem artifact bytes. SQLite evidence descriptors remain durable, and every planned, deleted, dry-run, or missing artifact outcome is recorded in `audit_events`.

## Kubernetes

Apply the deployment manifest:

```bash
kubectl apply -f deploy/kubernetes.yaml
```

Before production use, replace the `AH_API_KEYS` secret value with one or more entries:

```text
token|role|tenant|repo_prefixes
```

Use `;` between multiple keys. Use comma-separated repository prefixes inside one key.

Supported roles are `viewer`, `submitter`, `reviewer`, and `admin`.

## Required Environment

- `AH_WORKSPACE_MODE=local` or `AH_WORKSPACE_MODE=git`
- `AH_SECURITY_MODE=api-key`
- `AH_API_KEYS=token|role|tenant|repo_prefixes`
- `RUST_LOG=core=info`

Use `local` when workspaces already exist under the configured workspace root. Use `git` when the worker should clone the contract repository into the per-run workspace before gate execution.

Optional limits:

- `AH_RATE_LIMIT_PER_MINUTE`
- `AH_RUN_QUOTA_PER_HOUR`

## Metrics

The `/metrics` endpoint includes:

- `acceptability_engine_uptime_seconds`
- `acceptability_http_requests_total`
- `acceptability_http_responses_total`
- `acceptability_runs_submitted_total`
- `acceptability_security_denials_total`
