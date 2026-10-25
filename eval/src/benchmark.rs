#![forbid(unsafe_code)]

//! Benchmark runner with metrics computation (Master Prompt Phase 3).

use crate::types::Case;

pub struct BenchmarkRunner {
    pub cases: Vec<Case>,
}

impl BenchmarkRunner {
    pub fn new(cases: Vec<Case>) -> Self {
        Self { cases }
    }
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_runner() {
        let r = BenchmarkRunner::new(Vec::new());
        assert_eq!(r.case_count(), 0);
    }
}
