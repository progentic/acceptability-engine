# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
