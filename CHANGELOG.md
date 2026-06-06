# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.42] - 2026-06-06 - License Governance

### Added
- **Cargo Deny Policy** - Added `core/deny.toml` to validate advisories, bans, sources, and dependency licenses for Gate 8 supply-chain governance
- **License Governance Review** - Added `docs/reviews/LICENSE_GOVERNANCE.md` with the approved license list, review-required policy, prohibited-license rule, and exception process

### Changed
- **Local Crate License Metadata** - Declared the Rust authority crate license as MIT to match the repository license
- **D37-001 Closure** - Updated Phase 37 and PHASEMAP evidence to close the supply-chain license governance finding after `cargo deny check` passed

## [0.0.41] - 2026-06-06 - Placeholder Credential Rejection

### Added
- **Placeholder Credential Invariant** - Added the invariant that production API-key mode must not start with placeholder credentials
- **Startup Rejection Tests** - Added coverage for deployment placeholder, known placeholder tokens, empty token, whitespace token, and valid token acceptance

### Changed
- **API-Key Startup Validation** - `api-key` security mode now rejects known placeholder tokens before the server accepts requests
- **D37-002 Closure** - Updated deployment docs, startup runbook, phase map, and Phase 37 security assessment to close the placeholder admin API key release blocker

## [0.0.40] - 2026-06-06 - Security Assessment

### Added
- **Phase 37 Security Report** - Added `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md` with threat model, source-grounded penetration testing report, dependency assessment, and remediation inventory
- **Security Remediation Inventory** - Recorded D25-002 sandbox residual risk, D37-001 non-blocking v1.0 supply-chain license policy gap, and D37-002 release-blocking placeholder API key deployment risk

### Changed
- **Roadmap Security Status** - Updated `docs/PHASEMAP.md` to record Phase 37 acceptance evidence, validation commands, residual risks, and security-assessment deviations
- **Architecture Review Index** - Updated `docs/ARCHITECTURE.md` to link the Phase 37 security assessment as review evidence
- **Sandbox Review Wording** - Removed stale D25-001 dependency language from the Phase 31 sandbox network-model note

## [0.0.39] - 2026-06-05 - Candidate SHA Admission Boundary

### Added
- **Candidate Authority Field** - Added required `candidate_sha` and optional `candidate_ref` to contract, API, replay, policy trace, and browser UI models
- **Candidate Identity Migration** - Added SQLite migration and schema normalization for `contracts.candidate_sha`, `contracts.candidate_ref`, and `contracts.scopes_json`, with legacy rows backfilled to preserve historical evidence shape
- **D25-001 Closure Evidence** - Updated the candidate acquisition review and phase map with commit-SHA admission closure evidence

### Changed
- **Git Materialization** - Git mode now validates `base_sha`, validates `candidate_sha`, checks out `candidate_sha`, verifies `HEAD == candidate_sha`, and treats `candidate_ref` only as a fetch hint
- **Gate 3 Boundary** - Change-boundary evaluation now compares `base_sha..candidate_sha` instead of using workspace `HEAD` as implicit authority
- **Contract Persistence** - Reusing a contract id with different authority data, including scopes, is now rejected instead of silently reusing the stored contract row
- **Governance Docs** - Updated architecture, invariants, API, Phase 35/36 review notes, and the phase map to record `candidate_sha` as the admitted object

## [0.0.38] - 2026-06-05 - Performance Validation

### Added
- **Phase 36 Performance Report** - Added `docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md` with load, concurrency, queue saturation, storage, validation, and deviation evidence

### Changed
- **Deployment Performance Baseline** - Updated deployment and phase map documentation with the current bounded queue, single-worker, pooled SQLite, indexed storage, and local read-load validation model

## [0.0.37] - 2026-06-05 - Release Readiness Review

### Added
- **Phase 35 Release Review** - Added `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` with release result, unresolved issue inventory, security review inventory, architecture review inventory, operational review inventory, and release gate criteria
- **Candidate Acquisition Architecture** - Added `docs/reviews/CANDIDATE_ACQUISITION_ARCHITECTURE.md` defining `candidate_sha` as the future first-class admitted object for D25-001 closure

### Changed
- **Release Readiness Governance** - Updated architecture, invariants, and phase map records to classify D25-001 candidate acquisition as the remaining blocking production-release gap
- **Candidate Authority Model** - Documented that future `candidate_ref` metadata must not become admission authority and that Gate 3 must compare `base_sha..candidate_sha` once candidate acquisition is implemented

## [0.0.36] - 2026-06-05 - Browser UI Theme Refresh

### Added
- **Semantic UI Theme** - Added approved Acceptability Engine palette tokens and semantic color tokens for status, action, text, and surface usage

### Changed
- **Browser UI Styling** - Updated the browser dashboard, run list, run detail, review queue, gate timeline, evidence panels, and operations metrics to use semantic theme variables instead of raw component colors
- **Status Visualization** - Mapped queued/running, pending human review, approved, rejected, and failed-internal states to consistent semantic status classes while preserving visible status labels

## [0.0.35] - 2026-06-05 - Disaster Recovery Validation

