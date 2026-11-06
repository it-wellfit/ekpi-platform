#![forbid(unsafe_code)]

//! Channel adapter contract (domain side).
//!
//! Defines the trait boundary that platform-hub implements.
//! Implementation lives in platform-hub; this is the contract.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Capabilities a channel adapter supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCapabilities {
    pub fetch_orders: bool,
    pub push_inventory: bool,
    pub webhook: bool,
}

/// Channel adapter trait — domain-side contract.
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn platform_id(&self) -> &'static str;
    fn capabilities(&self) -> ChannelCapabilities;
    async fn fetch_orders(&self) -> Result<Vec<serde_json::Value>, ChannelError>;
    async fn push_inventory(&self, records: &[serde_json::Value]) -> Result<(), ChannelError>;
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
    async fn health(&self) -> Result<(), ChannelError>;
}

/// Bidirectional format derivation.
pub trait FormatDeriver: Send + Sync {
    fn platform_to_udm(&self, raw: &[u8]) -> Result<Vec<serde_json::Value>, ChannelError>;
    fn udm_to_platform(&self, records: &[serde_json::Value]) -> Result<Vec<u8>, ChannelError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("http error: {0}")]
    Http(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("rate limited")]
    RateLimited,
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_error_display() {
        let e = ChannelError::NotImplemented("test".into());
        assert!(e.to_string().contains("test"));
    }

    /// Contract lock: a well-behaved adapter's format round-trip must be
    /// lossless for the payloads it accepts. Demonstrated with a JSON-lines
    /// identity deriver over a single record.
    struct JsonLinesDeriver;

    impl FormatDeriver for JsonLinesDeriver {
        fn platform_to_udm(&self, raw: &[u8]) -> Result<Vec<serde_json::Value>, ChannelError> {
            serde_json::from_slice::<Vec<serde_json::Value>>(raw)
                .map_err(|e| ChannelError::Parse(e.to_string()))
        }

        fn udm_to_platform(&self, records: &[serde_json::Value]) -> Result<Vec<u8>, ChannelError> {
            serde_json::to_vec(records).map_err(|e| ChannelError::Parse(e.to_string()))
        }
    }

    #[test]
    fn format_deriver_round_trip_is_lossless() {
        let deriver = JsonLinesDeriver;
        let original = vec![
            serde_json::json!({"order_id": "A-1", "amount_cents": 12_500}),
            serde_json::json!({"order_id": "A-2", "amount_cents": 3_000}),
        ];
        let bytes = deriver
            .udm_to_platform(&original)
            .expect("serialize udm records");
        let parsed = deriver
            .platform_to_udm(&bytes)
            .expect("parse platform payload");
        assert_eq!(parsed, original, "round-trip must preserve records");
    }

    #[test]
    fn malformed_payload_is_a_parse_error_not_a_panic() {
        let deriver = JsonLinesDeriver;
        let err = deriver
            .platform_to_udm(b"not json")
            .expect_err("malformed payload must be a parse error");
        assert!(matches!(err, ChannelError::Parse(_)));
    }
}
