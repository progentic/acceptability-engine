use super::clock::current_unix_seconds;
use super::evidence::{create_evidence_bundle_record, EvidenceDescriptor};
use super::final_decisions::record_final_decision;
use super::runs::update_run_status;
use super::transaction::with_transaction;
use super::types::{EvidenceBundleId, ReviewDecisionId, RunId};
use crate::error::StoreError;
use rusqlite::{Connection, Row};

const PENDING_HUMAN_REVIEW: &str = "PENDING_HUMAN_REVIEW";

pub struct ReviewDecisionInput<'a> {
    pub run_id: RunId,
    pub tenant_id: &'a str,
    pub reviewer_actor: &'a str,
    pub reviewer_role: &'a str,
    pub decision: ReviewDecisionKind,
    pub reason: &'a str,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReviewDecisionKind {
    Approve,
    Reject,
}

pub struct RecordedReviewDecision {
    pub review_decision_id: ReviewDecisionId,
    pub evidence_bundle_id: EvidenceBundleId,
}

impl ReviewDecisionKind {
    pub fn status(self) -> &'static str {
        match self {
            Self::Approve => "APPROVED",
            Self::Reject => "REJECTED",
        }
    }

    fn evidence_label(self) -> &'static str {
        match self {
            Self::Approve => "Human review approved run",
            Self::Reject => "Human review rejected run",
        }
    }
}

pub fn finalize_human_review(
    conn: &Connection,
    input: ReviewDecisionInput<'_>,
) -> Result<RecordedReviewDecision, StoreError> {
    validate_review_reason(input.reason)?;
    with_transaction(conn, |conn| finalize_review_transaction(conn, &input))
}

fn finalize_review_transaction(
    conn: &Connection,
    input: &ReviewDecisionInput<'_>,
) -> Result<RecordedReviewDecision, StoreError> {
    ensure_reviewable_run(conn, input.run_id, input.tenant_id)?;
    let review_decision_id = insert_review_decision(conn, input)?;
    let evidence_bundle_id = insert_review_evidence(conn, input, review_decision_id)?;
    update_run_status(conn, input.run_id, input.decision.status())?;
    record_final_decision(
        conn,
        input.run_id,
        input.decision.status(),
        Some(input.reason),
    )?;
    Ok(RecordedReviewDecision {
        review_decision_id,
        evidence_bundle_id,
    })
}

fn validate_review_reason(reason: &str) -> Result<(), StoreError> {
    if !reason.trim().is_empty() {
        return Ok(());
    }
    Err(StoreError::InvalidParameter(
        "review reason must not be empty".to_string(),
    ))
}

fn ensure_reviewable_run(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<(), StoreError> {
    let Some(status) = fetch_tenant_run_status(conn, run_id, tenant_id)? else {
        return Err(StoreError::InvalidParameter(
            "run not found for tenant".to_string(),
        ));
    };
    if status == PENDING_HUMAN_REVIEW {
        return Ok(());
    }
    Err(StoreError::InvalidParameter(format!(
        "run must be PENDING_HUMAN_REVIEW before review decision, found {status}"
    )))
}

fn fetch_tenant_run_status(
    conn: &Connection,
    run_id: RunId,
    tenant_id: &str,
) -> Result<Option<String>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT status FROM runs WHERE id = ?1 AND tenant_id = ?2")
        .map_err(|source| StoreError::QueryFailed { source })?;
    let mut rows = stmt
        .query(rusqlite::params![run_id.get(), tenant_id])
        .map_err(|source| StoreError::QueryFailed { source })?;
    let Some(row) = rows
        .next()
        .map_err(|source| StoreError::QueryFailed { source })?
    else {
        return Ok(None);
    };
    read_string(row, 0).map(Some)
}

fn insert_review_decision(
    conn: &Connection,
    input: &ReviewDecisionInput<'_>,
) -> Result<ReviewDecisionId, StoreError> {
    conn.execute(
        "INSERT INTO review_decisions (
            run_id, tenant_id, reviewer_actor, reviewer_role, decision, reason, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            input.run_id.get(),
            input.tenant_id,
            input.reviewer_actor,
            input.reviewer_role,
            input.decision.status(),
            input.reason,
            current_unix_seconds()?
        ],
    )
    .map_err(|source| StoreError::InsertFailed { source })?;
    Ok(ReviewDecisionId::new(conn.last_insert_rowid()))
}

fn insert_review_evidence(
    conn: &Connection,
    input: &ReviewDecisionInput<'_>,
    review_decision_id: ReviewDecisionId,
) -> Result<EvidenceBundleId, StoreError> {
    create_evidence_bundle_record(
        conn,
        EvidenceDescriptor {
            run_id: input.run_id,
            attempt_id: None,
            gate_run_id: None,
            review_decision_id: Some(review_decision_id),
            kind: "human_review",
            label: input.decision.evidence_label(),
            storage_uri: None,
            sha256: None,
            byte_len: None,
            content_type: None,
            summary: input.reason,
        },
    )
}

fn read_string(row: &Row<'_>, index: usize) -> Result<String, StoreError> {
    row.get(index)
        .map_err(|source| StoreError::QueryFailed { source })
}
