#![forbid(unsafe_code)]

//! Runtime verification monitor (Master Prompt Phase 2, real implementation).
//!
//! Evaluates invariants against observed event streams and returns a verdict
//! with an optional trace. Invariant types:
//! - `Always(expr)`: expr holds at every position
//! - `Never(expr)`: expr never holds
//! - `Eventually(expr)`: expr holds at some position
//! - `Next(expr)`: expr holds at the next position (when exists)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorVerdict {
    Ok,
    Violation {
        invariant: String,
        trace: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Invariant {
    Always(String),
    Never(String),
    Eventually(String),
    Next(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub activity: String,
    pub timestamp: String,
    pub actor: String,
}

pub struct RuntimeMonitor {
    pub invariants: Vec<Invariant>,
}

impl RuntimeMonitor {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
        }
    }

    pub fn add_invariant(&mut self, inv: Invariant) {
        self.invariants.push(inv);
    }

    /// Evaluate all invariants against the event stream.
    pub fn evaluate(&self, events: &[MonitorEvent]) -> MonitorVerdict {
        for inv in &self.invariants {
            if let Some(violation) = check_invariant(inv, events) {
                return MonitorVerdict::Violation {
                    invariant: format!("{:?}", inv),
                    trace: violation,
                };
            }
        }
        MonitorVerdict::Ok
    }
}

impl Default for RuntimeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns Some(trace) if the invariant is violated.
fn check_invariant(inv: &Invariant, events: &[MonitorEvent]) -> Option<Vec<String>> {
    match inv {
        Invariant::Always(pred) => {
            for (i, ev) in events.iter().enumerate() {
                if !eval_predicate(pred, ev) {
                    return Some(vec![format!("position {}: {} failed", i, pred)]);
                }
            }
            None
        }
        Invariant::Never(pred) => {
            for (i, ev) in events.iter().enumerate() {
                if eval_predicate(pred, ev) {
                    return Some(vec![format!("position {}: {} occurred", i, pred)]);
                }
            }
            None
        }
        Invariant::Eventually(pred) => {
            if events.iter().any(|ev| eval_predicate(pred, ev)) {
                None
            } else {
                Some(vec![format!("{} never occurred", pred)])
            }
        }
        Invariant::Next(pred) => {
            for i in 0..events.len().saturating_sub(1) {
                if !eval_predicate(pred, &events[i + 1]) {
                    return Some(vec![format!("position {}: next failed {}", i, pred)]);
                }
            }
            None
        }
    }
}

/// Evaluate a simple predicate against an event.
/// Supports: "activity == X", "actor == X", "activity != X".
fn eval_predicate(pred: &str, ev: &MonitorEvent) -> bool {
    let pred = pred.trim();
    if let Some(rest) = pred.strip_prefix("activity ==") {
        return ev.activity == rest.trim();
    }
    if let Some(rest) = pred.strip_prefix("activity !=") {
        return ev.activity != rest.trim();
    }
    if let Some(rest) = pred.strip_prefix("actor ==") {
        return ev.actor == rest.trim();
    }
    if let Some(rest) = pred.strip_prefix("actor !=") {
        return ev.actor != rest.trim();
    }
    // Unknown predicate → false (conservative).
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(activity: &str, actor: &str) -> MonitorEvent {
        MonitorEvent {
            activity: activity.to_string(),
            timestamp: "t".to_string(),
            actor: actor.to_string(),
        }
    }

    #[test]
    fn empty_events_ok() {
        let m = RuntimeMonitor::new();
        assert_eq!(m.evaluate(&[]), MonitorVerdict::Ok);
    }

    #[test]
    fn always_holds() {
        let mut m = RuntimeMonitor::new();
        m.add_invariant(Invariant::Always("activity != execute_change".to_string()));
        let events = vec![ev("approve_change", "QA")];
        assert_eq!(m.evaluate(&events), MonitorVerdict::Ok);
    }

    #[test]
    fn never_violated() {
        let mut m = RuntimeMonitor::new();
        m.add_invariant(Invariant::Never("activity == execute_change".to_string()));
        let events = vec![ev("execute_change", "prod")];
        assert!(matches!(
            m.evaluate(&events),
            MonitorVerdict::Violation { .. }
        ));
    }

    #[test]
    fn eventually_holds() {
        let mut m = RuntimeMonitor::new();
        m.add_invariant(Invariant::Eventually(
            "activity == close_change".to_string(),
        ));
        let events = vec![ev("approve_change", "QA"), ev("close_change", "QA")];
        assert_eq!(m.evaluate(&events), MonitorVerdict::Ok);
    }

    #[test]
    fn eventually_violated() {
        let mut m = RuntimeMonitor::new();
        m.add_invariant(Invariant::Eventually(
            "activity == close_change".to_string(),
        ));
        let events = vec![ev("approve_change", "QA")];
        assert!(matches!(
            m.evaluate(&events),
            MonitorVerdict::Violation { .. }
        ));
    }
}
