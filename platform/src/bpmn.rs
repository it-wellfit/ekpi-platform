#![forbid(unsafe_code)]

//! BPMN 2.0 parser (Master Prompt Phase 4a).
//!
//! Parses BPMN XML into ProcessModel. Extracts tasks, gateways, events, sequence flows.

use serde::{Deserialize, Serialize};

/// BPMN element types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpmnElementType {
    Task,
    ExclusiveGateway,
    ParallelGateway,
    InclusiveGateway,
    StartEvent,
    EndEvent,
    IntermediateEvent,
    SubProcess,
    SequenceFlow,
}

impl BpmnElementType {
    pub fn name(&self) -> &'static str {
        match self {
            BpmnElementType::Task => "task",
            BpmnElementType::ExclusiveGateway => "exclusiveGateway",
            BpmnElementType::ParallelGateway => "parallelGateway",
            BpmnElementType::InclusiveGateway => "inclusiveGateway",
            BpmnElementType::StartEvent => "startEvent",
            BpmnElementType::EndEvent => "endEvent",
            BpmnElementType::IntermediateEvent => "intermediateEvent",
            BpmnElementType::SubProcess => "subProcess",
            BpmnElementType::SequenceFlow => "sequenceFlow",
        }
    }
}

/// A BPMN element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnElement {
    pub id: String,
    pub element_type: BpmnElementType,
    pub name: String,
    pub attributes: std::collections::BTreeMap<String, String>,
}

/// A BPMN sequence flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnSequenceFlow {
    pub id: String,
    pub source_ref: String,
    pub target_ref: String,
    pub condition: Option<String>,
}

/// Parsed BPMN model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BpmnModel {
    pub id: String,
    pub name: String,
    pub elements: Vec<BpmnElement>,
    pub flows: Vec<BpmnSequenceFlow>,
}

impl BpmnModel {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            elements: Vec::new(),
            flows: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: BpmnElement) {
        self.elements.push(element);
    }

    pub fn add_flow(&mut self, flow: BpmnSequenceFlow) {
        self.flows.push(flow);
    }

    pub fn tasks(&self) -> Vec<&BpmnElement> {
        self.elements
            .iter()
            .filter(|e| e.element_type == BpmnElementType::Task)
            .collect()
    }

    pub fn gateways(&self) -> Vec<&BpmnElement> {
        self.elements
            .iter()
            .filter(|e| {
                matches!(
                    e.element_type,
                    BpmnElementType::ExclusiveGateway
                        | BpmnElementType::ParallelGateway
                        | BpmnElementType::InclusiveGateway
                )
            })
            .collect()
    }

    pub fn start_events(&self) -> Vec<&BpmnElement> {
        self.elements
            .iter()
            .filter(|e| e.element_type == BpmnElementType::StartEvent)
            .collect()
    }

    pub fn end_events(&self) -> Vec<&BpmnElement> {
        self.elements
            .iter()
            .filter(|e| e.element_type == BpmnElementType::EndEvent)
            .collect()
    }

    pub fn flows_from(&self, source_id: &str) -> Vec<&BpmnSequenceFlow> {
        self.flows
            .iter()
            .filter(|f| f.source_ref == source_id)
            .collect()
    }

    pub fn flows_to(&self, target_id: &str) -> Vec<&BpmnSequenceFlow> {
        self.flows
            .iter()
            .filter(|f| f.target_ref == target_id)
            .collect()
    }

    /// Parse BPMN XML (simplified: extracts id, name, type from tags).
    pub fn parse(xml: &str) -> Result<Self, String> {
        let mut model = BpmnModel::default();
        // Extract process id and name
        if let Some(start) = xml.find("<bpmn:process") {
            if let Some(id_start) = xml[start..].find("id=\"") {
                let id_val_start = start + id_start + 4;
                if let Some(id_end) = xml[id_val_start..].find('"') {
                    model.id = xml[id_val_start..id_val_start + id_end].to_string();
                }
            }
            if let Some(name_start) = xml[start..].find("name=\"") {
                let name_val_start = start + name_start + 6;
                if let Some(name_end) = xml[name_val_start..].find('"') {
                    model.name = xml[name_val_start..name_val_start + name_end].to_string();
                }
            }
        }

        // Extract elements by tag type
        let tag_types = [
            ("bpmn:task", BpmnElementType::Task),
            ("bpmn:exclusiveGateway", BpmnElementType::ExclusiveGateway),
            ("bpmn:parallelGateway", BpmnElementType::ParallelGateway),
            ("bpmn:inclusiveGateway", BpmnElementType::InclusiveGateway),
            ("bpmn:startEvent", BpmnElementType::StartEvent),
            ("bpmn:endEvent", BpmnElementType::EndEvent),
            (
                "bpmn:intermediateThrowEvent",
                BpmnElementType::IntermediateEvent,
            ),
            ("bpmn:subProcess", BpmnElementType::SubProcess),
        ];

        for (tag, etype) in &tag_types {
            let mut search_pos = 0;
            while let Some(pos) = xml[search_pos..].find(&format!("<{}", tag)) {
                let abs_pos = search_pos + pos;
                // Extract id
                let id = extract_attr(xml, abs_pos, "id");
                let name = extract_attr(xml, abs_pos, "name").unwrap_or_default();
                if let Some(id) = id {
                    model.add_element(BpmnElement {
                        id,
                        element_type: *etype,
                        name,
                        attributes: std::collections::BTreeMap::new(),
                    });
                }
                search_pos = abs_pos + tag.len() + 1;
            }
        }

        // Extract sequence flows
        let mut search_pos = 0;
        while let Some(pos) = xml[search_pos..].find("<bpmn:sequenceFlow") {
            let abs_pos = search_pos + pos;
            let id = extract_attr(xml, abs_pos, "id").unwrap_or_default();
            let source = extract_attr(xml, abs_pos, "sourceRef").unwrap_or_default();
            let target = extract_attr(xml, abs_pos, "targetRef").unwrap_or_default();
            if !source.is_empty() && !target.is_empty() {
                model.add_flow(BpmnSequenceFlow {
                    id,
                    source_ref: source,
                    target_ref: target,
                    condition: None,
                });
            }
            search_pos = abs_pos + 20;
        }

        Ok(model)
    }
}

