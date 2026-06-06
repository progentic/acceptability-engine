# Architecture

## Purpose

The Acceptability Review Engine is a gatekeeper for proposed code changes.

The concrete object that enters the system is a contract. A contract names a repository, a base commit, a candidate commit, the allowed change scopes, the admission policy, and whether a human must review the result. The engine turns that contract into a run. A run creates one or more attempts. Each attempt executes a fixed sequence of gates. The system records the result, the gate telemetry, and the evidence artifacts.

The problem this solves is uncontrolled code admission. Generated code, agent-produced patches, and human-submitted changes can be useful, but they are not trusted merely because they were produced. The engine makes acceptance explicit. A change is admissible only after the required checks pass and any required human review is completed.

## Authority model

Rust is the authority boundary.

Rust owns contract validation, workspace selection, run state, gate execution, decision logic, evidence persistence, security enforcement, and API behavior.

TypeScript is an operator interface. It may display state and submit contracts through the HTTP API. It must not create authoritative decisions outside the Rust API.

SQLite is the durable evidence store. It records contracts, runs, attempts, gate runs, policy evaluations, final decisions, evidence bundle descriptors, and audit events.

The filesystem artifact store holds larger evidence payloads. SQLite stores descriptors for those artifacts, including storage URI, hash, byte length, content type, label, and summary.

External tools are evidence producers. Cargo, Git, cargo-deny, and cargo-audit may produce gate output, but they do not decide final admissibility by themselves. Rust interprets the results and records the decision.

## System layers

The system has six active layers.

### 1. HTTP and CLI entry points

The CLI accepts a contract file, a workspace root, a SQLite database path, and an artifact root.

The HTTP server exposes the control plane:

- `GET /health/live`
- `GET /health/ready`
- `GET /metrics`
- `POST /runs`
- `GET /runs`
- `GET /runs/:id`
- `GET /runs/:id/attempts`
- `GET /runs/:id/evidence`
- `GET /runs/:id/progress` as a WebSocket stream
- `POST /runs/:id/review/approve`
- `POST /runs/:id/review/reject`
- `GET /attempts/:id/gates`

`POST /runs` creates a queued run. It does not execute the full gate sequence inside the request lifecycle.

### 2. Trust controls

HTTP access is mediated by trust controls.

The default local development mode is disabled security. Production deployment must use API-key mode.

API-key mode accepts entries shaped as:

```text
token|role|tenant|repo_prefixes
```

The role controls read, submit, and review authority. The tenant scopes run and evidence access. The repository prefixes constrain which repositories a submitter may target.

Security denials are recorded as audit events. Accepted reads, submissions, and review decisions are also audited.

Authenticated attempts to read or review resources outside the caller tenant return an opaque not-found response. The server records a denied audit event with the caller identity and does not reveal whether the target exists for another tenant.

### 3. Run queue and worker

The HTTP server owns a bounded run queue. A submitted contract is converted into queued work and sent to the worker. The worker marks the run running, creates an attempt, executes the gates, and finalizes the run.

The queue protects the server from unbounded submission pressure. The worker keeps long-running validation outside request handling.

### 4. Orchestrator

The orchestrator is the lifecycle coordinator.

Its sequence is:

```text
create or receive run id
mark run RUNNING
create attempt
build run context
execute gates sequentially
derive final decision
write gate records
write evidence descriptors
update attempt status
update run status
write final decision when terminal
```

The final decision is one of:

- `APPROVED`
- `REJECTED`
- `PENDING_HUMAN_REVIEW`

Infrastructure failure is not the same as rejection. Gate runner infrastructure errors mark the attempt `ERROR` and the run `FAILED_INTERNAL`.

Human review is a separate authority boundary. When a run is `PENDING_HUMAN_REVIEW`, a reviewer may approve or reject it through the Rust API. The review transaction records the review decision, links evidence to the review record, updates the run status, and writes the final decision.

### 5. Gate runner

The gate runner is sequential and fail-fast. Later gates do not run after an earlier gate fails.

The active gate order is:

1. Contract validation
2. Local workspace verification
3. Change boundary check
4. Formatting check
5. Static analysis
6. Build
7. Test execution
8. Supply-chain checks

Gate 1 validates the contract shape.

Gate 2 verifies that the selected local workspace exists, is a directory, is a Git work tree, and contains the requested base commit.

Gate 3 compares `base_sha..candidate_sha` and rejects changed files outside the contract scopes.

Gate 4 runs `cargo fmt -- --check`.

Gate 5 runs `cargo clippy -- -D warnings`.

Gate 6 runs `cargo build`.

Gate 7 runs `cargo test -- -Z unstable-options --format json` and records parsed test metrics.

Gate 8 runs `cargo deny check` and then `cargo audit`.

### 6. Evidence store

The evidence model has these durable identities:

- Contract
- Run
- Attempt
- Gate run
- Policy evaluation
- Review decision
- Evidence bundle
- Final decision
- Audit event

A run may have multiple attempts. An attempt owns the gate run records for that execution. Evidence bundles may link to a run, an attempt, and a gate run.

Human-review evidence links to the review decision that produced it.

Admission policy evaluation runs after gate execution and before human-review suspension. The policy evaluates gate outputs against the contract policy, records a policy trace, and may reject a run before human review can be requested. Human review may suspend only a policy-passing run.

Gate telemetry artifacts are written to the artifact store before SQLite finalization. SQLite finalization then records gate rows, policy trace, evidence descriptors, attempt status, run status, and final decision in one transaction.

