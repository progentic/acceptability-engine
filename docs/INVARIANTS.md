# Invariants

## Purpose

This document defines rules that must remain true as the system changes.

An invariant is a rule the code must not violate. It protects the admission boundary. If an invariant becomes inconvenient, change the design explicitly. Do not work around the invariant in code.

## 1. Rust remains the authority boundary

Rust owns all authoritative decisions.

The API, CLI, orchestrator, gate runner, security checks, state transitions, and durable evidence writes must be implemented in Rust.

TypeScript may display and submit information through the API. It must not decide whether a run is approved, rejected, or pending review.

SQLite may store state. It must not contain trigger logic that silently changes the admission decision outside Rust.

## 2. A contract is untrusted input

Every submitted contract must be validated before it is used.

A valid contract must have:

- non-empty contract id
- safe contract id characters
- supported Git repository URL
- 40-character hexadecimal base SHA
- at least one normalized relative scope path
- no absolute scope path
- no parent traversal in scope paths
- no Windows path separators in scope paths

A contract id may select a local workspace only after the id is proven to be a single safe path segment.

## 3. Workspace paths must not escape the configured root

In local workspace mode, the workspace path is always:

```text
workspace_root / contract.id
```

The selected path must remain under the configured workspace root.

No contract field may directly provide an absolute workspace path.

No contract field may cause parent-directory traversal.

## 4. Workspace modes must fail closed

`local` and `git` are the implemented workspace modes.

Unset or empty `AH_WORKSPACE_MODE` means local mode.

`git` mode must materialize the repository under:

```text
workspace_root / contract.id
```

Git materialization must validate the contract before clone, reject unsafe roots and symlink workspace targets, clean only the selected per-run workspace path, clone without recursive submodules, disable Git credential prompts, detach `HEAD` at the requested `base_sha`, and verify the configured `origin` URL.

Unknown modes must fail at startup.

## 5. Gate execution is sequential

Gates must run in the defined order.

A failed gate stops the sequence.

Later gates must not run after an earlier gate fails.

The gate order is:

1. Contract validation
2. Local workspace verification
3. Change boundary check
4. Formatting check
5. Static analysis
6. Build
7. Test execution
8. Supply-chain checks

## 6. Gate numbers are stable public evidence

Gate numbers must remain stable.

Changing gate meaning, reordering gates, or inserting a new gate in the middle changes persisted evidence interpretation. Such a change requires an explicit migration and documentation update.

New gates should be appended unless there is a deliberate evidence-versioning change.

## 7. Rejection is different from internal failure

A rejection means the submitted change failed an expected validation gate.

An internal failure means the engine could not complete its own work.

Expected gate failure produces `REJECTED`.

Infrastructure, store, worker, join, process-runner, or evidence-finalization failure produces `FAILED_INTERNAL` or an error path. It must not be reported as approved.

## 8. Unknown is not approved

Only an explicit pass across all required gates may produce approval.

Missing evidence, skipped gates, missing attempts, parse failures that affect correctness, lost worker results, or ambiguous process results must not produce `APPROVED`.

## 9. Human review suspends final approval

If a contract requires human review and all gates pass, the run status is `PENDING_HUMAN_REVIEW`.

A pending human review run must not create an approved final decision.

A human review workflow must record reviewer identity, decision, reason, and timestamp before converting the run into a terminal approved or rejected state.

Human review approval and rejection must be performed through Rust API endpoints.

Human review must create evidence linked to the persisted review decision.

## 10. Attempts own gate records

Gate records belong to attempts, not directly to runs.

A run may have multiple attempts.

Run summaries may show the latest attempt, but the underlying attempt history must remain queryable.

Attempt numbering must be deterministic within a run.

## 11. Final decisions are unique per run

A run may have at most one persisted final decision.

`APPROVED` and `REJECTED` are terminal final decisions.

`PENDING_HUMAN_REVIEW` is not a final decision.

Changing this rule requires a new state model and migration.

## 12. Evidence must be durable before it is referenced

A filesystem artifact descriptor may be written to SQLite only after the artifact bytes are written successfully.

Each artifact descriptor must include:

- kind
- label
- storage URI
- SHA-256 hash
- byte length
- content type
- summary

Evidence linked to a gate must identify the run, attempt, and gate run when those identities exist.

Evidence linked to a human review decision must identify the run and review decision.

## 13. SQLite finalization must be transactional

For normal gate completion, the following writes must finalize together:

