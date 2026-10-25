#![forbid(unsafe_code)]

//! R2-6嚗ecision snapshot manifest + ?銝?湔改?蝖祇?瑼?H-6嚗?//!
//! manifest ??蝬?嚗摰撓?伐?獢辣鈭辣??????隞嗥?摰摰寥?皝??具摰?
//! ?????蝣箏??扯楝敺?閬?撘? + token replay + 瑼Ｙ揣嚗?
//! **銝??思遙雿??芋??*嚗ddendum A6嚗LM ??????瘙綽???
use serde::Serialize;

use crate::docs::sha256_hex;
use crate::engine::EngineOutcome;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct DecisionSnapshot {
    pub case_id: String,
    pub verdict: String,
    pub citations: Vec<String>,
    pub rule_ids_used: Vec<String>,
    /// doc_binding_shas field
    pub doc_binding_shas: Vec<String>,
    pub rule_ids_shas: Vec<String>,
    pub event_sequence_sha: String,
    pub manifest_sha: String,
}

impl DecisionSnapshot {
    /// build snapshot
    pub fn build(
        case_id: &str,
        outcome: &EngineOutcome,
        doc_binding_shas: Vec<String>,
        rule_ids_shas: Vec<String>,
        event_sequence_sha: String,
    ) -> DecisionSnapshot {
        let mut snap = DecisionSnapshot {
            case_id: case_id.to_string(),
            verdict: outcome.verdict.as_str().to_string(),
            citations: outcome.citations.clone(),
            rule_ids_used: outcome.rule_ids_used.clone(),
            doc_binding_shas,
            rule_ids_shas,
            event_sequence_sha,
            manifest_sha: String::new(),
        };
        snap.manifest_sha = snap.manifest_hash();
        snap
    }

    /// manifest_hash helper
    pub fn manifest_hash(&self) -> String {
        let payload = ManifestPayload {
            case_id: self.case_id.clone(),
            verdict: self.verdict.clone(),
            citations: self.citations.clone(),
            rule_ids_used: self.rule_ids_used.clone(),
            doc_binding_shas: self.doc_binding_shas.clone(),
            event_sequence_sha: self.event_sequence_sha.clone(),
        };
        let canonical =
            serde_json::to_string(&payload).unwrap_or_else(|_| "unserializable".to_string());
        sha256_hex(canonical.as_bytes())
    }

    pub fn replay_consistent(&self) -> bool {
        self.manifest_hash() == self.manifest_sha
    }
}

#[derive(Serialize)]
struct ManifestPayload {
    case_id: String,
    verdict: String,
    citations: Vec<String>,
    rule_ids_used: Vec<String>,
    doc_binding_shas: Vec<String>,
    event_sequence_sha: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineOutcome;
    use ekp_eval::types::Ver;

    fn dummy_outcome() -> EngineOutcome {
        EngineOutcome {
            verdict: Ver::Comply,
            citations: vec!["doc@1 §1".to_string()],
            rule_ids_used: vec!["RULE-1".to_string()],
            notes: Vec::new(),
            causal_results: Vec::new(),
        }
    }

    #[test]
    fn build_sets_manifest() {
        let o = dummy_outcome();
        let s = DecisionSnapshot::build(
            "C1",
            &o,
            vec!["d@1".to_string()],
            vec!["r@1".to_string()],
            "sha".to_string(),
        );
        assert_eq!(s.manifest_sha.len(), 64);
        assert!(s.replay_consistent());
    }

    #[test]
    fn stable_across_rebuilds() {
        let o = dummy_outcome();
        let s1 = DecisionSnapshot::build(
            "C1",
            &o,
            vec!["d@1".to_string()],
            vec!["r@1".to_string()],
            "sha".to_string(),
        );
        let s2 = DecisionSnapshot::build(
            "C1",
            &o,
            vec!["d@1".to_string()],
            vec!["r@1".to_string()],
            "sha".to_string(),
        );
        assert_eq!(s1.manifest_sha, s2.manifest_sha);
    }
}
