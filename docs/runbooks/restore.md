# Restore Runbook

## Purpose

Restore SQLite evidence and artifact storage after data loss or corruption.

Phase 32 defines the manual procedure. Phase 33 validates the backup artifact
shape and restore prerequisites. Phase 34 validates destructive evidence-store
restore through replay comparison.

## Restore Order

1. Stop the runtime with [Shutdown](shutdown.md).
2. Restore SQLite data under `/data`.
3. Restore artifact bytes under `/artifacts`.
4. Start the runtime with [Startup](startup.md).
5. Run a replay check for a known historical run.

Expected backup inputs are documented in [Backup](backup.md). The backup shape is:

```text
backup-root/
  evidence.db
  artifacts/
  replay/
    run-<id>-pre-backup.json
  inventory.txt
```

Use [Disaster recovery](disaster_recovery.md) when restore is part of a
destructive runtime rebuild.

## Validation

```bash
curl --fail http://127.0.0.1:8080/health/ready
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --replay-run-id 123
```

## Success Criteria

- Readiness returns `ready`.
- Replay succeeds for a known run.
- Replay evidence descriptors match the restored database.
- Missing artifact indicators are understood and documented.

## Notes

Do not fabricate final decisions, review decisions, gate records, or evidence
descriptors during restore.

If restored artifacts are incomplete, keep the SQLite descriptors intact and let
replay report missing artifact bytes.
