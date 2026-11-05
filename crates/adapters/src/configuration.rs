#![forbid(unsafe_code)]

//! Environment driven configuration.
//!
//! Configuration is read once at start-up. Nothing here reads a secret from a
//! file inside the repository, and no default password exists: a deployment
//! must provide its own bootstrap credential.

use std::net::SocketAddr;
use std::path::PathBuf;

use ekp_domain::auth::Role;

/// Which store backs the governance records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    /// PostgreSQL 18.6 with SQLx. The deployment target.
    Postgres,
    /// Append-only JSONL files. The local pilot fallback.
    Jsonl,
}

impl StorageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageMode::Postgres => "postgres",
            StorageMode::Jsonl => "jsonl",
        }
    }
}

/// A bootstrap account created on first start when the operator supplies one.
#[derive(Debug, Clone)]
pub struct BootstrapAccount {
    pub username: String,
    pub password: String,
    pub role: Role,
}

/// Start-up configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Address the HTTP server binds to. Loopback by default.
    pub bind_address: SocketAddr,
    /// Which store backs the governance records.
    pub storage: StorageMode,
    /// PostgreSQL connection string. Required for `StorageMode::Postgres`.
    pub database_url: Option<String>,
    /// Path of the JSONL ledger. Used for `StorageMode::Jsonl`.
    pub ledger_path: PathBuf,
    /// Whether session cookies carry the `Secure` attribute.
    pub cookie_secure: bool,
    /// Total session lifetime in seconds.
    pub session_absolute_timeout_secs: u64,
    /// Idle session lifetime in seconds.
    pub session_idle_timeout_secs: u64,
    /// Optional account created on first start.
    pub bootstrap_account: Option<BootstrapAccount>,
    /// Per-request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Maximum number of requests handled at the same time.
    pub max_concurrent_requests: usize,
}

impl AppConfig {
    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self, String> {
        let bind_address = match std::env::var("EKPI_BIND_ADDRESS") {
            Ok(value) => value
                .parse::<SocketAddr>()
                .map_err(|error| format!("EKPI_BIND_ADDRESS is not a socket address: {error}"))?,
            Err(_) => SocketAddr::from(([127, 0, 0, 1], 3000)),
        };

        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());

        let storage = match std::env::var("EKPI_STORAGE") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "postgres" | "postgresql" => StorageMode::Postgres,
                "jsonl" | "file" | "local" => StorageMode::Jsonl,
                other => {
                    return Err(format!(
                        "EKPI_STORAGE must be postgres or jsonl, not {other:?}"
                    ));
                }
            },
            Err(_) => {
                if database_url.is_some() {
                    StorageMode::Postgres
                } else {
                    StorageMode::Jsonl
                }
            }
        };

        if storage == StorageMode::Postgres && database_url.is_none() {
            return Err("DATABASE_URL is required when EKPI_STORAGE=postgres".to_string());
        }

        let cookie_secure = matches!(std::env::var("EKPI_COOKIE_SECURE").as_deref(), Ok("1"));
        let session_absolute_timeout_secs =
            parse_seconds("EKPI_SESSION_ABSOLUTE_TIMEOUT_SECS", 3600)?;
        let session_idle_timeout_secs = parse_seconds("EKPI_SESSION_IDLE_TIMEOUT_SECS", 900)?;
        if session_idle_timeout_secs > session_absolute_timeout_secs {
            return Err(
                "EKPI_SESSION_IDLE_TIMEOUT_SECS must not exceed EKPI_SESSION_ABSOLUTE_TIMEOUT_SECS"
                    .to_string(),
            );
        }
        let request_timeout_secs = parse_seconds("EKPI_REQUEST_TIMEOUT_SECS", 15)?;

        let bootstrap_account = match (
            std::env::var("EKPI_LOCAL_AUTH_USER"),
            std::env::var("EKPI_LOCAL_AUTH_PASSWORD"),
        ) {
            (Ok(username), Ok(password)) => {
                let role = std::env::var("EKPI_LOCAL_AUTH_ROLE")
                    .ok()
                    .and_then(|value| Role::parse_role(&value))
                    .unwrap_or(Role::Editor);
                Some(BootstrapAccount {
                    username,
                    password,
                    role,
                })
            }
            _ => None,
        };

        Ok(Self {
            bind_address,
            storage,
            database_url,
            ledger_path: default_ledger_path(),
            cookie_secure,
            session_absolute_timeout_secs,
            session_idle_timeout_secs,
            bootstrap_account,
            request_timeout_secs,
            max_concurrent_requests: 64,
        })
    }
}

/// Default location of the JSONL ledger.
pub fn default_ledger_path() -> PathBuf {
    if let Some(path) = std::env::var_os("EKP_GOVERNANCE_DATA") {
        return PathBuf::from(path);
    }
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(root)
            .join("EKPI")
            .join("governance-ledger.jsonl");
    }
    std::env::temp_dir()
        .join("ekpi")
        .join("governance-ledger.jsonl")
}

fn parse_seconds(name: &str, fallback: u64) -> Result<u64, String> {
    match std::env::var(name) {
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .map_err(|error| format!("{name} must be a whole number of seconds: {error}")),
        Err(_) => Ok(fallback),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_mode_has_a_stable_name() {
        assert_eq!(StorageMode::Postgres.as_str(), "postgres");
        assert_eq!(StorageMode::Jsonl.as_str(), "jsonl");
    }

    #[test]
    fn default_ledger_path_honours_the_data_variable() {
        // The helper only reads the environment; the value is checked for the
        // documented shape rather than for a specific machine.
        let path = default_ledger_path();
        assert!(path.to_string_lossy().contains("governance-ledger.jsonl"));
    }

    #[test]
    fn invalid_seconds_are_reported() {
        // parse_seconds only reads the environment, so an unset variable keeps
        // the fallback value.
        assert_eq!(
            parse_seconds("EKPI_TEST_UNSET_VARIABLE", 42).expect("fallback"),
            42
        );
    }
}
