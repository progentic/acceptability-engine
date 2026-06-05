# Phase 32 Operational Readiness

## Purpose

This report records the operational artifacts added for Phase 32.

The phase adds operator procedures and validation evidence. It does not add new
admission behavior.

## Operator Procedure Inventory

| Procedure | Artifact | Purpose |
| :--- | :--- | :--- |
| Startup | `docs/runbooks/startup.md` | Start the runtime and verify liveness, readiness, and metrics. |
| Shutdown | `docs/runbooks/shutdown.md` | Stop the runtime without deleting SQLite or artifact evidence. |
| Retention | `docs/runbooks/retention.md` | Run dry-run and live artifact byte retention through the audited CLI workflow. |
| Replay | `docs/runbooks/replay.md` | Produce read-only historical run reconstruction reports. |
| Incident response | `docs/runbooks/incident_response.md` | Preserve evidence, triage common failures, and route recovery. |
| Restore | `docs/runbooks/restore.md` | Define manual restore order for SQLite and artifact storage. |

## Monitoring Inventory

Operational visibility is documented in `docs/OPERATIONS.md`.

The monitored surfaces are:

- `GET /health/live`
- `GET /health/ready`
- `GET /metrics`
- Kubernetes readiness and liveness probes
- SQLite audit events
- CLI replay reports

## Alert Inventory

`docs/OPERATIONS.md` maps these alert classes to first-response runbooks:

- service not live
- service not ready
- pod restart loop
- security denial spike
- artifact storage pressure
- replay required
- restore required

## Authority Review

Phase 32 does not change the authority model.

Metrics, logs, probes, and progress streams remain operator observations. SQLite
evidence, filesystem artifact descriptors, review decisions, policy evaluations,
and final decisions remain authoritative.

## Deferred Validation

Backup and restore validation is deferred to Phase 33.

Disaster recovery exercise evidence is deferred to Phase 34.

## Validation

The following commands passed:

- `git diff --check`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`

## Conclusion

Phase 32 is complete when this report, the operations index, the runbooks, the
phase map entry, deployment links, and changelog entry are committed together.
