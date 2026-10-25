#![forbid(unsafe_code)]

//! Audit event log (Master Prompt Phase 9c).
//!
//! Append-only event log with SHA-256 hash chain.
//! Event types: document_import, rule_change, case_decision, process_conformance, evidence_review.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Audit event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    DocumentImport,
    RuleChange,
    CaseDecision,
    ProcessConformance,
    EvidenceReview,
    KnowledgeSearch,
}

impl AuditEventType {
    pub fn name(&self) -> &'static str {
        match self {
            AuditEventType::DocumentImport => "document_import",
            AuditEventType::RuleChange => "rule_change",
            AuditEventType::CaseDecision => "case_decision",
            AuditEventType::ProcessConformance => "process_conformance",
            AuditEventType::EvidenceReview => "evidence_review",
            AuditEventType::KnowledgeSearch => "knowledge_search",
        }
    }
}

/// An audit event with hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub actor: String,
    pub tenant_id: String,
    pub payload: String,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: String,
}

impl AuditEvent {
    pub fn new(
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
        prev_hash: &str,
    ) -> Self {
        let hash = Self::compute_hash(id, event_type, actor, tenant_id, payload, prev_hash);
        AuditEvent {
            id: id.to_string(),
            event_type,
            actor: actor.to_string(),
            tenant_id: tenant_id.to_string(),
            payload: payload.to_string(),
            prev_hash: prev_hash.to_string(),
            hash,
            timestamp: String::new(),
        }
    }

    fn compute_hash(
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(event_type.name().as_bytes());
        hasher.update(actor.as_bytes());
        hasher.update(tenant_id.as_bytes());
        hasher.update(payload.as_bytes());
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Audit log with hash chain integrity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(
        &mut self,
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
    ) {
        let prev_hash = self
            .events
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".to_string());
        let event = AuditEvent::new(id, event_type, actor, tenant_id, payload, &prev_hash);
        self.events.push(event);
    }

    pub fn verify_chain(&self) -> bool {
        if self.events.is_empty() {
            return true;
        }
        if self.events[0].prev_hash != "GENESIS" {
            return false;
        }
        for i in 1..self.events.len() {
            if self.events[i].prev_hash != self.events[i - 1].hash {
                return false;
            }
            let expected = AuditEvent::compute_hash(
                &self.events[i].id,
                self.events[i].event_type,
                &self.events[i].actor,
                &self.events[i].tenant_id,
                &self.events[i].payload,
                &self.events[i].prev_hash,
            );
            if expected != self.events[i].hash {
                return false;
            }
        }
        // Verify first entry
        let expected = AuditEvent::compute_hash(
            &self.events[0].id,
            self.events[0].event_type,
            &self.events[0].actor,
            &self.events[0].tenant_id,
            &self.events[0].payload,
            &self.events[0].prev_hash,
        );
        expected == self.events[0].hash
    }

    pub fn detect_tampering(&self) -> Vec<usize> {
        let mut tampered = Vec::new();
        for i in 0..self.events.len() {
            let prev_hash = if i == 0 {
                "GENESIS".to_string()
            } else {
                self.events[i - 1].hash.clone()
            };
            let expected = AuditEvent::compute_hash(
                &self.events[i].id,
                self.events[i].event_type,
                &self.events[i].actor,
                &self.events[i].tenant_id,
                &self.events[i].payload,
                &prev_hash,
            );
            if expected != self.events[i].hash || self.events[i].prev_hash != prev_hash {
                tampered.push(i);
            }
        }
        tampered
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events_by_type(&self, et: AuditEventType) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.event_type == et).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_verify() {
        let mut log = AuditLog::new();
        log.append(
            "A1",
            AuditEventType::DocumentImport,
            "user1",
            "T1",
            "doc imported",
        );
        log.append(
            "A2",
            AuditEventType::CaseDecision,
            "user1",
            "T1",
            "case decided",
        );
        assert!(log.verify_chain());
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn empty_log_verifies() {
        let log = AuditLog::new();
        assert!(log.verify_chain());
        assert!(log.is_empty());
    }

    #[test]
    fn tampering_detected() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "data");
        log.append("A2", AuditEventType::CaseDecision, "user1", "T1", "data2");
        log.events[1].payload = "tampered".to_string();
        assert!(!log.verify_chain());
        let tampered = log.detect_tampering();
        assert!(tampered.contains(&1));
    }

    #[test]
    fn events_by_type_filter() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "doc");
        log.append("A2", AuditEventType::CaseDecision, "user1", "T1", "case");
        log.append("A3", AuditEventType::DocumentImport, "user2", "T1", "doc2");
        assert_eq!(log.events_by_type(AuditEventType::DocumentImport).len(), 2);
        assert_eq!(log.events_by_type(AuditEventType::CaseDecision).len(), 1);
    }

    #[test]
    fn audit_event_type_names_unique() {
        let types = [
            AuditEventType::DocumentImport,
            AuditEventType::RuleChange,
            AuditEventType::CaseDecision,
            AuditEventType::ProcessConformance,
            AuditEventType::EvidenceReview,
            AuditEventType::KnowledgeSearch,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn genesis_prev_hash() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "data");
        assert_eq!(log.events[0].prev_hash, "GENESIS");
    }
}
