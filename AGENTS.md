# AGENTS.md

## Scope

This file applies to the entire repository unless a more specific AGENTS.md exists in a child directory.

This file is an agent navigation document. It tells agents which project documents control a change. It must not duplicate the full contents of those documents.

## Source of authority

Use the repository documents in this order.

1. `docs/INVARIANTS.md`

   Defines rules that must remain true.

   Read this before changing authority boundaries, run state, gate behavior, evidence persistence, tenant isolation, sandbox behavior, review behavior, or release governance.

2. `docs/ARCHITECTURE.md`

   Defines the system shape.

   Read this before changing module boundaries, APIs, persistence, worker flow, gate execution, UI/API relationships, deployment assumptions, or observability.

3. `docs/PHASEMAP.md`

   Defines roadmap sequencing and phase completion evidence.

   Read this before starting roadmap work, closing a phase, adding a phase, changing phase scope, or reporting phase completion.

4. `docs/CODING_STYLE.md`

   Defines how code must be structured.

   Read this before editing code, tests, scripts, examples, or generated implementation scaffolding.

5. `docs/DEPLOYMENT.md`

   Defines runtime and deployment behavior.

   Read this before changing health checks, metrics, container behavior, Kubernetes manifests, environment variables, runtime paths, or production operation notes.

6. `CHANGELOG.md`

   Records user-visible and governance-relevant changes.

   Update this when behavior, architecture, invariants, deployment, security, phase status, or public interfaces change.

## Do not create competing authority

Do not restate architecture rules in code comments.

Do not restate invariant rules in new roadmap text.

Do not restate coding style rules in feature documents.

Do not create a second roadmap outside `docs/PHASEMAP.md`.

Do not create a second architecture document outside `docs/ARCHITECTURE.md`.

Do not create a second invariant list outside `docs/INVARIANTS.md`.

If a rule belongs in an existing governance document, update that document instead of creating a parallel rule.

## Change discipline

Prefer small, focused changes.

Do not mix unrelated refactors, formatting churn, dependency changes, documentation rewrites, and behavior changes in one task.

Preserve existing public behavior unless the task explicitly requires changing it.

If behavior changes, update the relevant governance document in the same change.

If an invariant changes, update all of the following in the same change:

- `docs/INVARIANTS.md`
- `docs/ARCHITECTURE.md`
- `docs/PHASEMAP.md`
- `CHANGELOG.md`

## Phase discipline

A phase is complete only when the acceptance evidence in `docs/PHASEMAP.md` exists.

Do not mark a phase complete because code was written.

Do not skip documentation updates required by the phase.

Architecture review phases introduce no new product capability.

If implementation deviates from the phase plan, record the deviation in the phase notes and update the affected governance documents.

## Authority boundaries

Rust is the authority boundary for admission decisions, run state, gate execution, security checks, evidence writes, and API behavior.

TypeScript is an operator interface. It must not create authoritative decisions outside the Rust API.

SQLite and filesystem artifacts are evidence stores. They must not silently change admission decisions.

External tools produce evidence. Rust interprets that evidence.

## Platform requirements

Code must be multi-platform unless the architecture or task explicitly constrains the behavior.

Do not tailor behavior to the current local development host.

Avoid assumptions about:

- absolute local paths
- usernames
- shell-specific behavior
- machine-specific environment variables
- filesystem case sensitivity
- network access
- undeclared local tools

Use portable APIs and project-relative paths.

When platform-specific behavior is unavoidable, isolate it behind a small abstraction and cover it with tests or clear validation logic.

## Validation

Run the relevant project checks before reporting completion.

At minimum, run:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

When UI files change, also run the UI build documented by the repository.

When container or deployment files change, also validate the relevant Docker, Compose, or Kubernetes artifacts.

When documentation-only files change, inspect links, paths, headings, and cross-document alignment.

Report:

- what changed
- what validation ran
- what validation could not be run and why
- what documentation was updated
- what acceptance evidence exists, when working against a phase
