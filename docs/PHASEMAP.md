# PHASEMAP.md

## Purpose

This document defines the implementation roadmap for the Acceptability Review Engine.

A phase is complete only when:

* All phase goals are implemented.
* Required acceptance evidence exists.
* Required tests exist.
* Required documentation exists.
* Required validation commands pass.

A phase is not complete because code was written.

Evidence determines completion.

==========================
PHASEMAP GOVERNANCE RULES
=========================

1. Every phase must produce acceptance evidence.

2. Every phase must update documentation when behavior changes.

3. Any change to system behavior requires review of:

   * ARCHITECTURE.md
   * INVARIANTS.md
   * PHASEMAP.md
   * CHANGELOG.md

4. Every fifth phase is an Architecture Governance Review.

5. Architecture Review phases introduce no new product capability.

6. Production release requires:

   * zero unresolved architecture deviations
   * zero unresolved invariant violations
   * successful disaster recovery validation
   * successful security assessment
   * successful replay validation
   * successful release readiness review

7. A phase is complete only when evidence exists.

8. Unknown is not complete.

9. Missing evidence is not complete.

10. Notes and deviations must be recorded before phase closure.

========================
STANDARD PHASE TEMPLATE
========================

Task

Goal

Non-Goals

Steps

Acceptance Evidence

Documentation Updates

Commands Ran

Summary

Notes / Deviations

================================
PHASE 22 HUMAN REVIEW AUTHORITY
================================

Task

Implement complete human review workflow.

Goal

Introduce an authoritative reviewer decision boundary.

Non-Goals

* Multi-stage approvals
* Delegation chains
* External identity providers

Steps

1. Review decision model
2. Review persistence model
3. Approve endpoint
4. Reject endpoint
5. Reviewer audit trail
6. Decision evidence linkage

Acceptance Evidence

* `review_decisions` table exists in migration `0009_review_decisions.sql`.
* Reviewer actor, reviewer role, tenant, decision, reason, and decision timestamp are persisted.
* `POST /runs/:id/review/approve` finalizes pending review runs as `APPROVED`.
* `POST /runs/:id/review/reject` finalizes pending review runs as `REJECTED`.
* Reviewer authorization is enforced through the `reviewer` and `admin` roles.
* Submitter review attempts are rejected and audited.
* Successful review decisions are audited.
* Review evidence bundles link back to the persisted review decision.
* Store tests cover approval, rejection, review state validation, and evidence linkage.
* Handler/security tests cover approval, rejection, authorization, and audit event generation.

Documentation Updates

* `ARCHITECTURE.md` documents the human review authority boundary.
* `INVARIANTS.md` documents Rust-authoritative review and evidence linkage.
* `API.md` documents review roles, approval endpoint, and rejection endpoint.
* `DEPLOYMENT.md` documents the `reviewer` API key role.
* `CHANGELOG.md` records version `0.0.23 - Human Review Authority`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`
* `npm run build`

Summary

Human review is now a first-class authority boundary. Reviewer decisions are tenant-scoped, role-gated, audited, transactionally finalized, and linked to review evidence.

Notes / Deviations

* Multi-stage approval, delegation chains, and external identity providers remain out of scope for this phase.

==============================
PHASE 23 WEBSOCKET STREAMING
==============================

Task

Implement live execution telemetry.

Goal

Provide real-time run visibility.

Non-Goals

* Remote execution
* Interactive terminals
* Browser shells

Steps

1. Event schema
2. Progress publisher
3. WebSocket endpoint
4. Gate state streaming
5. Final decision events
6. Reconnect handling

Acceptance Evidence

* `GET /runs/:id/progress` upgrades to a WebSocket stream.
* Progress events include ordered `sequence`, `run_id`, `created_at`, and event type fields.
* Event inventory covers `queued`, `started`, `attempt_started`, `gate_started`, `gate_finished`, `finalized`, and `failed_internal`.
* Gate execution publishes start and finish events in sequential gate order.
* Reconnect replay supports `?after=<sequence>` against the recent bounded in-memory event buffer.
* WebSocket integration test validates replay plus live event delivery over a local Axum listener.
* Progress hub tests validate event ordering and replay filtering.
* The TypeScript API client exposes a typed progress WebSocket connection.
* The browser dashboard subscribes to the selected live run and refreshes on progress events.

Documentation Updates

* `ARCHITECTURE.md` documents the progress stream and reconnect behavior.
* `INVARIANTS.md` records that progress streams are observational only.
* `API.md` documents the WebSocket route, replay query, and event inventory.
* `CHANGELOG.md` records version `0.0.24 - WebSocket Progress Streaming`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`
* `npm run build`

