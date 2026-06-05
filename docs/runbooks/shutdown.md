# Shutdown Runbook

## Purpose

Stop the engine without losing durable evidence.

## Pre-Shutdown Checks

1. Record the current readiness state.

   ```bash
   curl --fail http://127.0.0.1:8080/health/ready
   ```

2. Capture current metrics.

   ```bash
   curl --fail http://127.0.0.1:8080/metrics
   ```

3. Preserve recent runtime logs through the platform log system.

## Compose Shutdown

```bash
docker compose down
```

Do not delete named volumes unless the intent is to remove local evidence.

## Kubernetes Shutdown

```bash
kubectl -n acceptability-engine scale deployment/acceptability-engine --replicas=0
kubectl -n acceptability-engine rollout status deployment/acceptability-engine
```

Do not delete persistent volume claims during ordinary shutdown.

## Success Criteria

- The runtime process has stopped.
- SQLite data storage remains present.
- Artifact storage remains present.
- No manual evidence mutation was performed.

## Failure Response

Use [Incident response](incident_response.md) if shutdown hangs or evidence
storage becomes unavailable.
