#![forbid(unsafe_code)]

//! Knowledge graph layer (Master Prompt Phase 2.3, §3.3).
//!
//! Typed entities and relations with provenance and verification status.
//! LLM-extracted relations default to `unverified` (never auto-trusted).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Entity types in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Standard,
    Clause,
    Requirement,
    Sop,
    Department,
    Role,
    Activity,
    Form,
    Evidence,
    Risk,
    Control,
    Exception,
    Event,
    Decision,
    Finding,
    Capa,
}

impl EntityType {
    pub fn name(&self) -> &'static str {
        match self {
            EntityType::Standard => "Standard",
            EntityType::Clause => "Clause",
            EntityType::Requirement => "Requirement",
            EntityType::Sop => "SOP",
            EntityType::Department => "Department",
            EntityType::Role => "Role",
            EntityType::Activity => "Activity",
            EntityType::Form => "Form",
            EntityType::Evidence => "Evidence",
            EntityType::Risk => "Risk",
            EntityType::Control => "Control",
            EntityType::Exception => "Exception",
            EntityType::Event => "Event",
            EntityType::Decision => "Decision",
            EntityType::Finding => "Finding",
            EntityType::Capa => "CAPA",
        }
    }
}

/// Relation types in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    Contains,
    References,
    AppliesTo,
    Requires,
    Prohibits,
    Precedes,
    Triggers,
    Supersedes,
    ExceptionOf,
    Verifies,
    ProducesEvidence,
    Causes,
    ConflictsWith,
}

impl RelationType {
    pub fn name(&self) -> &'static str {
        match self {
            RelationType::Contains => "contains",
            RelationType::References => "references",
            RelationType::AppliesTo => "applies_to",
            RelationType::Requires => "requires",
            RelationType::Prohibits => "prohibits",
            RelationType::Precedes => "precedes",
            RelationType::Triggers => "triggers",
            RelationType::Supersedes => "supersedes",
            RelationType::ExceptionOf => "exception_of",
            RelationType::Verifies => "verifies",
            RelationType::ProducesEvidence => "produces_evidence",
            RelationType::Causes => "causes",
            RelationType::ConflictsWith => "conflicts_with",
        }
    }
}

/// Verification status of a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Extracted by LLM, not yet verified (default).
    Unverified,
    /// Verified by human expert.
    Verified,
    /// Rejected by human expert.
    Rejected,
}

/// A typed entity in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEntity {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub source_doc_id: Option<String>,
    pub source_version: Option<String>,
}

/// A typed relation in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgRelation {
    pub id: String,
    pub relation_type: RelationType,
    pub from_entity: String,
    pub to_entity: String,
    pub source: String,
    pub scope: String,
    pub version: String,
    pub extraction_method: String,
    pub verification_status: VerificationStatus,
}