Summary

Operators receive live execution visibility through ordered, reconnectable WebSocket progress events.

Notes / Deviations

* Progress replay is a bounded in-memory reconnect aid. If older events age out, clients continue from available events and durable evidence remains in SQLite and filesystem artifacts.

==============================
PHASE 24 GIT MATERIALIZATION
=============================

Task

Implement repository acquisition.

Goal

Support AH_WORKSPACE_MODE=git.

Non-Goals

* Arbitrary credentials
* Submodule recursion
* Persistent repository mutation

Steps

1. Clone
2. Checkout
3. Verification
4. Cleanup
5. Isolation validation

Acceptance Evidence

* `AH_WORKSPACE_MODE=git` is accepted during startup.
* CLI and HTTP worker execution use the same workspace materialization helper.
* Git mode clones the contract repository into `workspace_root / contract.id`.
* Git mode cleans only the selected per-run workspace before cloning.
* Git mode clones without recursive submodules.
* Git mode rejects unsafe roots and symlink workspace targets before cleanup.
* Git mode detached `HEAD` at the requested `base_sha` at Phase 21 closure; the D25-001 closure track later changed Git mode to detach `HEAD` at `candidate_sha`.
* Git mode verifies `origin` matches the contract repository URL.
* Tests cover clone, checkout, cleanup, malicious path rejection, unsafe root rejection, symlink rejection, detached HEAD validation, and repository origin validation.

Documentation Updates

* `ARCHITECTURE.md` documents local and Git workspace modes.
* `INVARIANTS.md` documents Git materialization safety requirements.
* `DEPLOYMENT.md` documents `AH_WORKSPACE_MODE=git`.
* `README.md` documents Git materialization usage.
* `CHANGELOG.md` records version `0.0.25 - Git Materialization`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Engine can safely materialize repositories into isolated per-run workspaces before gate execution.

Notes / Deviations

* Git mode does not recurse submodules and does not add credential-provider behavior.

=================================
PHASE 25 ARCHITECTURE REVIEW I
=================================

Task

Validate roadmap alignment.

Goal

Verify implementation remains aligned with architecture.

Non-Goals

* New features
* New endpoints
* New persistence models

Steps

1. Architecture review
2. Invariant review
3. API review
4. Security review
5. Persistence review

Acceptance Evidence

* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the architecture review report.
* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the invariant compliance report.
* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the API inventory.
* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the persistence inventory.
* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the deviation register.
* `docs/reviews/PHASE25_ARCHITECTURE_REVIEW.md` contains the roadmap impact assessment.

Documentation Updates

* `ARCHITECTURE.md` links to the Phase 25 review record.
* `INVARIANTS.md` links to the Phase 25 compliance review record.
* `PHASEMAP.md` records Phase 25 acceptance evidence.
* `CHANGELOG.md` records version `0.0.26 - Architecture Review I`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Architecture remains aligned with documented Phase 25 deviations.

Notes / Deviations

* `D25-001` remained open at Phase 25 closure: Git mode materialized `base_sha` but did not yet model candidate-change acquisition. It is closed by the candidate acquisition implementation track below.
* `D25-002` remains open: process execution is hardened but not a complete adversarial kernel sandbox.
* `D25-003` is an accepted limitation: WebSocket replay is bounded and in memory.
* `D25-004` is an accepted limitation: API-key security does not include external identity providers.
* Phase 25 roadmap impact assessment identifies where deviations must be rechecked before v1.0.

=================================
PHASE 26 MULTI-TENANT HARDENING
=================================

Task

Strengthen tenant isolation.

Goal

Enforce strict tenant boundaries.

Non-Goals

* Tenant federation
* Shared visibility

Steps

1. Query audit
2. Boundary audit
3. Isolation testing
4. Negative-path testing

Acceptance Evidence

* `docs/reviews/PHASE26_TENANT_HARDENING.md` contains the query review report.
* `docs/reviews/PHASE26_TENANT_HARDENING.md` contains the boundary validation report.
* Tenant isolation tests cover cross-tenant run status, attempt gates, evidence, review attempts, progress access, run lists, and run summaries.
* Authorization tests cover API-key requirements, role denial, repository policy denial, reviewer authority, and submitter review denial.

