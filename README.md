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

The Acceptability Review Engine is an automated gatekeeper that checks software for correctness, safety, and reliability before any code is allowed into production. Acting as a zero-trust validation layer, the application ingests precise code alteration boundaries and subjects submitted patches to a rigorous execution gauntlet. The Kubernetes deployment combines runtime containment controls with Rust-owned admission decisions, durable evidence, replay, review, and real-time streaming instrumentation.

---

## System Shape

The authoritative system design is documented in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

The admitted object is `candidate_sha`. Contracts identify:

- repository URL
- `base_sha`
- `candidate_sha`
- optional `candidate_ref` provenance
- allowed scopes
- admission policy
- human review requirement

The Rust authority plane validates the contract, materializes the workspace,
executes the sequential gates, records evidence, evaluates policy, and finalizes
the run. The TypeScript UI is an operator interface that reads state and submits
requests through the Rust API.

The evidence store records contracts, runs, attempts, gate runs, policy
evaluations, review decisions, evidence descriptors, final decisions, and audit
events. Filesystem artifacts store larger evidence payloads referenced by
SQLite descriptors.

## Development Status

This repository is Pre-Alpha software and is under active development. For a complete timeline of historical implementations, system refinements, and framework updates, review [CHANGELOG.md](CHANGELOG.md).

## Workspace Mode

The engine supports explicit local workspace selection and Git materialization.

Set `AH_WORKSPACE_MODE=local` to run against workspaces that already exist under the configured `--workspace` root. Each contract ID resolves to a single child directory under that root, and Gate 2 verifies that directory is a local Git work tree with the requested `base_sha`.

Set `AH_WORKSPACE_MODE=git` to clone the contract repository into `--workspace/<contract-id>` before gate execution. Git mode cleans the selected per-run workspace, clones without recursive submodules, verifies `origin`, verifies `base_sha`, verifies `candidate_sha`, checks out `candidate_sha`, verifies `HEAD == candidate_sha`, and Gate 3 evaluates `base_sha..candidate_sha`.

## Sandbox Profile

Set `AH_SANDBOX_PROFILE=development` for local development.

Set `AH_SANDBOX_PROFILE=kubernetes-restricted` for the documented production
Kubernetes profile. That profile expects non-root execution, no privilege
escalation, dropped Linux capabilities, RuntimeDefault seccomp, a read-only root
filesystem, explicit writable mounts, CPU/memory limits, and denied pod egress.

## Browser UI

The repository includes a TypeScript observability UI under `ui/`.

Run the Rust API on port 8080:

```bash
cd core
cargo run -- --workspace ../scratch_workspace --database ../evidence.db --port 8080
```

Run the UI development server:

```bash
cd ui
npm install
npm run dev
```

The UI proxies `/api` to `http://127.0.0.1:8080` by default. Use the API field in the top bar to point at a different compatible server.

## Deployment

Deployment assets are under `Dockerfile`, `compose.yaml`, and `deploy/kubernetes.yaml`.

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for health probes, metrics, container runtime, Kubernetes, and production environment settings.

## License & Attribution

Distributed under the MIT License.

Built by the AJENTIC Development Group. Software For All Mankind.