### Added
- **Disaster Recovery Runbook** - Added `docs/runbooks/disaster_recovery.md` with recovery checklist, verification commands, success criteria, postmortem inputs, and Phase 34 validation notes
- **Phase 34 DR Report** - Added `docs/reviews/PHASE34_DISASTER_RECOVERY_VALIDATION.md` with exercise scope, recovery fixture, procedure, timing, postmortem review, validation evidence, and deviations
- **DR Replay Validation** - Added a file-backed recovery test that consumes the Phase 33 backup fixture, deletes originals, restores from backup, and verifies normalized replay equality

### Changed
- **Operational Routing** - Updated operations, deployment, incident response, restore, and phase map documentation so disaster recovery procedures are discoverable from the operator path
- **Recovery Scope** - Documented Phase 34 as local evidence-store recovery validation while leaving live Kubernetes rebuild validation separate

## [0.0.34] - 2026-06-05 - Backup Restore Validation

### Added
- **Backup Runbook** - Added `docs/runbooks/backup.md` with backup procedure, artifact shape, inventory, integrity validation, and restore prerequisites
- **Phase 33 Backup Report** - Added `docs/reviews/PHASE33_BACKUP_RESTORE_VALIDATION.md` with verification contract, recovery fixture, backup artifact shape, integrity validation, restore prerequisites, and validation evidence
- **Backup Fixture Validation** - Added a file-backed backup validation test that creates fixture run history, writes replay evidence, backs up SQLite and artifacts, and validates inventory hashes for copied evidence

### Changed
- **Restore Documentation** - Updated restore, operations, deployment, and phase map documentation so backup artifact shape and restore prerequisites are visible before disaster recovery

## [0.0.33] - 2026-06-04 - Operational Readiness

### Added
- **Operations Index** - Added `docs/OPERATIONS.md` with runbook links, monitoring inventory, metrics inventory, alert definitions, operator invariants, and Phase 32 validation evidence
- **Startup Runbook** - Added startup checks for deployment mode, production environment, writable paths, API key placeholder detection, Compose startup, Kubernetes startup, and readiness validation
- **Shutdown Runbook** - Added evidence-preserving Compose and Kubernetes shutdown procedures
- **Retention Runbook** - Added artifact retention dry-run, live deletion, success criteria, and restore handoff procedures
- **Replay Runbook** - Added read-only replay procedure, output handling, success criteria, and missing-run handling
- **Incident Response Runbook** - Added first-response checks, triage table, escalation evidence, and recovery handoffs
- **Restore Runbook** - Added manual restore order and validation procedure for SQLite and artifact recovery ahead of Phase 33 validation
- **Phase 32 Operational Report** - Added `docs/reviews/PHASE32_OPERATIONAL_READINESS.md` with procedure, monitoring, alert, authority, deferred-validation, and command evidence

### Changed
- **Operational Documentation** - Updated deployment and phase map records so health, metrics, retention, replay, incidents, and restore procedures are discoverable from the authoritative docs
- **Observability Framing** - Clarified that metrics are operational observations and do not replace durable SQLite and artifact evidence

## [0.0.32] - 2026-06-04 - Sandbox Hardening

### Added
- **Sandbox Profiles** - Added `AH_SANDBOX_PROFILE` validation with `development` and Linux-only `kubernetes-restricted` profiles
- **Restricted Kubernetes Profile** - Added non-root execution, no privilege escalation, dropped capabilities, RuntimeDefault seccomp, read-only root filesystem, explicit writable mounts, resource limits, and deny-all egress to the Kubernetes manifest
- **Compose Local Hardening** - Added dropped capabilities, no-new-privileges, read-only root filesystem, and `/tmp` tmpfs for local Compose runs
- **Phase 31 Sandbox Report** - Added `docs/reviews/PHASE31_SANDBOX_HARDENING.md` with namespace, filesystem, network, syscall, resource, containment, escape, deviation, and validation sections
- **Sandbox Coverage** - Added tests for sandbox profile defaults, restricted profile validation, unknown profile rejection, restricted kernel-control declarations, and sandbox runner to resource-limit wiring

### Changed
- **Sandbox Architecture** - Documented `kubernetes-restricted` as the production containment baseline and `development` as non-production hardening, with full closure of D25-002 requiring runtime enforcement validation
- **Sandbox Invariants** - Added fail-closed sandbox profile requirements to the invariant set

## [0.0.31] - 2026-06-04 - Architecture Review II

### Added
- **Phase 30 Review Evidence** - Added `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` with architecture, invariant, policy authority, replay completeness, security boundary, persistence, deviation, roadmap, and invariant assessment sections
- **Deviation Register Update** - Reconfirmed candidate-change acquisition as a release-critical gap and adversarial sandboxing as the other remaining open production architecture gap
- **Roadmap Impact Assessment** - Added Phase 30 guidance for sandbox hardening, candidate acquisition, security assessment, governance review, and production release readiness

### Changed
- **Governance Traceability** - Updated architecture, invariants, and phase map records so Phase 30 review evidence is discoverable from the authoritative documents

## [0.0.30] - 2026-06-04 - Admission Policy Engine