Documentation Updates

* `ARCHITECTURE.md` documents opaque cross-tenant responses and denied audit evidence.
* `INVARIANTS.md` requires opaque cross-tenant access and denied audit evidence.
* `CHANGELOG.md` records version `0.0.27 - Multi-Tenant Hardening`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Tenant boundaries are formally enforced and cross-tenant resource attempts produce denied audit evidence.

Notes / Deviations

* `D26-001` is an accepted limitation: tenant federation and shared visibility remain out of scope.
* `D26-002` is an accepted limitation: missing and hidden resources both return `404` to preserve tenant privacy.

============================
PHASE 27 ARTIFACT RETENTION
============================

Task

Evidence lifecycle management.

Goal

Implement retention controls.

Non-Goals

* Evidence mutation
* Silent deletion

Steps

1. Retention policy
2. Cleanup workflow
3. Audit integration
4. Validation

Acceptance Evidence

* Retention tests cover dry-run planning, cutoff filtering, and newer artifact preservation.
* Cleanup tests cover filesystem deletion, missing-file handling, traversal rejection, symlink rejection, and immutable evidence descriptors.
* Audit records are written for dry-run, planned, deleted, and missing retention outcomes.
* `docs/reviews/PHASE27_ARTIFACT_RETENTION.md` contains the artifact lifecycle report.

Documentation Updates

* `ARCHITECTURE.md` documents artifact retention as a CLI workflow that preserves SQLite evidence descriptors.
* `INVARIANTS.md` requires artifact retention to be explicit, audited, and descriptor-preserving.
* `DEPLOYMENT.md` documents dry-run and deletion retention commands.
* `CHANGELOG.md` records version `0.0.28 - Artifact Retention`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Evidence lifecycle is manageable through explicit, audited artifact retention.

Notes / Deviations

* `D27-001` is an accepted limitation: retention is a CLI workflow, not an HTTP API.
* `D27-002` is an accepted limitation: SQLite descriptors remain after artifact bytes are deleted.

========================
PHASE 28 REPLAY ENGINE
========================

Task

Deterministic replay.

Goal

Reconstruct historical runs.

Non-Goals

* Replay modification
* Historical mutation

Steps

1. Replay model
2. Replay API
3. Replay UI
4. Determinism validation

Acceptance Evidence

* Replay tests cover contract, run, attempt, gate, evidence, review decision, and final decision output.
* Determinism tests compare repeated replay output after normalizing `generated_at`.
* Replay demonstration covers the `--replay-run-id` CLI path.
* Replay evidence validation covers missing artifact indicators without descriptor mutation.
* `docs/reviews/PHASE28_REPLAY_ENGINE.md` contains the replay contract and validation evidence.

Documentation Updates

* `ARCHITECTURE.md` documents replay as a read-only reconstruction workflow.
* `INVARIANTS.md` requires replay to avoid execution and mutation.
* `DEPLOYMENT.md` documents replay CLI usage.
* `CHANGELOG.md` records version `0.0.29 - Replay Engine`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Historical execution is reproducible through read-only JSON replay reports.

Notes / Deviations

* `D28-001` is an accepted limitation: replay is exposed through the CLI first, not HTTP or UI.
* `D28-002` is an accepted limitation: replay reports artifact presence but does not read artifact bytes.

=====================================================================
PHASE 29
ADMISSION POLICY ENGINE
=======================

Task

Policy-based acceptance.

Goal

Allow configurable admission criteria.

Non-Goals

* Arbitrary scripting
* Runtime code execution

Steps

1. Policy schema
2. Evaluation order
3. Evaluator
4. Evidence linkage
5. Replay inclusion
6. Validation

Acceptance Evidence

* Contract schema accepts optional `admission_policy` and defaults to `strict-v1`.
* Policy evaluation order is documented as gate result, policy evaluation, human review requirement, final decision.
* `policy_evaluations` table exists in migration `0010_admission_policy.sql`.
* Policy evaluation tests cover required gates, failed gates, parse-error limits, unsupported policies, deterministic gate ordering, and attempts to weaken mandatory gates.
* Orchestrator tests cover policy-driven approval, rejection, human-review suspension, policy trace persistence, and transaction rollback.
* Replay output includes policy evaluations in deterministic order.
* `docs/reviews/PHASE29_ADMISSION_POLICY_ENGINE.md` contains the policy scope, evaluation order, evidence model, validation evidence, and deviations.

