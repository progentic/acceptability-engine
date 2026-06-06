# Phase 37 Security Assessment

## Scope

This assessment reviews the production-relevant security boundaries after D25-001
candidate acquisition closure.

In scope:

- candidate SHA authority
- repository policy
- tenant isolation
- review authorization
- admission policy evaluation
- sandbox profile
- retention safety
- replay integrity
- audit evidence
- D25-002 residual sandbox risk
- dependency and supply-chain assessment

Out of scope:

- external identity provider integration
- VM isolation
- multi-pod Kubernetes runtime testing
- third-party penetration testing
- patch, archive, or pull-request-number admission

## Assumptions

Production deployment uses:

- `AH_SECURITY_MODE=api-key`
- `AH_SANDBOX_PROFILE=kubernetes-restricted`
- non-placeholder API keys
- deployment-enforced Kubernetes restricted controls
- deny-all pod egress unless an explicit Git egress policy is added

Local `disabled` security mode and `development` sandbox profile are treated as
development-only modes.

## Threat Model

### Assets

| Asset | Security Goal |
| :--- | :--- |
| Contract authority fields | Prevent mutable refs or stale contract rows from changing what is admitted. |
| Run state and final decisions | Prevent unauthorized approval, rejection, or hidden state mutation. |
| Gate evidence and artifacts | Preserve auditability and replayability. |
| Tenant-scoped run data | Prevent cross-tenant disclosure. |
| API keys | Prevent unauthorized submit, read, or review actions. |
| Worker and gate execution host | Limit damage from untrusted candidate code. |
| Audit events | Preserve evidence of security denials and operator actions. |

### Trust Boundaries

| Boundary | Existing Controls |
| :--- | :--- |
| HTTP client to Rust API | API-key roles, tenant identity, rate limits, repository policy, audit events. |
| TypeScript UI to Rust API | UI is non-authoritative and calls Rust endpoints. |
| Contract JSON to orchestrator | Contract validation, candidate SHA validation, scope normalization. |
| Repository to workspace | Git mode verifies origin, base SHA, candidate SHA, ancestry, detached HEAD. |
| Rust runner to external tools | Sanitized environment, timeout, process-group cleanup, output limits, rlimits. |
| SQLite to replay/reads | Tenant-scoped read helpers for public HTTP paths; replay is read-only CLI behavior. |
| Artifact store to retention | URI validation, descriptor preservation, audit events. |

### Attacker Capabilities

Assumed:

- submit malformed contracts through HTTP when holding a submit-capable API key
- submit code that runs during Cargo gates
- attempt cross-tenant reads with a valid API key for another tenant
- attempt unauthorized review decisions with insufficient role
- trigger large output, long-running processes, or dependency checks
- attempt to exploit mutable Git refs as admitted-object authority

Not assumed:

- root access to the host or Kubernetes node
- direct SQLite file access outside the application
- direct artifact-store filesystem access outside the application
- compromise of the configured production API key secret

## Primary Target Review

| Target | Result | Evidence |
| :--- | :--- | :--- |
| Candidate SHA authority | Pass | `candidate_sha` is required, persisted, replayed, and used for Git checkout. `candidate_ref` is only provenance/fetch metadata. |
| Repository policy | Pass with accepted limitation | Submit authorization checks `repo_url` against API-key repository prefixes before run creation. Prefix matching is intentionally simple and depends on careful operator prefix configuration. |
| Tenant isolation | Pass | Public read/review paths use tenant-scoped helpers and opaque not-found behavior for hidden resources. |
| Review authorization | Pass | Review endpoints require reviewer/admin role and a tenant-owned `PENDING_HUMAN_REVIEW` run. |
| Policy evaluation | Pass | Policy ids, versions, required gates, parse-error thresholds, and gate-pass requirements fail closed. Policy trace includes candidate identity. |
| Sandbox profile | Residual risk | `kubernetes-restricted` defines production containment. D25-002 is now a residual release-risk decision until runtime enforcement is validated on Kubernetes. |
| Retention safety | Pass | Retention validates artifact URIs, preserves SQLite descriptors, records audit events, and rejects symlink-parent cleanup paths. |
| Replay integrity | Pass | Replay is read-only, deterministic except generation time, includes candidate identity and scopes, and reports missing artifacts without recreating them. |
| Audit evidence | Pass | Security denials, allowed reads/submissions/reviews, retention actions, and cross-tenant visibility denials are recorded when caller context exists. |

## Penetration Testing Report

This was a source-grounded and local-control assessment, not a third-party
penetration test.

Abuse paths reviewed:

