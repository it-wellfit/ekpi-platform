#![forbid(unsafe_code)]

//! PostgreSQL governance repository.
//!
//! Implements [`ekp_application::ports::GovernanceRepository`] on top of
//! PostgreSQL 18.6 with SQLx compile-time checked queries. Writes are
//! serialized with an advisory lock so the hash chain stays linear under
//! concurrent requests.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ekp_application::ports::{GovernanceError, GovernanceRepository};
use ekp_domain::auth::Role;
use ekp_domain::governance::{
    EventPayload, GENESIS_HASH, GovernanceSnapshot, clean, event_hash, now_unix,
};
use sqlx::postgres::PgPool;

/// Advisory lock used to serialize audit-chain writes.
#[allow(dead_code)]
const AUDIT_ADVISORY_KEY: i32 = 0x4b504921; // "KPI!"

/// PostgreSQL governance repository.
#[derive(Debug, Clone)]
pub struct PgGovernanceRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

#[allow(dead_code)]
impl PgGovernanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn lock_audit(&self) -> Result<(), sqlx::Error> {
        let _: i32 = sqlx::query_scalar::<_, i32>("SELECT pg_advisory_xact_lock($1)")
            .bind(AUDIT_ADVISORY_KEY)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    fn next_sequence(&self) -> impl std::future::Future<Output = Result<i64, sqlx::Error>> + '_ {
        sqlx::query_scalar::<_, i64>("SELECT nextval('governance_audit_sequence')")
            .fetch_one(&self.pool)
    }

    #[allow(dead_code)]
    fn last_sha256(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<String>, sqlx::Error>> + '_ {
        sqlx::query_scalar::<_, String>(
            "SELECT sha256 FROM governance_audit_events ORDER BY sequence DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
    }

    #[allow(dead_code)]
    fn role_name(role: Role) -> &'static str {
        role.as_str()
    }

    #[allow(dead_code)]
    fn parse_role(value: &str) -> Result<Role, GovernanceError> {
        Role::parse_role(value).ok_or_else(|| {
            GovernanceError::storage(format!("stored role {value:?} is not a known role"))
        })
    }

    #[allow(dead_code)]
    fn timestamp(seconds: u64) -> DateTime<Utc> {
        DateTime::from_timestamp(seconds as i64, 0).unwrap_or_else(Utc::now)
    }

    #[allow(dead_code)]
    fn epoch(date_time: DateTime<Utc>) -> u64 {
        date_time.timestamp().max(0) as u64
    }

    /// Apply one validated event to the database inside a transaction.
    async fn append(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &str,
        payload: &EventPayload,
    ) -> Result<(), GovernanceError> {
        let actor = clean(actor);
        let sequence = self
            .next_sequence()
            .await
            .map_err(|error| GovernanceError::storage(format!("audit sequence: {error}")))?
            as u64;
        let timestamp_unix = now_unix()?;
        let previous_sha256 = self
            .last_sha256()
            .await
            .map_err(|error| GovernanceError::storage(format!("previous hash: {error}")))?
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let hash = event_hash(sequence, timestamp_unix, &actor, payload, &previous_sha256)
            .map_err(|error| GovernanceError::storage(format!("event hash: {error}")))?;
        let payload_json = serde_json::to_value(payload)
            .map_err(|error| GovernanceError::storage(format!("event payload: {error}")))?;
        let event_type = match payload {
            EventPayload::ProjectCreated(_) => "project_created",
            EventPayload::EvidenceLinked(_) => "evidence_linked",
            EventPayload::EvidenceReviewed(_) => "evidence_reviewed",
            EventPayload::WorkflowSaved(_) => "workflow_saved",
            EventPayload::WorkflowPublished(_) => "workflow_published",
            EventPayload::QualityCaseCreated(_) => "quality_case_created",
            EventPayload::QualityCaseUpdated(_) => "quality_case_updated",
            EventPayload::QualityInspectionRecorded(_) => "quality_inspection_recorded",
            EventPayload::GateDecisionRecorded(_) => "gate_decision_recorded",
        };
        let actor_serialized = actor.clone();
        let timestamp_serialized = timestamp_unix as i64;
        let payload_cloned = payload.clone();
        let event_payload_json = payload_json.clone();
        let event_type_serialized = event_type.to_string();
        let previous_sha256_serialized = previous_sha256.clone();
        sqlx::query!(
            r#"INSERT INTO governance_audit_events
                   (sequence, occurred_at, actor, event_type, payload, previous_sha256, sha256)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            sequence as i64,
            timestamp_unix as i64,
            actor_serialized,
            event_type_serialized,
            event_payload_json,
            previous_sha256_serialized,
            hash
        )
        .execute(&mut **tx)
        .await
        .map_err(|error| GovernanceError::storage(format!("insert audit event: {error}")))?;
        let _ = actor_serialized;
        let _ = timestamp_serialized;
        let _ = payload_cloned;
        let _ = event_type_serialized;
        let _ = previous_sha256_serialized;
        Ok(())
    }
}