Documentation Updates

* `ARCHITECTURE.md` documents the policy decision boundary.
* `INVARIANTS.md` documents the policy-before-review rule and transactional policy trace persistence.
* `API.md` documents the optional `admission_policy` contract field.
* `CHANGELOG.md` records version `0.0.30 - Admission Policy Engine`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Admission is now policy driven through contract-scoped declarative policy evaluation.

Notes / Deviations

* `D29-001` is an accepted limitation: policy is contract-scoped and declarative, without server-global registry or dynamic scripting.
* `D29-002` is an accepted limitation: policy trace evidence is stored in SQLite, not as filesystem artifacts.

=====================================================================
PHASE 30
ARCHITECTURE REVIEW II
======================

Task

Second architecture governance review.

Goal

Prevent architectural drift.

Non-Goals

* Feature implementation

Steps

1. Architecture review
2. Invariant review
3. Policy authority review
4. Replay completeness review
5. Security boundary review
6. Persistence inventory update
7. Deviation register update
8. Roadmap impact assessment

Acceptance Evidence

* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the architecture review report.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the invariant compliance report.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the policy authority review.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the replay completeness review.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the security boundary review.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the persistence inventory update.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the deviation register update.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` contains the roadmap impact assessment.
* `docs/reviews/PHASE30_ARCHITECTURE_REVIEW.md` confirms whether any new invariant is needed before Phase 31.

Documentation Updates

* `ARCHITECTURE.md` links to the Phase 30 review record.
* `INVARIANTS.md` links to the Phase 30 invariant review record.
* `PHASEMAP.md` records Phase 30 acceptance evidence.
* `CHANGELOG.md` records version `0.0.31 - Architecture Review II`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Architecture remains coherent after policy, replay, retention, review, progress, and tenant hardening.

Notes / Deviations

* `D25-001` remained open and release-critical at Phase 30 closure. It is closed by the candidate acquisition implementation track below.
* `D25-002` remains open: process execution is hardened but not a complete adversarial kernel sandbox.
* `D25-003`, `D25-004`, `D28-001`, `D28-002`, `D29-001`, and `D29-002` remain accepted limitations.
* No new invariant is required before Phase 31; sandbox hardening is expected to update or add a sandbox invariant.

=====================================================================
PHASE 31
SANDBOX HARDENING
=================

Task

Production-grade containment.

Goal

Strengthen execution isolation.

Non-Goals

* Virtual machines
* Desktop execution

Acceptance Evidence

* Sandbox profile validation tests cover default, restricted, and unknown profiles.
* Containment tests cover restricted profile kernel-control declarations.
* Escape tests cover minimal command environment, proxy stripping, timeout cleanup, output caps, and deployment privilege restrictions.
* Resource limit tests cover process timeout, output limits, process-group cleanup, and `sandbox_runner` to `resource_limits` wiring.
* `docs/reviews/PHASE31_SANDBOX_HARDENING.md` contains the namespace, filesystem, network, syscall, and resource model.
* `docs/reviews/PHASE31_SANDBOX_HARDENING.md` contains the sandbox validation report.
* Kubernetes deployment uses non-root execution, no privilege escalation, dropped capabilities, RuntimeDefault seccomp, read-only root filesystem, explicit writable mounts, resource limits, and deny-all egress.
* Compose deployment uses local hardening with dropped capabilities, no-new-privileges, read-only root filesystem, and explicit writable mounts.

Documentation Updates

* `ARCHITECTURE.md` documents the sandbox profiles and containment boundary.
* `INVARIANTS.md` documents sandbox profile fail-closed behavior.
* `DEPLOYMENT.md` documents `AH_SANDBOX_PROFILE` and the Kubernetes restricted profile.
* `PHASEMAP.md` records Phase 31 acceptance evidence.
* `CHANGELOG.md` records version `0.0.32 - Sandbox Hardening`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`
* Compose YAML shape validation through Ruby YAML
* Kubernetes manifest shape validation through Ruby YAML

Summary

Sandboxing has a documented production profile backed by deployment controls and Rust runner hardening.

Notes / Deviations

