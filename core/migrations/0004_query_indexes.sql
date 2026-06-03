CREATE INDEX IF NOT EXISTS idx_runs_status_created_at
    ON runs(status, created_at);

CREATE INDEX IF NOT EXISTS idx_runs_contract_id
    ON runs(contract_id);

CREATE INDEX IF NOT EXISTS idx_attempts_run_number
    ON attempts(run_id, attempt_number);

CREATE INDEX IF NOT EXISTS idx_gate_runs_attempt_gate
    ON gate_runs(attempt_id, gate_num);

CREATE INDEX IF NOT EXISTS idx_evidence_bundles_run_created_at
    ON evidence_bundles(run_id, created_at);

CREATE INDEX IF NOT EXISTS idx_evidence_bundles_attempt
    ON evidence_bundles(attempt_id);

CREATE INDEX IF NOT EXISTS idx_evidence_bundles_gate_run
    ON evidence_bundles(gate_run_id);
