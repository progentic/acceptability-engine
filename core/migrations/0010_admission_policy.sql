CREATE TABLE IF NOT EXISTS policy_evaluations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    attempt_id INTEGER NOT NULL,
    policy_id TEXT NOT NULL,
    policy_version INTEGER NOT NULL,
    passed INTEGER NOT NULL,
    reason TEXT NOT NULL,
    trace_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id),
    FOREIGN KEY(attempt_id) REFERENCES attempts(id)
);

CREATE INDEX IF NOT EXISTS idx_policy_evaluations_run_created_at
    ON policy_evaluations(run_id, created_at);

CREATE INDEX IF NOT EXISTS idx_policy_evaluations_attempt
    ON policy_evaluations(attempt_id);
