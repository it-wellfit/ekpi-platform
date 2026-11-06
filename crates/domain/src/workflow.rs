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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    #[error("task submission invalid: {0}")]
    InvalidSubmission(String),
    #[error("stage {stage} has no escalation path")]
    NoEscalationPath { stage: String },
}

/// Input for submitting an approval task (bounded-context submission form).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmission {
    pub workflow_id: String,
    pub workflow_version: u32,
    pub amount_cents: i64,
    pub platform: String,
    pub classification: String,
    pub risk_score: i32,
    pub urgency: Urgency,
}

/// Result of a decided transition, recorded verbatim in the store event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDecision {
    pub task_id: String,
    pub action: TaskAction,
    pub from_state: TaskState,
    pub to_state: TaskState,
    pub from_stage: String,
    pub to_stage: Option<String>,
}

impl TaskAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Escalate => "escalate",
            Self::Expire => "expire",
            Self::Cancel => "cancel",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "approve" => Some(Self::Approve),
            "reject" => Some(Self::Reject),
            "escalate" => Some(Self::Escalate),
            "expire" => Some(Self::Expire),
            "cancel" => Some(Self::Cancel),
            _ => None,
        }
    }
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

impl ApprovalTask {
    /// An open task can still be decided; terminal tasks cannot.
    pub fn is_open(&self) -> bool {
        matches!(self.state, TaskState::Pending | TaskState::Escalated)
    }

    /// Overdue means: still open and past `due_at`. Terminal tasks are never
    /// overdue regardless of their timestamps.
    pub fn is_overdue(&self, now_unix: i64) -> bool {
        self.is_open() && self.due_at < now_unix
    }
}

impl WorkflowDefinition {
    /// The stage id that follows `after`, or `None` when `after` is the last
    /// stage. Callers must have already verified that `after` exists
    /// (`stage_index`); an unknown id yields `None`.
    pub fn next_stage_id(&self, after: &str) -> Option<&str> {
        let index = self.stage_index(after)?;
        self.stages.get(index + 1).map(|next| next.id.as_str())
    }

    /// Index of the stage with the given id, if present.
    pub fn stage_index(&self, stage_id: &str) -> Option<usize> {
        self.stages.iter().position(|stage| stage.id == stage_id)
    }
}

/// Creates a task on the first stage of a **published** definition.
///
/// The task deadline is `now + stage.sla_seconds` of the first stage (stage
/// SLA governs the stage; urgency orders the queue and is recorded on the
/// task). Actor is the session-derived submitter.
pub fn submit_task(
    definition: &WorkflowDefinition,
    input: &TaskSubmission,
    task_id: &str,
    _actor: &str,
    now_unix: i64,
) -> Result<ApprovalTask, WorkflowError> {
    if !definition.active {
        return Err(WorkflowError::InvalidSubmission(
            "workflow definition is not published".into(),
        ));
    }
    if definition.id != input.workflow_id || definition.version != input.workflow_version {
        return Err(WorkflowError::InvalidSubmission(format!(
            "task references workflow version {}@{} but the store holds {}@{}",
            input.workflow_id, input.workflow_version, definition.id, definition.version
        )));
    }
    let first = definition
        .stages
        .first()
        .ok_or_else(|| WorkflowError::InvalidDefinition("definition has no stages".into()))?;
    if task_id.trim().is_empty() {
        return Err(WorkflowError::InvalidSubmission("empty task id".into()));
    }
    Ok(ApprovalTask {
        id: task_id.to_string(),
        workflow_id: definition.id.clone(),
        workflow_version: definition.version,
        stage_id: first.id.clone(),
        state: TaskState::Pending,
        urgency: input.urgency,
        amount_cents: input.amount_cents,
        platform: input.platform.clone(),
        classification: input.classification.clone(),
        risk_score: input.risk_score,
        created_at: now_unix,
        due_at: now_unix + first.sla_seconds as i64,
        escalated_to: None,
        escalation_count: 0,
        agent_recommendation: None,
    })
}

