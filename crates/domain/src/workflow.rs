#![forbid(unsafe_code)]

//! Workflow domain model and business rules.
//!
//! Dynamic multi-level approval workflow with urgency, conditions,
//! escalation, and timeout handling. Deterministic evaluation only.

use serde::{Deserialize, Serialize};

/// Urgency level for an approval task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Low,
    Normal,
    High,
    Critical,
}

impl Urgency {
    pub fn sla_seconds(&self) -> u64 {
        match self {
            Self::Low => 86400 * 3,
            Self::Normal => 86400,
            Self::High => 14400,
            Self::Critical => 3600,
        }
    }
}

/// Condition expression for stage gating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionExpr {
    AmountAbove { threshold_cents: i64 },
    AmountBelow { threshold_cents: i64 },
    PlatformEquals { platform: String },
    ClassificationEquals { classification: String },
    RiskScoreAbove { threshold: i32 },
    AnomalyFlag { flag: String },
    And { conditions: Vec<ConditionExpr> },
    Or { conditions: Vec<ConditionExpr> },
    Not { condition: Box<ConditionExpr> },
}

impl ConditionExpr {
    /// Deterministic evaluation. No model inference.
    pub fn evaluate(&self, ctx: &EvaluationContext) -> bool {
        match self {
            Self::AmountAbove { threshold_cents } => ctx.amount_cents > *threshold_cents,
            Self::AmountBelow { threshold_cents } => ctx.amount_cents < *threshold_cents,
            Self::PlatformEquals { platform } => ctx.platform == *platform,
            Self::ClassificationEquals { classification } => ctx.classification == *classification,
            Self::RiskScoreAbove { threshold } => ctx.risk_score > *threshold,
            Self::AnomalyFlag { flag } => ctx.anomaly_flags.contains(flag),
            Self::And { conditions } => conditions.iter().all(|c| c.evaluate(ctx)),
            Self::Or { conditions } => conditions.iter().any(|c| c.evaluate(ctx)),
            Self::Not { condition } => !condition.evaluate(ctx),
        }
    }
}

/// Context for condition evaluation.
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    pub amount_cents: i64,
    pub platform: String,
    pub classification: String,
    pub risk_score: i32,
    pub anomaly_flags: Vec<String>,
}

/// Stage definition within a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub approver_rule: String,
    pub sla_seconds: u64,
    pub urgency: Urgency,
    pub conditions: Vec<ConditionExpr>,
    pub escalation_path: Vec<String>,
    pub on_timeout: TimeoutAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutAction {
    Escalate,
    AutoReject,
    AutoApprove,
    Cancel,
}

/// Workflow definition (versioned, unlimited stages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub stages: Vec<Stage>,
    pub active: bool,
}

impl WorkflowDefinition {
    /// Structural validation: at least one stage, unique stage ids, positive
    /// SLA on every stage, non-empty stage names, and escalation paths that
    /// only reference existing stage ids.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.id.trim().is_empty() {
            return Err(WorkflowError::InvalidDefinition("empty id".into()));
        }
        if self.name.trim().is_empty() {
            return Err(WorkflowError::InvalidDefinition("empty name".into()));
        }
        if self.stages.is_empty() {
            return Err(WorkflowError::InvalidDefinition("no stages".into()));
        }
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for stage in &self.stages {
            if stage.id.trim().is_empty() {
                return Err(WorkflowError::InvalidDefinition("empty stage id".into()));
            }
            if stage.name.trim().is_empty() {
                return Err(WorkflowError::InvalidDefinition(format!(
                    "stage {} has empty name",
                    stage.id
                )));
            }
            if stage.sla_seconds == 0 {
                return Err(WorkflowError::InvalidDefinition(format!(
                    "stage {} has zero sla",
                    stage.id
                )));
            }
            for escalated in &stage.escalation_path {
                if !self.stages.iter().any(|s| &s.id == escalated) {
                    return Err(WorkflowError::InvalidDefinition(format!(
                        "stage {} escalates to unknown stage {}",
                        stage.id, escalated
                    )));
                }
            }
            if !seen.insert(stage.id.as_str()) {
                return Err(WorkflowError::InvalidDefinition(format!(
                    "duplicate stage id {}",
                    stage.id
                )));
            }
        }
        Ok(())
    }
}

/// Approval task state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Approved,
    Rejected,
    Escalated,
    Expired,
    Cancelled,
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Approved | Self::Rejected | Self::Expired | Self::Cancelled
        )
    }

    /// Deterministic state-machine transition. Terminal states refuse every
    /// action; `Escalated` remains actionable (an escalated task can still be
    /// decided or cancelled); `Expired` and `Cancelled` are terminal.
    pub fn transition(&self, action: TaskAction) -> Result<TaskState, WorkflowError> {
        if self.is_terminal() {
            return Err(WorkflowError::InvalidTransition {
                from: *self,
                action,
            });
        }
        Ok(match action {
            TaskAction::Approve => TaskState::Approved,
            TaskAction::Reject => TaskState::Rejected,
            TaskAction::Escalate => TaskState::Escalated,
            TaskAction::Expire => TaskState::Expired,
            TaskAction::Cancel => TaskState::Cancelled,
        })
    }
}

/// Action applied to an approval task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskAction {
    Approve,
    Reject,
    Escalate,
    Expire,
    Cancel,
}

