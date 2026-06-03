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