### Added
- **Contract Admission Policy** - Added optional `admission_policy` contract input with `strict-v1` defaults for declarative policy evaluation
- **Policy Evaluation Store** - Added `policy_evaluations` persistence with run, attempt, policy id, policy version, outcome, reason, trace JSON, and timestamp
- **Policy Evidence** - Added transactional policy trace persistence during normal run finalization
- **Policy Replay Output** - Added policy evaluations to deterministic replay reports
- **TypeScript Policy Type** - Added optional admission policy fields to the browser API contract model
- **Phase 29 Policy Review** - Added `docs/reviews/PHASE29_ADMISSION_POLICY_ENGINE.md` with policy scope, evaluation order, evidence model, validation evidence, and deviations
- **Policy Coverage** - Added tests for required gates, failed gates, test parse-error limits, unsupported policy rejection, deterministic gate ordering, mandatory-gate weakening rejection, policy trace persistence, and rollback

### Changed
- **Admission Ordering** - Finalization now evaluates gate results, then admission policy, then human review requirement before final decision
- **Governance Documentation** - Updated architecture, invariants, API, and phase map records for the policy decision boundary

## [0.0.29] - 2026-06-04 - Replay Engine

### Added
- **Replay Contract** - Added `docs/reviews/PHASE28_REPLAY_ENGINE.md` defining replay input, deterministic JSON output, read-only rules, validation evidence, and deviations
- **Replay Report Model** - Added read-only replay reports with contract, run, attempts, gates, review decision, final decision, evidence descriptors, missing artifact indicators, generated timestamp, and source database identity
- **Replay CLI** - Added `--replay-run-id` for emitting replay JSON from the existing database and artifact root
- **Replay Coverage** - Added tests for complete replay reports, missing artifact indicators, deterministic replay content, and missing run behavior

### Changed
- **Replay Governance** - Updated architecture, invariants, deployment, and phase map records to document replay as read-only reconstruction

## [0.0.28] - 2026-06-04 - Artifact Retention

### Added
- **Artifact Retention CLI** - Added `--retention-days` and `--retention-dry-run` for explicit filesystem artifact lifecycle management
- **Retention Audit Evidence** - Added retention audit events for dry-run, planned, deleted, and missing artifact outcomes
- **Artifact Delete Safety** - Added artifact URI validation, traversal rejection, backslash rejection, and symlink root/parent checks before planning or deleting artifact files
- **Phase 27 Lifecycle Report** - Added `docs/reviews/PHASE27_ARTIFACT_RETENTION.md` with retention policy, cleanup workflow, audit record, validation, and deviation sections
- **Retention Coverage** - Added tests for dry-run behavior, artifact deletion, missing artifact audits, cutoff filtering, evidence descriptor preservation, and URI traversal rejection

### Changed
- **Evidence Lifecycle Documentation** - Updated architecture, invariants, deployment, and phase map records to document audited artifact retention while preserving SQLite evidence descriptors

## [0.0.27] - 2026-06-04 - Multi-Tenant Hardening

### Added
- **Phase 26 Tenant Evidence** - Added `docs/reviews/PHASE26_TENANT_HARDENING.md` with query review, boundary validation, tenant isolation test, authorization test, and deviation sections
- **Cross-Tenant Denial Audit** - Added denied audit events for authenticated hidden-resource run, attempt, evidence, progress, and review access attempts
- **Tenant Isolation Coverage** - Added HTTP negative-path tests for cross-tenant run status, attempt gates, evidence, and review attempts

### Changed
- **Tenant Boundary Documentation** - Updated architecture, invariants, and phase map records for opaque cross-tenant responses and durable denial evidence

## [0.0.26] - 2026-06-04 - Architecture Review I

### Added
- **Phase 25 Review Evidence** - Added `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` with architecture review, invariant compliance, API inventory, persistence inventory, and deviation register sections
- **Deviation Register** - Recorded Git candidate-change acquisition and full adversarial sandboxing as open production-readiness gaps
- **Roadmap Impact Assessment** - Added the Phase 25 bridge from open deviations to later roadmap checkpoints and production release gates

### Changed
- **Governance Traceability** - Updated architecture, invariant, phase map, and changelog records so Phase 25 completion evidence is discoverable from the authoritative documents

## [0.0.25] - 2026-06-04 - Git Materialization

### Added
- **Git Workspace Mode** - Added `AH_WORKSPACE_MODE=git` support for cloning contract repositories into per-run workspaces
- **Workspace Materializer** - Added shared CLI and HTTP worker materialization for local and Git workspace modes
- **Git Checkout Safety** - Added origin verification, detached HEAD checkout at `base_sha`, credential-prompt disabling, unsafe-root rejection, symlink rejection, and stale workspace cleanup
- **Git Materialization Coverage** - Added clone, checkout, cleanup, malicious path, unsafe root, symlink, detached HEAD, and origin validation tests

### Changed
- **CLI Workspace Semantics** - CLI execution now uses the same `workspace_root / contract.id` materialization path as HTTP execution
- **Workspace Documentation** - Updated architecture, invariants, deployment, phase map, and README documentation for Git mode

## [0.0.24] - 2026-06-04 - WebSocket Progress Streaming

