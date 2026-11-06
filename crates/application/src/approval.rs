#![forbid(unsafe_code)]

//! Approval use cases.
//!
//! Agent boundary: advisory only. Auto-approval lives in platform-hub.
//! Every use case is a thin, typed delegation over
//! [`crate::ports::ApprovalRepository`]; the repository owns persistence and
//! the domain owns the rules.

use crate::ports::{AgentAdvisor, ApprovalRepository, GovernanceError};
use ekp_domain::workflow::{TaskAction, TaskDecision, TaskSubmission, WorkflowDefinition};

/// Validate and persist a workflow definition; returns its id.
pub async fn create_workflow(
    repo: &mut dyn ApprovalRepository,
    definition: &WorkflowDefinition,
    actor: &str,
) -> Result<String, GovernanceError> {
    repo.create_workflow(definition, actor).await
}

/// Publish a workflow version, making it referenceable by tasks.
pub async fn publish_workflow(
    repo: &mut dyn ApprovalRepository,
    workflow_id: &str,
    version: u32,
    actor: &str,
) -> Result<(), GovernanceError> {
    repo.publish_workflow(workflow_id, version, actor).await
}

/// Submit an approval task; returns the generated task id.
pub async fn submit_task(
    repo: &mut dyn ApprovalRepository,
    input: &TaskSubmission,
    actor: &str,
    now_unix: i64,
) -> Result<String, GovernanceError> {
    repo.submit_task(input, actor, now_unix).await
}

/// Decide an approval task (human decision, Approver role upstream).
pub async fn decide_task(
    repo: &mut dyn ApprovalRepository,
    task_id: &str,
    action: TaskAction,
    actor: &str,
    note: &str,
    now_unix: i64,
) -> Result<TaskDecision, GovernanceError> {
    repo.decide_task(task_id, action, actor, note, now_unix)
        .await
}

/// Mark overdue open tasks expired; returns the count.
pub async fn cleanup_expired(
    repo: &mut dyn ApprovalRepository,
    now_unix: i64,
) -> Result<u64, GovernanceError> {
    repo.cleanup_expired(now_unix).await
}

/// List workflow definitions known to the store.
pub async fn list_workflows(
    repo: &dyn ApprovalRepository,
) -> Result<Vec<WorkflowDefinition>, GovernanceError> {
    repo.list_workflows().await
}

/// List approval tasks known to the store.
pub async fn list_tasks(
    repo: &dyn ApprovalRepository,
) -> Result<Vec<ekp_domain::workflow::ApprovalTask>, GovernanceError> {
    repo.list_tasks().await
}

