#![forbid(unsafe_code)]

//! Local-only authentication boundary for the governance web prototype.
//!
//! Provides password hashing (SHA-256 with salt, no external crate dependency),
//! session tokens with expiry, and role-based access control.
//! This is a local prototype — no enterprise authentication or external IdP.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub role: Role,
}

impl User {
    /// Create a new user with a hashed password.
    pub fn new(username: &str, password: &str, role: Role) -> Self {
        let salt = generate_salt(username);
        let hash = hash_password(password, &salt);
        Self {
            username: username.to_string(),
            password_hash: hash,
            salt,
            role,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let hash = hash_password(password, &self.salt);
        hash == self.password_hash
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
        now_secs() > self.expires_at
    }
}

pub struct AuthStore {
    users: BTreeMap<String, User>,
    sessions: BTreeMap<String, Session>,
    session_ttl_secs: u64,
}

impl AuthStore {
    pub fn new(session_ttl_secs: u64) -> Self {
        Self {
            users: BTreeMap::new(),
            sessions: BTreeMap::new(),
            session_ttl_secs,
        }
    }

    pub fn register(&mut self, username: &str, password: &str, role: Role) -> Result<(), String> {
        if self.users.contains_key(username) {
            return Err("user already exists".to_string());
        }
        self.users
            .insert(username.to_string(), User::new(username, password, role));
        Ok(())
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, String> {
        let user = self.users.get(username).ok_or("unknown user")?;
        if !user.verify_password(password) {
            return Err("invalid password".to_string());
        }
        let token = generate_token(username);
        let now = now_secs();
        let session = Session {
            token: token.clone(),
            username: username.to_string(),
            role: user.role,
            created_at: now,
            expires_at: now + self.session_ttl_secs,
        };
        self.sessions.insert(token, session.clone());
        Ok(session)
    }

    pub fn validate_session(&self, token: &str) -> Option<&Session> {
        let session = self.sessions.get(token)?;
        if session.is_expired() {
            return None;
        }
        Some(session)
    }

    pub fn logout(&mut self, token: &str) {
        self.sessions.remove(token);
    }

    pub fn has_role(&self, token: &str, required: Role) -> bool {
        match self.validate_session(token) {
            Some(s) => s.role >= required,
            None => false,
        }
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new(3600)
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_salt(username: &str) -> String {
    // Deterministic salt derived from username + constant (not cryptographically secure;
    // acceptable for local prototype only).
    format!("salt_{}", username)
}

fn hash_password(password: &str, salt: &str) -> String {
    // Simple hash using std::collections hash (not cryptographic; local prototype only).
    // In production, use bcrypt/argon2. This is a placeholder for the local-first phase.
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{}:{}", salt, password).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn generate_token(username: &str) -> String {
    format!("tok_{}_{}", username, now_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip() {
        let user = User::new("alice", "secret123", Role::Editor);
        assert!(user.verify_password("secret123"));
        assert!(!user.verify_password("wrong"));
    }

    #[test]
    fn login_and_validate() {
        let mut store = AuthStore::new(3600);
        store.register("alice", "secret123", Role::Editor).unwrap();
        let session = store.login("alice", "secret123").unwrap();
        assert!(store.validate_session(&session.token).is_some());
        assert!(store.has_role(&session.token, Role::Viewer));
        assert!(store.has_role(&session.token, Role::Editor));
        assert!(!store.has_role(&session.token, Role::Admin));
    }

    #[test]
    fn invalid_login_fails() {
        let mut store = AuthStore::new(3600);
        store.register("alice", "secret123", Role::Editor).unwrap();
        assert!(store.login("alice", "wrong").is_err());
        assert!(store.login("nobody", "pass").is_err());
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut store = AuthStore::new(3600);
        store.register("alice", "pass", Role::Viewer).unwrap();
        assert!(store.register("alice", "pass2", Role::Viewer).is_err());
    }

    #[test]
    fn logout_invalidates_session() {
        let mut store = AuthStore::new(3600);
        store.register("alice", "pass", Role::Viewer).unwrap();
        let session = store.login("alice", "pass").unwrap();
        store.logout(&session.token);
        assert!(store.validate_session(&session.token).is_none());
    }
}
