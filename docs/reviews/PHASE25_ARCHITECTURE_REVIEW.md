# Phase 25 Architecture Review I

## Scope

This review validates the implemented system against `docs/ARCHITECTURE.md`,
`docs/INVARIANTS.md`, `docs/API.md`, and `docs/PHASEMAP.md` after Phase 24.

This phase introduces no product capability, endpoint, table, or runtime
behavior.

## Architecture Review Report

| Area | Status | Evidence |
| :--- | :--- | :--- |
| Rust authority boundary | Aligned | Contract validation, run state, gate execution, review decisions, security checks, and evidence writes remain in Rust. |
| TypeScript authority boundary | Aligned | The UI calls Rust HTTP and WebSocket APIs and does not persist or decide final run status. |
| HTTP and CLI entry points | Aligned | CLI execution and HTTP worker execution share workspace materialization and orchestration paths. |
| Async execution model | Aligned | HTTP submission queues work; the background worker performs long-running execution outside request handling. |
| Gate sequence | Aligned | Gates 1 through 8 remain sequential and fail-fast. |
| Workspace model | Aligned with noted limitation | Local mode selects `workspace_root / contract.id`; Git mode materializes that path and checks out `base_sha`. |
| Evidence model | Aligned | Contracts, runs, attempts, gate runs, review decisions, evidence bundles, final decisions, and audit events are durable identities. |
| Human review | Aligned | Pending review is resolved only through Rust review endpoints with audit and evidence linkage. |
| Progress streaming | Aligned | WebSocket progress events are ordered, bounded, replayable observations and are not authority. |
| Deployment shape | Aligned | Health, metrics, container, Compose, Kubernetes, data paths, artifact paths, and required security/workspace environment are documented. |

## Invariant Compliance Report

| Invariant Area | Status | Notes |
| :--- | :--- | :--- |
| Rust authority | Compliant | Admission, review, security, state, and evidence decisions remain Rust-owned. |
| Contract validation | Compliant | Contract ids, repository URLs, base SHAs, and scope paths fail closed before workspace use. |
| Workspace containment | Compliant | Workspace selection is rooted at `workspace_root / contract.id`; Git mode rejects unsafe roots and symlink workspace targets. |
| Workspace modes | Compliant | `local` and `git` are implemented; unknown modes fail at startup. |
| Sequential gates | Compliant | Gate execution remains ordered and fail-fast. |
| Stable gate numbers | Compliant | Gate numbering remains unchanged. |
| Rejection versus internal failure | Compliant | Gate failures reject; infrastructure failures produce failed-internal paths. |
| Unknown is not approved | Compliant | Approval requires explicit completion of required gates and review when required. |
| Human review suspension | Compliant | `PENDING_HUMAN_REVIEW` has no final decision until reviewer action. |
| Attempt-owned gates | Compliant | Gate records belong to attempts, and summaries use the latest deterministic attempt. |
| Unique final decision | Compliant | Final decisions are unique per run. |
| Evidence durability | Compliant | Artifact descriptors are written after artifact bytes and link to run, attempt, gate, or review identities. |
| Transactional finalization | Compliant | Normal finalization writes gate records, evidence, attempt status, run status, and final decision atomically. |
| Tenant boundary before reads | Compliant | Public HTTP reads use tenant-aware store helpers after authorization. |
| Repository policy before submission | Compliant | Submitter repository policy is checked before queued run creation. |
| Security denials as evidence | Compliant | Denials are audited when server context is available. |
| Progress observational only | Compliant | Progress events do not mutate run state. |
| Bounded queue | Compliant | HTTP work submission uses a bounded queue and reports queue failures. |
| Blocking boundaries | Compliant | SQLite, Git, Cargo, and filesystem-heavy work are behind blocking execution boundaries. |
| Bounded output | Compliant | Process output and read previews are capped with truncation flags. |
| Controlled gate environment | Compliant | Gate commands clear inherited environment and set the required variables. |
| Mandatory timeouts | Compliant | External gate commands run with timeouts and cleanup. |
| Scope-based change boundary | Compliant in local mode | See deviation `D25-001` for Git mode candidate-change acquisition. |
| Supply-chain admission | Compliant | Gate 8 runs `cargo deny check` before `cargo audit`. |
| Metrics non-authoritative | Compliant | Metrics are operational signals only. |
| UI non-authoritative | Compliant | UI state comes from API responses and progress observations. |
| Explicit API models | Compliant | Public responses use explicit typed DTOs. |
| Migration preservation | Compliant | Legacy gate rows are attached to deterministic attempts. |
| Negative-path tests | Compliant | Authority paths include failure and denial coverage. |
| Documentation match | Compliant with deviations recorded | Current known limitations are recorded below. |
| Coding style | Compliant for review scope | This phase adds evidence documentation only. |

## API Inventory

