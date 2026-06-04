# Phase 29 Admission Policy Engine

## Policy Scope

Admission policy is part of the submitted contract.

The initial policy schema is `strict-v1`. It is declarative JSON and does not execute scripts or runtime code.

The policy may add admission criteria. It may not disable mandatory gates or weaken the requirement that gates 1 through 8 pass.

Unknown policy ids and unsupported policy versions fail closed during contract validation. Required gates must be exactly ordered from 1 through 8 so policy traces remain deterministic.

## Policy Evaluation Order

The authority order is:

```text
Gate Result
      ↓
Policy Evaluation
      ↓
Human Review Requirement
      ↓
Final Decision
```

This means a failed policy rejects the run before human review can suspend it. Human review can only apply to a policy-passing run.

## Policy Evidence Model

Policy evaluation records are persisted in `policy_evaluations`.

Each policy evaluation records:

- run id
- attempt id
- policy id
- policy version
- pass/fail outcome
- reason
- JSON trace
- timestamp

Policy evaluation is part of normal run finalization. Gate records, policy trace, evidence descriptors, attempt status, run status, and final decision are committed together.

## Validation Evidence

- Policy fixtures are represented by the default `strict-v1` contract policy.
- Policy evaluation tests cover required gates, failed gates, parse-error limits, unsupported policies, deterministic gate ordering, and attempts to weaken mandatory gates.
- Orchestrator tests cover policy-driven approval, rejection, human-review suspension, policy trace persistence, and transaction rollback.
- Replay includes policy evaluations in deterministic report output.

## Deviations

`D29-001`: Policy is contract-scoped and declarative. There is no server-global policy registry or dynamic policy scripting.

`D29-002`: Policy trace evidence is stored in SQLite. Filesystem artifact payloads for policy traces remain out of scope for this phase.
