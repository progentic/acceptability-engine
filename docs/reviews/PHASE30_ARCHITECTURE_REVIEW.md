# Phase 30 Architecture Review II

## Scope

This review validates the implemented system against `docs/ARCHITECTURE.md`,
`docs/INVARIANTS.md`, `docs/API.md`, and `docs/PHASEMAP.md` after Phase 29.

This phase introduces no product capability, endpoint, table, migration, or
runtime behavior.

## Architecture Review Report

| Area | Status | Evidence |
| :--- | :--- | :--- |
| Rust authority boundary | Aligned | Contract validation, gate execution, policy evaluation, review decisions, security checks, run state, and evidence writes remain Rust-owned. |
| Policy authority | Aligned | Policy is a Rust-interpreted contract rule. It is not a second authority layer, script runtime, database trigger, or UI decision. |
| Policy ordering | Aligned | The documented order is gate result, policy evaluation, human review requirement, final decision. |
| Replay model | Aligned | Replay reconstructs contract, run, attempts, gates, policy evaluations, review decision, final decision, evidence descriptors, and artifact presence without mutation. |
| Workspace model | Aligned with open deviation | Local and Git modes remain rooted at `workspace_root / contract.id`; Git mode still lacks candidate-change acquisition. |
| Sandbox model | Aligned with open deviation | Process execution is hardened but not a complete adversarial kernel sandbox. |
| Tenant boundary | Aligned | Tenant-scoped HTTP reads, progress access, evidence reads, and review actions require tenant visibility before data is returned. |
| Evidence model | Aligned | Review, retention, replay, and policy additions preserve SQLite descriptors as authority and filesystem artifacts as payloads. |
| UI boundary | Aligned | TypeScript models policy input but does not evaluate admission or create final decisions. |

## Invariant Compliance Report

| Invariant Area | Status | Notes |
| :--- | :--- | :--- |
| Rust remains authority | Compliant | Policy evaluation and review finalization are Rust-owned. |
| Contract is untrusted input | Compliant | Unsupported policy ids, unsupported versions, and invalid policy rules fail contract validation. |
| Workspace containment | Compliant | Workspace selection remains rooted and path-safe. |
| Workspace modes fail closed | Compliant with deviation | Unknown modes fail; Git mode remains limited by D25-001. |
| Sequential gates | Compliant | Policy runs after gate outputs and does not reorder gates. |
| Stable gate numbers | Compliant | Gate numbering remains unchanged. |
| Rejection versus internal failure | Compliant | Policy failure is an expected rejection; infrastructure failure remains failed-internal. |
| Unknown is not approved | Compliant | Approval requires required gates, policy pass, and review completion when required. |
| Human review suspension | Compliant | Human review applies only after gates and policy pass. |
| Attempt-owned gates | Compliant | Policy evaluations link to run and attempt; gate records remain attempt-owned. |
| Unique final decision | Compliant | Policy does not create additional final-decision rows. |
| Evidence durability | Compliant | Policy traces are persisted as durable SQLite evidence before terminal state is presented. |
| Transactional finalization | Compliant | Gate records, policy trace, evidence descriptors, attempt status, run status, and terminal final decision finalize together. |
| Tenant boundaries before reads | Compliant | Public read surfaces remain tenant-scoped. |
| Repository policy before submission | Compliant | Repository policy remains separate from admission policy and still runs before queued run creation. |
| Security denials as evidence | Compliant | Security denial audit behavior remains unchanged. |
| Progress observational only | Compliant | Progress does not participate in policy, review, replay, or final decision authority. |
| Blocking boundaries | Compliant | No new blocking execution path was introduced by Phase 29. |
| Controlled execution environment | Compliant with deviation | Command environment hardening remains in place; D25-002 remains open. |
| Metrics and UI non-authority | Compliant | Metrics and UI remain observation surfaces only. |

## Policy Authority Review

Policy evaluation did not become an independent authority layer.

The authority chain is:

```text
Gate Result
      ↓
Policy Evaluation
      ↓
Human Review Requirement
      ↓
Final Decision
```

Policy input is contract-scoped and declarative. Rust validates the policy,
interprets gate outputs against the policy, records a policy trace, and derives
the next state. Unsupported policy identities and versions fail closed during
contract validation.

Human review cannot convert a policy failure into `PENDING_HUMAN_REVIEW`.
A failed policy produces `REJECTED` unless a future explicitly documented
policy model introduces a different path.

## Replay Completeness Review

Replay remains read-only and reconstructive.

The replay report includes:

- contract fields, including `policy_json`
- run status
- attempts ordered by attempt number and id
- gates ordered by gate number and id
- policy evaluations ordered by creation time and id
- review decision, when present
- final decision, when present
- evidence descriptors
- missing artifact indicators
- generated timestamp
- source database identity when available

Replay does not execute gates, evaluate policy again, mutate run state, mutate
evidence descriptors, recreate deleted artifacts, or alter final decisions.

## Security Boundary Review

Tenant isolation remains coherent after progress, review, and policy work.

