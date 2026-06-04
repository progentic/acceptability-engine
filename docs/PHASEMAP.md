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

* review_decisions table
* reviewer identity persisted
* decision timestamp persisted
* approval tests
* rejection tests
* authorization tests
* audit events generated
* evidence linked to review record

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* API documentation
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Human review becomes a first-class authority boundary.

Notes / Deviations

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

* websocket integration tests
* event ordering validation
* reconnect validation
* live run demonstration
* telemetry event inventory

Documentation Updates

* ARCHITECTURE.md
* API documentation
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Operators receive live execution visibility.

Notes / Deviations

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

* clone tests
* checkout tests
* cleanup tests
* malicious path tests
* detached HEAD validation
* repository origin validation

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* DEPLOYMENT.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Engine can safely materialize repositories.

Notes / Deviations

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

* architecture review report
* invariant compliance report
* API inventory
* persistence inventory
* deviation register

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* PHASEMAP.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Architecture remains aligned.

Notes / Deviations

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

* tenant isolation tests
* authorization tests
* query review report
* boundary validation report

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Tenant boundaries formally enforced.

Notes / Deviations

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

* retention tests
* cleanup tests
* audit records
* artifact lifecycle report

Documentation Updates

* ARCHITECTURE.md
* DEPLOYMENT.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Evidence lifecycle becomes manageable.

Notes / Deviations

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

* replay tests
* determinism tests
* replay demonstrations
* replay evidence validation

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Historical execution becomes reproducible.

Notes / Deviations

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
2. Evaluator
3. Evidence linkage
4. Validation

Acceptance Evidence

* policy fixtures
* policy evaluation tests
* policy trace evidence
* policy inventory

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Admission becomes policy driven.

Notes / Deviations

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
3. Security review
4. Persistence review
5. Replay review

Acceptance Evidence

* architecture review report
* invariant review report
* replay review report
* security review report
* deviation register

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* PHASEMAP.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Architecture remains coherent.

Notes / Deviations

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

* containment tests
* escape tests
* resource limit tests
* sandbox validation report

Documentation Updates

* ARCHITECTURE.md
* DEPLOYMENT.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

Summary

Sandbox approaches production readiness.

Notes / Deviations

=====================================================================
PHASE 32
OPERATIONAL READINESS
=====================

Acceptance Evidence

* runbooks
* operational procedures
* alert validation
* monitoring validation

Documentation Updates

* DEPLOYMENT.md
* CHANGELOG.md

Commands Ran

cargo fmt -- --check

cargo clippy -- -D warnings

cargo test

=====================================================================
PHASE 33
BACKUP / RESTORE VALIDATION
===========================

Acceptance Evidence

* backup procedure
* restore procedure
* restore validation report
* integrity validation

Documentation Updates

* DEPLOYMENT.md
* CHANGELOG.md

=====================================================================
PHASE 34
DISASTER RECOVERY VALIDATION
============================

Acceptance Evidence

* DR exercise report
* recovery timing report
* recovery checklist
* postmortem review

Documentation Updates

* DEPLOYMENT.md
* CHANGELOG.md

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

* release readiness report
* unresolved issue inventory
* security review inventory
* architecture review inventory
* operational review inventory

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* PHASEMAP.md
* CHANGELOG.md

=====================================================================
PHASE 36
PERFORMANCE VALIDATION
======================

Acceptance Evidence

* load testing report
* concurrency testing report
* queue saturation report
* storage performance report

Documentation Updates

* DEPLOYMENT.md
* CHANGELOG.md

=====================================================================
PHASE 37
SECURITY ASSESSMENT
===================

Acceptance Evidence

* threat model
* penetration testing report
* dependency assessment
* remediation inventory

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* CHANGELOG.md

==============================
PHASE 38 DOCUMENTATION FREEZE
==============================

Acceptance Evidence

* documentation audit
* API audit
* deployment audit
* release documentation package

Documentation Updates

All documentation finalized.

============================
PHASE 39 RELEASE CANDIDATE
============================

Acceptance Evidence

* release candidate tag
* final validation report
* CI validation report
* deployment validation report

Documentation Updates

* CHANGELOG.md

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

* final architecture review
* final invariant review
* final security review
* final operational review
* final deviation register

Documentation Updates

* ARCHITECTURE.md
* INVARIANTS.md
* PHASEMAP.md
* CHANGELOG.md

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