### Added
- **Progress Event Hub** - Added ordered run progress events with bounded replay for reconnecting clients
- **Progress WebSocket** - Added `GET /runs/:id/progress` for live run visibility over WebSocket
- **Gate Progress Events** - Added queued, started, attempt started, gate started, gate finished, finalized, and failed-internal progress event types
- **Progress Reconnect Support** - Added `?after=<sequence>` replay for recent run progress events
- **Progress UI Surface** - Added typed TypeScript progress events, a WebSocket API client method, and selected-run live refresh
- **Progress Coverage** - Added event ordering, replay filtering, and WebSocket integration coverage

### Changed
- **Observability Contract** - Documented progress streams as observational signals that do not mutate run state or replace durable evidence

## [0.0.23] - 2026-06-04 - Human Review Authority

### Added
- **Review Decisions** - Added `review_decisions` persistence with reviewer actor, reviewer role, decision, reason, and timestamp
- **Review Endpoints** - Added `POST /runs/:id/review/approve` and `POST /runs/:id/review/reject` as Rust-authoritative human review decisions
- **Reviewer Role** - Added a `reviewer` API-key role that can read runs and finalize pending human review runs without submit authority
- **Review Evidence Linkage** - Added `review_decision_id` evidence linkage so human-review evidence points to the review record that produced it
- **API Documentation** - Added `docs/API.md` with authentication, run, evidence, attempt, review, and operations endpoint documentation
- **Review Coverage** - Added approval, rejection, authorization, persistence, timestamp, audit, and evidence-linkage tests

### Changed
- **Pending Review Finalization** - `PENDING_HUMAN_REVIEW` runs now become terminal only through a transactional review decision that writes evidence and a final decision
- **Governance Navigation** - `AGENTS.md` now points API changes at `docs/API.md`

## [0.0.22] - 2026-06-04 - CI Build Stabilization

### Added
- **GitHub CI Workflow** - Added cacheless GitHub Actions coverage for Rust formatting, clippy, tests, UI build, Docker image build, Compose validation, and Kubernetes manifest validation
- **Low-Memory CI Settings** - Added one-job Cargo build settings in CI so temporary runners do not rely on cache or high memory availability

### Changed
- **Docker Build Controls** - Docker builds now accept `CARGO_BUILD_JOBS` and disable incremental compilation to make runtime image builds more predictable on ephemeral CI runners
- **Offline Manifest Validation** - Kubernetes CI validation now parses manifest shape without contacting a cluster or relying on `kubectl` API discovery

## [0.0.21] - 2026-06-04 - Deployment Foundation

### Added
- **Health Probes** - Added `/health/live` and `/health/ready` endpoints for process and SQLite readiness checks
- **Prometheus Metrics** - Added `/metrics` with request, response, submission, security-denial, and uptime counters
- **Structured Tracing** - Added `tracing` initialization and per-request HTTP completion logs controlled by `RUST_LOG`
- **Container Runtime** - Added a Dockerfile and Compose file with persistent data, artifacts, workspace mounts, and HTTP health checks
- **Kubernetes Manifests** - Added namespace, secret, PVC, deployment, probes, resource limits, and service definitions under `deploy/`
- **Deployment Documentation** - Added deployment notes covering probes, metrics, container usage, Kubernetes, and required environment variables

### Changed
- **Server Startup** - HTTP server startup now wires telemetry state into app state and records request metrics through middleware
- **Submission Observability** - Accepted submissions and security denials now increment runtime counters in addition to durable audit rows

## [0.0.20] - 2026-06-04 - Security Trust Foundation

### Added
- **API Key Security Mode** - Added opt-in `AH_SECURITY_MODE=api-key` HTTP authentication using `AH_API_KEYS` entries shaped as `token|role|tenant|repo_prefixes`
- **Role Enforcement** - Added viewer, submitter, and admin roles so read and submit endpoints enforce separate permissions
- **Tenant Run Ownership** - Added `runs.tenant_id` and tenant-scoped run, attempt, gate, and evidence read helpers so API callers cannot inspect another tenant's run data
- **Repository Policy Model** - Added per-key repository prefix policy enforcement before accepting submitted contracts
- **Rate Limits and Run Quotas** - Added in-memory request rate limits and submission quotas through `AH_RATE_LIMIT_PER_MINUTE` and `AH_RUN_QUOTA_PER_HOUR`
- **Security Audit Log** - Added durable `audit_events` storage and HTTP audit writes for allowed and denied run API decisions
- **Security Coverage** - Added tests for API-key parsing, role rejection, repository policy rejection, rate limiting, tenant-scoped reads, and audit persistence

### Changed
- **HTTP Handler Trust Boundary** - Run submission and evidence read handlers now authorize before store access and record audit outcomes after decisions
- **Store Query Boundary** - Public HTTP paths now use tenant-aware store helpers while legacy local helpers are limited to CLI/internal test paths

## [0.0.19] - 2026-06-04 - Process Isolation Completion

