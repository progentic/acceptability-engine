ALTER TABLE gate_runs RENAME TO legacy_gate_runs;

INSERT INTO attempts (run_id, attempt_number, status, created_at)
SELECT id, 1, status, created_at
FROM runs
WHERE NOT EXISTS (
    SELECT 1
    FROM attempts
    WHERE attempts.run_id = runs.id
      AND attempts.attempt_number = 1
);

CREATE TABLE gate_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    attempt_id INTEGER NOT NULL,
    gate_num INTEGER NOT NULL,
    passed INTEGER NOT NULL,
    message TEXT NOT NULL,
    exit_code INTEGER,
    duration_ms INTEGER,
    stdout BLOB,
    stderr BLOB,
    test_passed INTEGER,
    test_failed INTEGER,
    test_ignored INTEGER,
    parse_errors INTEGER,
    FOREIGN KEY(attempt_id) REFERENCES attempts(id)
);

INSERT INTO gate_runs (
    attempt_id, gate_num, passed, message, exit_code, duration_ms,
    stdout, stderr, test_passed, test_failed, test_ignored, parse_errors
)
SELECT
    attempts.id,
    legacy_gate_runs.gate_num,
    legacy_gate_runs.passed,
    legacy_gate_runs.message,
    legacy_gate_runs.exit_code,
    legacy_gate_runs.duration_ms,
    legacy_gate_runs.stdout,
    legacy_gate_runs.stderr,
    legacy_gate_runs.test_passed,
    legacy_gate_runs.test_failed,
    legacy_gate_runs.test_ignored,
    legacy_gate_runs.parse_errors
FROM legacy_gate_runs
JOIN attempts
  ON attempts.run_id = legacy_gate_runs.run_id
 AND attempts.attempt_number = 1;

DROP TABLE legacy_gate_runs;