* `D25-002` is narrowed for the documented `kubernetes-restricted` profile; full closure requires runtime enforcement validation on Kubernetes.
* `D31-001` is an accepted limitation: `development` is not production containment.
* `D31-002` is an accepted limitation: non-Kubernetes production deployments must provide equivalent controls.
* `D31-003` is an accepted limitation: Git materialization with denied egress needs a future controlled egress design.
* `docker compose config` could not be run because Docker is unavailable in the local environment. Compose YAML was shape-validated by Ruby. Full Compose validation remains required where Docker is installed.

=====================================================================
PHASE 32
OPERATIONAL READINESS
=====================

Task

Document operator procedures.

Goal

Make the runtime operable by a human without changing admission authority.

Non-Goals

* New admission behavior
* Automated backup validation
* Disaster recovery exercise

Acceptance Evidence

* `docs/OPERATIONS.md` indexes operational procedures, monitoring inventory, metrics inventory, and alert definitions.
* `docs/runbooks/startup.md` documents startup checks, Compose startup, Kubernetes startup, readiness checks, and failure handoff.
* `docs/runbooks/shutdown.md` documents evidence-preserving shutdown.
* `docs/runbooks/retention.md` documents dry-run and live artifact retention procedures.
* `docs/runbooks/replay.md` documents read-only replay procedures and output handling.
* `docs/runbooks/incident_response.md` documents first response, triage, escalation evidence, and recovery handoff.
* `docs/runbooks/restore.md` documents the manual restore procedure that Phase 33 must validate.
* `docs/reviews/PHASE32_OPERATIONAL_READINESS.md` records the operator procedure inventory, monitoring inventory, alert inventory, authority review, deferred validation, and command evidence.
* Alert definitions map liveness, readiness, restart, denial-spike, artifact-pressure, replay, and restore conditions to runbooks.
* Monitoring validation maps health endpoints, metrics, probes, audit events, and replay reports to operator meaning.

Documentation Updates

* `DEPLOYMENT.md` links to the operations index and clarifies metrics as observational.
* `PHASEMAP.md` records Phase 32 acceptance evidence.
* `docs/reviews/PHASE32_OPERATIONAL_READINESS.md` records Phase 32 validation evidence.
* `CHANGELOG.md` records version `0.0.33 - Operational Readiness`.

Commands Ran

* `cargo fmt -- --check`

* `cargo clippy -- -D warnings`

* `cargo test`

Summary

Operational readiness now has runbooks, alert definitions, monitoring inventory, metrics inventory, and manual recovery procedures.

Notes / Deviations

* Backup and restore validation is intentionally deferred to Phase 33.
* Disaster recovery exercise evidence is intentionally deferred to Phase 34.

=====================================================================
PHASE 33
BACKUP / RESTORE VALIDATION
===========================

Task

Validate backup evidence.

Goal

Produce a reusable recovery fixture and backup evidence for Phase 34.

Non-Goals

* Destructive restore exercise
* Deployment rebuild
* External backup tooling

Acceptance Evidence

* `docs/runbooks/backup.md` documents backup procedure, artifact shape, inventory, integrity validation, and restore prerequisites.
* `docs/runbooks/restore.md` references the backup artifact shape.
* `docs/reviews/PHASE33_BACKUP_RESTORE_VALIDATION.md` records the backup validation report.
* `backup_validation_creates_reusable_recovery_fixture` creates fixture run history, writes a pre-backup replay baseline, backs up SQLite and artifacts, and validates backup inventory hashes.

Documentation Updates

* `DEPLOYMENT.md` links to the backup runbook.
* `OPERATIONS.md` links the backup runbook and alert route.
* `PHASEMAP.md` records Phase 33 acceptance evidence.
* `CHANGELOG.md` records version `0.0.34 - Backup Restore Validation`.

Commands Ran

