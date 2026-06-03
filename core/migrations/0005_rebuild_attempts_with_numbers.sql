ALTER TABLE attempts RENAME TO legacy_attempts;

CREATE TABLE attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id),
    UNIQUE(run_id, attempt_number)
);

INSERT INTO attempts (id, run_id, attempt_number, status, created_at)
SELECT
    id,
    run_id,
    (
        SELECT COUNT(*)
        FROM legacy_attempts previous
        WHERE previous.run_id = legacy_attempts.run_id
          AND previous.id <= legacy_attempts.id
    ),
    status,
    created_at
FROM legacy_attempts;

DROP TABLE legacy_attempts;