| Abuse Path | Result |
| :--- | :--- |
| Submit a contract without authoritative candidate identity | Blocked by contract validation. |
| Use `candidate_ref` as authority by moving a branch or PR ref | Blocked by `candidate_sha` checkout and `HEAD == candidate_sha` verification. |
| Reuse a contract id with different authority fields | Blocked by persisted authority comparison. |
| Submit outside repository policy | Blocked by submit authorization and audited. |
| Read another tenant's run, gates, evidence, or progress | Hidden behind tenant-scoped lookups and audited when the resource exists. |
| Review another tenant's run | Hidden before review finalization and audited. |
| Review with submitter role | Blocked by role authorization and audited. |
| Approve a non-pending run | Blocked by review finalization state check. |
| Preserve approval after partial finalization | Blocked by transactional finalization. |
| Delete evidence descriptors during retention | Blocked by retention design; only artifact bytes are deleted. |
| Recreate or mutate state through replay | Blocked by read-only replay implementation. |
| Exhaust gate process output | Blocked by output limit failure. |
| Leave descendant gate processes running after timeout | Covered on Unix by process-group cleanup test. |

## Dependency Assessment

Commands:

```text
cargo audit
cargo deny check
```

Results:

| Tool | Result | Notes |
| :--- | :--- | :--- |
| `cargo audit` | Pass | Loaded 1120 RustSec advisories and scanned 123 locked dependencies. |
| `cargo deny check` | Fail | `advisories ok`, `bans ok`, and `sources ok`; `licenses FAILED` because no deny config/license allowlist exists and the local crate has no license expression. |

The `cargo deny` result is a supply-chain governance gap, not evidence of a
known vulnerability in the dependency graph.

## Findings

### D25-002: Sandbox Runtime Enforcement Residual Risk

Severity: High

Status: Residual risk

Release blocker: Governance decision required

Required before v1.0: Governance decision required

Owner surface: Sandbox/deployment

Target phase: Phase 40 Production Governance Review

The engine has runner hardening, environment scrubbing, output limits,
timeouts, rlimits, and a restricted Kubernetes deployment profile. It does not
create a portable kernel namespace, chroot, seccomp, or VM isolation boundary in
Rust.

Impact:

Untrusted candidate code still relies on deployment-enforced containment for
production isolation.

Required closure:

Validate runtime enforcement on a real Kubernetes target or add a stronger
isolated runner design.

### D37-001: Supply-Chain License Policy Is Not Configured

Severity: Medium

Status: Open

Release blocker: No

Required before v1.0: Yes

Owner surface: Supply chain

Target phase: Phase 38 Documentation Freeze or Phase 39 Release Candidate

`cargo deny check` falls back to its default config and fails license checks for
normal open-source licenses and the local crate's missing license expression.

Impact:

Gate 8 can perform advisory and audit checks, but production supply-chain
governance cannot yet distinguish approved licenses from unreviewed ones.

Required closure:

Add an explicit `deny.toml` license/source policy and a local crate license
expression, then require `cargo deny check` to pass in validation.

### D37-002: Placeholder Kubernetes API Key Must Fail Closed

Severity: High

Status: Open

Release blocker: Yes

Required before v1.0: Yes

Owner surface: Deployment/security

Target phase: Phase 38 Documentation Freeze

The Kubernetes manifest contains `replace-me|admin|default|*` as a placeholder
secret value. The startup runbook instructs operators to replace it before
production use, but startup does not reject that exact placeholder in code.

Impact:

Applying the manifest unchanged would create a known admin wildcard credential.

Required closure:

Add startup rejection for known placeholder API keys and require deployment
validation or secret-management tooling that prevents the placeholder from
reaching production.

## Remediation Inventory

| ID | Severity | Release Blocker | Required Before v1.0 | Owner Surface | Target Phase | Required Action |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| D25-002 | High | Governance decision required | Governance decision required | Sandbox/deployment | Phase 40 | Validate `kubernetes-restricted` runtime enforcement or introduce stronger isolated execution. |
| D37-001 | Medium | No | Yes | Supply chain | Phase 38 or Phase 39 | Add `deny.toml`, approve licenses/sources explicitly, and make `cargo deny check` pass. |
| D37-002 | High | Yes | Yes | Deployment/security | Phase 38 | Prevent placeholder API keys from production startup and enforce replacement through deployment tooling. |

## Validation Evidence

Commands run:

```text
cargo test
cargo audit
cargo deny check
```

Results:

```text
cargo test: 142 passed
cargo audit: passed
cargo deny check: failed license policy; advisories/bans/sources ok
```

## Conclusion

Phase 37 validates the core admission security model after D25-001. Candidate
SHA authority, tenant isolation, review authorization, policy evaluation,
retention safety, replay integrity, and audit evidence are coherent.

Production release is blocked by D37-002 until placeholder credentials fail
closed or are prevented by deployment tooling. D25-002 remains a governance
decision about residual sandbox risk. D37-001 is not an immediate release
blocker, but it must be resolved before v1.0 supply-chain governance is
complete.
