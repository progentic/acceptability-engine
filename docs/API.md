# API

## Authentication

Production deployments use API-key mode.

Send the token with one of these headers:

```text
Authorization: Bearer token
X-API-Key: token
```

API keys are configured as:

```text
token|role|tenant|repo_prefixes
```

Roles:

- `viewer` can read runs and evidence.
- `submitter` can read runs and submit contracts.
- `reviewer` can read runs and make human review decisions.
- `admin` can read, submit, and review.

## Runs

### Submit Run

```text
POST /runs
```

Request:

```json
{
  "id": "run-001",
  "repo_url": "https://github.com/progentic/acceptability-engine.git",
  "base_sha": "a9993e364706816aba3e25717850c26c9cd0d89d",
  "candidate_sha": "b9993e364706816aba3e25717850c26c9cd0d89d",
  "candidate_ref": "refs/pull/12/head",
  "scopes": ["core/src"],
  "requires_human_review": true,
  "admission_policy": {
    "id": "strict-v1",
    "version": 1,
    "rules": {
      "require_all_gates_pass": true,
      "required_gates": [1, 2, 3, 4, 5, 6, 7, 8],
      "max_test_parse_errors": 0
    }
  }
}
```

`candidate_sha` is required and is the authoritative admitted object. `candidate_ref` is optional provenance metadata only; it must not decide what change is admitted.

`admission_policy` is optional. When omitted, the server applies `strict-v1`.
Unknown policy ids, unsupported policy versions, disabled mandatory gate checks, duplicate required gates, or reordered required gates are rejected during contract validation.

Response:

```json
{
  "run_id": 1,
  "status": "QUEUED",
  "reason": null
}
```

### List Runs

```text
GET /runs?status=PENDING_HUMAN_REVIEW&limit=50&offset=0
```

### Get Run

```text
GET /runs/:id
```

### List Run Attempts

```text
GET /runs/:id/attempts
```

### List Run Evidence

```text
GET /runs/:id/evidence
```

Evidence rows may include `review_decision_id` when the evidence was produced by a human review decision.

### Stream Run Progress

```text
GET /runs/:id/progress
```

This endpoint upgrades to a WebSocket connection.

Optional query:

```text
after=42
```

When `after` is present, the server replays recent events for that run with a higher sequence number before streaming live events.

Replay is bounded in memory. If older events have aged out, the server streams the available newer events and then continues with live events. Durable evidence remains available through the run, attempt, gate, and evidence read endpoints.

Events share this envelope:

```json
{
  "sequence": 1,
  "run_id": 1,
  "created_at": 1780545600,
  "type": "gate_started"
}
```

Event types:

- `queued`
- `started`
- `attempt_started`
- `gate_started`
- `gate_finished`
- `finalized`
- `failed_internal`

`attempt_started` includes `attempt_id`.

`gate_started` includes `gate_num`.

`gate_finished` includes `gate_num`, `passed`, and `message`.

`finalized` includes `status`.

`failed_internal` includes `reason`.

## Attempts

### List Attempt Gates

```text
GET /attempts/:id/gates
```

## Human Review

Review endpoints require the `reviewer` or `admin` role.

The run must be in `PENDING_HUMAN_REVIEW`.

### Approve Run

```text
POST /runs/:id/review/approve
```

Request:

```json
{
  "reason": "Reviewer confirmed the evidence bundle."
}
```

Response:

```json
{
  "run_id": 1,
  "status": "APPROVED",
  "review_decision_id": 1,
  "evidence_bundle_id": 9
}
```

### Reject Run

```text
POST /runs/:id/review/reject
```

Request:

```json
{
  "reason": "Required evidence was missing."
}
```

Response:

```json
{
  "run_id": 1,
  "status": "REJECTED",
  "review_decision_id": 1,
  "evidence_bundle_id": 9
}
```

## Operations

```text
GET /health/live
GET /health/ready
GET /metrics
```
