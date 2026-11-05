#![forbid(unsafe_code)]

//! PostgreSQL connection pool and migration runner.

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Pool tuning parameters.
#[derive(Debug, Clone, Copy)]
pub struct PoolSettings {
    pub max_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for PoolSettings {
    fn default() -> Self {
        Self {
            max_connections: 10,
            acquire_timeout_secs: 10,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        }
    }
}

/// Open a pool with explicit limits.
pub async fn connect(database_url: &str, settings: &PoolSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .acquire_timeout(Duration::from_secs(settings.acquire_timeout_secs))
        .idle_timeout(Some(Duration::from_secs(settings.idle_timeout_secs)))
        .max_lifetime(Some(Duration::from_secs(settings.max_lifetime_secs)))
        .connect(database_url)
        .await
}

/// Apply every migration in the workspace `migrations` directory.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}

/// Return `true` when the database answers a trivial query.
pub async fn is_reachable(pool: &PgPool) -> bool {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}