### Added
- **Sandbox Runner Process** - Gate commands now execute through an internal sandbox runner process boundary instead of being spawned directly by the orchestrator process
- **Resource Limit Hooks** - Added Unix CPU, address-space, and process-count limits for sandboxed gate commands behind a platform-isolated resource-limit module
- **Output Capture Limits** - Added per-stream stdout and stderr capture limits so gate output cannot grow without bound in memory or persistence
- **Process Isolation Coverage** - Added tests for runner-backed command execution and output-limit rejection

### Changed
- **Gate Command Launch Path** - Centralized sandbox environment, process-group cleanup, timeout metadata, and resource control in the gate process executor
- **Timeout Cleanup Semantics** - Timeout reporting now remains deterministic even if output reader cleanup observes pipe errors after process termination

## [0.0.18] - 2026-06-03 - UI Observability

### Added
- **TypeScript API Surface** - Added a typed frontend API client for runs, submissions, attempts, gates, and evidence read endpoints
- **Evidence Dashboard UI** - Added a Vite TypeScript browser dashboard with run status metrics, run list filtering, selected run details, gate output, attempts, and evidence descriptors
- **Review Queue UI** - Added a pending human review queue view driven by `PENDING_HUMAN_REVIEW` runs
- **Live Progress Polling** - Added polling refresh for run lists and selected active runs using the existing HTTP API

### Changed
- **Project Documentation** - Documented how to run the Rust API with the new UI development server
- **Build Ignore Rules** - Ignored nested `dist` directories generated by frontend builds

## [0.0.17] - 2026-06-03 - Coding Style Compliance

### Added
- **Legacy Migration SQL Files** - Added external SQL files for attempts rebuilds, legacy gate run migration, and evidence bundle rebuilds

### Changed
- **Artifact Finalization Order** - Gate telemetry artifacts are now written before SQLite finalization begins, keeping filesystem writes out of the database transaction
- **Schema Coordinator Shape** - Reduced `schema.rs` to migration coordination, schema-shape checks, and migration dispatch instead of embedding large legacy SQL bodies
- **Handler Narrative Order** - Moved HTTP handler tests below production handler code so `server/handlers.rs` follows top-down production flow

## [0.0.16] - 2026-06-03 - Production Evidence Store

### Added
- **Bounded SQLite Connection Pool** - Added a production store connection boundary that opens per-operation SQLite connections behind an async semaphore instead of serializing all HTTP and worker access through one mutex
- **External Migration Files** - Moved core table, gate run, evidence bundle, and query index SQL into versioned files under `core/migrations`
- **Filesystem Artifact Store** - Added a filesystem-backed artifact writer that returns storage URI, SHA-256, byte length, content type, kind, label, and summary descriptors
- **Gate Artifact Evidence** - Gate finalization now writes JSON telemetry artifacts and stores descriptor-backed evidence bundle rows linked to the run, attempt, and gate record
- **Evidence Store Coverage** - Added tests for pooled file-backed database reuse, filesystem artifact descriptors, and artifact-backed gate evidence metadata

### Changed
- **CLI Artifact Configuration** - Added `--artifact-root` so CLI and HTTP execution can write durable evidence artifacts outside SQLite
- **HTTP Worker Dependencies** - Threaded artifact storage through the supervised run worker so background runs capture artifact descriptors during finalization
- **Schema Ownership** - Kept legacy shape normalization in Rust while making the durable schema SQL inspectable as migration files
- **Store Concurrency Boundary** - Preserved the focused store query modules while removing the production `Arc<Mutex<Connection>>` bottleneck from the main runtime path

## [0.0.15] - 2026-06-03 - Evidence Store Descriptors

### Added
- **Evidence Artifact Descriptors** - Added `kind`, `label`, `storage_uri`, `sha256`, `byte_len`, and `content_type` metadata to evidence bundles
- **Descriptor Evidence Writes** - Added a descriptor-based evidence insert path while preserving the summary evidence helper used by orchestration
- **Production Query Indexes** - Added indexes for run status listing, contract lookup, attempt ordering, gate lookup, and evidence lookup paths
- **Evidence Descriptor Coverage** - Added tests for rich descriptor reads and production index creation

### Changed
- **Evidence Read API Shape** - Extended run evidence responses with artifact descriptor metadata while preserving run, attempt, gate, summary, and timestamp fields
- **Evidence Migration Safety** - Existing evidence bundle rows now receive descriptor defaults during schema normalization

## [0.0.14] - 2026-06-03 - Typed Orchestration IDs

### Added
- **Domain ID Types** - Added typed `RunId`, `AttemptId`, `GateRunId`, `EvidenceBundleId`, and `FinalDecisionId` wrappers for persistence identifiers
- **Typed ID Serialization** - Added transparent JSON serialization for ID wrappers so API responses keep numeric IDs while internal code gets stronger types

### Changed
- **Orchestration ID Flow** - Replaced raw run and attempt IDs across orchestration finalization with typed domain IDs
- **Store ID Boundary** - Updated store read and write helpers to accept typed IDs instead of interchangeable `i64` values
- **Worker Queue IDs** - Updated queued run work to carry `RunId` so background execution cannot confuse run IDs with other persistence IDs

## [0.0.13] - 2026-06-03 - Explicit Workspace Mode

