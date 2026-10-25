#![forbid(unsafe_code)]

//! PM4Py sidecar contract (Master Prompt Phase 4c).
//!
//! Hash-based artifact contract: input hash → process model, output hash → conformance report.
//! Sidecar invocation via subprocess with timeout.

use serde::{Deserialize, Serialize};

/// Sidecar input contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarInput {
    pub input_hash: String,
    pub xes_path: String,
    pub process_model_path: String,
    pub timeout_seconds: u64,
}

/// Sidecar output contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarOutput {
    pub output_hash: String,
    pub conformance_report: String,
    pub status: SidecarStatus,
}

/// Sidecar execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidecarStatus {
    Success,
    Timeout,
    Error,
    HashMismatch,
}

impl SidecarStatus {
    pub fn name(&self) -> &'static str {
        match self {
            SidecarStatus::Success => "success",
            SidecarStatus::Timeout => "timeout",
            SidecarStatus::Error => "error",
            SidecarStatus::HashMismatch => "hash_mismatch",
        }
    }
}

/// PM4Py sidecar contract manager.
pub struct Pm4pySidecar;

impl Pm4pySidecar {
    /// Validate input contract (hash must be non-empty, paths must be non-empty).
    pub fn validate_input(input: &SidecarInput) -> Result<(), String> {
        if input.input_hash.is_empty() {
            return Err("input_hash must not be empty".to_string());
        }
        if input.xes_path.is_empty() {
            return Err("xes_path must not be empty".to_string());
        }
        if input.process_model_path.is_empty() {
            return Err("process_model_path must not be empty".to_string());
        }
        if input.timeout_seconds == 0 {
            return Err("timeout_seconds must be > 0".to_string());
        }
        Ok(())
    }

    /// Verify output hash matches expected.
    pub fn verify_output(output: &SidecarOutput, expected_hash: &str) -> bool {
        output.output_hash == expected_hash
    }

    /// Compute a simple hash of input data (SHA-256 hex).
    pub fn compute_input_hash(data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Simulate sidecar execution (returns mock output for testing).
    pub fn simulate(input: &SidecarInput) -> SidecarOutput {
        let output_hash =
            Self::compute_input_hash(&format!("{}-{}", input.input_hash, input.xes_path));
        SidecarOutput {
            output_hash,
            conformance_report: r#"{"conformant": true, "fitness": 1.0}"#.to_string(),
            status: SidecarStatus::Success,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_input_ok() {
        let input = SidecarInput {
            input_hash: "abc123".to_string(),
            xes_path: "data.xes".to_string(),
            process_model_path: "model.bpmn".to_string(),
            timeout_seconds: 30,
        };
        assert!(Pm4pySidecar::validate_input(&input).is_ok());
    }

    #[test]
    fn validate_input_rejects_empty_hash() {
        let input = SidecarInput {
            input_hash: "".to_string(),
            xes_path: "data.xes".to_string(),
            process_model_path: "model.bpmn".to_string(),
            timeout_seconds: 30,
        };
        assert!(Pm4pySidecar::validate_input(&input).is_err());
    }

    #[test]
    fn validate_input_rejects_zero_timeout() {
        let input = SidecarInput {
            input_hash: "abc".to_string(),
            xes_path: "data.xes".to_string(),
            process_model_path: "model.bpmn".to_string(),
            timeout_seconds: 0,
        };
        assert!(Pm4pySidecar::validate_input(&input).is_err());
    }

    #[test]
    fn verify_output_matches() {
        let output = SidecarOutput {
            output_hash: "hash123".to_string(),
            conformance_report: "{}".to_string(),
            status: SidecarStatus::Success,
        };
        assert!(Pm4pySidecar::verify_output(&output, "hash123"));
        assert!(!Pm4pySidecar::verify_output(&output, "wrong"));
    }

    #[test]
    fn simulate_produces_success() {
        let input = SidecarInput {
            input_hash: "abc".to_string(),
            xes_path: "data.xes".to_string(),
            process_model_path: "model.bpmn".to_string(),
            timeout_seconds: 30,
        };
        let output = Pm4pySidecar::simulate(&input);
        assert_eq!(output.status, SidecarStatus::Success);
        assert!(!output.output_hash.is_empty());
    }

    #[test]
    fn compute_input_hash_deterministic() {
        let h1 = Pm4pySidecar::compute_input_hash("test data");
        let h2 = Pm4pySidecar::compute_input_hash("test data");
        assert_eq!(h1, h2);
        assert_ne!(h1, Pm4pySidecar::compute_input_hash("different"));
    }

    #[test]
    fn sidecar_status_names_unique() {
        let statuses = [
            SidecarStatus::Success,
            SidecarStatus::Timeout,
            SidecarStatus::Error,
            SidecarStatus::HashMismatch,
        ];
        let names: Vec<&str> = statuses.iter().map(|s| s.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }
}