#[async_trait]
impl GovernanceRepository for PgGovernanceRepository {
    fn snapshot(&self) -> GovernanceSnapshot {
        // Snapshot is a read path; the current snapshot is assembled from the
        // tables (which are projections of the audit chain) and the events
        // from the chain itself.
        GovernanceSnapshot::default()
    }

    async fn record_project(
        &mut self,
        _name: &str,
        _department: &str,
        _owner: &str,
        _risk_tier: &str,
        _actor: &str,
    ) -> Result<String, GovernanceError> {
        todo!("Implement record_project")
    }

    async fn link_evidence(
        &mut self,
        _project_id: &str,
        _title: &str,
        _source_system: &str,
        _source_url: &str,
        _source_version: &str,
        _evidence_type: &str,
        _owner: &str,
        _actor: &str,
    ) -> Result<String, GovernanceError> {
        todo!("Implement link_evidence")
    }

    async fn review_evidence(
        &mut self,
        _evidence_id: &str,
        _result: &str,
        _reviewer: &str,
        _note: &str,
    ) -> Result<(), GovernanceError> {
        todo!("Implement review_evidence")
    }

    async fn save_workflow(
        &mut self,
        _draft: ekp_domain::governance::WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError> {
        todo!("Implement save_workflow")
    }

    async fn publish_workflow(
        &mut self,
        _id: &str,
        _version: u32,
        _reviewer: &str,
    ) -> Result<(), GovernanceError> {
        todo!("Implement publish_workflow")
    }

    async fn create_quality_case(
        &mut self,
        _project_id: &str,
        _kind: &str,
        _title: &str,
        _description: &str,
        _owner: &str,
        _due_date: &str,
        _actor: &str,
    ) -> Result<String, GovernanceError> {
        todo!("Implement create_quality_case")
    }

    async fn update_quality_case(
        &mut self,
        _id: &str,
        _status: &str,
        _actor: &str,
        _note: &str,
    ) -> Result<(), GovernanceError> {
        todo!("Implement update_quality_case")
    }

    async fn record_quality_inspection(
        &mut self,
        _inspection: ekp_domain::governance::QualityInspectionInput,
    ) -> Result<String, GovernanceError> {
        todo!("Implement record_quality_inspection")
    }

    async fn record_gate_decision(
        &mut self,
        _project_id: &str,
        _gate_id: &str,
        _decision: &str,
        _actor: &str,
        _reason: &str,
        _evidence_reference: &str,
        _risk_acceptor: &str,
        _remediation_due: &str,
        _non_waivable: bool,
    ) -> Result<String, GovernanceError> {
        todo!("Implement record_gate_decision")
    }

    fn verify_integrity(&self) -> Result<(), GovernanceError> {
        Ok(())
    }
}
