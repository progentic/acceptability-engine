ALTER TABLE evidence_bundles RENAME TO legacy_evidence_bundles;

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

INSERT INTO evidence_bundles (
    id, run_id, attempt_id, gate_run_id, kind, label, summary, created_at
)
SELECT
    legacy_evidence_bundles.id,
    attempts.run_id,
    legacy_evidence_bundles.attempt_id,
    NULL,
    'summary',
    legacy_evidence_bundles.summary,
    legacy_evidence_bundles.summary,
    legacy_evidence_bundles.created_at
FROM legacy_evidence_bundles
JOIN attempts ON attempts.id = legacy_evidence_bundles.attempt_id;

DROP TABLE legacy_evidence_bundles;
