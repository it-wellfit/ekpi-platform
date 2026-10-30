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
        // Extract process id and name from the process start tag only. The
        // local name is matched so both `bpmn:process` and a default-namespace
        // `<process>` are accepted.
        if let Some(process_pos) = find_tag(xml, "process", 0) {
            if let Some(id) = extract_attr(xml, process_pos, "id") {
                model.id = id;
            }
            if let Some(name) = extract_attr(xml, process_pos, "name") {
                model.name = name;
            }
        }

        // Extract elements by local tag name. A longer local name such as
        // `taskSequence` must not be parsed as `task`, and an arbitrary
        // namespace prefix is accepted.
        let tag_types = [
            ("task", BpmnElementType::Task),
            ("exclusiveGateway", BpmnElementType::ExclusiveGateway),
            ("parallelGateway", BpmnElementType::ParallelGateway),
            ("inclusiveGateway", BpmnElementType::InclusiveGateway),
            ("startEvent", BpmnElementType::StartEvent),
            ("endEvent", BpmnElementType::EndEvent),
            ("intermediateThrowEvent", BpmnElementType::IntermediateEvent),
            ("subProcess", BpmnElementType::SubProcess),
        ];

        for (tag, etype) in &tag_types {
            let mut search_pos = 0;
            while let Some(abs_pos) = find_tag(xml, tag, search_pos) {
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
                search_pos = skip_tag(xml, abs_pos);
            }
        }

        // Extract sequence flows
        let mut search_pos = 0;
        while let Some(abs_pos) = find_tag(xml, "sequenceFlow", search_pos) {
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
            search_pos = skip_tag(xml, abs_pos);
        }

        Ok(model)
    }
}

/// Find the next start tag whose local name equals `local_name`.
///
/// The namespace prefix is intentionally ignored: both `<bpmn:process>` and a
/// default-namespace `<process>` are valid BPMN. Comments, CDATA sections and
/// processing instructions are skipped rather than scanned for element names.
fn find_tag(xml: &str, local_name: &str, from: usize) -> Option<usize> {
    let mut search_pos = from;
    while let Some(offset) = xml[search_pos..].find('<') {
        let start = search_pos + offset;
        let rest = &xml[start..];
        if rest.starts_with("<!--") {
            search_pos = start + rest.find("-->")? + 3;
            continue;
        }
        if rest.starts_with("<![CDATA[") {
            search_pos = start + rest.find("]]>")? + 3;
            continue;
        }
        if rest.starts_with("<?") {
            search_pos = start + rest.find("?>")? + 2;
            continue;
        }
        let next = xml.as_bytes().get(start + 1).copied();
        if matches!(next, Some(b'/') | Some(b'!') | Some(b'?')) {
            search_pos = skip_tag(xml, start);
            continue;
        }

        let name_start = start + 1;
        let bytes = xml.as_bytes();
        let mut name_end = name_start;
        while name_end < bytes.len() {
            let byte = bytes[name_end];
            if byte.is_ascii_whitespace() || byte == b'/' || byte == b'>' {
                break;
            }
            name_end += 1;
        }
        if name_end == name_start {
            search_pos = skip_tag(xml, start);
            continue;
        }
        let qualified_name = &xml[name_start..name_end];
        let local = qualified_name.rsplit(':').next().unwrap_or(qualified_name);
        if local == local_name {
            return Some(start);
        }
        search_pos = skip_tag(xml, start);
    }
    None
}

fn skip_tag(xml: &str, start: usize) -> usize {
    tag_end(xml, start).map(|end| end + 1).unwrap_or(start + 1)
}

/// Return the byte offset of the `>` that closes the tag starting at `pos`,
/// ignoring `>` characters inside quoted attribute values.
fn tag_end(xml: &str, pos: usize) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (offset, character) in xml[pos..].char_indices() {
        match (quote, character) {
            (Some(open), current) if current == open => quote = None,
            (None, '"') | (None, '\'') => quote = Some(character),
            (None, '>') => return Some(pos + offset),
            _ => {}
        }
    }
    None
}

