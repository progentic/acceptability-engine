# Startup Runbook

## Purpose

Start the engine and prove it is ready to accept work.

## Pre-Start Checks

1. Confirm the deployment mode.

   ```bash
   printenv AH_WORKSPACE_MODE
   printenv AH_SECURITY_MODE
   printenv AH_SANDBOX_PROFILE
   ```

2. Confirm production values.

   ```text
   AH_SECURITY_MODE=api-key
   AH_SANDBOX_PROFILE=kubernetes-restricted
   ```

3. Confirm the writable runtime paths exist.

   ```text
   /data
   /artifacts
   /workspaces
   /tmp
   ```

4. Confirm the API key secret is not the placeholder value.

   ```text
   replace-me|admin|default|*
   ```

   Startup rejects known placeholder tokens, including:

   ```text
   changeme
   change-me
   placeholder
   default
   example
   replace-me
   replace_this
   replace-this
   your-token-here
   ```

## Compose Startup

```bash
docker compose up --build
```

Compose uses `AH_SANDBOX_PROFILE=development`. It is for local operation and
does not provide the production containment baseline.

## Kubernetes Startup

```bash
kubectl apply -f deploy/kubernetes.yaml
kubectl -n acceptability-engine rollout status deployment/acceptability-engine
```

## Readiness Checks

```bash
curl --fail http://127.0.0.1:8080/health/live
curl --fail http://127.0.0.1:8080/health/ready
curl --fail http://127.0.0.1:8080/metrics
```

## Success Criteria

- Liveness returns `ok`.
- Readiness returns `ready`.
- Metrics include `acceptability_engine_uptime_seconds`.
- No startup log reports invalid workspace mode, invalid sandbox profile, placeholder API key, or SQLite readiness failure.

## Failure Response

Use [Incident response](incident_response.md) when startup checks fail.