/// Get agent advice (advisory only — never auto-approves).
pub async fn get_agent_advice(
    advisor: &dyn AgentAdvisor,
    context: &str,
) -> Result<String, GovernanceError> {
    advisor.advise(context).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::ApprovalRepository;
    use ekp_domain::workflow::{
        ApprovalTask, Stage, TaskSubmission, TimeoutAction, Urgency, WorkflowDefinition,
        WorkflowError,
    };
    use std::sync::Mutex;

    struct RecordingRepo {
        decided_actors: Mutex<Vec<String>>,
        decided_ids: Mutex<Vec<String>>,
        decided_actions: Mutex<Vec<TaskAction>>,
        created_definitions: Mutex<Vec<String>>,
        cleaned: Mutex<u64>,
    }

    impl RecordingRepo {
        fn new() -> Self {
            Self {
                decided_actors: Mutex::new(Vec::new()),
                decided_ids: Mutex::new(Vec::new()),
                decided_actions: Mutex::new(Vec::new()),
                created_definitions: Mutex::new(Vec::new()),
                cleaned: Mutex::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl ApprovalRepository for RecordingRepo {
        async fn create_workflow(
            &mut self,
            definition: &WorkflowDefinition,
            _actor: &str,
        ) -> Result<String, GovernanceError> {
            self.created_definitions
                .lock()
                .expect("definition log")
                .push(definition.id.clone());
            Ok(definition.id.clone())
        }
        async fn publish_workflow(
            &mut self,
            _workflow_id: &str,
            _version: u32,
            _actor: &str,
        ) -> Result<(), GovernanceError> {
            Ok(())
        }
        async fn submit_task(
            &mut self,
            _input: &TaskSubmission,
            _actor: &str,
            _now_unix: i64,
        ) -> Result<String, GovernanceError> {
            Ok("TSK-000001".into())
        }
        async fn decide_task(
            &mut self,
            task_id: &str,
            action: TaskAction,
            actor: &str,
            _note: &str,
            _now_unix: i64,
        ) -> Result<TaskDecision, GovernanceError> {
            self.decided_actors
                .lock()
                .expect("actor log")
                .push(actor.to_string());
            self.decided_ids
                .lock()
                .expect("id log")
                .push(task_id.to_string());
            self.decided_actions
                .lock()
                .expect("action log")
                .push(action);
            Ok(TaskDecision {
                task_id: task_id.to_string(),
                action,
                from_state: ekp_domain::workflow::TaskState::Pending,
                to_state: ekp_domain::workflow::TaskState::Approved,
                from_stage: "s1".into(),
                to_stage: None,
            })
        }
        async fn cleanup_expired(&mut self, _now_unix: i64) -> Result<u64, GovernanceError> {
            *self.cleaned.lock().expect("cleanup log") = 7;
            Ok(7)
        }
        async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>, GovernanceError> {
            Ok(vec![])
        }
        async fn list_tasks(&self) -> Result<Vec<ApprovalTask>, GovernanceError> {
            Ok(vec![])
        }
        fn verify(&self) -> Result<(), GovernanceError> {
            Ok(())
        }
    }

    fn definition() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf-1".into(),
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

    #[tokio::test]
    async fn create_workflow_delegates_definition_and_returns_id() {
        let mut repo = RecordingRepo::new();
        let id = create_workflow(&mut repo, &definition(), "approver-a")
            .await
            .unwrap();
        assert_eq!(id, "wf-1");
        assert_eq!(
            repo.created_definitions.lock().expect("log").as_slice(),
            ["wf-1"]
        );
    }

    #[tokio::test]
    async fn decide_task_forwards_task_id_action_and_actor_unchanged() {
        // The session-derived actor must reach the repository verbatim; the
        // use case is a pure delegation boundary, never a rewriter of actors.
        let mut repo = RecordingRepo::new();
        decide_task(
            &mut repo,
            "task-42",
            TaskAction::Reject,
            "reviewer-a",
            "",
            1,
        )
        .await
        .expect("delegation succeeds");
        assert_eq!(
            repo.decided_actors.lock().expect("actor log").as_slice(),
            ["reviewer-a"]
        );
        assert_eq!(
            repo.decided_ids.lock().expect("id log").as_slice(),
            ["task-42"]
        );
        assert_eq!(
            repo.decided_actions.lock().expect("action log").as_slice(),
            [TaskAction::Reject]
        );
    }

    #[tokio::test]
    async fn cleanup_expired_reports_repository_count() {
        let mut repo = RecordingRepo::new();
        let removed = cleanup_expired(&mut repo, 1)
            .await
            .expect("cleanup succeeds");
        assert_eq!(removed, 7);
    }

    #[tokio::test]
    async fn agent_advice_is_advisory_delegation() {
        struct Advisor;
        #[async_trait::async_trait]
        impl AgentAdvisor for Advisor {
            async fn advise(&self, context: &str) -> Result<String, GovernanceError> {
                Ok(format!("advice for {context}"))
            }
        }
        let advice = get_agent_advice(&Advisor, "task-1").await.unwrap();
        assert_eq!(advice, "advice for task-1");
        // Compile-time boundary reminder: AgentAdvisor has no approve method;
        // auto-approval is structurally impossible in this crate.
        let _ = std::marker::PhantomData::<WorkflowError>;
    }
}
