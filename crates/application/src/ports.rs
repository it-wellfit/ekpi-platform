#![forbid(unsafe_code)]

//! Ports: the interfaces the application layer needs from the outside world.
//!
//! Every trait here is implemented by an adapter. The application layer never
//! names a concrete database, filesystem or hash algorithm.

use std::collections::BTreeMap;

use async_trait::async_trait;
use ekp_domain::auth::{Role, Session, UserRecord};
use ekp_domain::governance::{GovernanceSnapshot, QualityInspectionInput, WorkflowDraftInput};

/// Why a use case failed. The distinction matters to callers: a validation
/// failure is the caller's fault and can be shown in a form, a storage failure
/// is an operator problem and must not be presented as invalid input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceError {
    /// The input violates a business rule.
    Validation(String),
    /// The referenced record does not exist.
    NotFound(String),
    /// The write conflicts with an existing record.
    Conflict(String),
    /// The backing store failed.
    Storage(String),
}

impl GovernanceError {
    pub fn validation(message: impl Into<String>) -> Self {
        GovernanceError::Validation(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        GovernanceError::NotFound(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        GovernanceError::Conflict(message.into())
    }

    pub fn storage(message: impl Into<String>) -> Self {
        GovernanceError::Storage(message.into())
    }

    /// The message safe to show to an operator.
    pub fn message(&self) -> &str {
        match self {
            GovernanceError::Validation(message)
            | GovernanceError::NotFound(message)
            | GovernanceError::Conflict(message)
            | GovernanceError::Storage(message) => message,
        }
    }

    /// Return `true` when the failure is caused by the submitted input.
    pub fn is_input_fault(&self) -> bool {
        !matches!(self, GovernanceError::Storage(_))
    }
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for GovernanceError {}

impl From<String> for GovernanceError {
    fn from(message: String) -> Self {
        GovernanceError::Validation(message)
    }
}

/// Storage port for governance records.
#[async_trait]
pub trait GovernanceRepository: Send {
    /// Return the current state of every governance record.
    fn snapshot(&self) -> GovernanceSnapshot;

    /// Register a project. Returns the new project identifier.
    async fn record_project(
        &mut self,
        name: &str,
        department: &str,
        owner: &str,
        risk_tier: &str,
        actor: &str,
    ) -> Result<String, GovernanceError>;

    /// Link a source reference to a project. Returns the new evidence identifier.
    #[allow(clippy::too_many_arguments)]
    async fn link_evidence(
        &mut self,
        project_id: &str,
        title: &str,
        source_system: &str,
        source_url: &str,
        source_version: &str,
        evidence_type: &str,
        owner: &str,
        actor: &str,
    ) -> Result<String, GovernanceError>;

    /// Record a human review of a source reference.
    async fn review_evidence(
        &mut self,
        evidence_id: &str,
        result: &str,
        reviewer: &str,
        note: &str,
    ) -> Result<(), GovernanceError>;

    /// Save a workflow draft. Returns the workflow identifier and version.
    async fn save_workflow(
        &mut self,
        draft: WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError>;

    /// Publish a workflow draft.
    async fn publish_workflow(
        &mut self,
        id: &str,
        version: u32,
        reviewer: &str,
    ) -> Result<(), GovernanceError>;

    /// Open a quality or risk case. Returns the new case identifier.
    #[allow(clippy::too_many_arguments)]
    async fn create_quality_case(
        &mut self,
        project_id: &str,
        kind: &str,
        title: &str,
        description: &str,
        owner: &str,
        due_date: &str,
        actor: &str,
    ) -> Result<String, GovernanceError>;

    /// Update the status of a quality case.
    async fn update_quality_case(
        &mut self,
        id: &str,
        status: &str,
        actor: &str,
        note: &str,
    ) -> Result<(), GovernanceError>;

    /// Record a complete quality inspection. Returns the new inspection identifier.
    async fn record_quality_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, GovernanceError>;

    /// Record a human gate decision. Returns the new decision identifier.
    #[allow(clippy::too_many_arguments)]
    async fn record_gate_decision(
        &mut self,
        project_id: &str,
        gate_id: &str,
        decision: &str,
        actor: &str,
        reason: &str,
        evidence_reference: &str,
        risk_acceptor: &str,
        remediation_due: &str,
        non_waivable: bool,
    ) -> Result<String, GovernanceError>;

    /// Verify that the stored audit chain is intact.
    fn verify_integrity(&self) -> Result<(), GovernanceError>;
}

/// Storage port for accounts.
#[async_trait]
pub trait UserRepository: Send {
    /// Return the account with this exact username, if any.
    async fn find_by_username(&self, username: &str) -> Option<UserRecord>;

    /// Create an account. Fails when the username is already taken.
    async fn create_user(&mut self, user: UserRecord) -> Result<(), GovernanceError>;

    /// Return every account, ordered by username.
    async fn list_users(&self) -> Vec<UserRecord>;

    /// Disable or re-enable an account.
    async fn set_disabled(&mut self, username: &str, disabled: bool)
    -> Result<(), GovernanceError>;
}

/// Storage port for sessions.
#[async_trait]
pub trait SessionRepository: Send {
    /// Create and persist a session.
    async fn create_session(&mut self, session: Session) -> Result<(), GovernanceError>;

    /// Return the session for this token when it is still valid at `now`.
    async fn find_session(&self, token: &str, now: u64) -> Option<Session>;

    /// Extend the idle deadline of a session.
    async fn touch_session(&mut self, token: &str, now: u64) -> Result<(), GovernanceError>;

    /// Remove a session.
    async fn delete_session(&mut self, token: &str) -> Result<(), GovernanceError>;

    /// Remove every session that expired before `now`. Returns the removed count.
    async fn purge_expired(&mut self, now: u64) -> Result<usize, GovernanceError>;

    /// Return the number of stored sessions.
    async fn session_count(&self) -> usize;
}

/// Port for one-way password hashing.
pub trait PasswordHasher: Send + Sync {
    /// Hash a password into an encoded hash.
    fn hash(&self, password: &str) -> Result<String, String>;

    /// Verify a password against an encoded hash.
    fn verify(&self, password: &str, encoded: &str) -> bool;
}

/// Port for the system clock.
pub trait Clock: Send + Sync {
    /// Seconds since the Unix epoch.
    fn now_unix(&self) -> u64;
}

/// System clock port backed by the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

/// Port for cryptographically secure random token generation.
pub trait TokenGenerator: Send + Sync {
    /// Return a lowercase hexadecimal token with `bytes` bytes of entropy.
    fn generate(&self, bytes: usize) -> Result<String, String>;
}

/// Port for throttling repeated failed logins.
#[async_trait]
pub trait LoginThrottle: Send + Sync {
    /// Record a failed login for `username`.
    async fn register_failure(&self, username: &str, now: u64) -> Result<(), String>;

    /// Clear the failure history for `username` after a successful login.
    async fn clear_failures(&self, username: &str) -> Result<(), String>;

    /// Return `true` when `username` is currently locked out.
    async fn is_locked(&self, username: &str, now: u64) -> Result<bool, String>;
}

/// Port for recording security relevant events.
pub trait AuditSink: Send + Sync {
    /// Record an event. Implementations must never store secrets.
    fn record(&self, event: AuditEvent) -> Result<(), String>;
}

/// A security relevant event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub occurred_at: u64,
    pub category: String,
    pub outcome: AuditOutcome,
    pub actor: String,
    pub subject: String,
    pub detail: String,
}

/// Outcome of an audited operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Denied,
    Failure,
}

impl AuditOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditOutcome::Success => "success",
            AuditOutcome::Denied => "denied",
            AuditOutcome::Failure => "failure",
        }
    }
}

/// In-memory audit sink used by tests and by the local pilot.
#[derive(Debug, Default)]
pub struct MemoryAuditSink {
    events: std::sync::Mutex<Vec<AuditEvent>>,
}

impl MemoryAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a copy of the recorded events.
    pub fn events(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}

impl AuditSink for MemoryAuditSink {
    fn record(&self, event: AuditEvent) -> Result<(), String> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| "audit sink lock is poisoned".to_string())?;
        events.push(event);
        Ok(())
    }
}

/// Read-only view over the quality checklist, exposed for adapters that need
/// to materialise the checklist into a database.
pub fn quality_checklist() -> &'static [ekp_domain::governance::QualityCheckDefinition] {
    &ekp_domain::governance::QUALITY_CHECKLIST
}

/// Convenience alias for a map of environment values.
pub type Environment = BTreeMap<String, String>;

/// Re-exported role type so adapters do not need to depend on the domain
/// crate directly for the most common type.
pub type UserRole = Role;
