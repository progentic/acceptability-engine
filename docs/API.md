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
  "scopes": ["core/src"],
  "requires_human_review": true
}
```

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