### Added
- **Workspace Mode Configuration** - Added `AH_WORKSPACE_MODE` parsing with explicit `local` workspace mode selection
- **Workspace Mode Coverage** - Added tests for default local mode, explicit local mode, unsupported Git mode, and unknown mode values

### Changed
- **Runtime Startup Validation** - CLI and HTTP startup now fail fast when `AH_WORKSPACE_MODE` selects an unsupported or invalid workspace mode
- **Server Workspace State** - Threaded workspace mode through HTTP state so contract submission resolves workspaces against the selected materialization mode
- **Workspace Mode Documentation** - Documented `AH_WORKSPACE_MODE=local` behavior and the reserved `git` mode in the README

## [0.0.12] - 2026-06-03 - Evidence Read API

### Added
- **Run Attempts Endpoint** - Added `GET /runs/:id/attempts` for listing durable attempts for a run
- **Attempt Gates Endpoint** - Added `GET /attempts/:id/gates` for reading detailed gate records, command output, and test metrics for an attempt
- **Run Evidence Endpoint** - Added `GET /runs/:id/evidence` for reading evidence bundle anchors linked to runs, attempts, and gate records
- **Evidence Read DTOs** - Added JSON response models for attempt summaries, attempt gate details, and evidence bundle summaries
- **Evidence Read Coverage** - Added store and HTTP handler coverage for attempt, gate, evidence, and missing-attempt read behavior

### Changed
- **Store Read Layout** - Split attempt and evidence read queries into focused modules so the HTTP layer remains a thin coordination boundary
- **Gate Output Preview** - Capped gate detail stdout/stderr previews at 8 KiB and added truncation flags for oversized command output

## [0.0.11] - 2026-06-03 - Attempt Evidence Model

### Added
- **Run Attempts Table** - Added an `attempts` table so each run execution has a durable attempt identity
- **Attempt Numbering** - Added deterministic `attempt_number` sequencing for run attempts
- **Final Decisions Table** - Added `final_decisions` with `UNIQUE(run_id)` for approved and rejected terminal outcomes
- **Evidence Bundles Table** - Added `evidence_bundles` linked to run, attempt, and gate evidence IDs after gate evidence is recorded
- **Human Review Suspension** - Added `PENDING_HUMAN_REVIEW` behavior when all gates pass and the contract requires human review
- **Attempt Model Regression Coverage** - Added tests for legacy migration, latest-attempt summaries, final decision uniqueness, transactional rollback, gate evidence links, internal-error finalization, and worker pending-review success

### Changed
- **Gate Run Ownership** - Changed `gate_runs` persistence from `run_id` ownership to `attempt_id` ownership
- **Run Summaries** - Preserved run summary responses by loading gates from the latest attempt for each run
- **Finalization Flow** - Approved and rejected outcomes now persist final decisions, while pending human review leaves final decision creation deferred
- **Finalization Atomicity** - Gate recording, evidence bundle writes, attempt status, run status, and final decision writes now occur in one SQLite transaction
- **Legacy Migration Safety** - Legacy gate evidence now migrates through attempt #1 for each existing run so old gate rows cannot be orphaned or assigned to later attempts
- **Internal Error Finalization** - Gate runner infrastructure errors now mark the attempt `ERROR`, mark the run `FAILED_INTERNAL`, and persist engine-error evidence from the orchestrator

## [0.0.10] - 2026-06-03 - Runtime Supervision Hardening

### Added
- **Run Worker Handle** - Added an explicit `RunWorker` handle around the background queue worker task
- **Process Pipe Error Variant** - Added `ProcessError::MissingPipe` for missing subprocess stdout/stderr pipes

### Changed
- **Worker Lifecycle Management** - The HTTP server now supervises the run worker and reports worker termination instead of dropping the task handle
- **Process Pipe Handling** - Replaced subprocess stdout/stderr `unwrap()` calls with predictable error returns

## [0.0.9] - 2026-06-03 - Store Access Refactor

### Added
- **Blocking Store Access Helper** - Added a store helper that runs SQLite operations on Tokio's blocking thread pool
- **Focused Store Modules** - Split store responsibilities into connection, schema, run writes, queries, row mappers, gate records, and DTO modules

### Changed
- **Async SQLite Boundary** - Moved direct SQLite work out of async request and orchestration futures
- **Storage Abstraction Layout** - Preserved the public `crate::store` API while isolating SQL, mapping, and persistence concerns below it

## [0.0.8] - 2026-06-03 - Sandbox Runner Environment

### Added
- **Sandbox Runner Policy** - Added a gate sandbox module that applies a minimal execution environment before process launch
- **Environment Isolation Tests** - Added coverage for proxy stripping, offline cargo mode, noninteractive Git prompts, and sandbox home selection

### Changed
- **Gate Command Execution** - All timeout-managed gate commands now run with cleared inherited environment, explicit `PATH`, isolated `HOME`, `CARGO_NET_OFFLINE=true`, `CARGO_TERM_COLOR=never`, and `GIT_TERMINAL_PROMPT=0`

## [0.0.7] - 2026-06-03 - Supply Chain Gate

