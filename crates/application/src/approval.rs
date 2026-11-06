#![forbid(unsafe_code)]

//! Approval use cases.
//!
//! Agent boundary: advisory only. Auto-approval lives in platform-hub.

use crate::ports::{AgentAdvisor, ApprovalRepository, GovernanceError};

/// Create a workflow definition.
pub async fn create_workflow(
    repo: &dyn ApprovalRepository,
    definition_json: &str,
) -> Result<String, GovernanceError> {
    repo.create_workflow(definition_json).await
}

/// Publish a workflow version.
pub async fn publish_workflow(
    repo: &dyn ApprovalRepository,
    workflow_id: &str,
    version: u32,
) -> Result<(), GovernanceError> {
    repo.publish_workflow(workflow_id, version).await
}

/// Submit an approval task.
pub async fn submit_task(
    repo: &dyn ApprovalRepository,
    task_json: &str,
) -> Result<String, GovernanceError> {
    repo.submit_task(task_json).await
}

/// Decide an approval task (human decision).
pub async fn decide_task(
    repo: &dyn ApprovalRepository,
    task_id: &str,
    decision: &str,
    actor: &str,
) -> Result<(), GovernanceError> {
    repo.decide_task(task_id, decision, actor).await
}

/// Escalate a task (timeout handling).
pub async fn escalate_task(
    repo: &dyn ApprovalRepository,
    task_id: &str,
) -> Result<(), GovernanceError> {
    repo.escalate_task(task_id).await
}

/// Cleanup expired tasks.
pub async fn cleanup_expired(repo: &dyn ApprovalRepository) -> Result<u64, GovernanceError> {
    repo.cleanup_expired().await
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
    use std::sync::Mutex;

    struct RecordingRepo {
        decided_actors: Mutex<Vec<String>>,
        decided_ids: Mutex<Vec<String>>,
        cleaned: Mutex<u64>,
    }

    impl RecordingRepo {
        fn new() -> Self {
            Self {
                decided_actors: Mutex::new(Vec::new()),
                decided_ids: Mutex::new(Vec::new()),
                cleaned: Mutex::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl ApprovalRepository for RecordingRepo {
        async fn create_workflow(&self, _definition: &str) -> Result<String, GovernanceError> {
            Ok("wf-1".into())
        }
        async fn publish_workflow(&self, _id: &str, _version: u32) -> Result<(), GovernanceError> {
            Ok(())
        }
        async fn submit_task(&self, _task: &str) -> Result<String, GovernanceError> {
            Ok("task-1".into())
        }
        async fn decide_task(
            &self,
            task_id: &str,
            _decision: &str,
            actor: &str,
        ) -> Result<(), GovernanceError> {
            self.decided_actors
                .lock()
                .expect("actor log")
                .push(actor.to_string());
            self.decided_ids
                .lock()
                .expect("id log")
                .push(task_id.to_string());
            Ok(())
        }
        async fn escalate_task(&self, _task_id: &str) -> Result<(), GovernanceError> {
            Ok(())
        }
        async fn cleanup_expired(&self) -> Result<u64, GovernanceError> {
            *self.cleaned.lock().expect("cleanup log") = 7;
            Ok(7)
        }
    }

    #[tokio::test]
    async fn create_workflow_delegates() {
        let repo = RecordingRepo::new();
        let id = create_workflow(&repo, "{}").await.unwrap();
        assert_eq!(id, "wf-1");
    }

    #[tokio::test]
    async fn decide_task_forwards_task_id_and_actor_unchanged() {
        // The session-derived actor must reach the repository verbatim; the
        // use case is a pure delegation boundary, never a rewriter of actors.
        let repo = RecordingRepo::new();
        decide_task(&repo, "task-42", "approved", "reviewer-a")
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
    }

    #[tokio::test]
    async fn cleanup_expired_reports_repository_count() {
        let repo = RecordingRepo::new();
        let removed = cleanup_expired(&repo).await.expect("cleanup succeeds");
        assert_eq!(removed, 7);
    }
}
