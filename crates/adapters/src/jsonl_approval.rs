#![forbid(unsafe_code)]

//! Append-only JSONL approval store.
//!
//! Implements [`ekp_application::ports::ApprovalRepository`] as a separate
//! bounded context from the governance ledger: its own event types
//! (`ekp_domain::workflow::ApprovalEventPayload`), its own domain-separated
//! hash chain, its own file. Replay is deterministic and fail closed:
//! tampered, torn or out-of-order files refuse to load entirely.
//!
//! This is a local pilot store, not an enterprise WORM service.

use std::fs;
use std::path::{Path, PathBuf};

use ekp_application::ports::GovernanceError;
use ekp_domain::governance::{now_unix, validate_actor};
use ekp_domain::hashing::sha256_hex;
use ekp_domain::workflow::{
    ApprovalEvent, ApprovalEventPayload, ApprovalProjection, TaskAction, TaskDecision,
    TaskSubmission, WorkflowDefinition, apply_approval_event, approval_event_hash,
};

use crate::local_lock::LocalFileLock;

/// The approval store: a hash-chained JSONL event file plus its projection.
#[derive(Debug)]
pub struct ApprovalStore {
    path: PathBuf,
    projection: ApprovalProjection,
    events: Vec<ApprovalEvent>,
    _lock: LocalFileLock,
}

fn sha256_file(path: &Path) -> Result<String, GovernanceError> {
    let content = fs::read(path).map_err(|error| {
        GovernanceError::storage(format!("read approval file {}: {error}", path.display()))
    })?;
    Ok(sha256_hex(&content))
}