- gate run records
- evidence bundle descriptors
- attempt status
- run status
- final decision when terminal

Partial finalization must not present a completed run without its supporting evidence.

## 14. Tenant boundaries apply before store reads

HTTP read and submit paths must authorize before accessing tenant-scoped run data.

Tenant-scoped helpers must be used for public HTTP paths.

A caller from one tenant must not be able to read runs, attempts, gates, evidence, or audit-derived state from another tenant.

Authenticated cross-tenant resource access must stay opaque to the caller and must be recorded as a denied audit event when the server has enough caller context.

## 15. Repository policy applies before submission

A submitter may submit only contracts whose repository URL matches the caller identity policy.

Repository policy must be enforced before creating queued run records.

A denied submission must be auditable.

## 16. Security denials are evidence

Authentication failures, authorization failures, rate limit failures, quota failures, and repository policy failures must be recorded as audit events when the server has enough context to write an event.

Audit events must include outcome and reason.

## 17. Progress streams are observational

WebSocket progress events may report current run execution state.

Progress events must not create, approve, reject, retry, cancel, or otherwise mutate runs.

Reconnect replay is an operator visibility aid. Durable evidence remains in SQLite and the artifact store.

## 18. Queues are bounded

The run queue must remain bounded.

If work cannot be queued, the run must not silently disappear.

A run that was created but cannot be queued must be marked as failed internal or returned as unavailable through a clear error path.

## 19. Blocking work must not block async executor threads

SQLite access, Git commands, Cargo commands, filesystem-heavy work, and other blocking operations must run through blocking boundaries.

Async request handlers and worker futures must not hold synchronous locks or perform blocking process execution directly.

## 20. Process output is bounded

Gate stdout and stderr capture must have a hard size limit.

Oversized output is an engine-visible failure condition.

Read APIs may return previews, but they must expose truncation flags when previews are shortened.

## 21. Gate command environment is controlled

Gate commands must not inherit the caller environment by default.

The gate process environment must clear inherited variables and set only the required command environment.

Network-dependent Cargo and Git behavior must be disabled unless a later design explicitly adds controlled network access.

## 22. Timeouts are mandatory for external commands

Every external gate command must have a timeout.

Timeout cleanup must terminate the process and descendants where the platform supports it.

A timeout must not be interpreted as a passing gate.

## 23. The change boundary is scope-based

Gate 3 must compare changed files from `base_sha` to `HEAD`.

Every changed file must fall under one of the contract scopes.

A path such as `src/api_backup/file.rs` must not match scope `src/api`.

## 24. Supply-chain checks are part of admission

The supply-chain gate is part of the admission sequence.

`cargo deny check` must pass before `cargo audit` success can matter.

A failed supply-chain command rejects the run.

## 25. Metrics are operational signals, not authority

Prometheus metrics and logs may help operators understand the system.

They do not determine run status.

Durable store state and evidence records are the authoritative record.

## 26. The UI is non-authoritative

The browser UI may call API endpoints and render their responses.

The UI must not synthesize gate results, rewrite statuses, suppress failures, or mark review decisions outside the API.

Polling is an observation mechanism. It is not a state transition mechanism.

## 27. API response models must remain explicit

Public API models must use explicit fields.

Do not return unstructured blobs when the domain has known fields.

Typed identifiers should not be interchangeable inside Rust code.

## 28. Migrations must preserve existing evidence

Schema changes must preserve existing contracts, runs, attempts, gate records, evidence bundles, final decisions, and audit events unless a deliberate destructive migration is documented.

Legacy migration code must attach old gate rows to deterministic attempt records.

## 29. Tests must cover negative paths

Any new authority path requires tests for failure behavior.

At minimum, tests should cover invalid input, denied authorization, missing records, failed gates, and internal error behavior when that path can produce those outcomes.

## 30. Documentation must match executable behavior

Architecture and invariant documentation must be updated when the gate sequence, API surface, state model, workspace model, security model, or persistence model changes.

The README must not claim a capability that the code fails closed on.

## 31. Coding style is part of the architecture

Code must follow `docs/CODING_STYLE.md`.

Authority code should use clear names, small functions, explicit state, typed errors, guard clauses, and direct control flow.

Avoid hidden state, unnecessary abstraction, deep nesting, swallowed errors, unbounded spawning, and blocking async executor paths.

## Compliance review record

Phase 25 invariant compliance evidence is recorded in `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md`.
That report is review evidence; this document remains the invariant authority.
