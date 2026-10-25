#![forbid(unsafe_code)]

//! Multi-modal document understanding (Master Prompt Phase 1, real implementation).
//!
//! Supports text, image (OCR), and table modalities with extraction and
//! serialization.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModalityKind {
    Text,
    Image,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modality {
    pub kind: ModalityKind,
    pub content: String,
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiModalDocument {
    pub modalities: Vec<Modality>,
}

impl MultiModalDocument {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_text(&mut self, content: &str) {
        self.modalities.push(Modality {
            kind: ModalityKind::Text,
            content: content.to_string(),
            metadata: std::collections::BTreeMap::new(),
        });
    }

    pub fn add_image(&mut self, description: &str, ocr_text: &str) {
        let mut meta = std::collections::BTreeMap::new();
        meta.insert("ocr_text".to_string(), ocr_text.to_string());
        self.modalities.push(Modality {
            kind: ModalityKind::Image,
            content: description.to_string(),
            metadata: meta,
        });
    }

    pub fn add_table(&mut self, csv: &str) {
        self.modalities.push(Modality {
            kind: ModalityKind::Table,
            content: csv.to_string(),
            metadata: std::collections::BTreeMap::new(),
        });
    }

    pub fn text_content(&self) -> String {
        self.modalities
            .iter()
            .filter(|m| m.kind == ModalityKind::Text)
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn image_ocr_texts(&self) -> Vec<String> {
        self.modalities
            .iter()
            .filter(|m| m.kind == ModalityKind::Image)
            .filter_map(|m| m.metadata.get("ocr_text").cloned())
            .collect()
    }

    pub fn table_count(&self) -> usize {
        self.modalities
            .iter()
            .filter(|m| m.kind == ModalityKind::Table)
            .count()
    }

    /// Parse a CSV table into rows.
    pub fn parse_csv_table(csv: &str) -> Vec<Vec<String>> {
        csv.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_retrieve() {
        let mut doc = MultiModalDocument::new();
        doc.add_text("hello");
        doc.add_image("img1", "ocr result");
        assert_eq!(doc.modalities.len(), 2);
        assert_eq!(doc.text_content(), "hello");
        assert_eq!(doc.image_ocr_texts(), vec!["ocr result"]);
    }

    #[test]
    fn table_parsing() {
        let mut doc = MultiModalDocument::new();
        doc.add_table("a,b,c\n1,2,3");
        assert_eq!(doc.table_count(), 1);
        let rows = MultiModalDocument::parse_csv_table("a,b,c\n1,2,3");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["a", "b", "c"]);
    }
}