impl ApprovalStore {
    /// Open (or create) the store at `path`, replaying and verifying every
    /// event. Any sequence gap, predecessor mismatch, hash mismatch, torn
    /// final line or projection violation refuses the whole load.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, GovernanceError> {
        let path = path.into();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|e| {
                GovernanceError::storage(format!(
                    "create approval directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        let lock = LocalFileLock::acquire(&path)?;
        let mut projection = ApprovalProjection::default();
        let mut events: Vec<ApprovalEvent> = Vec::new();
        if path.exists() {
            let contents = fs::read_to_string(&path).map_err(|e| {
                GovernanceError::storage(format!("read approval store {}: {e}", path.display()))
            })?;
            if !contents.is_empty() && !contents.ends_with('\n') {
                return Err(GovernanceError::validation(format!(
                    "approval store {} has a truncated final line; refusing to load",
                    path.display()
                )));
            }
            for (index, line) in contents.lines().enumerate() {
                if line.trim().is_empty() {
                    return Err(GovernanceError::validation(format!(
                        "approval store line {} is empty",
                        index + 1
                    )));
                }
                let event: ApprovalEvent = serde_json::from_str(line).map_err(|e| {
                    GovernanceError::storage(format!(
                        "parse approval event line {}: {e}",
                        index + 1
                    ))
                })?;
                if event.sequence != index as u64 + 1 {
                    return Err(GovernanceError::validation(format!(
                        "approval store sequence mismatch at line {}",
                        index + 1
                    )));
                }
                let expected_previous = events
                    .last()
                    .map(|last| last.sha256.clone())
                    .unwrap_or_else(|| GENESIS.to_string());
                if event.previous_sha256 != expected_previous {
                    return Err(GovernanceError::validation(format!(
                        "approval store predecessor mismatch at line {}",
                        index + 1
                    )));
                }
                let expected_hash = approval_event_hash(
                    event.sequence,
                    event.timestamp_unix,
                    &event.actor,
                    &event.payload,
                    &event.previous_sha256,
                )
                .map_err(|e| GovernanceError::validation(e.to_string()))?;
                if event.sha256 != expected_hash {
                    return Err(GovernanceError::validation(format!(
                        "approval store hash mismatch at line {}",
                        index + 1
                    )));
                }
                validate_actor(&event.actor).map_err(|error| {
                    GovernanceError::validation(format!("approval line {}: {error}", index + 1))
                })?;
                apply_approval_event(&mut projection, &event).map_err(|error| {
                    GovernanceError::validation(format!(
                        "approval store line {}: {error}",
                        index + 1
                    ))
                })?;
                events.push(event);
            }
        }
        Ok(Self {
            path,
            projection,
            events,
            _lock: lock,
        })
    }

    /// A working empty store backed by a unique temporary file. Used by tests
    /// and by `router(ledger, auth)`; the production binary opens the
    /// configured path via [`ApprovalStore::open`] instead.
    pub fn ephemeral() -> Result<Self, GovernanceError> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let name = format!(
            "ekpi-approval-{}-{}.jsonl",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        Self::open(std::env::temp_dir().join(name))
    }

    pub fn snapshot(&self) -> ApprovalProjection {
        self.projection.clone()
    }

    /// Verify the whole event chain (sequence, predecessor, hash).
    pub fn verify(&self) -> Result<(), GovernanceError> {
        let mut previous = GENESIS.to_string();
        for (index, event) in self.events.iter().enumerate() {
            if event.sequence != index as u64 + 1 || event.previous_sha256 != previous {
                return Err(GovernanceError::validation(format!(
                    "approval chain sequence/predecessor mismatch at event {}",
                    index + 1
                )));
            }
            let expected = approval_event_hash(
                event.sequence,
                event.timestamp_unix,
                &event.actor,
                &event.payload,
                &event.previous_sha256,
            )
            .map_err(|e| GovernanceError::validation(e.to_string()))?;
            if event.sha256 != expected {
                return Err(GovernanceError::validation(format!(
                    "approval chain hash mismatch at event {}",
                    index + 1
                )));
            }
            previous = event.sha256.clone();
        }
        Ok(())
    }

    /// Validate and append a `WorkflowCreated` event.
    pub fn create_workflow(
        &mut self,
        definition: WorkflowDefinition,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        definition
            .validate()
            .map_err(|e| GovernanceError::validation(e.to_string()))?;
        if self
            .projection
            .definitions
            .iter()
            .any(|d| d.id == definition.id && d.version == definition.version)
        {
            return Err(GovernanceError::validation(format!(
                "workflow {}@{} already exists",
                definition.id, definition.version
            )));
        }
        self.append(actor, ApprovalEventPayload::WorkflowCreated { definition })?;
        Ok(self
            .projection
            .definitions
            .last()
            .map(|d| d.id.clone())
            .unwrap_or_default())
    }

    /// Publish a workflow version.
    pub fn publish_workflow(
        &mut self,
        workflow_id: &str,
        version: u32,
        actor: &str,
    ) -> Result<(), GovernanceError> {
        let known = self
            .projection
            .definitions
            .iter()
            .any(|d| d.id == workflow_id && d.version == version && !d.active);
        if !known {
            return Err(GovernanceError::validation(format!(
                "cannot publish workflow {workflow_id}@{version}: not found or already published"
            )));
        }
        self.append(
            actor,
            ApprovalEventPayload::WorkflowPublished {
                workflow_id: workflow_id.to_string(),
                version,
            },
        )
    }

    /// Create a task on the first stage of a published definition.
    pub fn submit_task(
        &mut self,
        input: &TaskSubmission,
        actor: &str,
        now_unix: i64,
    ) -> Result<String, GovernanceError> {
        let definition = self
            .projection
            .definitions
            .iter()
            .find(|d| d.id == input.workflow_id && d.version == input.workflow_version)
            .ok_or_else(|| {
                GovernanceError::validation(format!(
                    "task references workflow version {}@{} which is not in the store",
                    input.workflow_id, input.workflow_version
                ))
            })?
            .clone();
        let task_id = self.next_task_id();
        let task = ekp_domain::workflow::submit_task(&definition, input, &task_id, actor, now_unix)
            .map_err(|e| GovernanceError::validation(e.to_string()))?;
        self.append(actor, ApprovalEventPayload::TaskSubmitted { task })?;
        Ok(task_id)
    }

    /// Apply a decision to an open task.
    pub fn decide_task(
        &mut self,
        task_id: &str,
        action: TaskAction,
        actor: &str,
        note: &str,
        now_unix: i64,
    ) -> Result<TaskDecision, GovernanceError> {
        let mut task = self
            .projection
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| GovernanceError::validation(format!("unknown task {task_id}")))?
            .clone();
        let definition = self
            .projection
            .definitions
            .iter()
            .find(|d| d.id == task.workflow_id && d.version == task.workflow_version)
            .ok_or_else(|| {
                GovernanceError::validation(format!("task {task_id} references unknown workflow"))
            })?
            .clone();
        let decision =
            ekp_domain::workflow::decide_task(&mut task, &definition, action, actor, now_unix)
                .map_err(|e| GovernanceError::validation(e.to_string()))?;
        let escalated_to = task.escalated_to.clone();
        self.append(
            actor,
            ApprovalEventPayload::TaskTransitioned {
                decision: decision.clone(),
                escalated_to,
                note: note.trim().to_string(),
            },
        )?;
        Ok(decision)
    }

    /// Mark overdue open tasks `Expired`; returns how many changed.
    ///
    /// Semantics note (IMP-020): this marks every overdue open task `Expired`
    /// and does **not** yet honor the stage's configured `TimeoutAction`
    /// (`Escalate`/`AutoReject`/`Cancel` are accepted by the model but await
    /// the scheduler slice; `AutoApprove` is refused at definition validation
    /// because ekpi-platform never auto-approves). Until then, "timeout means
    /// Expired" is the documented behavior — do not present it as SLA-action
    /// honoring.
    pub fn cleanup_expired(&mut self, now_unix: i64) -> Result<u64, GovernanceError> {
        let mut overdue: Vec<String> = self
            .projection
            .tasks
            .iter()
            .filter(|task| task.is_overdue(now_unix))
            .map(|task| task.id.clone())
            .collect();
        overdue.sort();
        if overdue.is_empty() {
            return Ok(0);
        }
        let count = overdue.len() as u64;
        self.append(
            actor_for_cleanup(),
            ApprovalEventPayload::TasksExpired { task_ids: overdue },
        )?;
        Ok(count)
    }

    pub fn list_workflows(&self) -> Vec<WorkflowDefinition> {
        self.projection.definitions.clone()
    }

    pub fn list_tasks(&self) -> Vec<ekp_domain::workflow::ApprovalTask> {
        self.projection.tasks.clone()
    }

    /// Actors of every event in order — read accessor for audit assertions.
    pub fn event_actors(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|event| event.actor.clone())
            .collect()
    }