* `cargo test backup_validation_creates_reusable_recovery_fixture`
* `cargo test sandbox_runner_invokes_resource_limit_configuration`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`

Summary

Backup validation now has a runbook, restore prerequisites, backup inventory expectations, and a reusable recovery fixture for Phase 34.

Notes / Deviations

* The backup fixture is test-scoped and does not add production backup tooling.
* External backup tooling remains deployment-specific.

=====================================================================
PHASE 34
DISASTER RECOVERY VALIDATION
============================

Task

Validate evidence-store disaster recovery.

Goal

Prove restored SQLite and artifact evidence can reproduce historical replay output after destructive loss of the original store.

Non-Goals

* Live Kubernetes destruction
* Cloud snapshot orchestration
* External backup tooling

Acceptance Evidence

* `docs/runbooks/disaster_recovery.md` documents the recovery checklist, verification commands, success criteria, and postmortem inputs.
* `docs/reviews/PHASE34_DISASTER_RECOVERY_VALIDATION.md` records the DR exercise report.
* `docs/reviews/PHASE34_DISASTER_RECOVERY_VALIDATION.md` records the recovery timing report.
* `docs/reviews/PHASE34_DISASTER_RECOVERY_VALIDATION.md` records the postmortem review.
* `disaster_recovery_restore_consumes_recovery_fixture` consumes the Phase 33 recovery fixture, deletes the live SQLite and artifact stores, restores from backup, and verifies normalized replay equality.

Documentation Updates

* `DEPLOYMENT.md` links to the disaster recovery runbook.
* `OPERATIONS.md` links the disaster recovery runbook and alert route.
* `PHASEMAP.md` records Phase 34 acceptance evidence.
* `CHANGELOG.md` records version `0.0.35 - Disaster Recovery Validation`.

Commands Ran

* `cargo test disaster_recovery_restore_consumes_recovery_fixture`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`

Summary

Disaster recovery now has a runbook and deterministic validation that restored evidence stores reproduce historical replay output from the Phase 33 fixture.

Notes / Deviations

* The automated exercise validates local evidence-store recovery, not live Kubernetes rebuild.

=====================================================================
PHASE 35
RELEASE READINESS REVIEW
========================

Task

Formal release gate.

Goal

Verify release eligibility.

Non-Goals

* New functionality

Acceptance Evidence

* `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` records the release readiness report.
* `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` records the unresolved issue inventory.
* `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` records the security review inventory.
* `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` records the architecture review inventory.
* `docs/reviews/PHASE35_RELEASE_READINESS_REVIEW.md` records the operational review inventory.
* `docs/reviews/CANDIDATE_ACQUISITION_ARCHITECTURE.md` defined `candidate_sha` as the selected admitted object for D25-001 closure.

Documentation Updates

* `ARCHITECTURE.md` recorded D25-001 as a release-critical gap and documented `candidate_sha` as the selected admitted object.
* `INVARIANTS.md` recorded that mutable refs must not become admission authority.
* `PHASEMAP.md` records Phase 35 acceptance evidence.
* `CHANGELOG.md` records version `0.0.37 - Release Readiness Review`.

Commands Ran

* `git diff --check`

Summary

Phase 35 completed the release-readiness review. At Phase 35 closure, the project was not production-release ready until D25-001 candidate acquisition was implemented with `candidate_sha` as the admitted object.

Notes / Deviations

* D25-001 was blocking at Phase 35 closure and is closed by the candidate acquisition implementation track below.
* D25-002 is narrowed by Phase 31 and remains a managed risk for Phase 37 security assessment.

=====================================================================
PHASE 36
PERFORMANCE VALIDATION
======================

Acceptance Evidence

* `docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md` records the load testing report.
* `docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md` records the concurrency testing report.
* `docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md` records the queue saturation report.
* `docs/reviews/PHASE36_PERFORMANCE_VALIDATION.md` records the storage performance report.

Documentation Updates

* `DEPLOYMENT.md` links to the Phase 36 performance baseline and documents current queue, worker, and storage model limits.
* `PHASEMAP.md` records Phase 36 acceptance evidence.
* `CHANGELOG.md` records version `0.0.38 - Performance Validation`.

Commands Ran

