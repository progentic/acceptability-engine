# Phase 33 Backup / Restore Validation

## Purpose

This report records the Phase 33 backup validation model.

Phase 33 produces the reusable recovery fixture that Phase 34 consumes during
destructive restore validation.

## Verification Contract

The backup contract is:

```text
create fixture run history
capture pre-backup replay JSON
backup SQLite + artifacts
validate backup inventory
record restore prerequisites
```

## Recovery Fixture

The fixture includes:

- contract data
- run status
- attempt data
- gate result
- policy evaluation
- review decision
- final decision
- evidence descriptor
- artifact bytes

## Backup Artifact Shape

The documented operator shape is:

```text
backup-root/
  evidence.db
  artifacts/
  replay/
    run-<id>-pre-backup.json
  inventory.txt
```

## Integrity Validation

Automated integrity validation is represented by
`backup_validation_creates_reusable_recovery_fixture`.

The test verifies:

- a file-backed SQLite database is created
- artifact bytes are created
- SQLite is copied to the backup location
- artifacts are copied to the backup location
- a pre-backup replay JSON file is written to the backup
- `inventory.txt` is written to the backup
- the inventory includes SQLite, replay, and artifact entries
- the inventory hashes match copied files
- at least one artifact file exists in the backup
- pre-backup replay output is generated from the fixture

## Restore Prerequisites

Operators must preserve:

- backup SQLite path
- backup artifact root
- pre-backup replay JSON
- selected replay run id
- target SQLite path
- target artifact root

## Validation Evidence

The following command validates the Phase 33 fixture:

```bash
cargo test backup_validation_creates_reusable_recovery_fixture
```

The full Phase 33 validation set is:

- `git diff --check`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`

## Notes / Deviations

- The backup helper is test-scoped and validates the evidence model without adding a production backup CLI.
- External backup tooling remains deployment-specific.
- Phase 34 consumes the same fixture for destructive restore validation.

## Conclusion

Phase 33 establishes backup evidence, backup inventory expectations, and a
reusable recovery fixture for Phase 34.