    /// Task ids number the *tasks*, not the events: definitions and
    /// publications must not shift task identifiers. The store is
    /// single-instance per file (file lock), so ids cannot collide.
    fn next_task_id(&self) -> String {
        format!("TSK-{:06}", self.projection.tasks.len() + 1)
    }

    fn append(
        &mut self,
        actor: &str,
        payload: ApprovalEventPayload,
    ) -> Result<(), GovernanceError> {
        validate_actor(actor).map_err(|e| GovernanceError::validation(e.to_string()))?;
        let actor = actor.trim().to_string();
        let sequence = self.events.len() as u64 + 1;
        let previous_sha256 = self
            .events
            .last()
            .map(|last| last.sha256.clone())
            .unwrap_or_else(|| GENESIS.to_string());
        let timestamp_unix = now_i64()?;
        let sha256 =
            approval_event_hash(sequence, timestamp_unix, &actor, &payload, &previous_sha256)
                .map_err(|e| GovernanceError::validation(e.to_string()))?;
        let event = ApprovalEvent {
            sequence,
            timestamp_unix,
            actor,
            payload,
            previous_sha256,
            sha256,
        };
        let mut next_projection = self.projection.clone();
        apply_approval_event(&mut next_projection, &event)
            .map_err(|e| GovernanceError::validation(e.to_string()))?;

        let mut line = serde_json::to_vec(&event)
            .map_err(|e| GovernanceError::storage(format!("encode approval event: {e}")))?;
        line.push(b'\n');

        // Atomic write: temp file + fsync + rename + checksum verify, mirroring
        // the governance ledger append.
        let temporary_path = self.path.with_extension("tmp");
        let existing = if self.path.exists() {
            fs::read(&self.path).map_err(|e| {
                GovernanceError::storage(format!(
                    "read existing approval store {}: {e}",
                    self.path.display()
                ))
            })?
        } else {
            Vec::new()
        };
        let mut temp_file = fs::File::create(&temporary_path).map_err(|e| {
            GovernanceError::storage(format!(
                "create temporary approval file {}: {e}",
                temporary_path.display()
            ))
        })?;
        use std::io::Write;
        temp_file
            .write_all(&existing)
            .and_then(|_| temp_file.write_all(&line))
            .and_then(|_| temp_file.sync_all())
            .map_err(|e| {
                GovernanceError::storage(format!(
                    "write temporary approval event {}: {e}",
                    temporary_path.display()
                ))
            })?;
        drop(temp_file);

        let checksum = sha256_file(&temporary_path)?;
        fs::rename(&temporary_path, &self.path).map_err(|e| {
            let _ = fs::remove_file(&temporary_path);
            GovernanceError::storage(format!(
                "atomic approval write: rename {}: {e}",
                self.path.display()
            ))
        })?;
        let actual = sha256_file(&self.path)?;
        if checksum != actual {
            return Err(GovernanceError::validation(format!(
                "approval store checksum mismatch after rename: expected {checksum}, got {actual}"
            )));
        }
        self.projection = next_projection;
        self.events.push(event);
        Ok(())
    }
}

const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Current unix time as i64, fail-closed on overflow (`now_unix` is u64).
fn now_i64() -> Result<i64, GovernanceError> {
    let now = now_unix()?;
    i64::try_from(now)
        .map_err(|_| GovernanceError::validation("system clock out of range".to_string()))
}

/// The system actor for automatic expiry transitions.
fn actor_for_cleanup() -> &'static str {
    "approval-cleanup"
}