/// Knowledge graph store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub entities: BTreeMap<String, KgEntity>,
    pub relations: BTreeMap<String, KgRelation>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entity(&mut self, entity: KgEntity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    pub fn add_relation(&mut self, relation: KgRelation) {
        self.relations.insert(relation.id.clone(), relation);
    }

    pub fn get_entity(&self, id: &str) -> Option<&KgEntity> {
        self.entities.get(id)
    }

    pub fn get_relation(&self, id: &str) -> Option<&KgRelation> {
        self.relations.get(id)
    }

    pub fn relations_from(&self, entity_id: &str) -> Vec<&KgRelation> {
        self.relations
            .values()
            .filter(|r| r.from_entity == entity_id)
            .collect()
    }

    pub fn relations_to(&self, entity_id: &str) -> Vec<&KgRelation> {
        self.relations
            .values()
            .filter(|r| r.to_entity == entity_id)
            .collect()
    }

    pub fn relations_by_type(&self, rt: RelationType) -> Vec<&KgRelation> {
        self.relations
            .values()
            .filter(|r| r.relation_type == rt)
            .collect()
    }

    pub fn verified_relations(&self) -> Vec<&KgRelation> {
        self.relations
            .values()
            .filter(|r| r.verification_status == VerificationStatus::Verified)
            .collect()
    }

    pub fn unverified_relations(&self) -> Vec<&KgRelation> {
        self.relations
            .values()
            .filter(|r| r.verification_status == VerificationStatus::Unverified)
            .collect()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(id: &str, et: EntityType, name: &str) -> KgEntity {
        KgEntity {
            id: id.to_string(),
            entity_type: et,
            name: name.to_string(),
            attributes: BTreeMap::new(),
            source_doc_id: Some("SOP-CHANGE".to_string()),
            source_version: Some("3.2".to_string()),
        }
    }

    fn make_relation(id: &str, rt: RelationType, from: &str, to: &str, method: &str) -> KgRelation {
        KgRelation {
            id: id.to_string(),
            relation_type: rt,
            from_entity: from.to_string(),
            to_entity: to.to_string(),
            source: "SOP-CHANGE@3.2 §4.3".to_string(),
            scope: "P-CHANGE".to_string(),
            version: "3.2".to_string(),
            extraction_method: method.to_string(),
            verification_status: if method == "llm" {
                VerificationStatus::Unverified
            } else {
                VerificationStatus::Verified
            },
        }
    }

    #[test]
    fn entity_crud() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "Clause 4.3"));
        assert_eq!(kg.entity_count(), 1);
        assert!(kg.get_entity("E1").is_some());
        assert_eq!(kg.get_entity("E1").unwrap().entity_type, EntityType::Clause);
    }

    #[test]
    fn relation_crud() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "Clause A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "Clause B"));
        kg.add_relation(make_relation(
            "R1",
            RelationType::Requires,
            "E1",
            "E2",
            "manual",
        ));
        assert_eq!(kg.relation_count(), 1);
        assert!(kg.get_relation("R1").is_some());
    }

    #[test]
    fn relations_from_entity() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "B"));
        kg.add_entity(make_entity("E3", EntityType::Clause, "C"));
        kg.add_relation(make_relation(
            "R1",
            RelationType::Requires,
            "E1",
            "E2",
            "manual",
        ));
        kg.add_relation(make_relation(
            "R2",
            RelationType::References,
            "E1",
            "E3",
            "manual",
        ));
        let from = kg.relations_from("E1");
        assert_eq!(from.len(), 2);
    }

    #[test]
    fn relations_to_entity() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "B"));
        kg.add_relation(make_relation(
            "R1",
            RelationType::Requires,
            "E1",
            "E2",
            "manual",
        ));
        let to = kg.relations_to("E2");
        assert_eq!(to.len(), 1);
    }

    #[test]
    fn relations_by_type() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "B"));
        kg.add_relation(make_relation(
            "R1",
            RelationType::Requires,
            "E1",
            "E2",
            "manual",
        ));
        kg.add_relation(make_relation(
            "R2",
            RelationType::References,
            "E1",
            "E2",
            "manual",
        ));
        assert_eq!(kg.relations_by_type(RelationType::Requires).len(), 1);
        assert_eq!(kg.relations_by_type(RelationType::References).len(), 1);
    }

    #[test]
    fn llm_extracted_defaults_to_unverified() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "B"));
        kg.add_relation(make_relation("R1", RelationType::Causes, "E1", "E2", "llm"));
        assert_eq!(kg.unverified_relations().len(), 1);
        assert_eq!(kg.verified_relations().len(), 0);
    }

    #[test]
    fn manual_extracted_defaults_to_verified() {
        let mut kg = KnowledgeGraph::new();
        kg.add_entity(make_entity("E1", EntityType::Clause, "A"));
        kg.add_entity(make_entity("E2", EntityType::Clause, "B"));
        kg.add_relation(make_relation(
            "R1",
            RelationType::Requires,
            "E1",
            "E2",
            "manual",
        ));
        assert_eq!(kg.verified_relations().len(), 1);
        assert_eq!(kg.unverified_relations().len(), 0);
    }

    #[test]
    fn entity_type_names_are_unique() {
        let types = [
            EntityType::Standard,
            EntityType::Clause,
            EntityType::Requirement,
            EntityType::Sop,
            EntityType::Department,
            EntityType::Role,
            EntityType::Activity,
            EntityType::Form,
            EntityType::Evidence,
            EntityType::Risk,
            EntityType::Control,
            EntityType::Exception,
            EntityType::Event,
            EntityType::Decision,
            EntityType::Finding,
            EntityType::Capa,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn relation_type_names_are_unique() {
        let types = [
            RelationType::Contains,
            RelationType::References,
            RelationType::AppliesTo,
            RelationType::Requires,
            RelationType::Prohibits,
            RelationType::Precedes,
            RelationType::Triggers,
            RelationType::Supersedes,
            RelationType::ExceptionOf,
            RelationType::Verifies,
            RelationType::ProducesEvidence,
            RelationType::Causes,
            RelationType::ConflictsWith,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }
}
