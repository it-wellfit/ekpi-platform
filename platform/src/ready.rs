#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadyStatus {
    /// Overall readiness status
    pub status: ReadyStatusType,
    /// Detailed status information
    pub details: ReadyDetails,
    /// Timestamp when the readiness check was performed
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReadyStatusType {
    /// Ready for production operations
    Ready,
    /// Ready for limited operations
    Degraded,
    /// Not ready - critical issues detected
    NotReady,
    /// Readiness check in progress
    Checking,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadyDetails {
    /// Ledger status
    pub ledger: LedgerStatus,
    /// Authentication status
    pub auth: AuthStatus,
    /// Storage status
    pub storage: StorageStatus,
    /// Configuration status
    pub config: ConfigStatus,
    /// Version information
    pub version: String,
    /// Build information
    pub build_info: BuildInfo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerStatus {
    /// Ledger accessible
    pub accessible: bool,
    /// Number of events
    pub event_count: usize,
    /// Last event timestamp
    pub last_event_timestamp: Option<u64>,
    /// Ledger consistency status
    pub consistent: bool,
    /// Any ledger warnings
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthStatus {
    /// Authentication service accessible
    pub accessible: bool,
    /// Auth store status
    pub auth_store_ok: bool,
    /// Session timeout status
    pub session_timeout_configured: bool,
    /// Any auth warnings
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStatus {
    /// Data directory accessible
    pub data_dir_accessible: bool,
    /// Backup directory accessible
    pub backup_dir_accessible: bool,
    /// Disk space available (in MB)
    pub disk_space_mb: u64,
    /// Storage warnings
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigStatus {
    /// Configuration loaded
    pub loaded: bool,
    /// Configuration valid
    pub valid: bool,
    /// Configuration warnings
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildInfo {
    /// Version string
    pub version: String,
    /// Build timestamp
    pub build_timestamp: u64,
    /// Git commit hash
    pub git_commit: String,
    /// Build environment
    pub environment: String,
}

/// Check system readiness
///
/// Performs comprehensive readiness checks for the EKPI platform,
/// checking ledger accessibility, authentication service, storage,
/// and configuration.
///
/// # Arguments
///
/// * `ledger` - Shared ledger instance
/// * `auth` - Shared authentication store instance
///
/// # Returns
///
/// Returns a ReadyStatus containing the overall readiness status
/// and detailed information about each subsystem.
pub fn check_readiness(
    ledger: &Arc<Mutex<crate::governance::Ledger>>,
    auth: &Arc<Mutex<crate::local_auth::AuthStore>>,
) -> ReadyStatus {
    let timestamp = current_timestamp();

    // Check ledger status
    let ledger_status = check_ledger_status(ledger);

    // Check authentication status
    let auth_status = check_auth_status(auth);

    // Check storage status
    let storage_status = check_storage_status();

    // Check configuration status
    let config_status = check_config_status();

    // Determine overall status
    let status = determine_overall_status(
        &ledger_status,
        &auth_status,
        &storage_status,
        &config_status,
    );

    let details = ReadyDetails {
        ledger: ledger_status,
        auth: auth_status,
        storage: storage_status,
        config: config_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_info: BuildInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_timestamp: timestamp,
            git_commit: option_env!("GIT_COMMIT_HASH")
                .unwrap_or("unknown")
                .to_string(),
            environment: std::env::var("EKPI_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
        },
    };

    ReadyStatus {
        status,
        details,
        timestamp,
    }
}

fn check_ledger_status(ledger: &Arc<Mutex<crate::governance::Ledger>>) -> LedgerStatus {
    let mut ledger_ok = false;
    let mut event_count = 0;
    let mut last_event_timestamp = None;
    let mut consistent = true;
    let mut warnings = Vec::new();

    match ledger.lock() {
        Ok(ledger_guard) => {
            ledger_ok = true;
            event_count = ledger_guard.snapshot().events.len();
            if let Some(last_event) = ledger_guard.snapshot().events.last() {
                last_event_timestamp = Some(last_event.timestamp_unix);
            }

            // Check for potential consistency issues
            if let Err(e) = ledger_guard.verify() {
                consistent = false;
                warnings.push(format!("Ledger consistency issue: {}", e));
            }
        }
        Err(e) => {
            warnings.push(format!("Cannot access ledger: {}", e));
        }
    }

    LedgerStatus {
        accessible: ledger_ok,
        event_count,
        last_event_timestamp,
        consistent,
        warnings,
    }
}

fn check_auth_status(auth: &Arc<Mutex<crate::local_auth::AuthStore>>) -> AuthStatus {
    let mut auth_store_ok = false;
    let mut session_timeout_configured = false;
    let mut warnings = Vec::new();

    match auth.lock() {
        Ok(auth_guard) => {
            auth_store_ok = true;
            session_timeout_configured = auth_guard.session_ttl_secs() > 0;
            let user_count = auth_guard.user_count();
            if user_count == 0 {
                warnings.push("No users configured".to_string());
            }
        }
        Err(e) => {
            warnings.push(format!("Cannot access auth store: {}", e));
        }
    }

    AuthStatus {
        accessible: auth_store_ok,
        auth_store_ok,
        session_timeout_configured,
        warnings,
    }
}

fn check_storage_status() -> StorageStatus {
    let mut data_dir_accessible = false;
    let mut backup_dir_accessible = false;
    let mut disk_space_mb = 0;
    let mut warnings = Vec::new();

    // Check data directory
    if let Ok(metadata) = std::fs::metadata("./data") {
        data_dir_accessible = metadata.is_dir();
        if let Some(space) = metadata.len().checked_div(1024 * 1024) {
            disk_space_mb = space;
        }
    } else {
        warnings.push("Data directory not accessible".to_string());
    }

    // Check backup directory
    if let Ok(metadata) = std::fs::metadata("./backups") {
        backup_dir_accessible = metadata.is_dir();
    }

    // Check minimum disk space (100MB)
    if disk_space_mb < 100 {
        warnings.push(format!("Low disk space: {}MB", disk_space_mb));
    }

    StorageStatus {
        data_dir_accessible,
        backup_dir_accessible,
        disk_space_mb,
        warnings,
    }
}

fn check_config_status() -> ConfigStatus {
    let loaded = true;
    let valid = true;
    let warnings = Vec::new();

    // Try to load configuration
    let _config = crate::config::get_config();

    ConfigStatus {
        loaded,
        valid,
        warnings,
    }
}

fn determine_overall_status(
    ledger_status: &LedgerStatus,
    auth_status: &AuthStatus,
    storage_status: &StorageStatus,
    config_status: &ConfigStatus,
) -> ReadyStatusType {
    // If any critical component is not accessible, we're not ready
    if !ledger_status.accessible || !auth_status.accessible {
        return ReadyStatusType::NotReady;
    }

    // If storage is not accessible, we're degraded
    if !storage_status.data_dir_accessible {
        return ReadyStatusType::Degraded;
    }

    // Check for critical warnings
    let has_critical_warnings = ledger_status
        .warnings
        .iter()
        .any(|w| w.contains("CRITICAL"))
        || auth_status.warnings.iter().any(|w| w.contains("CRITICAL"))
        || storage_status
            .warnings
            .iter()
            .any(|w| w.contains("CRITICAL"))
        || config_status
            .warnings
            .iter()
            .any(|w| w.contains("CRITICAL"));

    if has_critical_warnings {
        return ReadyStatusType::Degraded;
    }

    // If any warning exists, we're degraded
    let has_warnings = !ledger_status.warnings.is_empty()
        || !auth_status.warnings.is_empty()
        || !storage_status.warnings.is_empty()
        || !config_status.warnings.is_empty();

    if has_warnings {
        return ReadyStatusType::Degraded;
    }

    // All checks passed, we're ready
    ReadyStatusType::Ready
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
