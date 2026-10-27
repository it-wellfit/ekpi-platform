#![forbid(unsafe_code)]

//! Local-only authentication boundary for the governance web prototype.
//!
//! This module exists so the local workbench can demonstrate login, logout,
//! session expiry and role checks. It is **not** an enterprise identity
//! provider and **not** a production authentication system:
//!
//! - passwords are stored as PBKDF2-HMAC-SHA256 hashes with a random per-user
//!   salt; the default work factor is 600,000 iterations;
//! - session tokens are 256-bit values from the operating-system CSPRNG;
//! - there is no MFA, SSO, account recovery, password rotation, audit of login
//!   attempts, or persistent identity store;
//! - sessions live only in process memory and disappear on restart.
//!
//! The intended deployment is a single-user loopback process. Any shared or
//! network-exposed deployment requires the enterprise identity decision
//! (`DEC-12`) and a separate security review.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

/// OWASP-style PBKDF2-HMAC-SHA256 work factor for SHA-256 (2023 guidance).
pub const DEFAULT_PBKDF2_ITERATIONS: u32 = 600_000;

const SALT_BYTES: usize = 16;
const TOKEN_BYTES: usize = 32;
const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;
const MIN_USERNAME_LEN: usize = 1;
const MAX_USERNAME_LEN: usize = 64;
const MAX_FAILED_LOGINS: u32 = 5;
const LOCKOUT_WINDOW_SECS: u64 = 300;
const DUMMY_SALT: &str = "00000000000000000000000000000000";
const DUMMY_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    Viewer,
    Editor,
    Approver,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Viewer => "viewer",
            Role::Editor => "editor",
            Role::Approver => "approver",
            Role::Admin => "admin",
        }
    }

    pub fn parse_role(s: &str) -> Option<Self> {
        match s {
            "viewer" => Some(Role::Viewer),
            "editor" => Some(Role::Editor),
            "approver" => Some(Role::Approver),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
    pub role: Role,
}

impl User {
    /// Create a new user with a hashed password.
    pub fn new(username: &str, password: &str, role: Role) -> Result<Self, String> {
        Self::with_iterations(username, password, role, DEFAULT_PBKDF2_ITERATIONS)
    }

    /// Create a user with an explicit work factor. Tests use a lower value so
    /// the suite remains fast; production callers should use [`User::new`].
    pub fn with_iterations(
        username: &str,
        password: &str,
        role: Role,
        iterations: u32,
    ) -> Result<Self, String> {
        validate_username(username)?;
        validate_password(password)?;
        if iterations == 0 {
            return Err("PBKDF2 iterations must be greater than zero".to_string());
        }
        let salt = random_hex(SALT_BYTES)?;
        let password_hash = hash_password(password, &salt, iterations);
        Ok(Self {
            username: username.to_string(),
            password_hash,
            salt,
            iterations,
            role,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let candidate = hash_password(password, &self.salt, self.iterations);
        bool::from(candidate.as_bytes().ct_eq(self.password_hash.as_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub role: Role,
    pub created_at: u64,
    pub expires_at: u64,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        now_secs() >= self.expires_at
    }
}

pub struct AuthStore {
    users: BTreeMap<String, User>,
    sessions: BTreeMap<String, Session>,
    failed_logins: BTreeMap<String, (u32, u64)>,
    session_ttl_secs: u64,
    iterations: u32,
}

impl AuthStore {
    pub fn new(session_ttl_secs: u64) -> Self {
        Self::with_iterations(session_ttl_secs, DEFAULT_PBKDF2_ITERATIONS)
    }

    /// Construct a store with an explicit password work factor. Intended for
    /// tests and local drills; the HTTP server uses [`AuthStore::new`].
    pub fn with_iterations(session_ttl_secs: u64, iterations: u32) -> Self {
        Self {
            users: BTreeMap::new(),
            sessions: BTreeMap::new(),
            failed_logins: BTreeMap::new(),
            session_ttl_secs,
            iterations,
        }
    }

    pub fn register(&mut self, username: &str, password: &str, role: Role) -> Result<(), String> {
        if self.users.contains_key(username) {
            return Err("user already exists".to_string());
        }
        let user = User::with_iterations(username, password, role, self.iterations)?;
        self.users.insert(username.to_string(), user);
        Ok(())
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, String> {
        let now = now_secs();
        self.clear_expired_lockout(username, now);
        if self.is_locked_out(username, now) {
            return Err("too many failed login attempts; try again later".to_string());
        }

        let snapshot = self.users.get(username).map(|user| {
            (
                user.salt.clone(),
                user.iterations,
                user.password_hash.clone(),
            )
        });
        let (salt, iterations, expected_hash) = snapshot.unwrap_or_else(|| {
            (
                DUMMY_SALT.to_string(),
                self.iterations,
                DUMMY_HASH.to_string(),
            )
        });
        let candidate = hash_password(password, &salt, iterations);
        let password_matches = bool::from(candidate.as_bytes().ct_eq(expected_hash.as_bytes()));
        let user_exists = self.users.contains_key(username);

        if !(password_matches && user_exists) {
            self.record_failure(username, now);
            return Err("invalid username or password".to_string());
        }

        self.failed_logins.remove(username);
        let token = random_hex(TOKEN_BYTES)?;
        let session = Session {
            token: token.clone(),
            username: username.to_string(),
            role: self
                .users
                .get(username)
                .map(|user| user.role)
                .unwrap_or(Role::Viewer),
            created_at: now,
            expires_at: now.saturating_add(self.session_ttl_secs),
        };
        self.sessions.insert(token, session.clone());
        Ok(session)
    }

    pub fn validate_session(&self, token: &str) -> Option<&Session> {
        self.sessions
            .get(token)
            .filter(|session| !session.is_expired())
    }

    pub fn logout(&mut self, token: &str) {
        self.sessions.remove(token);
    }

    pub fn has_role(&self, token: &str, required: Role) -> bool {
        match self.validate_session(token) {
            Some(session) => session.role >= required,
            None => false,
        }
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    fn clear_expired_lockout(&mut self, username: &str, now: u64) {
        if let Some((_, first_failure)) = self.failed_logins.get(username) {
            if now.saturating_sub(*first_failure) >= LOCKOUT_WINDOW_SECS {
                self.failed_logins.remove(username);
            }
        }
    }

    fn is_locked_out(&self, username: &str, now: u64) -> bool {
        match self.failed_logins.get(username) {
            Some((count, first_failure)) => {
                *count >= MAX_FAILED_LOGINS
                    && now.saturating_sub(*first_failure) < LOCKOUT_WINDOW_SECS
            }
            None => false,
        }
    }

    fn record_failure(&mut self, username: &str, now: u64) {
        let entry = self
            .failed_logins
            .entry(username.to_string())
            .or_insert((0, now));
        if now.saturating_sub(entry.1) >= LOCKOUT_WINDOW_SECS {
            *entry = (0, now);
        }
        entry.0 = entry.0.saturating_add(1);
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new(3600)
    }
}

fn default_iterations() -> u32 {
    DEFAULT_PBKDF2_ITERATIONS
}

fn validate_username(username: &str) -> Result<(), String> {
    let length = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&length) {
        return Err(format!(
            "username must contain between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters"
        ));
    }
    if username.chars().any(|c| c.is_control()) {
        return Err("username must not contain control characters".to_string());
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    let length = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&length) {
        return Err(format!(
            "password must contain between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters"
        ));
    }
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn random_hex(length: usize) -> Result<String, String> {
    let mut bytes = vec![0u8; length];
    getrandom::getrandom(&mut bytes)
        .map_err(|error| format!("operating-system random source unavailable: {error}"))?;
    Ok(to_hex(&bytes))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn hash_password(password: &str, salt: &str, iterations: u32) -> String {
    let mut output = [0u8; 32];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
        &mut output,
    );
    to_hex(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_store() -> AuthStore {
        AuthStore::with_iterations(3600, 1_000)
    }

    #[test]
    fn password_roundtrip() {
        let user = User::with_iterations("alice", "secret123", Role::Editor, 1_000).unwrap();
        assert!(user.verify_password("secret123"));
        assert!(!user.verify_password("wrong"));
    }

    #[test]
    fn salts_and_hashes_are_unique() {
        let first = User::with_iterations("alice", "secret123", Role::Editor, 1_000).unwrap();
        let second = User::with_iterations("alice", "secret123", Role::Editor, 1_000).unwrap();
        assert_ne!(first.salt, second.salt);
        assert_ne!(first.password_hash, second.password_hash);
        assert_ne!(first.password_hash, "secret123");
    }

    #[test]
    fn login_and_validate() {
        let mut store = fast_store();
        store.register("alice", "secret123", Role::Editor).unwrap();
        let session = store.login("alice", "secret123").unwrap();
        assert_eq!(session.token.len(), TOKEN_BYTES * 2);
        assert!(store.validate_session(&session.token).is_some());
        assert!(store.has_role(&session.token, Role::Viewer));
        assert!(store.has_role(&session.token, Role::Editor));
        assert!(!store.has_role(&session.token, Role::Admin));
    }

    #[test]
    fn invalid_login_fails_without_leaking_user_existence() {
        let mut store = fast_store();
        store.register("alice", "secret123", Role::Editor).unwrap();
        let wrong_password = store.login("alice", "wrong-password").unwrap_err();
        let unknown_user = store.login("nobody", "wrong-password").unwrap_err();
        assert_eq!(wrong_password, unknown_user);
        assert_eq!(wrong_password, "invalid username or password");
    }

    #[test]
    fn repeated_failures_trigger_a_lockout() {
        let mut store = fast_store();
        store.register("alice", "secret123", Role::Editor).unwrap();
        for _ in 0..MAX_FAILED_LOGINS {
            assert!(store.login("alice", "wrong-password").is_err());
        }
        let error = store.login("alice", "secret123").unwrap_err();
        assert!(error.contains("too many failed login attempts"));
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut store = fast_store();
        store.register("alice", "password1", Role::Viewer).unwrap();
        assert!(store.register("alice", "password2", Role::Viewer).is_err());
    }

    #[test]
    fn weak_or_short_passwords_are_rejected() {
        let mut store = fast_store();
        assert!(store.register("alice", "short", Role::Viewer).is_err());
        assert!(store.register("alice", "", Role::Viewer).is_err());
    }

    #[test]
    fn logout_invalidates_session() {
        let mut store = fast_store();
        store.register("alice", "password1", Role::Viewer).unwrap();
        let session = store.login("alice", "password1").unwrap();
        store.logout(&session.token);
        assert!(store.validate_session(&session.token).is_none());
        assert_eq!(store.session_count(), 0);
    }

    #[test]
    fn expired_sessions_are_not_valid() {
        let mut store = fast_store();
        store.register("alice", "password1", Role::Viewer).unwrap();
        let session = store.login("alice", "password1").unwrap();
        let expired = Session {
            expires_at: session.created_at,
            ..session
        };
        assert!(expired.is_expired());
    }
}
