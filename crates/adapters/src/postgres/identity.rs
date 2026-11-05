#![forbid(unsafe_code)]

//! PostgreSQL identity adapter.
//!
//! Session tokens are stored as SHA-256 digests, never in clear text, so a
//! database disclosure does not yield usable session cookies. The CSRF token
//! is stored in clear text because the server has to present it to the browser
//! inside the page.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ekp_application::ports::{GovernanceError, LoginThrottle, SessionRepository, UserRepository};
use ekp_domain::auth::{Role, Session, UserRecord};
use ekp_domain::hashing::sha256_hex;
use sqlx::postgres::PgPool;

/// PostgreSQL account store.
#[derive(Debug, Clone)]
pub struct PgUserStore {
    pool: PgPool,
}

impl PgUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn parse_role(value: &str) -> Result<Role, GovernanceError> {
    Role::parse_role(value).ok_or_else(|| {
        GovernanceError::storage(format!("stored role {value:?} is not a known role"))
    })
}

fn role_name(role: Role) -> &'static str {
    role.as_str()
}

#[async_trait]
impl UserRepository for PgUserStore {
    async fn find_by_username(&self, username: &str) -> Option<UserRecord> {
        let row = sqlx::query!(
            r#"SELECT username, password_hash, role, created_at, disabled
               FROM users
               WHERE username = $1"#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(UserRecord {
            username: row.username,
            password_hash: row.password_hash,
            role: parse_role(&row.role).ok()?,
            created_at: row.created_at.timestamp().max(0) as u64,
            disabled: row.disabled,
        })
    }

    async fn create_user(&mut self, user: UserRecord) -> Result<(), GovernanceError> {
        let created_at: DateTime<Utc> =
            DateTime::from_timestamp(user.created_at as i64, 0).unwrap_or_else(Utc::now);
        let result = sqlx::query!(
            r#"INSERT INTO users (username, password_hash, role, created_at, disabled)
               VALUES ($1, $2, $3, $4, $5)"#,
            user.username,
            user.password_hash,
            role_name(user.role),
            created_at,
            user.disabled
        )
        .execute(&self.pool)
        .await
        .map_err(|error| {
            if let Some(database_error) = error.as_database_error()
                && database_error.code().as_deref() == Some("23505")
            {
                return GovernanceError::conflict("user already exists");
            }
            GovernanceError::storage(format!("insert user: {error}"))
        })?;
        let _ = result;
        Ok(())
    }

    async fn list_users(&self) -> Vec<UserRecord> {
        let rows = sqlx::query!(
            r#"SELECT username, password_hash, role, created_at, disabled
               FROM users
               ORDER BY username"#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .filter_map(|row| {
                Some(UserRecord {
                    username: row.username,
                    password_hash: row.password_hash,
                    role: parse_role(&row.role).ok()?,
                    created_at: row.created_at.timestamp().max(0) as u64,
                    disabled: row.disabled,
                })
            })
            .collect()
    }

    async fn set_disabled(
        &mut self,
        username: &str,
        disabled: bool,
    ) -> Result<(), GovernanceError> {
        let result = sqlx::query!(
            r#"UPDATE users SET disabled = $2 WHERE username = $1"#,
            username,
            disabled
        )
        .execute(&self.pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("update user: {error}")))?;
        if result.rows_affected() == 0 {
            return Err(GovernanceError::not_found("user not found"));
        }
        Ok(())
    }
}

/// PostgreSQL session store.
#[derive(Debug, Clone)]
pub struct PgSessionStore {
    pool: PgPool,
    idle_timeout_secs: u64,
}

impl PgSessionStore {
    pub fn new(pool: PgPool, idle_timeout_secs: u64) -> Self {
        Self {
            pool,
            idle_timeout_secs,
        }
    }

    fn token_digest(token: &str) -> String {
        sha256_hex(token.as_bytes())
    }
}

