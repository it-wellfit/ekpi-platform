#![forbid(unsafe_code)]

//! OCR and layout parsing module (Master Prompt Phase 1).
//!
//! Trait-based OCR with a stub engine (no external binary required) and a
//! layout grouper for reading-order reconstruction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl BBox {
    pub fn area(&self) -> f64 {
        self.w * self.h
    }
    pub fn top(&self) -> f64 {
        self.y
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn left(&self) -> f64 {
        self.x
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub bbox: BBox,
    pub confidence: f64,
}

pub trait OcrEngine {
    fn recognize(&self, image_bytes: &[u8]) -> Result<Vec<OcrResult>, String>;
    fn name(&self) -> &str {
        "unknown"
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StubOcrEngine;

impl OcrEngine for StubOcrEngine {
    fn recognize(&self, _image_bytes: &[u8]) -> Result<Vec<OcrResult>, String> {
        Ok(Vec::new())
    }
    fn name(&self) -> &str {
        "stub"
    }
}

pub fn mean_confidence(results: &[OcrResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    results.iter().map(|r| r.confidence).sum::<f64>() / results.len() as f64
}

/// Group OCR results into reading order (top-to-bottom, left-to-right).
/// Uses a simple line-grouping heuristic: results within `line_tol` vertical
/// pixels of each other are grouped into the same line.
pub fn reading_order(results: &[OcrResult], line_tol: f64) -> Vec<OcrResult> {
    let mut sorted = results.to_vec();
    // Sort by top, then left.
    sorted.sort_by(|a, b| {
        let ta = a.bbox.top();
        let tb = b.bbox.top();
        if (ta - tb).abs() < line_tol {
            a.bbox
                .left()
                .partial_cmp(&b.bbox.left())
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_empty() {
        let engine = StubOcrEngine;
        let res = engine.recognize(b"fake").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn mean_confidence_empty() {
        assert_eq!(mean_confidence(&[]), 0.0);
    }

    #[test]
    fn mean_confidence_values() {
        let results = vec![
            OcrResult {
                text: "a".into(),
                bbox: BBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                confidence: 0.8,
            },
            OcrResult {
                text: "b".into(),
                bbox: BBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                confidence: 0.6,
            },
        ];
        assert!((mean_confidence(&results) - 0.7).abs() < 1e-9);
    }

    #[test]
    fn reading_order_sorts() {
        let results = vec![
            OcrResult {
                text: "bottom".into(),
                bbox: BBox {
                    x: 0.0,
                    y: 100.0,
                    w: 1.0,
                    h: 1.0,
                },
                confidence: 1.0,
            },
            OcrResult {
                text: "top".into(),
                bbox: BBox {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                confidence: 1.0,
            },
        ];
        let ordered = reading_order(&results, 5.0);
        assert_eq!(ordered[0].text, "top");
        assert_eq!(ordered[1].text, "bottom");
    }
}