fn extract_attr(xml: &str, pos: usize, attr: &str) -> Option<String> {
    let search = format!("{}=\"", attr);
    if let Some(attr_pos) = xml[pos..].find(&search) {
        let val_start = pos + attr_pos + search.len();
        if let Some(val_end) = xml[val_start..].find('"') {
            return Some(xml[val_start..val_start + val_end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_bpmn() {
        let xml = r#"<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
            <bpmn:process id="P-001" name="Test Process">
                <bpmn:startEvent id="start1" name="Start"/>
                <bpmn:task id="task1" name="Do Something"/>
                <bpmn:endEvent id="end1" name="End"/>
                <bpmn:sequenceFlow id="f1" sourceRef="start1" targetRef="task1"/>
                <bpmn:sequenceFlow id="f2" sourceRef="task1" targetRef="end1"/>
            </bpmn:process>
        </bpmn:definitions>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.id, "P-001");
        assert_eq!(model.name, "Test Process");
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(model.start_events().len(), 1);
        assert_eq!(model.end_events().len(), 1);
        assert_eq!(model.flows.len(), 2);
    }

    #[test]
    fn parse_gateway() {
        let xml = r#"<bpmn:process id="P-002" name="Gateway Process">
            <bpmn:task id="t1" name="Task 1"/>
            <bpmn:exclusiveGateway id="gw1" name="Decision"/>
            <bpmn:task id="t2" name="Task 2"/>
            <bpmn:sequenceFlow id="f1" sourceRef="t1" targetRef="gw1"/>
            <bpmn:sequenceFlow id="f2" sourceRef="gw1" targetRef="t2"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.gateways().len(), 1);
        assert_eq!(model.flows_from("gw1").len(), 1);
    }

    #[test]
    fn flows_to_works() {
        let xml = r#"<bpmn:process id="P-003" name="Flow Test">
            <bpmn:task id="t1" name="Task 1"/>
            <bpmn:task id="t2" name="Task 2"/>
            <bpmn:sequenceFlow id="f1" sourceRef="t1" targetRef="t2"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.flows_to("t2").len(), 1);
    }

    #[test]
    fn empty_process() {
        let xml = r#"<bpmn:process id="P-004" name="Empty"/>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.id, "P-004");
        assert!(model.elements.is_empty());
    }

    #[test]
    fn bpmn_element_type_names_unique() {
        let types = [
            BpmnElementType::Task,
            BpmnElementType::ExclusiveGateway,
            BpmnElementType::ParallelGateway,
            BpmnElementType::InclusiveGateway,
            BpmnElementType::StartEvent,
            BpmnElementType::EndEvent,
            BpmnElementType::IntermediateEvent,
            BpmnElementType::SubProcess,
            BpmnElementType::SequenceFlow,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }
}
