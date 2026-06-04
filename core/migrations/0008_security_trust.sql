CREATE TABLE IF NOT EXISTS audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tenant_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    role TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    outcome TEXT NOT NULL,
    reason TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_runs_tenant_status_created_at
    ON runs(tenant_id, status, created_at);

CREATE INDEX IF NOT EXISTS idx_audit_events_tenant_created_at
    ON audit_events(tenant_id, created_at);

CREATE INDEX IF NOT EXISTS idx_audit_events_action_created_at
    ON audit_events(action, created_at);
