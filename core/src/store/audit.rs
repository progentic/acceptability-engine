use super::clock::current_unix_seconds;
use super::types::AuditEvent;
use crate::error::StoreError;
use rusqlite::Connection;

pub fn record_audit_event(conn: &Connection, event: &AuditEvent) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO audit_events (
            tenant_id, actor, role, action, resource_type, resource_id, outcome, reason, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            event.tenant_id,
            event.actor,
            event.role,
            event.action,
            event.resource_type,
            event.resource_id,
            event.outcome,
            event.reason,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(())
}
