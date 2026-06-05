# Phase 35 Release Readiness Review

## Scope

This review evaluates release eligibility after Phase 34 and UI-001.

Phase 35 introduces no runtime functionality, endpoint, table, migration,
policy behavior, replay behavior, review workflow, security behavior, or UI
behavior.

## Release Readiness Result

The project is not production-release ready.

The blocking release gap is:

```text
D25-001 Candidate acquisition model
```

The system has strong evidence, review, recovery, operations, replay, policy,
tenant, and sandbox posture for its current stage. It still lacks a first-class
model for the proposed Git change being admitted.

## Capability Inventory

| Capability | Status | Evidence |
| :--- | :--- | :--- |
| Rust authority model | Complete | `docs/ARCHITECTURE.md`, `docs/INVARIANTS.md` |
| Human review | Complete | Review endpoints, review decisions, final decisions |
| Progress streaming | Complete | WebSocket progress route and bounded replay |
| Git materialization | Partial | Repository clone and `base_sha` checkout exist; candidate identity is missing |
| Tenant isolation | Complete | Tenant-scoped reads and denial audit evidence |
| Artifact retention | Complete | Retention CLI and audit evidence |
| Replay | Complete | Read-only replay reports |
| Admission policy | Complete | Contract-scoped Rust-evaluated policy |
| Sandbox hardening | Complete for documented profile | `kubernetes-restricted` profile, deployment controls, runner hardening |
| Operational runbooks | Complete | Operations index and runbooks |
| Backup evidence | Complete | Backup fixture, inventory, replay baseline, hashes |
| Disaster recovery evidence | Complete | Replay-preserving recovery validation |
| Browser UI theme | Complete | UI-001 semantic theme commit |

## Unresolved Issue Inventory

| ID | Severity | Status | Release Impact |
| :--- | :--- | :--- | :--- |
| D25-001 | Blocking | Open | The engine does not yet identify the proposed Git change as a first-class admitted object. |
| D25-002 | Managed risk | Narrowed | Production containment depends on documented Kubernetes/container controls plus runner hardening. |
| D25-004 | Accepted limitation | Open by design | API-key identity remains deployment-specific until external identity is required. |

## Security Review Inventory

| Area | Status | Notes |
| :--- | :--- | :--- |
| Tenant isolation | Ready for Phase 37 assessment | Cross-tenant reads and review attempts are opaque and audited. |
| Repository authorization | Ready for Phase 37 assessment | Repository policy runs before submission. |
| Sandbox profile | Ready for Phase 37 assessment | Production profile is documented, but runtime enforcement should be assessed in target Kubernetes. |
| API-key identity | Accepted limitation | External identity provider integration remains deployment-specific. |
| Candidate acquisition | Blocking | `candidate_sha` must be added before production release. |

## Architecture Review Inventory

| Review | Result |
| :--- | :--- |
| Phase 25 Architecture Review I | Identified D25-001 and D25-002. |
| Phase 30 Architecture Review II | Reconfirmed D25-001 as release-critical. |
| Candidate Acquisition Architecture | Defines `candidate_sha` as the future admitted object. |
| Phase 31 Sandbox Hardening | Narrows D25-002 for `kubernetes-restricted`. |

## Operational Review Inventory

| Area | Status |
| :--- | :--- |
| Startup | Runbook exists. |
| Shutdown | Runbook exists. |
| Retention | Runbook and CLI workflow exist. |
| Replay | Runbook and CLI workflow exist. |
| Incident response | Runbook exists. |
| Backup | Runbook and validation evidence exist. |
| Restore | Runbook exists. |
| Disaster recovery | Runbook and replay-preserving validation evidence exist. |
| Health and metrics | Endpoints and operational mapping exist. |

## Candidate Acquisition Readiness Decision

Use `candidate_sha` as the first-class admitted object.

Minimum future contract boundary:

```text
repo_url + base_sha + candidate_sha
```

Optional future provenance:

```text
candidate_ref
```

`candidate_ref` must not become authority.

## Release Readiness Gate

Before production release, D25-001 must be closed by implementation and review.

Minimum closure criteria:

- contract accepts and validates `candidate_sha`
- persistence stores `candidate_sha`
- Git materialization resolves `candidate_sha` inside `repo_url`
- workspace `HEAD` equals `candidate_sha` during gates
- Gate 3 evaluates `base_sha..candidate_sha`
- replay reports `candidate_sha`
- policy can reference persisted candidate identity
- legacy contracts without `candidate_sha` have explicit non-production handling

## Validation Evidence

Documentation validation:

```text
git diff --check
```

No runtime validation is required for this phase because no runtime behavior
changes.

## Conclusion

Phase 35 completes the release-readiness review but does not grant production
release readiness.

The next release-critical implementation target is D25-001 candidate
acquisition with `candidate_sha` as the admitted object.
