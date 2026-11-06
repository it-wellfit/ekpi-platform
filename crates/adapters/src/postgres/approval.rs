#![forbid(unsafe_code)]

//! PostgreSQL approval repository.
//!
//! Implements [`ekp_application::ports::ApprovalRepository`] on top of
//! PostgreSQL 18.6, following the same advisory-lock + audit-chain pattern
//! as `governance.rs`.

use async_trait::async_trait;
use ekp_application::ports::{ApprovalRepository, GovernanceError};
use sqlx::postgres::PgPool;

/// PostgreSQL approval repository (skeleton).
///
/// All operations are fail-closed: they return `Err(Storage("not yet
/// implemented"))` until the migration and use cases are wired. The pool
/// accessor is the only read path, which keeps the struct live without
/// `allow(dead_code)`.
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
    async fn create_workflow(&self, _definition: &str) -> Result<String, GovernanceError> {
        // TODO: implement after migration lands
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn publish_workflow(&self, _id: &str, _version: u32) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn submit_task(&self, _task: &str) -> Result<String, GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn decide_task(
        &self,
        _task_id: &str,
        _decision: &str,
        _actor: &str,
    ) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn escalate_task(&self, _task_id: &str) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn cleanup_expired(&self) -> Result<u64, GovernanceError> {
        Ok(0)
    }
}