### Added
- **Gate 8: Supply Chain** - Added a fail-fast supply-chain gate after tests
- **Cargo Deny Check** - Gate 8 runs `cargo deny check` for policy, license, advisory, and ban checks
- **Cargo Audit Scan** - Gate 8 runs `cargo audit` for RustSec advisory scanning after `cargo deny` succeeds
- **Supply Chain Unit Coverage** - Added tests for command construction and successful evidence merging

### Changed
- **Gate Runner Capacity** - Expanded the sequential gate runner from seven to eight gates

## [0.0.6] - 2026-06-03 - Process Group Timeout Cleanup

### Added
- **Process Group Isolation** - Gate commands now launch in their own Unix process group before execution
- **Descendant Timeout Test** - Added regression coverage proving timeout cleanup kills child processes spawned by a shell command

### Changed
- **Timeout Cleanup** - Timeout handling now sends `SIGKILL` to the process group, falls back to direct child kill, reaps the process, and joins stdout/stderr reader threads

## [0.0.5] - 2026-06-03 - Local Workspace Mode Lock

### Added
- **Local Workspace Verification** - Gate 2 now requires the runtime workspace to already exist as a directory
- **Git Repository Verification** - Gate 2 validates that the workspace is a Git work tree and that `base_sha` resolves to a commit
- **Workspace Validation Tests** - Added tests for missing and non-directory local workspaces

### Changed
- **Workspace Mode Semantics** - Locked the engine into explicit local-workspace mode instead of creating empty workspace directories
- **Gate 2 Success Message** - Updated the workspace gate result to describe local Git workspace verification

## [0.0.4] - 2026-06-03 - Async Run Queue Worker

### Added
- **Run Queue** - Added a bounded Tokio run queue for HTTP-submitted contracts
- **Background Worker** - Added a server worker that consumes queued runs, marks them `RUNNING`, executes gates, and finalizes evidence/status asynchronously
- **Queued Run Status** - Added `QUEUED` run creation support and tests for queued run persistence

### Changed
- **HTTP Submit Semantics** - Changed `POST /runs` to return `202 Accepted` with `run_id` and `QUEUED` status instead of executing gates inside the request lifecycle
- **Worker Failure Handling** - Marks queued work `FAILED_INTERNAL` when execution or queue handoff fails

## [0.0.3] - 2026-06-03 - Orchestration Lock Boundary Refactor

### Added
- **Orchestrator Lifecycle Helpers** - Added explicit helpers for run record creation, run context construction, final decision derivation, and run finalization
- **Orchestrator Unit Coverage** - Added tests for approval/rejection decision logic, store-independent run context construction, and persisted final status with gate evidence

### Changed
- **Run Contract Flow** - Refactored `run_contract` into a high-level lifecycle sequence so gate execution remains outside SQLite lock scopes
- **Store Concurrency Documentation** - Updated the SQLite connection comment to describe per-operation locking instead of long-running gate lock retention

## [0.0.2] - 2026-06-03 - Contract Workspace Validation Hardening

### Added
- **Contract Validation Rules** - Added validation for safe contract IDs, supported Git repository URLs, and normalized relative scope paths
- **Workspace Containment Check** - Added runtime workspace resolution that rejects contract IDs which escape the configured workspace root

### Changed
- **HTTP Path Extractor Import** - Aliased Axum's path extractor so filesystem path validation can use `std::path::Path` without ambiguity

### Updated
- **Validation Coverage** - Added unit tests for path traversal IDs, unsafe scope paths, repeated scope separators, trailing scope separators, unsupported repo URLs, SSH repo URLs, and workspace containment

## [0.0.1] - 2026-06-03 - Compile Baseline Stabilization

### Added
- **Cargo Lockfile** - Generated `core/Cargo.lock` for reproducible binary crate builds
- **Nested Target Ignore** - Added `**/target/` ignore coverage so `core/target/` build artifacts stay out of git status

### Changed
- **Orchestrator Storage Boundary** - Updated `run_contract` to accept shared database state and lock SQLite only around create, record, and status update operations
- **HTTP Submit Handler** - Removed the long-held database guard across awaited gate execution, allowing the Axum handler future to satisfy framework requirements
- **CLI Execution Path** - Wrapped the single-shot SQLite connection in shared state so CLI and HTTP continue to invoke the same orchestrator entrypoint

### Updated
- **Blocking Gate Closures** - Added explicit result types for Gate 2 workspace setup and Gate 7 test execution closures
- **Process Error Construction** - Replaced `std::io::Error::new(ErrorKind::Other, ...)` with `std::io::Error::other(...)`
- **Source Formatting** - Applied `cargo fmt` across the core crate

### Removed
- **Unused Future Placeholders** - Removed currently-unused error variants and run context fields that prevented `cargo clippy -- -D warnings` from passing

### Fixed
- **Build Failure** - Fixed the private `StoreError` import in the HTTP handlers module
- **Compile Baseline** - Restored successful `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt -- --check`

## [0.0.0] - 2026-06-02 - Blank Bootstrap Build

