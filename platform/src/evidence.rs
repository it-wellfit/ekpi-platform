#![forbid(unsafe_code)]

//! Evidence and claim layer (Master Prompt Phase 2.6, §3.6).
//!
//! Every important conclusion must have a Claim-Evidence link.
//! Distinguishes: direct facts, rule-derived conclusions, event-derived judgments,
//! statistical inferences, causal claims, unverified hypotheses, undetermined questions.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Claim type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    /// Direct fact from source document.
    Fact,
    /// Derived from a rule application.
    RuleDerived,
    /// Derived from event log analysis.
    EventDerived,
    /// Statistical inference.
    Statistical,
    /// Causal claim (requires causal module validation).
    Causal,
    /// Unverified hypothesis.
    Hypothesis,
    /// Cannot be determined.
    Undetermined,
}

impl ClaimType {
    pub fn name(&self) -> &'static str {
        match self {
            ClaimType::Fact => "fact",
            ClaimType::RuleDerived => "rule_derived",
            ClaimType::EventDerived => "event_derived",
            ClaimType::Statistical => "statistical",
            ClaimType::Causal => "causal",
            ClaimType::Hypothesis => "hypothesis",
            ClaimType::Undetermined => "undetermined",
        }
    }
}

/// Human review status of a claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    RequiresRevision,
}

/// A piece of evidence with source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub source_doc_id: String,
    pub source_version: String,
    pub section: String,
    pub page: Option<u32>,
    pub paragraph: Option<u32>,
    pub verbatim_text: String,
    pub hash: String,
    pub access_level: String,
    pub timestamp: String,
}

impl Evidence {
    pub fn new(id: &str, doc_id: &str, version: &str, section: &str, verbatim: &str) -> Self {
        let hash = sha256_hex(verbatim.as_bytes());
        Evidence {
            id: id.to_string(),
            source_doc_id: doc_id.to_string(),
            source_version: version.to_string(),
            section: section.to_string(),
            page: None,
            paragraph: None,
            verbatim_text: verbatim.to_string(),
            hash,
            access_level: "internal".to_string(),
            timestamp: String::new(),
        }
    }
}

/// A claim with supporting/contradicting evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub claim_type: ClaimType,
    pub supporting_evidence: Vec<String>,
    pub contradicting_evidence: Vec<String>,
    pub scope: String,
    pub reasoning_steps: Vec<String>,
    pub confidence: f64,
    pub human_review_status: ReviewStatus,
}

impl Claim {
    pub fn new(id: &str, text: &str, claim_type: ClaimType) -> Self {
        Claim {
            id: id.to_string(),
            text: text.to_string(),
            claim_type,
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
            scope: String::new(),
            reasoning_steps: Vec::new(),
            confidence: 0.0,
            human_review_status: ReviewStatus::Pending,
        }
    }

    pub fn add_supporting(&mut self, evidence_id: &str) {
        if !self.supporting_evidence.contains(&evidence_id.to_string()) {
            self.supporting_evidence.push(evidence_id.to_string());
        }
    }

    pub fn add_contradicting(&mut self, evidence_id: &str) {
        if !self
            .contradicting_evidence
            .contains(&evidence_id.to_string())
        {
            self.contradicting_evidence.push(evidence_id.to_string());
        }
    }

    pub fn is_supported(&self) -> bool {
        !self.supporting_evidence.is_empty() && self.contradicting_evidence.is_empty()
    }

    pub fn has_conflict(&self) -> bool {
        !self.supporting_evidence.is_empty() && !self.contradicting_evidence.is_empty()
    }

    pub fn set_confidence(&mut self, c: f64) {
        self.confidence = c.clamp(0.0, 1.0);
    }
}

/// Evidence store with claim-evidence linking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceStore {
    pub evidence: BTreeMap<String, Evidence>,
    pub claims: BTreeMap<String, Claim>,
}

