# Backup Runbook

## Purpose

Create backup evidence for SQLite state and filesystem artifacts.

Backup must preserve the evidence chain used by replay:

- SQLite database under `/data`
- artifact bytes under `/artifacts`
- pre-backup replay JSON for at least one known run
- backup inventory describing what was captured

## Backup Artifact Shape

Use this layout for operator-created backups:

```text
backup-root/
  evidence.db
  artifacts/
  replay/
    run-<id>-pre-backup.json
  inventory.txt
```

The database and artifacts are copied as evidence. Replay JSON is verification
evidence and must not be used to reconstruct SQLite state.

## Procedure

1. Select a known run id for validation.
2. Capture pre-backup replay output.

   ```bash
   accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --replay-run-id 123 > backup-root/replay/run-123-pre-backup.json
   ```

3. Copy SQLite data.

   ```bash
   cp /data/evidence.db backup-root/evidence.db
   ```

4. Copy artifact bytes.

   ```bash
   cp -R /artifacts backup-root/artifacts
   ```

5. Write backup inventory.

   ```text
   created_at=<unix-seconds>
   source_database=/data/evidence.db
   source_artifact_root=/artifacts
   database_file_name=evidence.db
   database_sha256=<sha256>
   replay_file_name=replay/run-123-pre-backup.json
   replay_sha256=<sha256>
   artifact_count=<count>
   artifact.0.path=<relative-artifact-path>
   artifact.0.sha256=<sha256>
   ```

## Integrity Validation

Verify:

- `backup-root/evidence.db` exists.
- `backup-root/artifacts` exists.
- at least one artifact file exists when the selected replay references artifact bytes.
- pre-backup replay JSON exists.
- inventory hashes match the copied SQLite database, replay JSON, and artifact files.
- restore prerequisites are documented in `inventory.txt`.

## Restore Prerequisites

Before restore, operators need:

- backup database path
- backup artifact root path
- target database path
- target artifact root path
- run id used for replay comparison
- pre-backup replay JSON

Phase 34 consumes this backup shape during disaster recovery validation.
