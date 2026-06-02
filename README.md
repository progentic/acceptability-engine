<p align="center">
  <h1 align="center">Acceptability Review Engine</h1>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/language-TypeScript-blue?style=flat-square" alt="TypeScript">
  <img src="https://img.shields.io/badge/runtime-Node.js-green?style=flat-square" alt="Node.js">
  <img src="https://img.shields.io/badge/database-SQLite-lightgrey?style=flat-square" alt="SQLite">
  <img src="https://img.shields.io/badge/sandbox-Linux-purple?style=flat-square" alt="Linux Sandbox">
</p>

---

## Product Overview

The Acceptability Review Engine is an automated gatekeeper that checks software for correctness, safety, and reliability before any code is allowed into production. Acting as an uncompromising, zero-trust validation layer, the application ingests precise code alteration boundaries and subjects submitted patches to a rigorous, isolated execution gauntlet. By combining autonomous kernel-level sandboxing with intelligent error diagnostics and real-time streaming instrumentation, the platform empowers engineering teams to eliminate flawed logic, catch breaking mutations, and verify software compliance completely on autopilot.

---

## Core Data Flow Matrix

```text
[ HTTP POST /runs OR CLI --contract ] 
               │
               ▼  [ Consumer Contract Ingestion ]
   ┌───────────────────────────┐
   │  Rust Orchestrator        │ ◄─── Updates DB & Generates Telemetry
   │  (Axum Server / CLI)      │
   └───────────┬───────────────┘
               │
               ▼  [ Materialize Workspace ]
 ┌───────────────────────────────┐
 │  Gate 2: Workspace Setup      │ ─── (fs::create_dir_all, Zero-Network)
 └─────────────┬─────────────────┘
               │
               ▼  [ Sequential Gate Evaluation Pipeline ]
 ┌───────────────────────────────────────────────────────────────────────────────┐
 │ Gate 1: Contract Validation (Static Schema Checks)                            │
 │    │                                                                          │
 │    ▼                                                                          │
 │ Gate 3: Change Boundaries (git diff --name-only vs base_sha)                  │
 │    │                                                                          │
 │    ▼                                                                          │
 │ Gate 4: Code Formatting (cargo fmt --check) ──► 300s timeout                  │
 │    │                                                                          │
 │    ▼                                                                          │
 │ Gate 5: Static Analysis (cargo clippy -D warnings) ──► 300s timeout           │
 │    │                                                                          │
 │    ▼                                                                          │
 │ Gate 6: Compilation (cargo build) ──► 600s timeout                            │
 │    │                                                                          │
 │    ▼                                                                          │
 │ Gate 7: Test Suite (cargo test --format json) ──► 1800s timeout               │
 └────────────────────────────────┬──────────────────────────────────────────────┘
                                  │
         ┌────────────────────────┴────────────────────────┐
         ▼                                                 ▼
[ APPROVE: Admissible Output ]                   [ REJECT: Telemetry Dump ]
  • INSERT SQLite: runs.status='APPROVED'          • INSERT SQLite: runs.status='REJECTED'
  • INSERT gate_runs: 7 rows with metrics          • INSERT gate_runs: partial rows up to failure
  • HTTP 200: {"status":"APPROVED"}                • HTTP 200: {"status":"REJECTED","reason":"Gate N..."}
  • CLI exit 0                                     • CLI exit 2

[ HTTP GET /runs OR GET /runs/:id ]
               │
               ▼  [ Observability Plane ]
   ┌───────────────────────────┐
   │  SQLite Evidence Store    │ ◄─── Arc<Mutex<Connection>>
   │  (contracts, runs,        │      CONCURRENCY: Serialized writes.
   │   gate_runs tables)       │      Layer 8: migrate to sqlx::SqlitePool
   └───────────────────────────┘
```

## Development Status

This repository is Pre-Alpha software and is under active development. For a complete timeline of historical implementations, system refinements, and framework updates, please review the project (CHANGELOG)[CHANGELOG].

## License & Attribution

Distributed under the MIT License.

Built by the AJENTIC Development Group. Software For All Mankind.
