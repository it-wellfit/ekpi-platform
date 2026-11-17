#![forbid(unsafe_code)]

//! Runtime configuration for the `governance_web` binary.
//!
//! Stage 1 supports loopback binds only. The resolver is kept separate from
//! process-environment access so the parsing and validation rules can be
//! tested without mutating global state.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

const DEFAULT_BIND: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;

/// Resolved runtime settings for the workbench.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub bind_ip: IpAddr,
    pub port: u16,
    pub data_dir: PathBuf,
}

impl RuntimeConfig {
    /// The socket address the server binds to.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_ip, self.port)
    }
}

/// Resolve settings from optional raw values.
///
/// Empty or absent values fall back to the Stage 1 defaults. This function
/// performs no filesystem access; use [`validate_data_dir`] for that.
pub fn resolve(
    bind_ip: Option<&str>,
    port: Option<&str>,
    data_dir: Option<&str>,
) -> Result<RuntimeConfig, String> {
    let bind_value = bind_ip
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_BIND);

    let bind_ip: IpAddr = bind_value.parse().map_err(|_| {
        if bind_value.parse::<SocketAddr>().is_ok() {
            format!(
                "EKPI_BIND_IP 只接受 IP（不可含埠）；目前值 {bind_value:?} 含埠，埠請改用 EKPI_WEB_PORT"
            )
        } else {
            format!("EKPI_BIND_IP 不是合法 IP：{bind_value:?}（例如 127.0.0.1 或 ::1）")
        }
    })?;

    if !bind_ip.is_loopback() {
        return Err(format!(
            "EKPI_BIND_IP 在 Stage 1 只允許 loopback（127.0.0.0/8 或 ::1）；目前值 {bind_ip} 非 loopback，LAN 暴露需 Stage 2 核准"
        ));
    }

    let port = match port.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value
            .parse::<u16>()
            .map_err(|_| format!("EKPI_WEB_PORT 必須是 1–65535 的整數；目前值 {value:?}"))?,
        None => DEFAULT_PORT,
    };
    if port == 0 {
        return Err("EKPI_WEB_PORT 不可為 0；Stage 1 需固定埠（預設 3000）".to_string());
    }

    let data_dir = match data_dir.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => PathBuf::from(value),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
    };

    Ok(RuntimeConfig {
        bind_ip,
        port,
        data_dir,
    })
}

/// Validate the data directory.
///
/// Returns `Ok(Some(warning))` when the default directory is absent: the
/// workbench can still serve ledger pages, and `/search` will report an honest
/// load error. An explicitly configured missing path is a startup error.
pub fn validate_data_dir(path: &Path, explicit: bool) -> Result<Option<String>, String> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(None)
            } else {
                Err(format!(
                    "EKPI_DATA_DIR 必須是目錄，目前不是目錄：{}",
                    path.display()
                ))
            }
        }
        Err(error) => {
            if explicit {
                Err(format!(
                    "EKPI_DATA_DIR 指向的目錄不存在或不可讀：{} ({error})",
                    path.display()
                ))
            } else {
                Ok(Some(format!(
                    "資料目錄不存在，/search 將無法載入文件庫：{} ({error})",
                    path.display()
                )))
            }
        }
    }
}

/// Read the Stage 1 environment variables and resolve the configuration.
pub fn from_env() -> Result<RuntimeConfig, String> {
    let bind_ip = std::env::var("EKPI_BIND_IP").ok();
    let port = std::env::var("EKPI_WEB_PORT").ok();
    let data_dir = std::env::var("EKPI_DATA_DIR").ok();

    let explicit_data_dir = data_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some();

    let config = resolve(bind_ip.as_deref(), port.as_deref(), data_dir.as_deref())?;
    if let Some(warning) = validate_data_dir(&config.data_dir, explicit_data_dir)? {
        eprintln!("警告：{warning}");
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ekpi-runtime-config-{name}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn defaults_are_loopback_port_3000_and_manifest_data() {
        let config = resolve(None, None, None).expect("defaults resolve");
        assert_eq!(config.bind_ip, IpAddr::from([127, 0, 0, 1]));
        assert_eq!(config.port, 3000);
        assert_eq!(
            config.data_dir,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
        );
        assert_eq!(
            config.socket_addr().to_string(),
            "127.0.0.1:3000",
            "default socket address must stay loopback-only"
        );
    }

    #[test]
    fn accepts_ipv4_and_ipv6_loopback() {
        let ipv4 = resolve(Some("127.0.0.5"), Some("3100"), None).expect("ipv4 loopback");
        assert_eq!(ipv4.bind_ip, IpAddr::from([127, 0, 0, 5]));
        assert_eq!(ipv4.port, 3100);

        let ipv6 = resolve(Some("::1"), None, None).expect("ipv6 loopback");
        assert_eq!(ipv6.bind_ip, IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]));
        assert_eq!(ipv6.socket_addr().to_string(), "[::1]:3000");
    }

    #[test]
    fn refuses_non_loopback_binds() {
        let error = resolve(Some("0.0.0.0"), None, None).expect_err("wildcard must be refused");
        assert!(error.contains("loopback"), "unexpected error: {error}");
        assert!(error.contains("Stage 2"), "unexpected error: {error}");

        let error = resolve(Some("192.168.1.10"), None, None).expect_err("LAN IP must be refused");
        assert!(error.contains("非 loopback"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_bind_values_that_include_a_port() {
        let error =
            resolve(Some("127.0.0.1:3000"), None, None).expect_err("IP:port must be refused");
        assert!(error.contains("EKPI_WEB_PORT"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_malformed_bind_values() {
        let error = resolve(Some("localhost"), None, None).expect_err("hostname must be refused");
        assert!(error.contains("不是合法 IP"), "unexpected error: {error}");
    }

    #[test]
    fn port_parsing_is_fail_closed() {
        assert!(resolve(None, Some("not-a-number"), None).is_err());
        assert!(resolve(None, Some(""), None).is_ok());
        assert!(resolve(None, Some("0"), None).is_err());
        assert!(resolve(None, Some("65536"), None).is_err());
        let config = resolve(None, Some(" 3001 "), None).expect("trimmed port");
        assert_eq!(config.port, 3001);
    }

    #[test]
    fn explicit_missing_data_dir_is_an_error() {
        let missing = std::env::temp_dir().join(format!(
            "ekpi-runtime-config-missing-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let error = validate_data_dir(&missing, true).expect_err("explicit missing dir");
        assert!(error.contains("EKPI_DATA_DIR"), "unexpected error: {error}");
    }

    #[test]
    fn default_missing_data_dir_produces_a_warning_not_an_error() {
        let missing = std::env::temp_dir().join(format!(
            "ekpi-runtime-config-default-missing-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let warning = validate_data_dir(&missing, false)
            .expect("default missing dir is only a warning")
            .expect("a warning is returned");
        assert!(warning.contains("/search"), "unexpected warning: {warning}");
    }

    #[test]
    fn existing_data_dir_is_accepted() {
        let dir = temp_dir("existing");
        assert_eq!(validate_data_dir(&dir, true).expect("existing dir"), None);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn a_file_is_not_a_data_dir() {
        let path = std::env::temp_dir().join(format!(
            "ekpi-runtime-config-file-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, b"not a directory").expect("write file");
        let error = validate_data_dir(&path, true).expect_err("file must be refused");
        assert!(error.contains("必須是目錄"), "unexpected error: {error}");
        std::fs::remove_file(&path).expect("cleanup");
    }
}
