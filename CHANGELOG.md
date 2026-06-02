# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
