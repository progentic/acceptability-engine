CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY,
    repo_url TEXT NOT NULL,
    base_sha TEXT NOT NULL,
    requires_human_review INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(contract_id) REFERENCES contracts(id)
);

CREATE TABLE IF NOT EXISTS attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id),
    UNIQUE(run_id, attempt_number)
);

CREATE TABLE IF NOT EXISTS final_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL UNIQUE,
    decision TEXT NOT NULL,
    reason TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES runs(id)
);
