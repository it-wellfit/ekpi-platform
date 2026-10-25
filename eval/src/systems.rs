#![forbid(unsafe_code)]

use crate::types::{Case, Decision, Ver};

pub trait System {
    fn name(&self) -> &'static str;
    fn decide(&self, case: &Case) -> Decision;
}

/// Always answers comply. Exists only to prove the hard gates fire on a bad
/// system; it is one of the three deterministic stubs, not a baseline.
pub struct AllowAll;

impl System for AllowAll {
    fn name(&self) -> &'static str {
        "stub:allow_all"
    }

    fn decide(&self, case: &Case) -> Decision {
        Decision {
            case_id: case.case_id.clone(),
            system: self.name().to_string(),
            verdict: Ver::Comply,
            confidence: 1.0,
            cited_ids: Vec::new(),
            rule_ids_used: Vec::new(),
            latency_ms: 1,
            cost_tokens: 0,
        }
    }
}

/// Always refuses. Acceptable only where refusal is required; a permanent
/// refusal on definitive cases is the measured second failure mode.
pub struct RefuseAll;

impl System for RefuseAll {
    fn name(&self) -> &'static str {
        "stub:refuse_all"
    }

    fn decide(&self, case: &Case) -> Decision {
        Decision {
            case_id: case.case_id.clone(),
            system: self.name().to_string(),
            verdict: Ver::InsufficientEvidence,
            confidence: 0.95,
            cited_ids: Vec::new(),
            rule_ids_used: Vec::new(),
            latency_ms: 1,
            cost_tokens: 0,
        }
    }
}

/// Reads annotations. Proves the harness reaches 100% when verdicts are right.
pub struct Oracle;

impl System for Oracle {
    fn name(&self) -> &'static str {
        "stub:oracle"
    }

    fn decide(&self, case: &Case) -> Decision {
        let verdict = case.expected().expect("verdicts validated at load time");
        let confidence = if case.abstain_required { 0.99 } else { 1.0 };
        Decision {
            case_id: case.case_id.clone(),
            system: self.name().to_string(),
            verdict,
            confidence,
            cited_ids: case.citations.clone(),
            rule_ids_used: case.rules.clone(),
            latency_ms: 2,
            cost_tokens: 1,
        }
    }
}

pub fn registry() -> Vec<Box<dyn System>> {
    vec![Box::new(AllowAll), Box::new(RefuseAll), Box::new(Oracle)]
}