| Route | Purpose | Authority | Role Requirement |
| :--- | :--- | :--- | :--- |
| `GET /health/live` | Process liveness | Operational only | None in local mode; configured security applies globally where enabled. |
| `GET /health/ready` | SQLite readiness | Operational only | None in local mode; configured security applies globally where enabled. |
| `GET /metrics` | Prometheus metrics | Operational only | None in local mode; configured security applies globally where enabled. |
| `POST /runs` | Submit a contract and queue a run | Rust API | `submitter` or `admin` |
| `GET /runs` | List tenant-scoped runs | Rust API | `viewer`, `submitter`, `reviewer`, or `admin` |
| `GET /runs/:id` | Read tenant-scoped run status | Rust API | `viewer`, `submitter`, `reviewer`, or `admin` |
| `GET /runs/:id/attempts` | Read tenant-scoped attempts | Rust API | `viewer`, `submitter`, `reviewer`, or `admin` |
| `GET /runs/:id/evidence` | Read tenant-scoped evidence descriptors | Rust API | `viewer`, `submitter`, `reviewer`, or `admin` |
| `GET /runs/:id/progress` | Observe ordered run progress | Observational | `viewer`, `submitter`, `reviewer`, or `admin` |
| `POST /runs/:id/review/approve` | Approve pending human review | Rust API | `reviewer` or `admin` |
| `POST /runs/:id/review/reject` | Reject pending human review | Rust API | `reviewer` or `admin` |
| `GET /attempts/:id/gates` | Read tenant-scoped gate records | Rust API | `viewer`, `submitter`, `reviewer`, or `admin` |

## Persistence Inventory

| Store | Durable | Purpose |
| :--- | :--- | :--- |
| `contracts` | Yes | Original submitted contract fields. |
| `runs` | Yes | Tenant-owned run state and reason. |
| `attempts` | Yes | Deterministic per-run execution attempts. |
| `gate_runs` | Yes | Attempt-owned gate results and telemetry. |
| `evidence_bundles` | Yes | Artifact descriptors linked to runs, attempts, gates, or review decisions. |
| `final_decisions` | Yes | Unique terminal approved or rejected decision per run. |
| `review_decisions` | Yes | Reviewer identity, role, decision, reason, and timestamp. |
| `audit_events` | Yes | Security, authorization, submission, read, and review audit trail. |
| Filesystem artifacts | Yes | Larger evidence payloads addressed by SQLite descriptors. |
| Progress replay buffer | No | Bounded in-memory reconnect aid; not authoritative evidence. |
| Metrics | No | Operational counters only. |

## Deviation Register

| ID | Status | Deviation | Impact | Disposition |
| :--- | :--- | :--- | :--- | :--- |
| D25-001 | Open | Git mode materializes and detaches `HEAD` at `base_sha`; the contract does not yet model a separate candidate ref or patch source. | In Git mode, Gate 3 can only compare the checked-out base commit to itself unless the workspace is modified by another controlled future step. | Add candidate-change materialization in a later roadmap phase before relying on Git mode for remote proposed-change admission. |
| D25-002 | Open | The process boundary is not a complete adversarial kernel sandbox. | Production isolation still depends on container, namespace, filesystem, network, and syscall policy around the runner. | Keep as production hardening work before production acceptance. |
| D25-003 | Accepted limitation | WebSocket replay is bounded and in memory. | Old progress observations can age out after disconnects. | Durable evidence remains in SQLite and filesystem artifacts; no action required unless operators require durable event history. |
| D25-004 | Accepted limitation | Security uses API-key identities and does not include external identity providers. | Operator identity integration is deployment-specific. | External identity remains out of scope until a future security phase requires it. |

## Roadmap Impact Assessment

| Phase | Impact | Required Attention |
| :--- | :--- | :--- |
| Phase 26 Multi-Tenant Hardening | No expected architecture change. | Preserve tenant-aware read helpers and authorization-before-store-read invariant. |
| Phase 27 Artifact Retention | No expected architecture change. | Retention must not silently delete authoritative evidence or break descriptor integrity. |
| Phase 28 Replay Engine | Architecture-sensitive. | Replay must reconstruct from SQLite and artifacts without treating progress events as authority. |
| Phase 29 Admission Policy Engine | Architecture-sensitive. | Policy inputs, outputs, traces, and final-decision influence require contract, invariant, and evidence review. |
| Phase 30 Architecture Review II | Expected checkpoint. | Re-evaluate this deviation register and confirm whether D25-001 or D25-002 changed scope. |
| Phase 31 Sandbox Hardening | Expected architecture change. | D25-002 should drive containment design for namespace, filesystem, network, and syscall policy. |
| Phase 35 Release Readiness Review | Release-sensitive. | Open deviations must be classified as closed, accepted, or blocking before release readiness. |
| Phase 37 Security Assessment | Security-sensitive. | D25-002 and tenant/security assumptions require explicit assessment evidence. |
| Phase 40 Production Governance Review | Release gate. | The final deviation register must have no unresolved production-blocking architecture or invariant gaps. |
| Phase 41 Production Release | Production gate. | D25-001 and D25-002 must be closed or explicitly accepted as non-blocking before v1.0. |

## Conclusion

The implementation remains aligned with the documented architecture for the
current roadmap stage. Phase 25 records two open production-readiness gaps:
candidate-change acquisition for Git mode and full adversarial sandboxing.
Neither gap changes current executable behavior, and both are now explicit in
the review evidence.
