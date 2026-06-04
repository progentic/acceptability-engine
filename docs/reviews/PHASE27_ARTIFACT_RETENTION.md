# Phase 27 Artifact Retention

## Scope

This report documents the filesystem artifact retention workflow added in
Phase 27.

Retention applies only to artifact files addressed by `artifact://` storage
URIs. It does not delete or mutate SQLite evidence descriptors.

## Retention Policy

Operators run retention from the CLI with:

```text
--retention-days N
```

The engine selects artifact-backed evidence rows whose `created_at` timestamp
is older than the computed cutoff.

Operators may preview eligible artifacts with:

```text
--retention-days N --retention-dry-run
```

Dry runs record audit evidence but do not delete files.

## Cleanup Workflow

1. Select eligible `evidence_bundles` rows with a non-null `storage_uri`.
2. Record a planned or dry-run audit event before any delete.
3. Resolve the `artifact://` URI to a path under the configured artifact root.
4. Reject invalid URIs, traversal, and symlinked parent directories.
5. Delete the artifact file when not in dry-run mode.
6. Record `DELETED` or `MISSING` audit evidence.
7. Leave the SQLite evidence descriptor unchanged.

## Audit Records

Retention audit events use:

| Field | Value |
| :--- | :--- |
| `tenant_id` | `system` |
| `actor` | `artifact-retention` |
| `role` | `system` |
| `action` | `artifacts.retention` |
| `resource_type` | `evidence_bundle` |
| `resource_id` | Evidence bundle id |
| `outcome` | `DRY_RUN`, `PLANNED`, `DELETED`, or `MISSING` |
| `reason` | Artifact storage URI |

## Validation Evidence

| Test | Coverage |
| :--- | :--- |
| `dry_run_records_plans_without_deleting_artifacts` | Dry-run retention records audit evidence and preserves artifact files. |
| `cleanup_deletes_artifact_and_keeps_evidence_descriptor` | Cleanup deletes filesystem bytes, records audit evidence, and leaves evidence rows intact. |
| `cleanup_records_missing_artifacts` | Missing files are audited as missing. |
| `retention_ignores_newer_artifacts` | Newer artifacts remain untouched. |
| `dry_run_rejects_symlink_parent_before_audit_planning` | Dry-run validates artifact paths before writing planning audit records. |
| `deletes_artifact_file_under_root` | Artifact store deletes files addressed by valid internal artifact URIs. |
| `rejects_artifact_uri_traversal` | Artifact store rejects traversal outside the artifact root. |
| `rejects_artifact_uri_backslashes` | Artifact store rejects platform-dependent backslash separators. |
| `rejects_symlink_artifact_root` | Artifact store rejects deletion through a symlink artifact root on Unix. |
| `rejects_symlink_artifact_parent` | Artifact store rejects deletion through a symlink parent directory on Unix. |

## Deviation Register

| ID | Status | Deviation | Disposition |
| :--- | :--- | :--- | :--- |
| D27-001 | Accepted limitation | Retention is a CLI workflow, not an HTTP API. | Keeps deletion out of the public API surface for this phase. |
| D27-002 | Accepted limitation | SQLite descriptors remain after artifact bytes are deleted. | Preserves immutable evidence records and leaves replay/inspection to report missing bytes explicitly. |

## Conclusion

Artifact retention is explicit, auditable, and constrained to filesystem
artifacts under the configured artifact root. It does not silently delete
evidence descriptors or create a second evidence authority.
