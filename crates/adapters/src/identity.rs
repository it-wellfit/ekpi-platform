#![forbid(unsafe_code)]

//! In-memory identity adapter for the local pilot.
//!
//! Accounts, sessions and login throttling live in process memory only. They
//! disappear when the process stops, so this adapter is a development and test
//! helper, never a production identity store.

use std::collections::BTreeMap;

use async_trait::async_trait;
use ekp_application::ports::{GovernanceError, LoginThrottle, SessionRepository, UserRepository};
use ekp_domain::auth::{Session, UserRecord};

/// Failed logins that trigger a temporary lockout.
pub const MAX_FAILED_LOGINS: u32 = 5;
/// Length of the lockout window in seconds.
pub const LOCKOUT_WINDOW_SECS: u64 = 300;

/// In-memory account store.
#[derive(Debug, Default)]
pub struct UserStore {
    users: BTreeMap<String, UserRecord>,
}

impl UserStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepository for UserStore {
    async fn find_by_username(&self, username: &str) -> Option<UserRecord> {
        self.users.get(username).cloned()
    }

    async fn create_user(&mut self, user: UserRecord) -> Result<(), GovernanceError> {
        if self.users.contains_key(&user.username) {
            return Err(GovernanceError::conflict("user already exists"));
        }
        self.users.insert(user.username.clone(), user);
        Ok(())
    }

    async fn list_users(&self) -> Vec<UserRecord> {
        self.users.values().cloned().collect()
    }

    async fn set_disabled(
        &mut self,
        username: &str,
        disabled: bool,
    ) -> Result<(), GovernanceError> {
        let user = self
            .users
            .get_mut(username)
            .ok_or_else(|| GovernanceError::not_found("user not found"))?;
        user.disabled = disabled;
        Ok(())
    }
}

/// In-memory session store.
#[derive(Debug)]
pub struct SessionStore {
    sessions: BTreeMap<String, Session>,
    idle_timeout_secs: u64,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::with_idle_timeout(900)
    }

    /// Build a store that renews the idle deadline with this window.
    pub fn with_idle_timeout(idle_timeout_secs: u64) -> Self {
        Self {
            sessions: BTreeMap::new(),
            idle_timeout_secs,
        }
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for SessionStore {
    async fn create_session(&mut self, session: Session) -> Result<(), GovernanceError> {
        self.sessions.insert(session.token.clone(), session);
        Ok(())
    }

    async fn find_session(&self, token: &str, now: u64) -> Option<Session> {
        self.sessions
            .get(token)
            .filter(|session| !session.is_expired_at(now))
            .cloned()
    }

    async fn touch_session(&mut self, token: &str, now: u64) -> Result<(), GovernanceError> {
        let idle_timeout_secs = self.idle_timeout_secs;
        let session = self
            .sessions
            .get_mut(token)
            .ok_or_else(|| GovernanceError::not_found("session not found"))?;
        if session.is_expired_at(now) {
            self.sessions.remove(token);
            return Err(GovernanceError::not_found("session expired"));
        }
        session.idle_deadline = now.saturating_add(idle_timeout_secs);
        Ok(())
    }

    async fn delete_session(&mut self, token: &str) -> Result<(), GovernanceError> {
        self.sessions.remove(token);
        Ok(())
    }

    async fn purge_expired(&mut self, now: u64) -> Result<usize, GovernanceError> {
        let before = self.sessions.len();
        self.sessions
            .retain(|_, session| !session.is_expired_at(now));
        Ok(before - self.sessions.len())
    }

    async fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// In-memory login throttle.
#[derive(Debug, Default)]
pub struct LoginThrottleStore {
    failures: std::sync::Mutex<BTreeMap<String, (u32, u64)>>,
}

impl LoginThrottleStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_if_expired(failures: &mut BTreeMap<String, (u32, u64)>, username: &str, now: u64) {
        if let Some((_, first_failure)) = failures.get(username)
            && now.saturating_sub(*first_failure) >= LOCKOUT_WINDOW_SECS
        {
            failures.remove(username);
        }
    }
}

#[async_trait]
impl LoginThrottle for LoginThrottleStore {
    async fn register_failure(&self, username: &str, now: u64) -> Result<(), String> {
        let mut guard = self
            .failures
            .lock()
            .map_err(|_| "login throttle lock is poisoned".to_string())?;
        Self::clear_if_expired(&mut guard, username, now);
        let entry = guard.entry(username.to_string()).or_insert((0, now));
        if now.saturating_sub(entry.1) >= LOCKOUT_WINDOW_SECS {
            *entry = (0, now);
        }
        entry.0 = entry.0.saturating_add(1);
        Ok(())
    }

    async fn clear_failures(&self, username: &str) -> Result<(), String> {
        let mut guard = self
            .failures
            .lock()
            .map_err(|_| "login throttle lock is poisoned".to_string())?;
        guard.remove(username);
        Ok(())
    }

    async fn is_locked(&self, username: &str, now: u64) -> Result<bool, String> {
        let mut guard = self
            .failures
            .lock()
            .map_err(|_| "login throttle lock is poisoned".to_string())?;
        Self::clear_if_expired(&mut guard, username, now);
        Ok(guard.get(username).is_some_and(|(count, first_failure)| {
            *count >= MAX_FAILED_LOGINS && now.saturating_sub(*first_failure) < LOCKOUT_WINDOW_SECS
        }))
    }
}

/// The complete in-memory identity subsystem.
#[derive(Debug, Default)]
pub struct InMemoryIdentity {
    pub users: UserStore,
    pub sessions: SessionStore,
    pub throttle: LoginThrottleStore,
}

impl InMemoryIdentity {
    pub fn new() -> Self {
        Self::default()
    }
}
