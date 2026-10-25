#![forbid(unsafe_code)]

//! Benchmark framework B0-B9 (Master Prompt Phase 3).

#[derive(Debug, Clone)]
pub struct Baseline {
    pub id: String,
    pub name: String,
}

pub const BASELINES: [&str; 10] = ["B0", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9"];

pub fn baseline_list() -> Vec<Baseline> {
    BASELINES
        .iter()
        .map(|b| Baseline {
            id: b.to_string(),
            name: b.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_baselines() {
        assert_eq!(baseline_list().len(), 10);
    }
}
