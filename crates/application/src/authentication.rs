#![forbid(unsafe_code)]

//! Authentication use cases.
//!
//! The service never reveals whether an account exists: every failed login
//! returns the same [`AuthError::InvalidCredentials`], and the password
//! verification cost is paid even for unknown accounts.

use ekp_domain::auth::{
    Role, Session, SessionPolicy, UserRecord, validate_password, validate_username,
};

use crate::ports::{
    Clock, GovernanceError, LoginThrottle, PasswordHasher, SessionRepository, TokenGenerator,
    UserRepository,
};

/// Bytes of entropy in a session token.
pub const SESSION_TOKEN_BYTES: usize = 32;
/// Bytes of entropy in the synchronizer CSRF token.
pub const CSRF_TOKEN_BYTES: usize = 32;

/// Why an authentication operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// The username or the password was wrong. The message never says which.
    InvalidCredentials,
    /// The session token is unknown or expired.
    InvalidSession,
    /// The session is valid but the presented CSRF token does not match.
    InvalidCsrfToken,
    /// The account is disabled.
    AccountDisabled,
    /// The operation requires a stronger role.
    Forbidden,
    /// The backing store failed.
    Internal(String),
}

impl AuthError {
    /// The message that may be shown to the caller.
    pub fn message(&self) -> &str {
        match self {
            AuthError::InvalidCredentials => "invalid username or password",
            AuthError::InvalidSession => "authentication required",
            AuthError::InvalidCsrfToken => "invalid CSRF token",
            AuthError::AccountDisabled => "account is disabled",
            AuthError::Forbidden => "insufficient role for this operation",
            AuthError::Internal(message) => message,
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for AuthError {}

impl From<GovernanceError> for AuthError {
    fn from(error: GovernanceError) -> Self {
        AuthError::Internal(error.message().to_string())
    }
}

/// Authentication use cases bound to their ports.
pub struct AuthenticationService<'a> {
    users: &'a mut dyn UserRepository,
    sessions: &'a mut dyn SessionRepository,
    hasher: &'a dyn PasswordHasher,
    clock: &'a dyn Clock,
    tokens: &'a dyn TokenGenerator,
    throttle: &'a dyn LoginThrottle,
    policy: SessionPolicy,
}

impl<'a> AuthenticationService<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        users: &'a mut dyn UserRepository,
        sessions: &'a mut dyn SessionRepository,
        hasher: &'a dyn PasswordHasher,
        clock: &'a dyn Clock,
        tokens: &'a dyn TokenGenerator,
        throttle: &'a dyn LoginThrottle,
        policy: SessionPolicy,
    ) -> Self {
        Self {
            users,
            sessions,
            hasher,
            clock,
            tokens,
            throttle,
            policy,
        }
    }

    /// Create an account.
    pub async fn register(
        &mut self,
        username: &str,
        password: &str,
        role: Role,
    ) -> Result<UserRecord, AuthError> {
        validate_username(username).map_err(AuthError::Internal)?;
        validate_password(password).map_err(AuthError::Internal)?;
        let hash = self.hasher.hash(password).map_err(AuthError::Internal)?;
        let user = UserRecord {
            username: username.to_string(),
            password_hash: hash,
            role,
            created_at: self.clock.now_unix(),
            disabled: false,
        };
        self.users.create_user(user.clone()).await?;
        Ok(user)
    }

    /// Authenticate a username and password and open a session.
    pub async fn login(&mut self, username: &str, password: &str) -> Result<Session, AuthError> {
        let now = self.clock.now_unix();
        if self
            .throttle
            .is_locked(username, now)
            .await
            .unwrap_or(false)
        {
            // The lockout is not disclosed: the caller sees the same message as
            // for a wrong password.
            return Err(AuthError::InvalidCredentials);
        }

        let user = self.users.find_by_username(username).await;
        let stored = user.as_ref().map(|user| user.password_hash.clone());
        let hash = self.hasher.hash(password).map_err(AuthError::Internal)?;
        let matches = match stored {
            Some(expected) => self.hasher.verify(password, &expected),
            None => {
                // The hash above already paid the verification cost for an
                // unknown account, so the response time does not reveal
                // whether the account exists.
                let _ = hash;
                false
            }
        };

        let Some(user) = user else {
            // Unknown accounts are not tracked in the throttle, otherwise an
            // unauthenticated caller could grow the failure map without bound.
            return Err(AuthError::InvalidCredentials);
        };

        if !matches {
            let _ = self.throttle.register_failure(username, now).await;
            return Err(AuthError::InvalidCredentials);
        }

        if user.disabled {
            return Err(AuthError::AccountDisabled);
        }

        self.throttle
            .clear_failures(username)
            .await
            .map_err(AuthError::Internal)?;

        let token = self
            .tokens
            .generate(SESSION_TOKEN_BYTES)
            .map_err(AuthError::Internal)?;
        let csrf_token = self
            .tokens
            .generate(CSRF_TOKEN_BYTES)
            .map_err(AuthError::Internal)?;
        let (expires_at, idle_deadline) = self.policy.deadlines(now);
        let session = Session {
            token,
            username: user.username.clone(),
            role: user.role,
            created_at: now,
            expires_at,
            idle_deadline,
            csrf_token,
        };
        self.sessions.create_session(session.clone()).await?;
        Ok(session)
    }

    /// Resolve a session token into a live session.
    pub async fn authenticate(&mut self, token: &str) -> Result<Session, AuthError> {
        let now = self.clock.now_unix();
        let session = self
            .sessions
            .find_session(token, now)
            .await
            .ok_or(AuthError::InvalidSession)?;
        self.sessions.touch_session(token, now).await?;
        Ok(session)
    }

    /// End a session.
    pub async fn logout(&mut self, token: &str) -> Result<(), AuthError> {
        self.sessions.delete_session(token).await?;
        Ok(())
    }

    /// Verify the synchronizer token that accompanies a state-changing request.
    pub fn verify_csrf(session: &Session, presented: &str) -> Result<(), AuthError> {
        if presented.is_empty() {
            return Err(AuthError::InvalidCsrfToken);
        }
        let expected = session.csrf_token.as_bytes();
        let actual = presented.as_bytes();
        if expected.len() != actual.len() {
            return Err(AuthError::InvalidCsrfToken);
        }
        let mut difference = 0u8;
        for (left, right) in expected.iter().zip(actual.iter()) {
            difference |= left ^ right;
        }
        if difference == 0 {
            Ok(())
        } else {
            Err(AuthError::InvalidCsrfToken)
        }
    }

    /// Return the number of stored sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.session_count().await
    }

    /// Remove sessions that expired before now.
    pub async fn purge_expired(&mut self) -> Result<usize, AuthError> {
        let now = self.clock.now_unix();
        Ok(self.sessions.purge_expired(now).await?)
    }
}