#[async_trait]
impl SessionRepository for PgSessionStore {
    async fn create_session(&mut self, session: Session) -> Result<(), GovernanceError> {
        let created_at = timestamp(session.created_at);
        let expires_at = timestamp(session.expires_at);
        let idle_deadline = timestamp(session.idle_deadline);
        sqlx::query!(
            r#"INSERT INTO sessions
                   (token_hash, username, role, csrf_token, created_at, expires_at, idle_deadline)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            Self::token_digest(&session.token),
            session.username,
            role_name(session.role),
            session.csrf_token,
            created_at,
            expires_at,
            idle_deadline
        )
        .execute(&self.pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("insert session: {error}")))?;
        Ok(())
    }

    async fn find_session(&self, token: &str, now: u64) -> Option<Session> {
        let now = timestamp(now);
        let row = sqlx::query!(
            r#"SELECT username, role, csrf_token, created_at, expires_at, idle_deadline
               FROM sessions
               WHERE token_hash = $1 AND expires_at > $2 AND idle_deadline > $2"#,
            Self::token_digest(token),
            now
        )
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(Session {
            token: token.to_string(),
            username: row.username,
            role: parse_role(&row.role).ok()?,
            created_at: row.created_at.timestamp().max(0) as u64,
            expires_at: row.expires_at.timestamp().max(0) as u64,
            idle_deadline: row.idle_deadline.timestamp().max(0) as u64,
            csrf_token: row.csrf_token,
        })
    }

    async fn touch_session(&mut self, token: &str, now: u64) -> Result<(), GovernanceError> {
        let idle_deadline = timestamp(now.saturating_add(self.idle_timeout_secs));
        let result = sqlx::query!(
            r#"UPDATE sessions
               SET idle_deadline = $3
               WHERE token_hash = $1 AND expires_at > $2 AND idle_deadline > $2"#,
            Self::token_digest(token),
            timestamp(now),
            idle_deadline
        )
        .execute(&self.pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("touch session: {error}")))?;
        if result.rows_affected() == 0 {
            return Err(GovernanceError::not_found("session expired"));
        }
        Ok(())
    }

    async fn delete_session(&mut self, token: &str) -> Result<(), GovernanceError> {
        sqlx::query!(
            r#"DELETE FROM sessions WHERE token_hash = $1"#,
            Self::token_digest(token)
        )
        .execute(&self.pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("delete session: {error}")))?;
        Ok(())
    }

    async fn purge_expired(&mut self, now: u64) -> Result<usize, GovernanceError> {
        let result = sqlx::query!(
            r#"DELETE FROM sessions WHERE expires_at <= $1 OR idle_deadline <= $1"#,
            timestamp(now)
        )
        .execute(&self.pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("purge sessions: {error}")))?;
        Ok(result.rows_affected() as usize)
    }

    async fn session_count(&self) -> usize {
        sqlx::query_scalar!("SELECT count(*) AS count FROM sessions")
            .fetch_one(&self.pool)
            .await
            .map(|count| count.unwrap_or(0) as usize)
            .unwrap_or(0)
    }
}

/// PostgreSQL login throttle.
#[derive(Debug, Clone)]
pub struct PgLoginThrottle {
    pool: PgPool,
    max_failures: u32,
    window_secs: u64,
}

impl PgLoginThrottle {
    pub fn new(pool: PgPool, max_failures: u32, window_secs: u64) -> Self {
        Self {
            pool,
            max_failures,
            window_secs,
        }
    }
}

#[async_trait]
impl LoginThrottle for PgLoginThrottle {
    async fn register_failure(&self, username: &str, now: u64) -> Result<(), String> {
        let window = self.window_secs;
        sqlx::query!(
            r#"INSERT INTO login_failures (username, failures, first_failure_at)
               VALUES ($1, 1, $2)
               ON CONFLICT (username) DO UPDATE
               SET failures = CASE
                   WHEN $2 - login_failures.first_failure_at >= $3 THEN 1
                   ELSE login_failures.failures + 1
               END,
               first_failure_at = CASE
                   WHEN $2 - login_failures.first_failure_at >= $3 THEN $2
                   ELSE login_failures.first_failure_at
               END"#,
            username,
            timestamp(now),
            window as i64
        )
        .execute(&self.pool)
        .await
        .map_err(|error| format!("register login failure: {error}"))?;
        Ok(())
    }

    async fn clear_failures(&self, username: &str) -> Result<(), String> {
        sqlx::query!(
            r#"DELETE FROM login_failures WHERE username = $1"#,
            username
        )
        .execute(&self.pool)
        .await
        .map_err(|error| format!("clear login failures: {error}"))?;
        Ok(())
    }

    async fn is_locked(&self, username: &str, now: u64) -> Result<bool, String> {
        let row = sqlx::query!(
            r#"SELECT failures, first_failure_at FROM login_failures WHERE username = $1"#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| format!("read login failures: {error}"))?;

        let Some(row) = row else {
            return Ok(false);
        };
        let elapsed = now as i64 - row.first_failure_at.timestamp();
        if elapsed >= self.window_secs as i64 {
            return Ok(false);
        }
        Ok(row.failures as u32 >= self.max_failures)
    }
}

fn timestamp(seconds: u64) -> DateTime<Utc> {
    DateTime::from_timestamp(seconds as i64, 0).unwrap_or_else(Utc::now)
}