* Local HTTP read-load smoke: 600 health/readiness/metrics requests, 8 workers, 0 failures
* `cargo test creates_bounded_run_queue`
* `cargo test pooled_connection_reuses_file_backed_store`
* `cargo test creates_production_query_indexes`
* `cargo test rejects_requests_above_limit`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`
* `git diff --check`

Summary

Phase 36 establishes baseline local performance evidence for read availability, queue behavior, request limiting, pooled SQLite access, and indexed storage queries.

Notes / Deviations

* Phase 36 is not a production maximum-throughput benchmark.
* No long-running soak test or multi-pod Kubernetes load test was executed.
* Queue depth is not yet exported as a metric.

=====================================================================
CANDIDATE ACQUISITION IMPLEMENTATION TRACK
D25-001 CLOSURE
================

Task

Close the release-critical admitted-object gap identified by Phase 25, Phase 30, and Phase 35.

Goal

Make `candidate_sha` the first-class admitted object for Git-mode admission.

Non-Goals

* Patch upload admission
* Archive admission
* Pull-request-number admission
* Mutable-ref admission authority

Acceptance Evidence

* Contracts require and validate `candidate_sha`.
* Contracts preserve optional `candidate_ref` as provenance metadata only.
* SQLite contracts persist `candidate_sha`, optional `candidate_ref`, and `scopes_json`.
* Legacy contract rows are migrated by backfilling `candidate_sha = base_sha` and `scopes_json = []`.
* Git materialization validates `base_sha`, validates `candidate_sha`, checks out `candidate_sha`, and verifies workspace `HEAD == candidate_sha`.
* Gate 3 evaluates `base_sha..candidate_sha`.
* Replay output includes `candidate_sha`, optional `candidate_ref`, and scopes.
* Policy evaluation trace includes candidate identity.
* Browser UI submission and detail models expose candidate identity.
* `docs/reviews/CANDIDATE_ACQUISITION_ARCHITECTURE.md` records D25-001 closure.

Documentation Updates

* `ARCHITECTURE.md` records candidate-SHA admission as implemented.
* `INVARIANTS.md` records the candidate-based change boundary.
* `API.md` documents `candidate_sha` and optional `candidate_ref`.
* `PHASEMAP.md` records D25-001 closure evidence.
* `CHANGELOG.md` records version `0.0.39 - Candidate SHA Admission Boundary`.

Summary

D25-001 is closed for commit-SHA candidate admission. The authoritative admitted object is `candidate_sha`; the admission boundary includes `repo_url`, `base_sha`, `candidate_sha`, scopes, and admission policy. `candidate_ref` remains provenance metadata only.

Notes / Deviations

* Legacy rows are preserved by setting missing `candidate_sha` to `base_sha` and missing scopes to `[]`; that migration is compatibility evidence, not proof that legacy rows represented remote candidate admission.
* Patch, archive, and pull-request-number admission remain out of scope.

=====================================================================
PHASE 37
SECURITY ASSESSMENT
===================

Acceptance Evidence

* `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md` records the threat model.
* `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md` records the source-grounded penetration testing report.
* `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md` records the dependency assessment.
* `docs/reviews/PHASE37_SECURITY_ASSESSMENT.md` records the remediation inventory.

Documentation Updates

* `ARCHITECTURE.md` records the Phase 37 review artifact.
* `PHASEMAP.md` records Phase 37 acceptance evidence.
* `CHANGELOG.md` records version `0.0.40 - Security Assessment`.

Commands Ran

* `cargo test`
* `cargo audit`
* `cargo deny check`

Summary

Phase 37 validates the core admission security model after D25-001. Candidate SHA authority, repository policy, tenant isolation, review authorization, policy evaluation, retention safety, replay integrity, and audit evidence are coherent.

Notes / Deviations

* D25-002 is residual risk: production containment architecture exists, and Phase 40 must decide whether deployment-enforced Kubernetes containment plus Rust runner hardening is sufficient for release.
* D37-001 is closed: `core/deny.toml` defines advisory, ban, source, and license policy; the local crate declares its MIT license; `docs/reviews/LICENSE_GOVERNANCE.md` records approved licenses and the exception process; and `cargo deny check` passes.
* D37-002 is closed: startup rejects known placeholder API key tokens, including the Kubernetes manifest example token, before the server accepts requests.
* The penetration testing report is source-grounded and local-control based; no third-party dynamic penetration test was executed.
* `INVARIANTS.md` was inspected; no new invariant was required.

==============================
PHASE 38 DOCUMENTATION FREEZE
==============================

Task

Freeze release documentation.

Goal

Produce evidence that documented capabilities, implemented capabilities, and
authoritative documents agree before release-candidate work begins.

Non-Goals

* New runtime behavior
* New API behavior
* New persistence behavior
* New security behavior

Acceptance Evidence

* `docs/reviews/PHASE38_DOCUMENTATION_FREEZE.md` records the documentation audit.
* Capability coverage review records implemented, documented, authoritative document, and evidence status for release-relevant capabilities.
* Documentation coverage review records API, contract, gate, state, evidence, deployment, operator, supply-chain, and README coverage.
* Cross-document consistency review compares architecture, invariants, phase map, deployment, API, operations, license governance, runbooks, and review artifacts.
* Release documentation package is inventoried.

Documentation Updates

* `README.md` removed stale pre-production flow text and now points to the authoritative architecture.
* `CHANGELOG.md` records version `0.0.43 - Documentation Freeze`.

Commands Ran

* `git diff --check`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`
* `cargo deny check`
* `cargo audit --no-fetch --stale`

