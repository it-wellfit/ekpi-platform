#![forbid(unsafe_code)]

//! PostgreSQL approval repository.
//!
//! Implements [`ekp_application::ports::ApprovalRepository`] on top of
//! PostgreSQL 18.6, following the same advisory-lock + audit-chain pattern
//! as `governance.rs`. All operations are **fail-closed**: they return
//! `Err(Storage("not yet implemented"))` until the migration
//! (`migrations/20261106000000_approval_workflow.sql`) and the queries are
//! wired. The local JSONL store is the working implementation.

use async_trait::async_trait;
use ekp_application::ports::{ApprovalRepository, GovernanceError};
use ekp_domain::workflow::{
    ApprovalTask, TaskAction, TaskDecision, TaskSubmission, WorkflowDefinition,
};
use sqlx::postgres::PgPool;

/// PostgreSQL approval repository (skeleton).
#[derive(Debug, Clone)]
pub struct PgApprovalRepository {
    pool: PgPool,
}

impl PgApprovalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ApprovalRepository for PgApprovalRepository {
    async fn create_workflow(
        &mut self,
        _definition: &WorkflowDefinition,
        _actor: &str,
    ) -> Result<String, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn publish_workflow(
        &mut self,
        _workflow_id: &str,
        _version: u32,
        _actor: &str,
    ) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn submit_task(
        &mut self,
        _input: &TaskSubmission,
        _actor: &str,
        _now_unix: i64,
    ) -> Result<String, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn decide_task(
        &mut self,
        _task_id: &str,
        _action: TaskAction,
        _actor: &str,
        _note: &str,
        _now_unix: i64,
    ) -> Result<TaskDecision, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn cleanup_expired(&mut self, _now_unix: i64) -> Result<u64, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn list_tasks(&self) -> Result<Vec<ApprovalTask>, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    fn verify(&self) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }
}
