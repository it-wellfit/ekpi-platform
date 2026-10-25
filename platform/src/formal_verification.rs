#![forbid(unsafe_code)]

//! Formal verification (Master Prompt Phase 2).
//!
//! Defines property specifications and a verifier boundary. Real TLA+/Alloy
//! model checking requires an external `tlc`/`alloy` binary; this module
//! provides the property parser and a stub verifier that returns `Unknown`
//! unless a real backend is wired in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalOperator {
    Always,     // []
    Eventually, // <>
    Next,       // X
    Until,      // U
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationResult {
    Verified,
    Violated(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub expression: String,
    pub operator: Option<TemporalOperator>,
}

impl Property {
    /// Parse a property expression like "[] (x > 0)" or "<> done".
    pub fn parse(name: &str, expr: &str) -> Self {
        let expr = expr.trim();
        let operator = if expr.starts_with("[]") {
            Some(TemporalOperator::Always)
        } else if expr.starts_with("<>") {
            Some(TemporalOperator::Eventually)
        } else if expr.starts_with("X ") || expr.starts_with("X(") {
            Some(TemporalOperator::Next)
        } else {
            None
        };
        Property {
            name: name.to_string(),
            expression: expr.to_string(),
            operator,
        }
    }

    pub fn is_temporal(&self) -> bool {
        self.operator.is_some()
    }
}

pub struct FormalVerifier;

impl FormalVerifier {
    /// Stub: returns Unknown unless a real backend is configured.
    pub fn check(_property: &Property, _model: &str) -> VerificationResult {
        VerificationResult::Unknown
    }

    /// Validate that a property expression is well-formed (non-empty, has operator or predicate).
    pub fn validate(prop: &Property) -> bool {
        !prop.name.is_empty() && !prop.expression.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_always() {
        let p = Property::parse("safety", "[] (x > 0)");
        assert_eq!(p.operator, Some(TemporalOperator::Always));
        assert!(p.is_temporal());
    }

    #[test]
    fn parse_eventually() {
        let p = Property::parse("liveness", "<> done");
        assert_eq!(p.operator, Some(TemporalOperator::Eventually));
    }

    #[test]
    fn parse_no_operator() {
        let p = Property::parse("basic", "x > 0");
        assert!(p.operator.is_none());
        assert!(!p.is_temporal());
    }

    #[test]
    fn validate_ok() {
        let p = Property::parse("safety", "[] (x > 0)");
        assert!(FormalVerifier::validate(&p));
    }

    #[test]
    fn stub_returns_unknown() {
        let p = Property::parse("safety", "[] (x > 0)");
        assert_eq!(
            FormalVerifier::check(&p, "model"),
            VerificationResult::Unknown
        );
    }
}
