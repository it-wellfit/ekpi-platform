#![forbid(unsafe_code)]

/// Configuration management for the EKPI platform.
///
/// This module provides configuration loading, validation, and access for all
/// platform settings. It ensures type safety and validates configuration values
/// against schema requirements.
///
/// Configuration is loaded from config.toml files and environment variables,
/// with appropriate defaults for different deployment environments.
///
/// Configuration values include:
/// - Server settings (host, port, timeout)
/// - Database/storage configurations
/// - Security settings (authentication, CORS, rate limiting)
/// - Logging and monitoring configurations
/// - Feature flags and experimental settings
///
/// All configuration values are validated at startup to ensure the platform
/// can start successfully with valid settings.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Database/storage configuration
    pub storage: StorageConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging and monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable HTTPS
    pub https_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS private key path
    pub tls_key_path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StorageConfig {
    /// Data directory path
    pub data_dir: String,
    /// Ledger file path
    pub ledger_path: String,
    /// Backup directory path
    pub backup_dir: String,
    /// Maximum ledger events to load into memory
    pub max_ledger_events: usize,
    /// Enable atomic writes
    pub atomic_writes: bool,
    /// Backup retention days
    pub backup_retention_days: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SecurityConfig {
    /// Cookie secure flag
    pub cookie_secure: bool,
    /// CSRF token length
    pub csrf_token_length: usize,
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Maximum failed login attempts before lockout
    pub max_failed_logins: u32,
    /// Lockout duration in seconds
    pub lockout_duration_secs: u64,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    /// Rate limiting per minute per IP
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MonitoringConfig {
    /// Enable metrics endpoint
    pub metrics_enabled: bool,
    /// Metrics port
    pub metrics_port: Option<u16>,
    /// Log level
    pub log_level: String,
    /// Enable structured logging
    pub structured_logging: bool,
    /// Health check endpoint enabled
    pub health_check_enabled: bool,
    /// Readiness check endpoint enabled
    pub readiness_check_enabled: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FeatureFlags {
    /// Enable experimental features
    pub experimental: bool,
    /// Enable AI features
    pub ai_enabled: bool,
    /// Enable advanced search
    pub advanced_search_enabled: bool,
    /// Enable governance workflows
    pub workflows_enabled: bool,
    /// Enable multi-tenancy
    pub multi_tenant_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                request_timeout_secs: 15,
                max_concurrent_requests: 64,
                https_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
                ledger_path: "ledger.jsonl".to_string(),
                backup_dir: "./backups".to_string(),
                max_ledger_events: 10000,
                atomic_writes: true,
                backup_retention_days: 7,
            },
            security: SecurityConfig {
                cookie_secure: false,
                csrf_token_length: 32,
                session_timeout_secs: 3600,
                max_failed_logins: 5,
                lockout_duration_secs: 300,
                cors_allowed_origins: vec!["http://localhost:8080".to_string()],
                rate_limit_per_minute: 60,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: Some(9090),
                log_level: "info".to_string(),
                structured_logging: true,
                health_check_enabled: true,
                readiness_check_enabled: true,
            },
            features: FeatureFlags {
                experimental: false,
                ai_enabled: false,
                advanced_search_enabled: true,
                workflows_enabled: true,
                multi_tenant_enabled: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file (TOML format)
    ///
    /// # Returns
    ///
    /// Returns a Config instance with values loaded from the file,
    /// or the default configuration if the file cannot be read or parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or
    /// if the parsed values fail validation.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read config file '{}': {}", path, e))?;

        let mut config: Self = toml::from_str(&content)
            .map_err(|e| format!("Cannot parse config file '{}': {}", path, e))?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from environment variables
    ///
    /// Environment variables should be prefixed with "EKPI_" and use
    /// uppercase with underscores (e.g., "EKPI_SERVER_PORT").
    ///
    /// # Returns
    ///
    /// Returns a Config instance with values loaded from environment variables,
    /// with file-based configuration taking precedence.
    pub fn from_env() -> Result<Self, String> {
        let mut config = Self::default();

        // Override from environment variables
        if let Ok(host) = std::env::var("EKPI_SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("EKPI_SERVER_PORT") {
            config.server.port = port
                .parse()
                .map_err(|e| format!("Invalid EKPI_SERVER_PORT: {}", e))?;
        }
        if let Ok(cookie_secure) = std::env::var("EKPI_COOKIE_SECURE") {
            config.security.cookie_secure = cookie_secure.parse().unwrap_or(false);
        }
        if let Ok(data_dir) = std::env::var("EKPI_DATA_DIR") {
            config.storage.data_dir = data_dir;
        }
        if let Ok(backup_dir) = std::env::var("EKPI_BACKUP_DIR") {
            config.storage.backup_dir = backup_dir;
        }
        if let Ok(log_level) = std::env::var("EKPI_LOG_LEVEL") {
            config.monitoring.log_level = log_level;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    ///
    /// Performs comprehensive validation of all configuration values to ensure
    /// the platform can start successfully with the given settings.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation passes, or an error with details about
    /// what failed validation.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration value is invalid or inconsistent
    /// with other configuration values.
    pub fn validate(&mut self) -> Result<(), String> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        if self.server.host.is_empty() {
            return Err("Server host cannot be empty".to_string());
        }
        if self.server.request_timeout_secs == 0 {
            return Err("Request timeout cannot be 0".to_string());
        }

        // Validate storage configuration
        if self.storage.data_dir.is_empty() {
            return Err("Data directory cannot be empty".to_string());
        }
        if self.storage.ledger_path.is_empty() {
            return Err("Ledger path cannot be empty".to_string());
        }
        if self.storage.backup_retention_days == 0 {
            return Err("Backup retention days cannot be 0".to_string());
        }

        // Validate security configuration
        if self.security.csrf_token_length < 16 {
            return Err("CSRF token length must be at least 16".to_string());
        }
        if self.security.session_timeout_secs == 0 {
            return Err("Session timeout cannot be 0".to_string());
        }
        if self.security.max_failed_logins == 0 {
            return Err("Maximum failed logins cannot be 0".to_string());
        }
        if self.security.lockout_duration_secs == 0 {
            return Err("Lockout duration cannot be 0".to_string());
        }

        // Validate monitoring configuration
        if self.monitoring.metrics_enabled && self.monitoring.metrics_port.is_none() {
            return Err("Metrics port must be specified when metrics are enabled".to_string());
        }
        if !["trace", "debug", "info", "warn", "error"]
            .contains(&self.monitoring.log_level.as_str())
        {
            return Err("Invalid log level".to_string());
        }

        // Validate feature flags
        if self.features.multi_tenant_enabled && self.features.ai_enabled {
            return Err("Multi-tenancy and AI cannot both be enabled simultaneously".to_string());
        }

        // Create directories if they don't exist
        if let Err(e) = std::fs::create_dir_all(&self.storage.data_dir) {
            return Err(format!(
                "Cannot create data directory '{}': {}",
                self.storage.data_dir, e
            ));
        }
        if let Err(e) = std::fs::create_dir_all(&self.storage.backup_dir) {
            return Err(format!(
                "Cannot create backup directory '{}': {}",
                self.storage.backup_dir, e
            ));
        }

        Ok(())
    }

    /// Get the configuration as a TOML string
    ///
    /// # Returns
    ///
    /// Returns a TOML representation of the configuration, suitable for
    /// saving to a configuration file.
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|e| format!("Cannot serialize config to TOML: {}", e))
    }
}

/// Global configuration instance
static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();

/// Get the global configuration instance
///
/// # Returns
///
/// Returns a reference to the global configuration instance, creating it
/// with values from environment variables if it hasn't been initialized yet.
///
/// # Panics
///
/// Panics if the configuration cannot be loaded.
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| match Config::from_env() {
        Ok(config) => config,
        Err(e) => panic!("Failed to load configuration: {}", e),
    })
}

/// Load configuration from a specific file and set it as the global configuration
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Panics
///
/// Panics if the configuration cannot be loaded.
pub fn set_config_from_file(path: &str) {
    let config = Config::from_file(path).unwrap_or_else(|e| {
        panic!("Failed to load configuration from '{}': {}", path, e);
    });
    // Set the global configuration
    let _ = CONFIG.set(config);
}
