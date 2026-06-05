# Replay Runbook

## Purpose

Reconstruct a historical run decision from durable evidence.

Replay is read-only. It does not execute gates, mutate run state, recreate
artifact bytes, or change final decisions.

## Command

```bash
accessibility-engine --workspace /workspaces --database /data/evidence.db --artifact-root /artifacts --replay-run-id 123
```

Replace `123` with the target run id.

## Output Handling

Preserve the JSON output with the incident or audit record that required replay.

The `generated_at` field is intentionally new for each replay. Historical run
content should remain deterministic apart from that timestamp.

## Success Criteria

- The command exits `0`.
- The report includes contract, run, attempts, gates, policy evaluations,
  review decision when present, final decision when present, and evidence
  descriptors.
- Missing artifact bytes appear as missing indicators.

## Failure Response

- Exit `2` means the run was not found.
- Store or artifact-root errors require [Incident response](incident_response.md).