Summary

Phase 38 freezes the current documentation package. The audit found no
undocumented release-relevant capability, no documented-but-missing current
capability, and no contradictory authoritative document.

Notes / Deviations

* Historical review artifacts intentionally retain the status that was true at their phase closure. Later PHASEMAP entries record closure or reclassification.
* D25-002 remains the only open release-risk item and is carried to Phase 40 as a production governance decision.

============================
PHASE 39 RELEASE CANDIDATE
============================

Task

Assemble release-candidate evidence.

Goal

Produce a formal release candidate evidence binder from completed roadmap
evidence and identify whether the project may proceed to Phase 40.

Non-Goals

* New runtime behavior
* New API behavior
* New persistence behavior
* New security behavior
* New deployment behavior

Acceptance Evidence

* `docs/reviews/PHASE39_RELEASE_CANDIDATE.md` records the release candidate evidence binder.
* Release candidate identity is `v0.0.44-rc.1`.
* Evidence coverage matrix aggregates Phases 25, 30, 32, 33, 34, 35, 36, 37, and 38.
* Validation inventory records final validation commands and results.
* Security, documentation, and operational inventories identify no open blocker except D25-002.
* D25-002 is explicitly deferred to Phase 40 Production Governance Review.

Documentation Updates

* CHANGELOG.md

Commands Ran

* `git diff --check`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`
* `cargo deny check`
* `cargo audit --no-fetch --stale`

Summary

Phase 39 creates the release-candidate evidence binder for `v0.0.44-rc.1`.
The project may proceed to Phase 40 because no implementation, documentation,
security remediation, operational, performance, replay, retention, recovery, or
license-governance blocker remains open.

Notes / Deviations

* D25-002 remains open and is carried to Phase 40 as the sole production governance decision.

=======================================
PHASE 40 PRODUCTION GOVERNANCE REVIEW
======================================

Task

Final governance review.

Goal

Validate production release criteria.

Non-Goals

* New functionality

Acceptance Evidence

* `docs/reviews/PHASE40_PRODUCTION_GOVERNANCE_REVIEW.md` records the production governance decision.
* Release candidate `v0.0.44-rc.1` is reviewed.
* D25-002 is evaluated against current controls, residual risks, and deployment assumptions.
* D25-002 is accepted as v1.0 residual risk for the documented deployment model.
* Phase 41 readiness determination records that no open release blocker remains.

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* PHASEMAP.md
* CHANGELOG.md

Commands Ran

* `git diff --check`
* `cargo fmt -- --check`
* `cargo clippy -- -D warnings`
* `cargo test`
* `cargo deny check`
* `cargo audit --no-fetch --stale`
* `docker --version`
* `docker-compose --version`
* `docker-compose config`
* `docker compose version`
* `docker info`

Summary

Phase 40 accepts D25-002 as v1.0 residual risk for controlled Kubernetes
deployments using the `kubernetes-restricted` profile plus Rust runner
hardening. Phase 41 may proceed.

Notes / Deviations

* D25-002 acceptance is scoped to the documented deployment assumptions.
* Local Docker client and standalone Docker Compose exist; Docker daemon availability was not present during supplemental validation.
* Stronger isolation remains future hardening, not a v1.0 release blocker under the accepted deployment model.

===================================
PHASE 41 PRODUCTION RELEASE (v1.0)
===================================

Task

Release v1.0.

Goal

Declare production readiness.

Acceptance Evidence

* v1.0 release tag
* production release checklist
* successful CI validation
* successful DR validation
* successful replay validation
* successful security validation
* successful operational validation

Required Documentation

* ARCHITECTURE.md
* INVARIANTS.md
* CODING_STYLE.md
* DEPLOYMENT.md
* PHASEMAP.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

docker build .

docker compose config

Summary

Acceptability Engine v1.0 production release.

Notes / Deviations

None unresolved.
