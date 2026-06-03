CREATE TABLE evidence_bundles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    attempt_id INTEGER,
    gate_run_id INTEGER,
    kind TEXT NOT NULL DEFAULT 'summary',
    label TEXT NOT NULL DEFAULT 'Evidence summary',
    storage_uri TEXT,
    sha256 TEXT,
    byte_len INTEGER,
    content_type TEXT,
    summary TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id),
    FOREIGN KEY(attempt_id) REFERENCES attempts(id),
    FOREIGN KEY(gate_run_id) REFERENCES gate_runs(id)
);