/// Default approval store path, colocated with the governance ledger.
pub fn default_approval_path() -> PathBuf {
    if let Some(path) = std::env::var_os("EKP_APPROVAL_DATA") {
        return PathBuf::from(path);
    }
    super::jsonl_ledger::default_ledger_path().join("approvals.jsonl")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ekp_domain::workflow::{Stage, TaskSubmission, TimeoutAction, Urgency, WorkflowDefinition};

    fn two_stage_definition() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf-two".into(),
            name: "採購簽核".into(),
            version: 1,
            stages: vec![
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
            ],
            active: false,
        }
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

    /// C1：重開後狀態還原。
    #[test]
    fn reopening_store_replays_identical_projection() {
        let path = std::env::temp_dir().join(format!(
            "ekpi-approval-c1-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let mut store = ApprovalStore::open(&path).expect("store opens");
            let definition = two_stage_definition();
            store
                .create_workflow(definition.clone(), "approver-a")
                .expect("create workflow");
            store
                .publish_workflow("wf-two", 1, "approver-a")
                .expect("publish");
            store
                .submit_task(&submission(), "editor-a", 1_000_000)
                .expect("submit");
            store
                .decide_task(
                    "TSK-000001",
                    TaskAction::Approve,
                    "approver-a",
                    "",
                    1_000_100,
                )
                .expect("decide");
            store.verify().expect("chain verifies");
        }
        let reopened = ApprovalStore::open(&path).expect("reopen");
        assert_eq!(reopened.list_workflows().len(), 1);
        assert!(reopened.list_workflows()[0].active, "publication replayed");
        assert_eq!(reopened.list_tasks().len(), 1);
        assert_eq!(
            reopened.list_tasks()[0].stage_id,
            "s2",
            "stage advance replayed"
        );
        reopened.verify().expect("reopened chain verifies");
        let _ = fs::remove_file(&path);
    }

    /// C2：竄改拒載。
    #[test]
    fn tampered_store_refuses_to_load() {
        let path = std::env::temp_dir().join(format!(
            "ekpi-approval-c2-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let mut store = ApprovalStore::open(&path).expect("store opens");
            store
                .create_workflow(two_stage_definition(), "approver-a")
                .expect("create workflow");
        }
        let mut content = fs::read_to_string(&path).expect("read store");
        content = content.replace("採購簽核", "竄改名稱");
        fs::write(&path, content).expect("tampered copy writes");
        let error = ApprovalStore::open(&path).expect_err("tampered store refuses to load");
        assert!(
            error.to_string().contains("hash mismatch"),
            "unexpected error: {error}"
        );
        let _ = fs::remove_file(&path);
    }

    /// C3：斷行拒載。
    #[test]
    fn torn_final_line_refuses_to_load() {
        let path = std::env::temp_dir().join(format!(
            "ekpi-approval-c3-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let mut store = ApprovalStore::open(&path).expect("store opens");
            store
                .create_workflow(two_stage_definition(), "approver-a")
                .expect("create workflow");
        }
        let content = fs::read_to_string(&path).expect("read store");
        fs::write(&path, content.trim_end_matches('\n')).expect("torn copy writes");
        let error = ApprovalStore::open(&path).expect_err("torn store refuses to load");
        assert!(
            error.to_string().contains("truncated"),
            "unexpected: {error}"
        );
        let _ = fs::remove_file(&path);
    }

    /// C4：事件鏈可驗證。
    #[test]
    fn event_chain_verifies_after_writes() {
        let mut store = ApprovalStore::ephemeral().expect("ephemeral store");
        let mut definition = two_stage_definition();
        definition.stages[0].escalation_path = vec!["s2".into()];
        store
            .create_workflow(definition, "approver-a")
            .expect("create");
        store
            .publish_workflow("wf-two", 1, "approver-a")
            .expect("publish");
        store
            .submit_task(&submission(), "editor-a", 1_000_000)
            .expect("submit");
        store
            .decide_task(
                "TSK-000001",
                TaskAction::Escalate,
                "approver-a",
                "需上層確認",
                1_000_100,
            )
            .expect("escalate");
        store
            .decide_task(
                "TSK-000001",
                TaskAction::Approve,
                "approver-a",
                "",
                1_000_200,
            )
            .expect("approve");
        store.verify().expect("chain verifies");
        let tasks = store.list_tasks();
        assert_eq!(tasks[0].escalation_count, 1, "escalation count persisted");
        assert_eq!(tasks[0].stage_id, "s2");
    }

    /// C5：獨佔鎖。
    #[test]
    fn second_open_on_same_path_is_refused() {
        let path = std::env::temp_dir().join(format!(
            "ekpi-approval-c5-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let _store = ApprovalStore::open(&path).expect("first open");
            let second = ApprovalStore::open(&path);
            assert!(second.is_err(), "second concurrent open must be refused");
        }
        // After the first store is dropped the lock is released.
        ApprovalStore::open(&path).expect("open after release");
        let _ = fs::remove_file(&path);
    }
}