impl EvidenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_evidence(&mut self, ev: Evidence) {
        self.evidence.insert(ev.id.clone(), ev);
    }

    pub fn add_claim(&mut self, claim: Claim) {
        self.claims.insert(claim.id.clone(), claim);
    }

    pub fn get_evidence(&self, id: &str) -> Option<&Evidence> {
        self.evidence.get(id)
    }

    pub fn get_claim(&self, id: &str) -> Option<&Claim> {
        self.claims.get(id)
    }

    pub fn get_claim_mut(&mut self, id: &str) -> Option<&mut Claim> {
        self.claims.get_mut(id)
    }

    pub fn link_support(&mut self, claim_id: &str, evidence_id: &str) -> bool {
        if let Some(claim) = self.claims.get_mut(claim_id) {
            if self.evidence.contains_key(evidence_id) {
                claim.add_supporting(evidence_id);
                return true;
            }
        }
        false
    }

    pub fn link_contradiction(&mut self, claim_id: &str, evidence_id: &str) -> bool {
        if let Some(claim) = self.claims.get_mut(claim_id) {
            if self.evidence.contains_key(evidence_id) {
                claim.add_contradicting(evidence_id);
                return true;
            }
        }
        false
    }

    pub fn supported_claims(&self) -> Vec<&Claim> {
        self.claims.values().filter(|c| c.is_supported()).collect()
    }

    pub fn conflicting_claims(&self) -> Vec<&Claim> {
        self.claims.values().filter(|c| c.has_conflict()).collect()
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }

    pub fn claim_count(&self) -> usize {
        self.claims.len()
    }
}

// Helper for SHA-256
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_creation() {
        let claim = Claim::new("C1", "test claim", ClaimType::Fact);
        assert_eq!(claim.id, "C1");
        assert_eq!(claim.claim_type, ClaimType::Fact);
        assert_eq!(claim.confidence, 0.0);
        assert_eq!(claim.human_review_status, ReviewStatus::Pending);
    }

    #[test]
    fn evidence_creation() {
        let ev = Evidence::new("E1", "DOC-1", "1.0", "§1", "verbatim text");
        assert_eq!(ev.id, "E1");
        assert_eq!(ev.source_doc_id, "DOC-1");
        assert!(!ev.hash.is_empty());
    }

    #[test]
    fn link_support() {
        let mut store = EvidenceStore::new();
        store.add_evidence(Evidence::new("E1", "DOC-1", "1.0", "§1", "text"));
        store.add_claim(Claim::new("C1", "claim", ClaimType::Fact));
        assert!(store.link_support("C1", "E1"));
        let claim = store.get_claim("C1").unwrap();
        assert!(claim.is_supported());
        assert!(!claim.has_conflict());
    }

    #[test]
    fn link_contradiction() {
        let mut store = EvidenceStore::new();
        store.add_evidence(Evidence::new("E1", "DOC-1", "1.0", "§1", "text1"));
        store.add_evidence(Evidence::new("E2", "DOC-1", "1.0", "§2", "text2"));
        store.add_claim(Claim::new("C1", "claim", ClaimType::Fact));
        store.link_support("C1", "E1");
        store.link_contradiction("C1", "E2");
        let claim = store.get_claim("C1").unwrap();
        assert!(claim.has_conflict());
    }

    #[test]
    fn confidence_clamping() {
        let mut claim = Claim::new("C1", "claim", ClaimType::Fact);
        claim.set_confidence(1.5);
        assert_eq!(claim.confidence, 1.0);
        claim.set_confidence(-0.5);
        assert_eq!(claim.confidence, 0.0);
        claim.set_confidence(0.75);
        assert_eq!(claim.confidence, 0.75);
    }

    #[test]
    fn supported_claims_filter() {
        let mut store = EvidenceStore::new();
        store.add_evidence(Evidence::new("E1", "DOC-1", "1.0", "§1", "text"));
        store.add_claim(Claim::new("C1", "supported", ClaimType::Fact));
        store.add_claim(Claim::new("C2", "unsupported", ClaimType::Fact));
        store.link_support("C1", "E1");
        assert_eq!(store.supported_claims().len(), 1);
    }

    #[test]
    fn claim_type_names_unique() {
        let types = [
            ClaimType::Fact,
            ClaimType::RuleDerived,
            ClaimType::EventDerived,
            ClaimType::Statistical,
            ClaimType::Causal,
            ClaimType::Hypothesis,
            ClaimType::Undetermined,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn link_to_nonexistent_evidence_fails() {
        let mut store = EvidenceStore::new();
        store.add_claim(Claim::new("C1", "claim", ClaimType::Fact));
        assert!(!store.link_support("C1", "NONEXISTENT"));
    }
}