/// Applies a decision to an open task under its definition (aggregate rule).
///
/// - `Approve`: advance to the next stage (task returns to `Pending`, which
///   resolves any escalation) or complete the task when on the last stage.
/// - `Reject`: terminal rejection from any stage.
/// - `Escalate`: stay on the stage, record the first escalation target and
///   bump the counter. A stage without an escalation path refuses.
/// - `Expire`/`Cancel`: terminal, used by cleanup and explicit cancellation.
///
/// The returned [`TaskDecision`] carries the exact before/after values so the
/// store event can be replay-validated deterministically.
pub fn decide_task(
    task: &mut ApprovalTask,
    definition: &WorkflowDefinition,
    action: TaskAction,
    _actor: &str,
    _now_unix: i64,
) -> Result<TaskDecision, WorkflowError> {
    if task.workflow_id != definition.id || task.workflow_version != definition.version {
        return Err(WorkflowError::InvalidSubmission(format!(
            "task {} references {}@{} but the definition is {}@{}",
            task.id, task.workflow_id, task.workflow_version, definition.id, definition.version
        )));
    }
    let from_state = task.state;
    let from_stage = task.stage_id.clone();
    let stage_index = definition.stage_index(&from_stage).ok_or_else(|| {
        WorkflowError::InvalidSubmission(format!(
            "task {} sits on unknown stage {}",
            task.id, from_stage
        ))
    })?;

    // Validity gate: terminal states and unknown actions are refused here.
    from_state.transition(action)?;
    let mut to_stage: Option<String> = None;
    match action {
        TaskAction::Approve => {
            if let Some(next) = definition.stages.get(stage_index + 1) {
                to_stage = Some(next.id.clone());
                task.stage_id = next.id.clone();
                task.due_at = _now_unix + next.sla_seconds as i64;
                // Advancing resolves any escalation: the task is waiting at a
                // new stage and is actionable again.
                task.state = TaskState::Pending;
                task.escalated_to = None;
            } else {
                task.state = TaskState::Approved;
            }
        }
        TaskAction::Reject => {
            task.state = TaskState::Rejected;
        }
        TaskAction::Escalate => {
            let stage = &definition.stages[stage_index];
            let target =
                stage
                    .escalation_path
                    .first()
                    .ok_or_else(|| WorkflowError::NoEscalationPath {
                        stage: from_stage.clone(),
                    })?;
            task.state = TaskState::Escalated;
            task.escalated_to = Some(target.clone());
            task.escalation_count = task.escalation_count.saturating_add(1);
        }
        TaskAction::Expire => {
            task.state = TaskState::Expired;
        }
        TaskAction::Cancel => {
            task.state = TaskState::Cancelled;
        }
    }
    Ok(TaskDecision {
        task_id: task.id.clone(),
        action,
        from_state,
        to_state: task.state,
        from_stage,
        to_stage,
    })
}

/// Marks every overdue open task `Expired`; returns the ids that changed.
pub fn expire_overdue_tasks(tasks: &mut [&mut ApprovalTask], now_unix: i64) -> Vec<String> {
    let mut expired = Vec::new();
    for task in tasks.iter_mut() {
        if task.is_overdue(now_unix) {
            task.state = TaskState::Expired;
            expired.push(task.id.clone());
        }
    }
    expired
}

/// Domain-separated hash input version for the approval event chain.
pub const APPROVAL_CHAIN_VERSION: &str = "ekpi-approval-v1";

/// Append-only approval store event payloads. The projection is derived
/// purely from these events; replay must be deterministic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApprovalEventPayload {
    WorkflowCreated {
        definition: WorkflowDefinition,
    },
    WorkflowPublished {
        workflow_id: String,
        version: u32,
    },
    TaskSubmitted {
        task: ApprovalTask,
    },
    TaskTransitioned {
        decision: TaskDecision,
        escalated_to: Option<String>,
        note: String,
    },
    TasksExpired {
        task_ids: Vec<String>,
    },
}

/// A hash-chained approval store event (same shape as the governance ledger).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvent {
    pub sequence: u64,
    pub timestamp_unix: i64,
    pub actor: String,
    pub payload: ApprovalEventPayload,
    pub previous_sha256: String,
    pub sha256: String,
}

/// In-memory projection of the approval event chain.
#[derive(Debug, Clone, Default)]
pub struct ApprovalProjection {
    pub definitions: Vec<WorkflowDefinition>,
    pub tasks: Vec<ApprovalTask>,
}