Artifact retention is an operator CLI workflow. It may remove filesystem artifact bytes after writing audit evidence, but it must not delete or mutate SQLite evidence descriptors.

Replay is a read-only reconstruction workflow. It accepts a run id and emits a deterministic JSON report from SQLite evidence descriptors and artifact presence checks. Replay must not execute gates, mutate run state, mutate evidence descriptors, recreate deleted artifacts, or change final decisions.

## Workspace model

The supported workspace modes are local and Git.

In local mode, the runtime workspace is selected as:

```text
workspace_root / contract.id
```

The contract id must be a single safe path segment. It must not escape the workspace root.

In Git mode, the worker materializes the repository into the same runtime workspace path before gate execution. The materializer rejects unsafe roots and symlink workspace targets, cleans any stale per-run workspace, clones the contract repository without recursive submodules, verifies `origin`, verifies `base_sha`, verifies `candidate_sha`, verifies `base_sha` is an ancestor of `candidate_sha`, detaches `HEAD` at `candidate_sha`, and verifies the workspace `HEAD`.

The admitted change boundary is:

```text
repo_url + base_sha + candidate_sha = admitted change boundary
```

The admission boundary also includes the contract scopes and admission policy.

`candidate_sha` is the admitted object. `candidate_ref` is optional provenance metadata and may be used only as a fetch hint or operator explanation. Branch names, pull request refs, tags, and other mutable names must not become admission authority.

Gate 3 evaluates `base_sha..candidate_sha` and the workspace `HEAD` must equal `candidate_sha` during gate execution.

## Sandbox and execution model

Gate commands execute through the process execution boundary.

The command environment is cleared. The engine sets a minimal environment:

- `PATH`
- `HOME`
- `CARGO_NET_OFFLINE=true`
- `CARGO_TERM_COLOR=never`
- `GIT_TERMINAL_PROMPT=0`

Process execution has timeouts. On Unix, gate commands run in a process group so timeout cleanup can kill descendants. Output capture is bounded. The sandbox runner applies CPU, address-space, and process-count rlimits where the platform supports them.

Sandbox profile `development` is for local development and does not claim production containment.

Sandbox profile `kubernetes-restricted` is the production containment baseline. It combines Rust runner hardening with Kubernetes/container runtime controls: pod namespaces, non-root execution, no privilege escalation, dropped Linux capabilities, RuntimeDefault seccomp, read-only root filesystem, explicit writable mounts, CPU and memory limits, and deny-all pod egress by default.

The engine validates `AH_SANDBOX_PROFILE` at startup. Unknown profiles fail closed.

## API and UI relationship

The TypeScript UI is a browser dashboard. It talks to the Rust API. It can submit contracts, list runs, inspect a run, inspect attempts, inspect gate records, inspect evidence descriptors, render the human review queue, and call the Rust review endpoints.

The UI must not bypass the API. It must not infer a final decision that the API has not recorded.

## Deployment shape

The deployment assets define a container runtime, Compose configuration, Kubernetes manifest, health probes, metrics, persistent data paths, artifact paths, and workspace mounts.

Runtime paths:

- SQLite data: `/data`
- Evidence artifacts: `/artifacts`
- Local Git workspaces: `/workspaces`

Required production environment:

- `AH_WORKSPACE_MODE=local` or `AH_WORKSPACE_MODE=git`
- `AH_SECURITY_MODE=api-key`
- `AH_API_KEYS=token|role|tenant|repo_prefixes`
- `RUST_LOG=core=info`

Optional limits:

- `AH_RATE_LIMIT_PER_MINUTE`
- `AH_RUN_QUOTA_PER_HOUR`

## Observability

The server exposes liveness, readiness, and Prometheus metrics.

The current metrics cover uptime, HTTP requests, HTTP response classes, submitted runs, and security denials.

HTTP requests are traced with method, path, status, and duration.

Durable audit events record tenant, actor, role, action, resource type, resource id, outcome, reason, and timestamp.

Run progress is published as ordered WebSocket events. The progress stream is observational only. It reports queueing, run start, attempt start, gate start, gate finish, finalization, and internal failure events. Clients may reconnect with the last received sequence number to replay recent events from the bounded in-memory progress buffer. If older progress events have aged out, durable evidence remains available through the read APIs.

## Architecture review record

Phase 25 review evidence is recorded in `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md`.
Phase 30 review evidence is recorded in `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md`.
Phase 31 sandbox review evidence is recorded in `docs/reviews/PHASE31_SANDBOX_HARDENING.md`.
Phase 37 security assessment evidence is recorded in `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md`.
Phase 40 production governance evidence is recorded in `docs/reviews/PHASE40_PRODUCTION_GOVERNANCE_REVIEW.md`.
Phase 41 production release evidence is recorded in `docs/reviews/PHASE41_PRODUCTION_RELEASE.md`.
Those reports are review evidence; this document remains the architecture authority.

## Non-goals for the current architecture

The current architecture does not make the LLM authoritative.

It does not provide a full multi-user identity provider.

It does not implement multi-stage approvals or external identity-provider integration for review decisions.

It does not replace CI/CD. It supplies an evidence-producing admission boundary that can integrate with CI/CD later.

It does not claim VM isolation or portable Rust-created namespaces. Non-Kubernetes production deployments must provide controls equivalent to the documented `kubernetes-restricted` profile.
