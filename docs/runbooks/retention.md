# Artifact Retention Runbook

## Purpose

Remove old artifact bytes while preserving SQLite evidence descriptors.

Retention is an audited operator workflow. It must not be replaced with manual
filesystem deletion.

## Dry Run

Run the dry-run command first.

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90 --retention-dry-run
```

## Live Deletion

Run deletion only after the dry-run candidate set is understood.

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --retention-days 90
```

## Success Criteria

- Dry-run records planned retention audit events.
- Live deletion records deleted or missing retention audit events.
- SQLite `evidence_bundles` rows remain present.
- Replay reports deleted artifact bytes as missing rather than failing.

## Rollback

Retention does not mutate SQLite descriptors. Deleted artifact bytes can be
restored only from an external artifact backup.

Use [Restore](restore.md) when artifact bytes must be recovered.
