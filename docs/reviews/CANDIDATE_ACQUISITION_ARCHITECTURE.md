# Candidate Acquisition Architecture

## Scope

This review defines the admitted change object for Git-mode admission.

Implementation is now complete for commit-SHA candidate admission. Patch,
archive, and pull-request-number admission remain out of scope.

## Decision

The admitted object is `candidate_sha`.

The admission boundary is:

```text
repo_url + base_sha + candidate_sha
```

The change being admitted is the Git diff:

```text
base_sha..candidate_sha
```

`candidate_ref` may exist as provenance metadata to help explain or fetch the
candidate. It is not authority.

## Rejected Alternatives

| Alternative | Reason Rejected |
| :--- | :--- |
| Branch name | Mutable names can move after submission. |
| Tag name | Tags can be mutable unless separately verified and policy-bound. |
| Pull request number | Host-specific metadata, not a stable Git object. |
| Pull request ref | Useful fetch metadata, but still mutable host state. |
| Patch text only | Requires a separate patch identity, application model, and artifact verification model. |
| Archive upload | Requires object storage, signature, extraction, and provenance design before it can be authoritative. |

## Contract Model

Contracts carry:

```text
repo_url
base_sha
candidate_sha
candidate_ref optional
scopes
requires_human_review
admission_policy
```

`candidate_sha` must be a 40-character hexadecimal commit SHA.

`candidate_ref` must not be used as the admitted object. It may be recorded as
provenance metadata only.

## Materialization Flow

Git materialization:

1. validate the contract shape
2. clone `repo_url` into the per-run workspace
3. verify `origin` matches `repo_url`
4. verify `base_sha` resolves inside `repo_url`
5. verify `candidate_sha` resolves inside `repo_url`
6. verify `base_sha` is an ancestor of `candidate_sha`
7. detach `HEAD` at `candidate_sha`
8. verify workspace `HEAD` equals `candidate_sha`
9. run gates against that workspace

Gate 3 must evaluate:

```text
git diff --name-only base_sha..candidate_sha
```

During gate execution:

```text
workspace HEAD == candidate_sha
```

## Evidence Chain

The durable evidence chain is:

```text
contract
  -> candidate identity
  -> scopes and admission policy
  -> materialized workspace at candidate_sha
  -> gate evidence
  -> policy trace
  -> review decision when required
  -> final decision
```

The contract record must preserve `repo_url`, `base_sha`, `candidate_sha`,
optional `candidate_ref`, scopes, and admission policy.

Gate evidence must be attributable to the exact `candidate_sha` that was
executed.

## Replay Impact

Replay includes `candidate_sha` and optional `candidate_ref` in the contract
section.

Replay must not resolve mutable refs. It should reconstruct the historical
admission record from persisted evidence.

## Retention Impact

Retention may delete artifact bytes only through the existing audited artifact
retention workflow.

Retention must not remove the persisted candidate identity from SQLite evidence.

## Policy Impact

Admission policy may reference gate outputs and persisted candidate identity.

Policy must not treat `candidate_ref` as authority. Any policy that needs the
admitted change must use `candidate_sha`.

## Security Impact

Repository policy remains enforced before submission.

Implementation ensures both `base_sha` and `candidate_sha` resolve inside the
authorized `repo_url`. A SHA that resolves only in another repository must fail.

Network access for fetching candidates remains controlled by workspace mode,
sandbox profile, and deployment egress policy.

## Migration Impact

Adding `candidate_sha` required:

- contract schema migration
- API model update
- CLI contract model update
- TypeScript model update
- replay model update
- Git materialization update
- Gate 3 comparison update
- legacy contract handling decision

Legacy contracts without `candidate_sha` are migrated by backfilling
`candidate_sha = base_sha`. That preserves historical evidence shape without
claiming that legacy rows represented production Git-mode admission of a remote
proposed change.

Legacy contracts without persisted scopes are migrated with `scopes_json = []`
for preservation only. New contracts persist the validated scope list.

## Open Questions

| Question | Current Position |
| :--- | :--- |
| Should `base_sha` have to be an ancestor? | Yes. The implemented model requires ancestor verification. |
| Should `candidate_ref` be persisted? | Yes, as optional provenance metadata only. |
| Should patch/archive candidates be supported? | Not before commit-SHA admission is implemented and reviewed. |
| Should Git egress be allowed under `kubernetes-restricted`? | Requires a controlled egress design; denied egress remains the default. |

## Conclusion

D25-001 is closed for commit-SHA candidate admission. `candidate_sha` is
implemented as the admitted object, Gate 3 compares `base_sha..candidate_sha`,
and the workspace is detached at `candidate_sha`.
