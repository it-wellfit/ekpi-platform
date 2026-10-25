#![forbid(unsafe_code)]

//! Failure case database (Master Prompt Phase 3).

#[derive(Debug, Clone)]
pub struct FailureCase {
    pub case_id: String,
    pub reason: String,
}

#[derive(Default)]
pub struct FailureCaseDb {
    pub cases: Vec<FailureCase>,
}

impl FailureCaseDb {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, case_id: &str, reason: &str) {
        self.cases.push(FailureCase {
            case_id: case_id.to_string(),
            reason: reason.to_string(),
        });
    }
    pub fn len(&self) -> usize {
        self.cases.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_failure() {
        let mut db = FailureCaseDb::new();
        db.add("C1", "wrong verdict");
        assert_eq!(db.len(), 1);
    }
}