/// Canonical hash over an approval event (domain-separated by
/// [`APPROVAL_CHAIN_VERSION`]).
pub fn approval_event_hash(
    sequence: u64,
    timestamp_unix: i64,
    actor: &str,
    payload: &ApprovalEventPayload,
    previous_sha256: &str,
) -> Result<String, String> {
    let canonical = serde_json::json!({
        "version": APPROVAL_CHAIN_VERSION,
        "sequence": sequence,
        "timestamp_unix": timestamp_unix,
        "actor": actor,
        "payload": payload,
        "previous_sha256": previous_sha256,
    });
    let bytes =
        serde_json::to_vec(&canonical).map_err(|e| format!("encode approval event: {e}"))?;
    Ok(crate::hashing::sha256_hex(&bytes))
}

/// Applies one event to the projection with deterministic replay validation.
pub fn apply_approval_event(
    projection: &mut ApprovalProjection,
    event: &ApprovalEvent,
) -> Result<(), WorkflowError> {
    match &event.payload {
        ApprovalEventPayload::WorkflowCreated { definition } => {
            if projection
                .definitions
                .iter()
                .any(|d| d.id == definition.id && d.version == definition.version)
            {
                return Err(WorkflowError::InvalidDefinition(format!(
                    "workflow {}@{} already exists",
                    definition.id, definition.version
                )));
            }
            definition.validate()?;
            projection.definitions.push(definition.clone());
        }
        ApprovalEventPayload::WorkflowPublished {
            workflow_id,
            version,
        } => {
            let definition = projection
                .definitions
                .iter_mut()
                .find(|d| &d.id == workflow_id && d.version == *version)
                .ok_or_else(|| {
                    WorkflowError::InvalidDefinition(format!(
                        "unknown workflow {workflow_id}@{version}"
                    ))
                })?;
            if definition.active {
                return Err(WorkflowError::InvalidDefinition(format!(
                    "workflow {workflow_id}@{version} already published"
                )));
            }
            definition.active = true;
        }
        ApprovalEventPayload::TaskSubmitted { task } => {
            if projection.tasks.iter().any(|t| t.id == task.id) {
                return Err(WorkflowError::InvalidSubmission(format!(
                    "task {} already exists",
                    task.id
                )));
            }
            let definition = projection
                .definitions
                .iter()
                .find(|d| d.id == task.workflow_id && d.version == task.workflow_version)
                .ok_or_else(|| {
                    WorkflowError::InvalidSubmission(format!(
                        "task {} references unknown workflow {}@{}",
                        task.id, task.workflow_id, task.workflow_version
                    ))
                })?;
            if !definition.active {
                return Err(WorkflowError::InvalidSubmission(format!(
                    "task {} references unpublished workflow {}@{}",
                    task.id, definition.id, definition.version
                )));
            }
            let expected_stage = definition
                .stages
                .first()
                .map(|s| s.id.clone())
                .unwrap_or_default();
            if task.stage_id != expected_stage {
                return Err(WorkflowError::InvalidSubmission(format!(
                    "task {} must start on stage {expected_stage}",
                    task.id
                )));
            }
            projection.tasks.push(task.clone());
        }
        ApprovalEventPayload::TaskTransitioned {
            decision,
            escalated_to,
            ..
        } => {
            let task = projection
                .tasks
                .iter_mut()
                .find(|t| t.id == decision.task_id)
                .ok_or_else(|| {
                    WorkflowError::InvalidSubmission(format!("unknown task {}", decision.task_id))
                })?;
            if task.state != decision.from_state || task.stage_id != decision.from_stage {
                return Err(WorkflowError::InvalidSubmission(format!(
                    "task {} replay mismatch: store has {:?}/{} but event claims {:?}/{}",
                    task.id, task.state, task.stage_id, decision.from_state, decision.from_stage
                )));
            }
            if decision.to_state.is_terminal() && decision.to_stage.is_some() {
                return Err(WorkflowError::InvalidSubmission(format!(
                    "task {} terminal state {:?} must not carry a next stage",
                    task.id, decision.to_state
                )));
            }
            task.state = decision.to_state;
            if let Some(next) = decision.to_stage.clone() {
                task.stage_id = next;
            }
            if decision.action == TaskAction::Escalate {
                task.escalated_to = escalated_to.clone();
                task.escalation_count = task.escalation_count.saturating_add(1);
            } else if decision.to_state == TaskState::Pending {
                task.escalated_to = None;
            }
        }
        ApprovalEventPayload::TasksExpired { task_ids } => {
            for id in task_ids {
                let task = projection
                    .tasks
                    .iter_mut()
                    .find(|t| t.id == *id)
                    .ok_or_else(|| {
                        WorkflowError::InvalidSubmission(format!("unknown task {id}"))
                    })?;
                if !task.is_open() {
                    return Err(WorkflowError::InvalidSubmission(format!(
                        "task {id} is not open and cannot expire"
                    )));
                }
                task.state = TaskState::Expired;
            }
        }
    }
    Ok(())
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

    // ── B 組：任務生命週期（BDD 場景鎖，見 docs/quality/bdd-scenarios-approval.md）──

    fn two_stage_definition() -> WorkflowDefinition {
        let mut def = valid_definition();
        def.id = "wf-two".into();
        def.active = true;
        def.stages = vec![
            Stage {
                id: "s1".into(),
                name: "主管複核".into(),
                approver_rule: "role:approver".into(),
                sla_seconds: 3600,
                urgency: Urgency::Normal,
                conditions: vec![],
                escalation_path: vec![],
                on_timeout: TimeoutAction::Escalate,
            },
            Stage {
                id: "s2".into(),
                name: "部門核准".into(),
                approver_rule: "role:approver".into(),
                sla_seconds: 86400,
                urgency: Urgency::Normal,
                conditions: vec![],
                escalation_path: vec![],
                on_timeout: TimeoutAction::AutoReject,
            },
        ];
        def
    }

    fn submission() -> TaskSubmission {
        TaskSubmission {
            workflow_id: "wf-two".into(),
            workflow_version: 1,
            amount_cents: 15_000,
            platform: "momo".into(),
            classification: "internal".into(),
            risk_score: 20,
            urgency: Urgency::Normal,
        }
    }

    /// B1：提交落在第一階段，due=now+該階段 SLA。
    #[test]
    fn submitting_task_places_it_in_first_stage_with_stage_sla() {
        let def = two_stage_definition();
        let now = 1_000_000;
        let task = submit_task(&def, &submission(), "TSK-000001", "editor-a", now)
            .expect("submission on published definition succeeds");
        assert_eq!(task.stage_id, "s1");
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.due_at, now + 3600);
        assert_eq!(task.workflow_id, "wf-two");
        assert_eq!(task.workflow_version, 1);
        assert_eq!(task.escalation_count, 0);
        assert!(task.escalated_to.is_none());
    }

    /// B2：核准中間階段前進到下一階段，仍 Pending。
    #[test]
    fn approving_mid_stage_advances_to_next_stage() {
        let def = two_stage_definition();
        let mut task = submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000)
            .expect("submission succeeds");
        let decision = decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_100,
        )
        .expect("first approval succeeds");
        assert_eq!(decision.from_stage, "s1");
        assert_eq!(decision.to_stage.as_deref(), Some("s2"));
        assert_eq!(task.stage_id, "s2");
        assert_eq!(task.state, TaskState::Pending);
    }

    /// B3：核准最終階段完成任務。
    #[test]
    fn approving_final_stage_completes_task() {
        let def = two_stage_definition();
        let mut task =
            submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000).unwrap();
        decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_100,
        )
        .unwrap();
        let decision = decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_200,
        )
        .expect("final approval succeeds");
        assert_eq!(decision.to_state, TaskState::Approved);
        assert_eq!(decision.to_stage, None);
        assert_eq!(task.state, TaskState::Approved);
    }

    /// B4：任何階段駁回皆為終態。
    #[test]
    fn rejecting_task_is_terminal_from_any_stage() {
        let def = two_stage_definition();
        let mut task =
            submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000).unwrap();
        decide_task(&mut task, &def, TaskAction::Reject, "approver-a", 1_000_100).unwrap();
        assert_eq!(task.state, TaskState::Rejected);
        let err = decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_200,
        )
        .expect_err("terminal task refuses decisions");
        assert!(matches!(err, WorkflowError::InvalidTransition { .. }));
    }

    /// B5：升級記錄對象並累計次數（不推進階段）。
    #[test]
    fn escalating_task_records_target_and_count() {
        let mut def = two_stage_definition();
        def.stages[0].escalation_path = vec!["s2".into()];
        let mut task =
            submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000).unwrap();
        decide_task(
            &mut task,
            &def,
            TaskAction::Escalate,
            "approver-a",
            1_000_100,
        )
        .unwrap();
        assert_eq!(task.state, TaskState::Escalated);
        assert_eq!(task.stage_id, "s1", "escalation does not advance the stage");
        assert_eq!(task.escalation_count, 1);
        assert_eq!(task.escalated_to.as_deref(), Some("s2"));
    }

    /// B6：終態拒絕一切操作。
    #[test]
    fn deciding_terminal_task_is_refused() {
        let def = two_stage_definition();
        let mut task =
            submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000).unwrap();
        decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_100,
        )
        .unwrap();
        decide_task(
            &mut task,
            &def,
            TaskAction::Approve,
            "approver-a",
            1_000_200,
        )
        .unwrap();
        assert!(task.state.is_terminal());
        for action in [
            TaskAction::Approve,
            TaskAction::Reject,
            TaskAction::Escalate,
        ] {
            assert!(decide_task(&mut task, &def, action, "approver-a", 1_000_300).is_err());
        }
    }

    /// B7：未發布的定義不可提任務。
    #[test]
    fn submitting_task_against_unpublished_definition_is_refused() {
        let mut def = two_stage_definition();
        def.active = false;
        let err = submit_task(&def, &submission(), "TSK-000001", "editor-a", 1_000_000)
            .expect_err("unpublished definition must refuse tasks");
        assert!(err.to_string().contains("not published"));
    }

    /// B8：逾期判定尊重 due_at 與終態。
    #[test]
    fn overdue_detection_respects_due_at_and_terminal_states() {
        let def = two_stage_definition();
        let now = 1_000_000;
        let mut task = submit_task(&def, &submission(), "TSK-000001", "editor-a", now).unwrap();
        assert!(
            !task.is_overdue(now + 3600),
            "exactly at due is not overdue"
        );
        assert!(task.is_overdue(now + 3601), "past due is overdue");
        decide_task(&mut task, &def, TaskAction::Approve, "approver-a", now + 10).unwrap();
        decide_task(&mut task, &def, TaskAction::Approve, "approver-a", now + 20).unwrap();
        assert!(
            !task.is_overdue(now + 999_999),
            "terminal tasks are never overdue"
        );
    }

    /// B9：cleanup 只標記逾期未決任務為 Expired。
    #[test]
    fn cleanup_expires_only_overdue_open_tasks() {
        let def = two_stage_definition();
        let now = 1_000_000;
        let mut overdue =
            submit_task(&def, &submission(), "TSK-000001", "editor-a", now - 7200).unwrap();
        let mut fresh = submit_task(&def, &submission(), "TSK-000002", "editor-a", now).unwrap();
        let mut done =
            submit_task(&def, &submission(), "TSK-000003", "editor-a", now - 7200).unwrap();
        decide_task(
            &mut done,
            &def,
            TaskAction::Approve,
            "approver-a",
            now - 7000,
        )
        .unwrap();
        decide_task(
            &mut done,
            &def,
            TaskAction::Approve,
            "approver-a",
            now - 6900,
        )
        .unwrap();

        let expired = expire_overdue_tasks(&mut [&mut overdue, &mut fresh, &mut done], now + 3000);
        assert_eq!(expired, vec!["TSK-000001".to_string()]);
        assert_eq!(overdue.state, TaskState::Expired);
        assert_eq!(fresh.state, TaskState::Pending, "fresh task untouched");
        assert_eq!(done.state, TaskState::Approved, "decided task untouched");
    }

    /// B10：任務必須引用已存在的已發布版本。
    #[test]
    fn task_must_reference_existing_published_version() {
        let def = two_stage_definition();
        let mut input = submission();
        input.workflow_version = 2;
        let err = submit_task(&def, &input, "TSK-000001", "editor-a", 1_000_000)
            .expect_err("unknown version must be refused");
        assert!(err.to_string().contains("version"));
    }
}
