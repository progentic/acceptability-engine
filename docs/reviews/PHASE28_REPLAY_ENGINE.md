# Phase 28 Replay Engine

## Replay Contract

Replay input:

```text
run_id
```

Replay output:

```text
deterministic JSON report
```

Replay includes:

- contract
- run status
- attempts
- gate sequence
- gate results
- policy evaluations
- review decision, if present
- final decision, if present
- evidence descriptors
- missing artifact indicators
- replay generated_at
- replay source database identity, if available

Replay must not:

- execute gates
- mutate run state
- mutate evidence descriptors
- recreate deleted artifacts
- change final decisions

## Determinism Rules

The replay report orders collections by stable persisted identities:

- attempts by `attempt_number ASC, id ASC`
- gates by `gate_num ASC, id ASC`
- evidence descriptors by `created_at ASC, id ASC`

The replay report includes `generated_at` because the report itself is a new
observation. Historical content must come only from persisted evidence and
artifact existence checks.

## Authority Model

Replay is read-only. SQLite remains the authority for contracts, run state,
attempts, gate runs, review decisions, final decisions, and evidence
descriptors.

Filesystem artifacts are not authority. Replay may report whether an artifact
referenced by a descriptor is present or missing, but it must not recreate or
modify artifact bytes.

Progress events and metrics are not replay inputs.

## Validation Evidence

| Test | Coverage |
| :--- | :--- |
| `replay_report_includes_run_history` | Report includes contract, run, attempts, gates, evidence, review, and final decision data. |
| `replay_report_marks_missing_artifacts` | Report keeps evidence descriptors and marks deleted artifact bytes missing. |
| `replay_is_deterministic_except_generated_at` | Historical report content is stable across repeated replay generation. |
| `replay_missing_run_returns_none` | Missing run ids do not fabricate replay output. |

## Deviation Register

| ID | Status | Deviation | Disposition |
| :--- | :--- | :--- | :--- |
| D28-001 | Accepted limitation | Replay is exposed through the CLI first, not the HTTP API or UI. | Keeps the first implementation focused on the authoritative replay model; HTTP and UI replay can build on the same report shape later. |
| D28-002 | Accepted limitation | Replay reports artifact presence but does not read artifact bytes. | Preserves replay determinism and avoids large payload transport in this phase. |

## Conclusion

Replay reconstructs historical run state from durable evidence without running
gates, mutating state, or treating progress events as authority.
