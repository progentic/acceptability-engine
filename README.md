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

## Installation

Install from a release archive when you want the packaged command-line binary
and bundled operator documentation. Download the archive for your platform from
the GitHub Release, unpack it, and put the `acceptability-engine` binary on
your `PATH`.

```bash
tar -xzf acceptability-engine-v1.0.0-linux-x86_64.tar.gz
cd acceptability-engine-v1.0.0-linux-x86_64
./acceptability-engine --help
```

Build from source when you are developing the engine:

```bash
cd core
cargo build --release --locked
```

The built binary is available at `core/target/release/core`. The release
workflow publishes it as `acceptability-engine`.

## First Run

Create the runtime directories first:

```bash
mkdir -p workspaces artifacts
```

Start the HTTP API for local development:

```bash
AH_WORKSPACE_MODE=local \
AH_SANDBOX_PROFILE=development \
core/target/release/core \
  --workspace ./workspaces \
  --database ./evidence.db \
  --artifact-root ./artifacts \
  --port 8080
```

Check the service:

```bash
curl http://127.0.0.1:8080/health/live
curl http://127.0.0.1:8080/health/ready
```

Run one contract directly from the CLI:

```bash
AH_WORKSPACE_MODE=git \
AH_SANDBOX_PROFILE=development \
core/target/release/core \
  --contract ./contract.json \
  --workspace ./workspaces \
  --database ./evidence.db \
  --artifact-root ./artifacts
```

A contract must identify the admitted Git object with `candidate_sha`:

```json
{
  "id": "example-run",
  "repo_url": "https://github.com/progentic/acceptability-engine.git",
  "base_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "candidate_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "candidate_ref": "refs/pull/123/head",
  "scopes": ["core/src"],
  "requires_human_review": false,
  "admission_policy": {
    "id": "strict-v1",
    "version": 1
  }
}
```

`candidate_ref` is provenance only. The admitted change is the diff from
`base_sha` to `candidate_sha`.

## Workspace Mode

The engine supports two workspace modes.

Use `AH_WORKSPACE_MODE=local` when each contract workspace already exists under
the configured `--workspace` root. The runtime resolves each contract ID to one
child directory and verifies that it is a Git work tree with the requested
history.

Use `AH_WORKSPACE_MODE=git` when the runtime should materialize the workspace.
Git mode clones the contract repository into `--workspace/<contract-id>`,
verifies `origin`, verifies `base_sha`, verifies `candidate_sha`, checks out
`candidate_sha`, verifies `HEAD == candidate_sha`, and Gate 3 evaluates
`base_sha..candidate_sha`.

## Sandbox Profile

Sandbox mode is selected with `AH_SANDBOX_PROFILE`.

Use `AH_SANDBOX_PROFILE=development` for local development. This profile keeps
the runner easy to start on a developer machine. It is not the production
containment boundary.

Use `AH_SANDBOX_PROFILE=kubernetes-restricted` for the documented production
Kubernetes deployment. That profile expects the pod runtime to enforce:

- non-root execution
- no privilege escalation
- dropped Linux capabilities
- RuntimeDefault seccomp
- read-only root filesystem
- explicit writable mounts
- CPU and memory limits
- denied pod egress by default

The Rust runner also applies process hardening, timeout cleanup, bounded output,
and environment scrubbing. The restricted profile is the accepted v1.0
deployment model; it is not a microVM or custom seccomp sandbox.

## Container

Build the local image:

```bash
docker build -t acceptability-engine:local .
```

Run the local Compose stack:

```bash
docker compose up --build
```

Compose uses the `development` sandbox profile with local container hardening.
Use Kubernetes plus `AH_SANDBOX_PROFILE=kubernetes-restricted` for the
production deployment model.

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

Production API-key mode requires `AH_API_KEYS` entries in this form:

```text
token|role|tenant|repo_prefixes
```

The server rejects empty, whitespace-only, and known placeholder API key tokens.

## License & Attribution

Distributed under the MIT License.

Built by the AJENTIC Development Group. Software For All Mankind.