Public HTTP read paths must prove tenant visibility before returning run,
attempt, gate, evidence, or progress data. Cross-tenant review attempts remain
opaque to the caller and auditable internally.

Admission policy does not create a new tenant boundary. It is submitted through
the existing contract path and validated by Rust before execution.

The open security limitations are unchanged:

- API-key identity is not an external identity provider.
- Full kernel-mediated sandboxing is not implemented.

## Persistence Inventory Update

| Store | Durable | Purpose |
| :--- | :--- | :--- |
| `contracts` | Yes | Submitted contract fields and serialized admission policy. |
| `runs` | Yes | Tenant-owned run state. |
| `attempts` | Yes | Deterministic per-run execution attempts. |
| `gate_runs` | Yes | Attempt-owned gate results and telemetry. |
| `policy_evaluations` | Yes | Run- and attempt-linked policy outcome, reason, and trace. |
| `evidence_bundles` | Yes | Artifact descriptors linked to run, attempt, gate, or review identities. |
| `review_decisions` | Yes | Reviewer identity, role, decision, reason, and timestamp. |
| `final_decisions` | Yes | Unique terminal approved or rejected decision per run. |
| `audit_events` | Yes | Security, authorization, submission, read, review, and retention audit trail. |
| Filesystem artifacts | Yes | Larger evidence payloads addressed by SQLite descriptors. |
| Progress replay buffer | No | Bounded in-memory reconnect aid; not authoritative evidence. |
| Metrics | No | Operational counters only. |

## Deviation Register Update

| ID | Status | Deviation | Impact | Disposition |
| :--- | :--- | :--- | :--- | :--- |
| D25-001 | Open, release-critical | Git mode validates `base_sha` but the contract does not yet model candidate ref, candidate SHA, patch URI, archive, or pull request identity. | Gate 3 is meaningful only when a controlled external step has already modified the workspace. The engine still lacks a first-class model for the change being admitted. | Must be resolved before relying on Git mode for remote proposed-change admission or production release. |
| D25-002 | Open | Process execution is hardened but not a complete adversarial kernel sandbox. | Production isolation still depends on outer container, namespace, filesystem, network, and syscall policy. | Phase 31 must define and test the sandbox architecture. |
| D25-003 | Accepted limitation | WebSocket replay is bounded and in memory. | Old progress observations can age out after disconnects. | Durable evidence remains authoritative. |
| D25-004 | Accepted limitation | API-key security does not include external identity providers. | Operator identity integration is deployment-specific. | External identity remains deferred. |
| D28-001 | Accepted limitation | Replay is CLI-first, not HTTP or UI. | Operators use CLI replay for deterministic reconstruction. | No action required unless API replay becomes a roadmap target. |
| D28-002 | Accepted limitation | Replay reports artifact presence but does not read artifact bytes. | Replay proves descriptor and presence state, not artifact payload content. | Artifact content inspection remains a separate evidence API concern. |
| D29-001 | Accepted limitation | Policy is contract-scoped and declarative; no server-global registry or scripting exists. | Policies are intentionally constrained to Rust-validated JSON. | Revisit only if future phases require centralized policy administration. |
| D29-002 | Accepted limitation | Policy trace evidence is stored in SQLite, not filesystem artifacts. | Policy traces are durable but not separate artifact payloads. | No action required unless trace size or artifact export requirements change. |

## Roadmap Impact Assessment

| Phase | Impact | Required Attention |
| :--- | :--- | :--- |
| Phase 31 Sandbox Hardening | Expected architecture change. | Close or narrow D25-002 with namespace, filesystem, network, resource, and syscall policy evidence. |
| Phase 32 Candidate Acquisition | Expected architecture change if added. | Close D25-001 by defining candidate identity in the contract and materialization flow. |
| Phase 35 Release Readiness Review | Release-sensitive. | Recheck open deviations and classify each as closed, accepted, or blocking. |
| Phase 37 Security Assessment | Security-sensitive. | Reassess tenant isolation, API-key limitations, audit evidence, and sandbox assumptions. |
| Phase 40 Production Governance Review | Release gate. | Ensure no unresolved production-blocking architecture or invariant gaps remain. |
| Phase 41 Production Release | Production gate. | D25-001 must be closed. D25-002 must be closed or explicitly accepted as non-blocking before v1.0. |

## New Invariant Assessment

No additional invariant is required before Phase 31.

The existing invariants already cover policy ordering, Rust authority,
transactional policy trace persistence, replay read-only behavior, tenant
boundaries, and non-authoritative UI/progress/metrics surfaces.

Phase 31 is expected to require a new or revised sandbox invariant once the
kernel isolation model is defined.

## Conclusion

The implementation remains aligned with the documented architecture after the
admission policy engine. Policy is Rust-interpreted evidence, not a second
authority layer. Replay remains complete for the current evidence model.

The two production-relevant open architecture gaps remain D25-001 candidate
acquisition and D25-002 adversarial sandboxing. D25-001 is release-critical
because the engine still lacks a first-class model for the change being
admitted.
