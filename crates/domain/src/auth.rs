#![forbid(unsafe_code)]

//! Identity and authorization domain concepts.
//!
//! Only the rules live here: who may do what, how a session ages, and what a
//! password or username may look like. Credential storage and hashing are
//! infrastructure concerns and belong to the adapter layer.

use serde::{Deserialize, Serialize};

/// Minimum accepted password length in characters.
pub const MIN_PASSWORD_LEN: usize = 8;
/// Maximum accepted password length in characters.
pub const MAX_PASSWORD_LEN: usize = 128;
/// Minimum accepted username length in characters.
pub const MIN_USERNAME_LEN: usize = 1;
/// Maximum accepted username length in characters.
pub const MAX_USERNAME_LEN: usize = 64;

/// Ordered role ladder. A higher role satisfies every requirement of a lower
/// role, so comparisons such as `role >= Role::Editor` are meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    pub fn parse_role(value: &str) -> Option<Self> {
        match value {
            "viewer" => Some(Role::Viewer),
            "editor" => Some(Role::Editor),
            "approver" => Some(Role::Approver),
            "admin" => Some(Role::Admin),
            _ => None,
        }
    }

    /// Return `true` when this role satisfies the required role.
    pub fn satisfies(self, required: Role) -> bool {
        self >= required
    }
}

/// Capability required by an operation. Authorization is expressed as
/// capabilities rather than as scattered role comparisons so that the
/// application layer can fail closed on a single, reviewable table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Read the workbench and its audit trail.
    ViewGovernance,
    /// Create and edit governance records.
    EditGovernance,
    /// Review evidence, publish workflows and record gate decisions.
    ApproveGovernance,
    /// Create, disable and re-role accounts.
    ManageUsers,
}

impl Permission {
    /// Return the lowest role that is allowed to exercise this permission.
    pub fn minimum_role(self) -> Role {
        match self {
            Permission::ViewGovernance => Role::Viewer,
            Permission::EditGovernance => Role::Editor,
            Permission::ApproveGovernance => Role::Approver,
            Permission::ManageUsers => Role::Admin,
        }
    }
}

/// A stored account. The password hash is opaque to the domain layer; only
/// the adapter that produced it knows the algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub username: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: u64,
    #[serde(default)]
    pub disabled: bool,
}

/// An authenticated session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub role: Role,
    pub created_at: u64,
    /// Absolute deadline: the session is invalid after this instant.
    pub expires_at: u64,
    /// Idle deadline: the session is invalid after this instant without use.
    pub idle_deadline: u64,
    /// Synchronizer token bound to this session for state-changing requests.
    pub csrf_token: String,
}

impl Session {
    /// Return `true` when the session is past its absolute or idle deadline.
    pub fn is_expired_at(&self, now: u64) -> bool {
        now >= self.expires_at || now >= self.idle_deadline
    }
}

/// Server-side session lifetime policy.
#[derive(Debug, Clone, Copy)]
pub struct SessionPolicy {
    /// How long a session may live in total.
    pub absolute_timeout_secs: u64,
    /// How long a session may stay idle before it must be renewed.
    pub idle_timeout_secs: u64,
}

impl SessionPolicy {
    pub const fn new(absolute_timeout_secs: u64, idle_timeout_secs: u64) -> Self {
        Self {
            absolute_timeout_secs,
            idle_timeout_secs,
        }
    }

    /// Return the absolute and idle deadlines for a session started at `now`.
    pub fn deadlines(&self, now: u64) -> (u64, u64) {
        let absolute = now.saturating_add(self.absolute_timeout_secs);
        let idle = now.saturating_add(self.idle_timeout_secs.min(self.absolute_timeout_secs));
        (absolute, idle)
    }
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self::new(3600, 900)
    }
}

/// Validate a username shape. Control characters and whitespace are rejected
/// because the value is used as a ledger actor and as a lookup key.
pub fn validate_username(username: &str) -> Result<(), String> {
    let length = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&length) {
        return Err(format!(
            "username must contain between {MIN_USERNAME_LEN} and {MAX_USERNAME_LEN} characters"
        ));
    }
    if username.chars().any(char::is_control) {
        return Err("username must not contain control characters".to_string());
    }
    if username.chars().any(char::is_whitespace) {
        return Err("username must not contain whitespace".to_string());
    }
    Ok(())
}

/// Validate a password shape. The hash strength policy belongs to the
/// adapter; only the length envelope is a domain rule.
pub fn validate_password(password: &str) -> Result<(), String> {
    let length = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&length) {
        return Err(format!(
            "password must contain between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_ladder_is_ordered() {
        assert!(Role::Admin.satisfies(Role::Viewer));
        assert!(Role::Editor.satisfies(Role::Editor));
        assert!(!Role::Viewer.satisfies(Role::Editor));
        assert!(!Role::Editor.satisfies(Role::Approver));
    }

    #[test]
    fn permissions_map_to_minimum_roles() {
        assert_eq!(Permission::ViewGovernance.minimum_role(), Role::Viewer);
        assert_eq!(Permission::EditGovernance.minimum_role(), Role::Editor);
        assert_eq!(Permission::ApproveGovernance.minimum_role(), Role::Approver);
        assert_eq!(Permission::ManageUsers.minimum_role(), Role::Admin);
    }

    #[test]
    fn session_expires_on_absolute_and_idle_deadlines() {
        let session = Session {
            token: "t".to_string(),
            username: "alice".to_string(),
            role: Role::Viewer,
            created_at: 100,
            expires_at: 200,
            idle_deadline: 150,
            csrf_token: "c".to_string(),
        };
        assert!(!session.is_expired_at(149));
        assert!(session.is_expired_at(150));
        assert!(session.is_expired_at(200));
    }

    #[test]
    fn session_policy_never_exceeds_the_absolute_timeout() {
        let policy = SessionPolicy::new(600, 900);
        let (absolute, idle) = policy.deadlines(0);
        assert_eq!(absolute, 600);
        assert_eq!(idle, 600);
    }

    #[test]
    fn username_rules_reject_whitespace_and_controls() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username(" alice").is_err());
        assert!(validate_username("ali\tce").is_err());
        assert!(validate_username("").is_err());
    }

    #[test]
    fn password_rules_reject_short_and_long_secrets() {
        assert!(validate_password("password1").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password(&"x".repeat(MAX_PASSWORD_LEN + 1)).is_err());
    }
}