### Added
- **Layer 0: Build Infrastructure** - Initial `core` crate with `tokio`, `axum`, `rusqlite`, `serde`, `clap`, `wait-timeout` dependencies
- **Layer 1: Domain Models** - `Contract` struct with SHA-40 + scope validation
- **Layer 1: Result Types** - `GateResult`, `ExecutionResult`, `TestMetrics`, `GateOutput` enum for typed gate telemetry
- **Layer 1: Error Hierarchy** - Complete `thiserror` taxonomy: `ValidationError`, `StoreError`, `GitError`, `ProcessError`, `GateError`, `OrchestratorError`, `ContractLoadError`
- **Layer 1: Storage Engine** - SQLite schema: `contracts`, `runs`, `gate_runs` tables with FK constraints
- **Layer 1: Storage Primitives** - `create_run`, `update_run_status`, `record_gate_run`, `fetch_run_summary`, `list_runs` with pagination guards
- **Layer 2: State Machine** - `FinalDecision::Approve/Reject`, `Run` context struct
- **Layer 2: Orchestrator** - `run_contract` coordination loop with fail-fast semantics and DB persistence
- **Layer 3: Process Driver** - `execute_with_timeout` with concurrent stdout/stderr readers, `wait_timeout` kill+reap, panic-safe thread joins
- **Layer 4: Sequential Runner** - 7-gate pipeline: Contract → Workspace → Boundary → Formatting → Clippy → Build → Tests
- **Gate 1**: Static contract schema validation, zero I/O
- **Gate 2**: Workspace materialization via `fs::create_dir_all`, base_sha validation
- **Gate 3**: Change boundary enforcement using `git diff --name-only base_sha HEAD` with scope prefix matching
- **Gate 4**: `cargo fmt --check` with 300s timeout
- **Gate 5**: `cargo clippy -D warnings` with 300s timeout
- **Gate 6**: `cargo build` with 600s timeout
- **Gate 7**: `cargo test --format json` with 1800s timeout, JSON event parsing, `suite_failed` sentinel handling
- **Layer 6: CLI Interface** - `clap` args: `--contract`, `--workspace`, `--database`, `--port` with exit codes 0/2/3
- **Layer 7: HTTP Control Plane** - Axum server with `POST /runs`, `GET /runs/:id`, `GET /runs?status&limit&offset`
- **Layer 8: Administrative API** - Paginated run listing with `RunListItem` DTO, query param validation
- **Observability** - `RUNNING/APPROVED/REJECTED` status tracking, per-gate `duration_ms`, `exit_code`, `test_metrics` persistence
- **Documentation** - Rule 20 comments for Cargo JSON instability, Rule 10 comments for 10MB blob limits
- **Tests** - `test_process_timeout`, `test_scope_boundary`, `test_fetch_run_not_found`, `test_list_runs_pagination`

### Changed
- **StoreError**: Added `QueryFailed` variant for read-side failures, `InvalidParameter` for pagination guards
- **GateOutput**: Added `#[allow(clippy::large_enum_variant)]` to accommodate `ExecutionResult` size
- **main.rs**: Migrated from hardcoded contract to dual-mode CLI/HTTP entrypoint using `clap::Parser`
- **orchestrator**: Refactored to accept `workspace: PathBuf` parameter instead of hardcoded path

### Updates
- **Concurrency Model**: Documented `Arc<Mutex<Connection>>` constraint - writes serialized during 30min test runs. Migration to `sqlx::SqlitePool` scheduled for Layer 9
- **Test Metrics**: `TestMetrics::parse_errors` counter tracks non-JSON lines from `cargo test` unstable output
- **Schema Evolution**: Added `test_passed`, `test_failed`, `test_ignored`, `parse_errors` columns to `gate_runs` table
- **Error Handling**: Replaced `Box<dyn Error>` with concrete `ContractLoadError::ReadFailed/ParseFailed` for typed CLI failures
- **Exit Codes**: Standardized 0=approve, 2=code reject, 3=infra panic for CI triage
- **Logging**: `println!` for orchestration start, `eprintln!` for panics, HTTP bind address logged on server start

### Security
- **Path Traversal**: `runtime_workspace.push(&contract.id)` prevents `../etc/passwd` via Gate 2 `create_dir_all`
- **SQL Injection**: All queries use `rusqlite::params!` prepared statements
- **Resource Limits**: Gate timeouts 300s/600s/1800s prevent infinite hangs. Pagination `limit <= 100` prevents table scans

### Known Limitations
- **Network Isolation**: No Linux namespace/chroot sandbox. Subprocesses inherit host network. Zero-network enforced by convention only
- **Concurrent Reads**: `GET /runs` blocks during active `POST /runs` due to `Mutex<Connection>`
- **Blob Storage**: `stdout/stderr` cloned into SQLite. Artifacts >10MB should use LFS in Gate 8
- **Supply Chain**: No `cargo audit` or `cargo deny` gates. Dependencies not scanned for RUSTSEC advisories

### Dependencies
- tokio 1.38 - Async runtime
- axum 0.7.5 - HTTP server
- rusqlite 0.31.0 - SQLite with bundled lib
- serde 1.0.204 - Serialization
- clap 4.5 - CLI parsing
- wait-timeout 0.2.0 - Subprocess timeout handling
- thiserror 1.0.61 - Error derives