/// Read an attribute from the single start tag beginning at `pos`.
///
/// The scanner walks attribute names and respects both XML quote styles, so
/// `id="x"`, `id='x'`, and whitespace around `=` are accepted. It never
/// searches inside another attribute value, so text such as `name="id='fake'"`
/// cannot spoof a missing attribute.
fn extract_attr(xml: &str, pos: usize, attr: &str) -> Option<String> {
    let end = tag_end(xml, pos)?;
    let tag = &xml[pos..end];
    let bytes = tag.as_bytes();
    let mut index = 1;
    while index < bytes.len()
        && !bytes[index].is_ascii_whitespace()
        && bytes[index] != b'/'
        && bytes[index] != b'>'
    {
        index += 1;
    }

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] == b'/' || bytes[index] == b'>' {
            break;
        }

        let name_start = index;
        while index < bytes.len()
            && !bytes[index].is_ascii_whitespace()
            && bytes[index] != b'='
            && bytes[index] != b'/'
            && bytes[index] != b'>'
        {
            index += 1;
        }
        let name = &tag[name_start..index];

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let quote = bytes[index];
        if quote != b'"' && quote != b'\'' {
            break;
        }
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        if name == attr {
            return Some(tag[value_start..index].to_string());
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_does_not_panic_on_multibyte_text_after_a_tag_name() {
        // A malformed document must return a model or an error, never panic
        // while advancing the scan position.
        let _ = BpmnModel::parse("<bpmn:task中");
        let _ = BpmnModel::parse("<bpmn:sequenceFlow中");
    }

    #[test]
    fn attributes_do_not_leak_from_a_later_tag() {
        let xml = r#"<bpmn:process id="P-005" name="Attribute isolation">
            <bpmn:task name="An unidentified task"/>
            <bpmn:task id="task2" name="Second"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(
            model.tasks().len(),
            1,
            "a tag without an id must not borrow the next tag's id"
        );
        assert_eq!(model.tasks()[0].id, "task2");
    }

    #[test]
    fn tag_prefix_requires_a_delimiter() {
        let xml = r#"<bpmn:process id="P-006" name="Prefix boundary">
            <bpmn:taskSequence id="not-a-task"/>
            <bpmn:task id="real-task"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(
            model.tasks().len(),
            1,
            "a longer tag name must not be parsed as bpmn:task"
        );
        assert_eq!(model.tasks()[0].id, "real-task");
    }

    #[test]
    fn process_metadata_does_not_leak_from_a_later_tag() {
        let xml = r#"<bpmn:process name="No process id">
            <bpmn:task id="task-id" name="Task"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(
            model.id, "",
            "a missing process id must not be filled from a child tag"
        );
    }

    #[test]
    fn attribute_names_require_a_delimiter() {
        let xml = r#"<bpmn:process id="P-007" name="Attribute boundary">
            <bpmn:task data-id="decoy" id="real-id"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(
            model.tasks()[0].id,
            "real-id",
            "a longer attribute name must not be parsed as id"
        );
    }

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
    fn parses_single_quoted_attributes() {
        let xml = r#"<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
            <bpmn:process id='P-008' name='Single quote process'>
                <bpmn:task id='t1' name='Do it'/>
                <bpmn:sequenceFlow id='f1' sourceRef='t1' targetRef='t2'/>
            </bpmn:process>
        </bpmn:definitions>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.id, "P-008");
        assert_eq!(model.name, "Single quote process");
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(model.tasks()[0].id, "t1");
        assert_eq!(model.flows.len(), 1);
        assert_eq!(model.flows[0].source_ref, "t1");
        assert_eq!(model.flows[0].target_ref, "t2");
    }

    #[test]
    fn parses_mixed_quotes_and_whitespace_around_equals() {
        let xml = r#"<bpmn:process id = 'P-009' name="Mixed 'quotes'">
            <bpmn:task id= 't1' name = "Task with 'single' text"/>
            <bpmn:task id="t2" name='Task with "double" text'/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.id, "P-009");
        assert_eq!(model.name, "Mixed 'quotes'");
        assert_eq!(model.tasks().len(), 2);
        assert_eq!(model.tasks()[0].id, "t1");
        assert_eq!(model.tasks()[0].name, "Task with 'single' text");
        assert_eq!(model.tasks()[1].id, "t2");
        assert_eq!(model.tasks()[1].name, "Task with \"double\" text");
    }

    #[test]
    fn attribute_scanner_does_not_match_text_inside_another_value() {
        let xml = r#"<bpmn:process id="P-010" name="id='fake'">
            <bpmn:task name="id='decoy'" id='real'/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.id, "P-010");
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(model.tasks()[0].id, "real");
        assert_eq!(model.tasks()[0].name, "id='decoy'");
    }

    #[test]
    fn parses_default_namespace_and_arbitrary_prefixes() {
        let default_namespace = r#"<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL">
            <process id="P-011" name="Default namespace">
                <task id="t1" name="Task"/>
                <sequenceFlow id="f1" sourceRef="t1" targetRef="t1"/>
            </process>
        </definitions>"#;
        let model = BpmnModel::parse(default_namespace).unwrap();
        assert_eq!(model.id, "P-011");
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(model.tasks()[0].id, "t1");
        assert_eq!(model.flows.len(), 1);

        let custom_prefix = r#"<bpmn2:process id='P-012' name='Custom prefix'>
            <bpmn2:startEvent id='s1'/>
        </bpmn2:process>"#;
        let model = BpmnModel::parse(custom_prefix).unwrap();
        assert_eq!(model.id, "P-012");
        assert_eq!(model.start_events().len(), 1);
        assert_eq!(model.start_events()[0].id, "s1");
    }

    #[test]
    fn comments_and_cdata_do_not_create_elements() {
        let xml = r#"<bpmn:process id="P-013" name="Comment safe">
            <!-- <bpmn:task id="comment-decoy"/> -->
            <![CDATA[<bpmn:task id="cdata-decoy"/>]]>
            <bpmn:task id="real"/>
        </bpmn:process>"#;
        let model = BpmnModel::parse(xml).unwrap();
        assert_eq!(model.tasks().len(), 1);
        assert_eq!(model.tasks()[0].id, "real");
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
