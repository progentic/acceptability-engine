# Incident Response Runbook

## Purpose

Stabilize the runtime, preserve evidence, and avoid accidental admission-state
changes during operational incidents.

## First Response

1. Preserve current metrics.

   ```bash
   curl --fail http://127.0.0.1:8080/metrics
   ```

2. Check process liveness and readiness.

   ```bash
   curl --fail http://127.0.0.1:8080/health/live
   curl --fail http://127.0.0.1:8080/health/ready
   ```

3. Preserve platform logs for the affected time window.

4. Do not manually edit SQLite state.

5. Do not manually delete artifact files.

## Triage

| Symptom | Likely Area | Action |
| :--- | :--- | :--- |
| Live fails | Runtime process | Restart through Compose or Kubernetes after preserving logs. |
| Ready fails | SQLite or migration path | Verify `/data/evidence.db` availability and filesystem permissions. |
| Security denials spike | Authentication, role, tenant, rate, quota, or repo policy | Review API key rollout and tenant/repo policy changes. |
| Run stuck running | Worker or gate execution | Preserve logs and replay the run after the runtime stabilizes. |
| Artifact read missing | Retention or storage loss | Use replay to confirm descriptor state and restore artifact bytes from backup if required. |

## Escalation Evidence

Attach these artifacts to the incident record:

- Health responses.
- Metrics output.
- Platform logs.
- Relevant replay report.
- Retention audit rows if artifacts are involved.
- Deployment revision or image digest.

## Recovery

Use [Startup](startup.md) after restarting the service.

Use [Restore](restore.md) if SQLite or artifact storage must be recovered.

Use [Disaster recovery](disaster_recovery.md) when runtime and evidence stores
must be rebuilt after destructive failure.
