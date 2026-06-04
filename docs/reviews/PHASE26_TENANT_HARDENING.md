# Phase 26 Multi-Tenant Hardening

## Scope

This review validates tenant isolation after Phase 25. It covers public HTTP
read paths, review authority paths, store query helpers, audit behavior, and
negative-path coverage.

This phase does not add tenant federation or shared visibility.

## Query Review Report

| Query Surface | Tenant Boundary | Result |
| :--- | :--- | :--- |
| Run list | `list_runs_for_tenant` filters `runs.tenant_id`. | Tenant-scoped. |
| Run status | `fetch_run_summary_for_tenant` checks `runs.id` and `runs.tenant_id` before loading latest-attempt gates. | Tenant-scoped. |
| Run attempts | `list_run_attempts_for_tenant` checks parent run ownership before loading attempts. | Tenant-scoped. |
| Attempt gates | `list_attempt_gates_for_tenant` joins attempts to runs and checks `runs.tenant_id` before loading gate rows. | Tenant-scoped. |
| Run evidence | `list_run_evidence_for_tenant` checks parent run ownership before loading evidence descriptors. | Tenant-scoped. |
| Human review finalization | `finalize_human_review` checks run status through `run_id` and `tenant_id` inside the transaction. | Tenant-scoped. |
| Progress stream setup | `GET /runs/:id/progress` checks run visibility for the caller tenant before subscription or replay. | Tenant-scoped. |
| Audit events | Public handlers write allowed and denied audit events with the authenticated tenant identity. | Tenant-attributed. |

## Boundary Validation Report

| Boundary | Validation |
| :--- | :--- |
| Authorization before reads | Public read handlers call read authorization before tenant-scoped store access. |
| Authorization before submission | Submission checks role and repository policy before creating a queued run. |
| Authorization before review | Review handlers require `reviewer` or `admin` before tenant-scoped review finalization. |
| Cross-tenant read behavior | Existing resources outside the caller tenant return `404`, not the hidden resource. |
| Cross-tenant audit behavior | Authenticated hidden-resource reads and review attempts write denied audit events with reason `resource not found or not visible`. |
| Progress replay boundary | Progress replay is unavailable until run visibility is proven for the caller tenant. |
| Store helper boundary | Public HTTP paths use tenant-aware helpers; local helpers are test-only or internal local-mode helpers. |

## Tenant Isolation Tests

| Test | Coverage |
| :--- | :--- |
| `tenant_scoped_run_list_excludes_other_tenants` | Run list hides other tenant runs. |
| `tenant_scoped_run_summary_hides_other_tenant_run` | Run status hides other tenant runs. |
| `cross_tenant_run_status_is_hidden_and_audited` | HTTP run status returns `404` and writes denied audit evidence. |
| `cross_tenant_attempt_gates_are_hidden_and_audited` | HTTP attempt gates return `404` and write denied audit evidence. |
| `cross_tenant_run_evidence_is_hidden_and_audited` | HTTP evidence returns `404` and writes denied audit evidence. |
| `cross_tenant_review_is_hidden_and_audited` | HTTP review returns `404`, leaves run pending, and writes denied audit evidence. |
| `cross_tenant_progress_is_hidden_and_audited` | Progress setup returns `404` before replay or subscription and writes denied audit evidence. |
| `missing_authenticated_run_is_not_hidden_resource_denial` | Missing run status returns `404` without a hidden-resource denial audit. |
| `missing_progress_run_is_not_hidden_resource_denial` | Missing progress run setup returns `404` without a hidden-resource denial audit. |

## Authorization Tests

| Test | Coverage |
| :--- | :--- |
| `submit_requires_api_key_when_security_is_enabled` | Submission requires authentication in API-key mode. |
| `rejects_viewer_submission` | Viewer cannot submit. |
| `rejects_repo_outside_policy` | Submitter cannot submit outside repository policy. |
| `authorizes_reviewer_decisions` | Reviewer role can enter review path. |
| `rejects_submitter_review_decision` | Submitter cannot review. |
| `submitter_cannot_review_run` | HTTP review endpoint denies submitter role and audits denial. |

## Deviation Register

| ID | Status | Deviation | Disposition |
| :--- | :--- | :--- | :--- |
| D26-001 | Accepted limitation | API-key mode has tenant strings but no tenant federation or shared visibility. | Explicit non-goal for Phase 26. |
| D26-002 | Accepted limitation | Hidden-resource reads return `404` for both missing and cross-tenant resources. | Preserves tenant privacy; audit evidence records the denied authenticated attempt. |

## Conclusion

Tenant boundaries are formally enforced across the current public HTTP read,
submit, review, and progress surfaces. Cross-tenant resource access remains
opaque to callers and now leaves durable denied audit evidence.
