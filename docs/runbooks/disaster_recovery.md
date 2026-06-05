# Disaster Recovery Runbook

## Purpose

Recover service after loss of runtime state, SQLite evidence storage, or
filesystem artifact storage.

Disaster recovery must preserve admission evidence. Do not fabricate run state,
gate records, policy evaluations, review decisions, final decisions, or evidence
descriptors.

## Recovery Checklist

1. Declare the incident and preserve available logs.
2. Record last known health and metrics output when endpoints are reachable.
3. Stop the runtime with [Shutdown](shutdown.md).
4. Restore SQLite data under `/data`.
5. Restore artifact bytes under `/artifacts`.
6. Start the runtime with [Startup](startup.md).
7. Verify readiness.
8. Generate replay output for a known run with [Replay](replay.md).
9. Compare replay output against the pre-backup replay evidence from [Backup](backup.md) when available.
10. Document missing artifact indicators and any evidence gaps.

## Verification Commands

```bash
curl --fail http://127.0.0.1:8080/health/live
curl --fail http://127.0.0.1:8080/health/ready
curl --fail http://127.0.0.1:8080/metrics
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --replay-run-id 123
```

## Success Criteria

- Runtime health checks pass after restore.
- SQLite evidence is readable.
- Artifact root is readable.
- Replay succeeds for the selected run.
- Normalized post-restore replay matches pre-restore replay when a baseline exists.
- Missing artifact bytes are documented instead of rewritten.

## Postmortem Inputs

Record:

- incident start and recovery end timestamps
- affected deployment revision
- backup source
- restored database identity
- artifact restore source
- replay comparison result
- evidence gaps
- follow-up actions

## Phase 34 Validation

Phase 34 validates the replay comparison model by consuming the Phase 33 backup
fixture in a local destructive evidence-store exercise. Full deployment
destruction and restore remains an operator exercise to run in the target
environment.