/// Domain error for workflow rules.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorkflowError {
    #[error("invalid transition from {from:?} via action {action:?}")]
    InvalidTransition { from: TaskState, action: TaskAction },
    #[error("workflow definition invalid: {0}")]
    InvalidDefinition(String),
}

/// Approval task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalTask {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub stage_id: String,
    pub state: TaskState,
    pub urgency: Urgency,
    pub amount_cents: i64,
    pub platform: String,
    pub classification: String,
    pub risk_score: i32,
    pub created_at: i64,
    pub due_at: i64,
    pub escalated_to: Option<String>,
    pub escalation_count: u32,
    pub agent_recommendation: Option<AgentRecommendation>,
}

/// Agent recommendation (advisory within ekpi-platform).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendation {
    pub risk_level: String,
    pub confidence: f32,
    pub reason: String,
    pub classification: String,
}

/// Agent decision record (append-only audit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDecision {
    pub task_id: String,
    pub recommendation: AgentRecommendation,
    pub auto_approved: bool,
    pub decided_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_evaluation_deterministic() {
        let ctx = EvaluationContext {
            amount_cents: 150_000,
            platform: "momo".into(),
            classification: "internal".into(),
            risk_score: 30,
            anomaly_flags: vec![],
        };
        let cond = ConditionExpr::And {
            conditions: vec![
                ConditionExpr::AmountAbove {
                    threshold_cents: 100_000,
                },
                ConditionExpr::PlatformEquals {
                    platform: "momo".into(),
                },
            ],
        };
        assert!(cond.evaluate(&ctx));
    }

    #[test]
    fn task_state_terminal() {
        assert!(TaskState::Approved.is_terminal());
        assert!(!TaskState::Pending.is_terminal());
        assert!(!TaskState::Escalated.is_terminal());
    }

    #[test]
    fn urgency_sla() {
        assert!(Urgency::Critical.sla_seconds() < Urgency::Low.sla_seconds());
    }

    #[test]
    fn task_transitions_from_pending_are_valid() {
        use TaskAction::*;
        let from = TaskState::Pending;
        assert_eq!(from.transition(Approve), Ok(TaskState::Approved));
        assert_eq!(from.transition(Reject), Ok(TaskState::Rejected));
        assert_eq!(from.transition(Escalate), Ok(TaskState::Escalated));
        assert_eq!(from.transition(Expire), Ok(TaskState::Expired));
        assert_eq!(from.transition(Cancel), Ok(TaskState::Cancelled));
    }

    #[test]
    fn escalated_task_stays_actionable() {
        use TaskAction::*;
        let from = TaskState::Escalated;
        assert_eq!(from.transition(Approve), Ok(TaskState::Approved));
        assert_eq!(from.transition(Reject), Ok(TaskState::Rejected));
        assert_eq!(from.transition(Escalate), Ok(TaskState::Escalated));
        assert_eq!(from.transition(Cancel), Ok(TaskState::Cancelled));
    }

    #[test]
    fn terminal_states_refuse_every_action() {
        use TaskAction::*;
        for from in [
            TaskState::Approved,
            TaskState::Rejected,
            TaskState::Expired,
            TaskState::Cancelled,
        ] {
            for action in [Approve, Reject, Escalate, Expire, Cancel] {
                let err = from
                    .transition(action)
                    .expect_err("terminal states refuse actions");
                assert_eq!(
                    err,
                    WorkflowError::InvalidTransition { from, action },
                    "error must name the refusing state and action"
                );
            }
        }
    }

    fn valid_definition() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf-purchase".into(),
            name: "採購簽核".into(),
            version: 1,
            stages: vec![Stage {
                id: "s1".into(),
                name: "主管複核".into(),
                approver_rule: "role:approver".into(),
                sla_seconds: 3600,
                urgency: Urgency::Normal,
                conditions: vec![],
                escalation_path: vec![],
                on_timeout: TimeoutAction::Escalate,
            }],
            active: false,
        }
    }

    #[test]
    fn valid_definition_passes_validation() {
        assert_eq!(valid_definition().validate(), Ok(()));
    }

    #[test]
    fn definition_without_stages_is_invalid() {
        let mut def = valid_definition();
        def.stages.clear();
        let err = def.validate().expect_err("no stages must be invalid");
        assert!(err.to_string().contains("no stages"));
    }

    #[test]
    fn duplicate_stage_ids_are_invalid() {
        let mut def = valid_definition();
        let second = def.stages[0].clone();
        def.stages.push(second);
        let err = def.validate().expect_err("duplicate ids must be invalid");
        assert!(err.to_string().contains("duplicate stage id"));
    }

    #[test]
    fn zero_sla_is_invalid() {
        let mut def = valid_definition();
        def.stages[0].sla_seconds = 0;
        let err = def.validate().expect_err("zero sla must be invalid");
        assert!(err.to_string().contains("zero sla"));
    }

    #[test]
    fn escalation_to_unknown_stage_is_invalid() {
        let mut def = valid_definition();
        def.stages[0].escalation_path = vec!["ghost".into()];
        let err = def
            .validate()
            .expect_err("unknown escalation target must be invalid");
        assert!(err.to_string().contains("unknown stage"));
    }

    #[test]
    fn empty_definition_id_is_invalid() {
        let mut def = valid_definition();
        def.id = "  ".into();
        assert!(def.validate().is_err());
    }
}
