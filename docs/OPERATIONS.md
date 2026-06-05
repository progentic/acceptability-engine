# Operations

## Purpose

This document is the operator index for running the Acceptability Review Engine.

It links the runbooks, inventories the observable signals, and defines the alert
conditions operators must monitor. It does not create admission authority.
Durable state remains in SQLite and filesystem artifacts.

## Runbooks

- [Startup](runbooks/startup.md)
- [Shutdown](runbooks/shutdown.md)
- [Artifact retention](runbooks/retention.md)
- [Replay](runbooks/replay.md)
- [Incident response](runbooks/incident_response.md)
- [Restore](runbooks/restore.md)

## Monitoring Inventory

| Signal | Source | Operator Meaning |
| :--- | :--- | :--- |
| Liveness | `GET /health/live` | Process is running. |
| Readiness | `GET /health/ready` | SQLite is reachable and request handling may proceed. |
| Metrics | `GET /metrics` | Prometheus counters for traffic, submissions, denials, and uptime. |
| Kubernetes readiness probe | `deploy/kubernetes.yaml` | Pod should receive traffic only when ready. |
| Kubernetes liveness probe | `deploy/kubernetes.yaml` | Runtime should restart a wedged process. |
| Audit events | SQLite `audit_events` | Durable operator evidence for security and retention decisions. |
| Replay reports | CLI `--replay-run-id` | Read-only reconstruction of historical run evidence. |

## Metrics Inventory

| Metric | Type | Use |
| :--- | :--- | :--- |
| `acceptability_engine_uptime_seconds` | Gauge | Detect restarts and unstable runtime. |
| `acceptability_http_requests_total` | Counter | Track request volume. |
| `acceptability_http_responses_total` | Counter | Track response classes. |
| `acceptability_runs_submitted_total` | Counter | Track run intake. |
| `acceptability_security_denials_total` | Counter | Track authentication, authorization, tenant, rate, quota, and policy denials. |

## Alert Definitions

| Alert | Condition | First Runbook |
| :--- | :--- | :--- |
| Service not live | `/health/live` fails for two consecutive checks. | [Startup](runbooks/startup.md) |
| Service not ready | `/health/ready` fails for two consecutive checks. | [Incident response](runbooks/incident_response.md) |
| Pod restart loop | Kubernetes restart count increases repeatedly. | [Incident response](runbooks/incident_response.md) |
| Security denial spike | `acceptability_security_denials_total` rises faster than the normal tenant baseline. | [Incident response](runbooks/incident_response.md) |
| Artifact storage pressure | Artifact volume exceeds the local retention threshold. | [Artifact retention](runbooks/retention.md) |
| Replay required | Operator must reconstruct a historical decision. | [Replay](runbooks/replay.md) |
| Restore required | SQLite or artifact storage is unavailable or corrupted. | [Restore](runbooks/restore.md) |

## Operator Invariants

- Do not edit SQLite evidence rows by hand.
- Do not delete artifact files outside the retention workflow.
- Do not treat WebSocket progress or metrics as admission authority.
- Do not run production with `AH_SECURITY_MODE=disabled`.
- Do not run production with `AH_SANDBOX_PROFILE=development`.
- Preserve replay output, audit rows, and relevant logs during incidents.

## Validation Evidence

Phase 32 operational readiness is complete when these artifacts exist:

- Startup, shutdown, retention, replay, incident response, and restore runbooks.
- Monitoring and metrics inventory.
- Alert definitions mapped to first-response procedures.
- Phase 32 operational readiness report.
- Deployment documentation linking operators to this index.
- Phase map and changelog entries recording the operational evidence.
