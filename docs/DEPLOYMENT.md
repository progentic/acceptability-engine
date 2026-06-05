# Deployment

## Runtime Endpoints

- `GET /health/live` reports process liveness.
- `GET /health/ready` checks SQLite readiness.
- `GET /metrics` exposes Prometheus text metrics.

Operational runbooks, alert definitions, and monitoring inventories are indexed
in `docs/OPERATIONS.md`.

Disaster recovery procedures are documented in
`docs/runbooks/disaster_recovery.md`.

Backup procedures are documented in `docs/runbooks/backup.md`.

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

Compose runs with the `development` sandbox profile. It drops Linux
capabilities, sets `no-new-privileges`, uses a read-only root filesystem, and
mounts `/tmp`, `/data`, `/artifacts`, and `/workspaces` as writable paths. This
is local hardening, not production containment.

Run artifact retention from the engine CLI against the same database and artifact root:

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90 --retention-dry-run
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90
```

Retention deletes only filesystem artifact bytes. SQLite evidence descriptors remain durable, and every planned, deleted, dry-run, or missing artifact outcome is recorded in `audit_events`.

Generate a replay report from the engine CLI against the same database and artifact root:

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --replay-run-id 123
```

Replay emits JSON to stdout. It is read-only and reports missing artifact bytes without recreating them.

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
- `AH_SANDBOX_PROFILE=kubernetes-restricted`
- `AH_API_KEYS=token|role|tenant|repo_prefixes`
- `RUST_LOG=core=info`

Use `local` when workspaces already exist under the configured workspace root. Use `git` when the worker should clone the contract repository into the per-run workspace before gate execution.

Optional limits:

- `AH_RATE_LIMIT_PER_MINUTE`
- `AH_RUN_QUOTA_PER_HOUR`

## Performance Baseline

Phase 36 performance validation is recorded in
`docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md`.

The current runtime model is intentionally conservative:

- HTTP submissions enter a bounded queue with capacity 64.
- One background worker processes gate runs sequentially.
- SQLite access uses a file-backed pooled connection model with blocking
  boundaries.
- Query indexes exist for production read paths.

The Phase 36 local smoke validated health, readiness, and metrics read
availability under small concurrent load. It is not a production throughput
claim. Operators should run target-environment load and soak tests before
raising queue capacity, worker count, or storage limits.

## Sandbox Profiles

`AH_SANDBOX_PROFILE=development` is the default for local development.

`AH_SANDBOX_PROFILE=kubernetes-restricted` is the documented production
containment baseline and requires a Linux container runtime. The Kubernetes
manifest enforces:

- non-root container execution
- no privilege escalation
- dropped Linux capabilities
- RuntimeDefault seccomp
- read-only root filesystem
- explicit writable mounts for `/data`, `/artifacts`, `/workspaces`, and `/tmp`
- CPU and memory requests and limits
- deny-all pod egress by default

The restricted Kubernetes profile is compatible with local workspace mode.
Git materialization that requires outbound clone access needs a deliberate
future egress policy and is still constrained by the release-critical
candidate-change acquisition gap.

## Metrics

The `/metrics` endpoint includes:

- `acceptability_engine_uptime_seconds`
- `acceptability_http_requests_total`
- `acceptability_http_responses_total`
- `acceptability_runs_submitted_total`
- `acceptability_security_denials_total`

Operators should treat metrics as operational observations. Durable admission
state remains in SQLite and filesystem evidence artifacts.
