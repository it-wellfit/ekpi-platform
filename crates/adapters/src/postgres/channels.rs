#![forbid(unsafe_code)]

//! PostgreSQL channel repository (skeleton).

use async_trait::async_trait;
use ekp_application::ports::{ChannelRepository, GovernanceError};
use sqlx::postgres::PgPool;

/// PostgreSQL channel repository (skeleton).
///
/// Fail-closed like [`PgApprovalRepository`]: registration is refused until
/// wired; health reports `Ok(false)` (unknown/unhealthy is the honest default)
/// and the channel list is genuinely empty.
#[derive(Debug, Clone)]
pub struct PgChannelRepository {
    pool: PgPool,
}

impl PgChannelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ChannelRepository for PgChannelRepository {
    async fn register_channel(
        &self,
        _platform_id: &str,
        _config: &str,
    ) -> Result<(), GovernanceError> {
        Err(GovernanceError::Storage("not yet implemented".into()))
    }

    async fn channel_health(&self, _platform_id: &str) -> Result<bool, GovernanceError> {
        Ok(false)
    }

    async fn list_channels(&self) -> Result<Vec<String>, GovernanceError> {
        Ok(vec![])
    }
}
