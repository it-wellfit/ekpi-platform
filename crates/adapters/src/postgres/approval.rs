#![forbid(unsafe_code)]

//! PostgreSQL approval repository (PG-A).
//!
//! A full implementation of [`ekp_application::ports::ApprovalRepository`] on
//! PostgreSQL 18, semantically identical to the JSONL store
//! (`crate::jsonl_approval`): the canonical state is an append-only,
//! domain-separated hash chain (`approval_events`, migration
//! `20261111000000`), and reads replay the events into an in-memory
//! projection via `ekp_domain::workflow::apply_approval_event`.
//!
//! Concurrency model: **one writer per database**. Every append takes a
//! transaction-scoped advisory lock and then verifies that the stored chain
//! head still matches this instance's mirror; a foreign append is refused
//! (fail closed) instead of forking the chain.
//!
//! Projection: `approval_tasks` is maintained inside the same transaction as
//! the event insert (state, stage, due window, escalation columns) and the
//! S10 idempotency key sits behind a unique partial index. The Phase C tables
//! `approval_workflows` / `approval_stages` are intentionally not populated:
//! definitions live inside the event chain, and their current key shape
//! (id-only primary key) cannot represent versioned definitions; a later
//! migration will reshape them if SQL-side definition queries are needed.
//!
//! This is the deployment-target store for the single-host pilot; it is not
//! an enterprise WORM service.

use async_trait::async_trait;
use ekp_application::ports::{ApprovalRepository, GovernanceError};
use ekp_domain::governance::{now_unix, validate_actor};
use ekp_domain::workflow::{
    ApprovalEvent, ApprovalEventPayload, ApprovalProjection, TaskAction, TaskDecision, TaskState,
    TaskSubmission, Urgency, WorkflowDefinition, apply_approval_event, approval_event_hash,
};
use sqlx::postgres::PgPool;

/// Advisory lock key that serializes approval-chain appends ("APRV").
const APPROVAL_ADVISORY_KEY: i64 = 0x4150_5256;

/// Predecessor hash of the first event (mirrors `jsonl_approval::GENESIS`).
const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Current unix time as i64, fail-closed on overflow (`now_unix` is u64).
fn now_i64() -> Result<i64, GovernanceError> {
    let now = now_unix()?;
    i64::try_from(now)
        .map_err(|_| GovernanceError::validation("system clock out of range".to_string()))
}

/// `TaskState` → stored text (matches the `approval_tasks.state` CHECK list).
fn state_text(state: TaskState) -> &'static str {
    match state {
        TaskState::Pending => "pending",
        TaskState::Approved => "approved",
        TaskState::Rejected => "rejected",
        TaskState::Escalated => "escalated",
        TaskState::Expired => "expired",
        TaskState::Cancelled => "cancelled",
    }
}

/// `Urgency` → stored text (matches the `approval_tasks.urgency` CHECK list).
fn urgency_text(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Low => "low",
        Urgency::Normal => "normal",
        Urgency::High => "high",
        Urgency::Critical => "critical",
    }
}

/// u32 → i32 for the INTEGER columns, fail-closed instead of wrapping.
fn int32(value: u32, field: &str) -> Result<i32, GovernanceError> {
    i32::try_from(value).map_err(|_| {
        GovernanceError::storage(format!(
            "{field} value {value} does not fit a 32-bit column"
        ))
    })
}

/// PostgreSQL approval repository: hash-chained events plus a replayed
/// projection, exactly like the JSONL store but durable in PostgreSQL.
///
/// Single instance per database: the in-memory mirror is loaded by
/// [`PgApprovalRepository::open`] and updated only by successful appends of
/// this instance. A second instance that appended elsewhere is detected by
/// the chain-head guard at the next append.
#[derive(Debug)]
pub struct PgApprovalRepository {
    pool: PgPool,
    events: Vec<ApprovalEvent>,
    projection: ApprovalProjection,
}

impl PgApprovalRepository {
    /// Open the repository over `pool`, replaying and verifying every stored
    /// event. Any sequence gap, predecessor mismatch, hash mismatch or
    /// projection violation refuses the whole load (fail closed).
    pub async fn open(pool: PgPool) -> Result<Self, GovernanceError> {
        let rows = sqlx::query!(
            r#"SELECT sequence, occurred_at, actor, payload_jsonb::text AS "payload_text!",
                      previous_sha256, sha256
               FROM approval_events ORDER BY sequence ASC"#
        )
        .fetch_all(&pool)
        .await
        .map_err(|error| GovernanceError::storage(format!("load approval events: {error}")))?;

        let mut projection = ApprovalProjection::default();
        let mut events: Vec<ApprovalEvent> = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let position = index + 1;
            let expected_sequence = position as u64;
            if row.sequence < 0 || row.sequence as u64 != expected_sequence {
                return Err(GovernanceError::validation(format!(
                    "approval chain sequence mismatch at event {position}: stored {}",
                    row.sequence
                )));
            }
            let payload: ApprovalEventPayload =
                serde_json::from_str(&row.payload_text).map_err(|error| {
                    GovernanceError::storage(format!(
                        "parse approval payload at event {position}: {error}"
                    ))
                })?;
            let event = ApprovalEvent {
                sequence: expected_sequence,
                timestamp_unix: row.occurred_at,
                actor: row.actor.clone(),
                payload,
                previous_sha256: row.previous_sha256.clone(),
                sha256: row.sha256.clone(),
            };
            let expected_previous = events
                .last()
                .map(|last| last.sha256.clone())
                .unwrap_or_else(|| GENESIS.to_string());
            if event.previous_sha256 != expected_previous {
                return Err(GovernanceError::validation(format!(
                    "approval chain predecessor mismatch at event {position}"
                )));
            }
            let expected_hash = approval_event_hash(
                event.sequence,
                event.timestamp_unix,
                &event.actor,
                &event.payload,
                &event.previous_sha256,
            )
            .map_err(GovernanceError::validation)?;
            if event.sha256 != expected_hash {
                return Err(GovernanceError::validation(format!(
                    "approval chain hash mismatch at event {position}"
                )));
            }
            validate_actor(&event.actor).map_err(|error| {
                GovernanceError::validation(format!("approval event {position}: {error}"))
            })?;
            apply_approval_event(&mut projection, &event).map_err(|error| {
                GovernanceError::validation(format!("approval event {position}: {error}"))
            })?;
            events.push(event);
        }
        Ok(Self {
            pool,
            events,
            projection,
        })
    }

    /// The underlying pool (read accessor for health checks and tests).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// A copy of the replayed projection (parity checks, future wiring).
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
            .map_err(GovernanceError::validation)?;
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

    /// Task ids number the *tasks*, not the events (JSONL parity).
    fn next_task_id(&self) -> String {
        format!("TSK-{:06}", self.projection.tasks.len() + 1)
    }

    /// Append one event: validate, hash, stage the projection, then persist
    /// the event row and the `approval_tasks` projection inside a single
    /// transaction guarded by the advisory lock and the chain-head check.
    async fn append(
        &mut self,
        actor: &str,
        payload: ApprovalEventPayload,
    ) -> Result<(), GovernanceError> {
        validate_actor(actor).map_err(|error| GovernanceError::validation(error.to_string()))?;
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
                .map_err(GovernanceError::validation)?;
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
            .map_err(|error| GovernanceError::validation(error.to_string()))?;
        let payload_value = serde_json::to_value(&event.payload).map_err(|error| {
            GovernanceError::storage(format!("encode approval payload: {error}"))
        })?;

        let mut tx =
            self.pool.begin().await.map_err(|error| {
                GovernanceError::storage(format!("begin approval append: {error}"))
            })?;
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(APPROVAL_ADVISORY_KEY)
            .execute(&mut *tx)
            .await
            .map_err(|error| {
                GovernanceError::storage(format!("approval advisory lock: {error}"))
            })?;
        let head = sqlx::query!(
            "SELECT sequence, sha256 FROM approval_events ORDER BY sequence DESC LIMIT 1"
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|error| GovernanceError::storage(format!("approval chain head: {error}")))?;
        let cached_head = self
            .events
            .last()
            .map(|last| (last.sequence, last.sha256.clone()));
        let head_matches = match (head, cached_head) {
            (None, None) => true,
            (Some(row), Some((sequence, sha256))) => {
                row.sequence == sequence as i64 && row.sha256 == sha256
            }
            _ => false,
        };
        if !head_matches {
            return Err(GovernanceError::validation(
                "approval chain head changed since this store was opened; reopen the store \
                 (single writer per database)"
                    .to_string(),
            ));
        }
        self.project_tasks(&mut tx, &next_projection, &event)
            .await?;
        sqlx::query!(
            r#"INSERT INTO approval_events
                   (sequence, occurred_at, actor, payload_jsonb, previous_sha256, sha256)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            event.sequence as i64,
            event.timestamp_unix,
            &event.actor,
            &payload_value,
            &event.previous_sha256,
            &event.sha256
        )
        .execute(&mut *tx)
        .await
        .map_err(|error| GovernanceError::storage(format!("insert approval event: {error}")))?;
        tx.commit().await.map_err(|error| {
            GovernanceError::storage(format!("commit approval append: {error}"))
        })?;
        self.projection = next_projection;
        self.events.push(event);
        Ok(())
    }

    /// Maintain the `approval_tasks` projection for task payloads; workflow
    /// payloads have no projection row (see the module docs).
    async fn project_tasks(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        projection: &ApprovalProjection,
        event: &ApprovalEvent,
    ) -> Result<(), GovernanceError> {
        match &event.payload {
            ApprovalEventPayload::TaskSubmitted {
                task,
                idempotency_key,
            } => {
                let stored = projection
                    .tasks
                    .iter()
                    .find(|candidate| candidate.id == task.id)
                    .ok_or_else(|| {
                        GovernanceError::storage(format!(
                            "projection lost task {} after replay",
                            task.id
                        ))
                    })?;
                let recommendation = stored
                    .agent_recommendation
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(|error| {
                        GovernanceError::storage(format!("encode recommendation: {error}"))
                    })?;
                sqlx::query!(
                    r#"INSERT INTO approval_tasks
                           (id, workflow_id, workflow_version, stage_id, state, urgency,
                            amount_cents, platform, classification, risk_score,
                            created_at, due_at, escalated_to, escalation_count,
                            agent_recommendation_json, idempotency_key)
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                               to_timestamp($11::bigint::double precision),
                               to_timestamp($12::bigint::double precision),
                               $13, $14, $15, $16)"#,
                    &stored.id,
                    &stored.workflow_id,
                    int32(stored.workflow_version, "workflow_version")?,
                    &stored.stage_id,
                    state_text(stored.state),
                    urgency_text(stored.urgency),
                    stored.amount_cents,
                    &stored.platform,
                    &stored.classification,
                    stored.risk_score,
                    stored.created_at,
                    stored.due_at,
                    stored.escalated_to.as_deref(),
                    int32(stored.escalation_count, "escalation_count")?,
                    recommendation.as_deref(),
                    idempotency_key.as_deref()
                )
                .execute(&mut **tx)
                .await
                .map_err(|error| GovernanceError::storage(format!("project task: {error}")))?;
            }
            ApprovalEventPayload::TaskTransitioned { decision, .. } => {
                let stored = projection
                    .tasks
                    .iter()
                    .find(|candidate| candidate.id == decision.task_id)
                    .ok_or_else(|| {
                        GovernanceError::storage(format!(
                            "projection lost task {} after replay",
                            decision.task_id
                        ))
                    })?;
                self.update_task_row(tx, stored).await?;
            }
            ApprovalEventPayload::TasksExpired { task_ids } => {
                for task_id in task_ids {
                    let stored = projection
                        .tasks
                        .iter()
                        .find(|candidate| candidate.id == *task_id)
                        .ok_or_else(|| {
                            GovernanceError::storage(format!(
                                "projection lost task {task_id} after replay"
                            ))
                        })?;
                    self.update_task_row(tx, stored).await?;
                }
            }
            ApprovalEventPayload::WorkflowCreated { .. }
            | ApprovalEventPayload::WorkflowPublished { .. } => {}
        }
        Ok(())
    }

    /// Write one replayed task's mutable columns back to `approval_tasks`.
    async fn update_task_row(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        task: &ekp_domain::workflow::ApprovalTask,
    ) -> Result<(), GovernanceError> {
        sqlx::query!(
            r#"UPDATE approval_tasks
               SET state = $2, stage_id = $3,
                   due_at = to_timestamp($4::bigint::double precision),
                   escalated_to = $5, escalation_count = $6
               WHERE id = $1"#,
            &task.id,
            state_text(task.state),
            &task.stage_id,
            task.due_at,
            task.escalated_to.as_deref(),
            int32(task.escalation_count, "escalation_count")?
        )
        .execute(&mut **tx)
        .await
        .map_err(|error| GovernanceError::storage(format!("update task projection: {error}")))?;
        Ok(())
    }
}

#[async_trait]
impl ApprovalRepository for PgApprovalRepository {
    async fn create_workflow(
        &mut self,
        definition: &WorkflowDefinition,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        definition
            .validate()
            .map_err(|error| GovernanceError::validation(error.to_string()))?;
        if self
            .projection
            .definitions
            .iter()
            .any(|stored| stored.id == definition.id && stored.version == definition.version)
        {
            return Err(GovernanceError::validation(format!(
                "workflow {}@{} already exists",
                definition.id, definition.version
            )));
        }
        self.append(
            actor,
            ApprovalEventPayload::WorkflowCreated {
                definition: definition.clone(),
            },
        )
        .await?;
        Ok(self
            .projection
            .definitions
            .last()
            .map(|stored| stored.id.clone())
            .unwrap_or_default())
    }

    async fn publish_workflow(
        &mut self,
        workflow_id: &str,
        version: u32,
        actor: &str,
    ) -> Result<(), GovernanceError> {
        let known =
            self.projection.definitions.iter().any(|stored| {
                stored.id == workflow_id && stored.version == version && !stored.active
            });
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
        .await
    }

    async fn submit_task(
        &mut self,
        input: &TaskSubmission,
        actor: &str,
        now_unix: i64,
        idempotency_key: Option<&str>,
    ) -> Result<(String, bool), GovernanceError> {
        let key = idempotency_key.map(str::trim).filter(|key| !key.is_empty());
        let key = match key {
            Some(key)
                if key.len() > 128 || !key.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) =>
            {
                return Err(GovernanceError::validation(
                    "idempotency key must be 1-128 printable ASCII characters".to_string(),
                ));
            }
            other => other,
        };
        if let Some(key) = key {
            for event in &self.events {
                if let ApprovalEventPayload::TaskSubmitted {
                    task,
                    idempotency_key: Some(existing),
                } = &event.payload
                    && existing == key
                {
                    return Ok((task.id.clone(), true));
                }
            }
        }
        let definition = self
            .projection
            .definitions
            .iter()
            .find(|stored| {
                stored.id == input.workflow_id && stored.version == input.workflow_version
            })
            .ok_or_else(|| {
                GovernanceError::validation(format!(
                    "task references workflow version {}@{} which is not in the store",
                    input.workflow_id, input.workflow_version
                ))
            })?
            .clone();
        let task_id = self.next_task_id();
        let task = ekp_domain::workflow::submit_task(&definition, input, &task_id, actor, now_unix)
            .map_err(|error| GovernanceError::validation(error.to_string()))?;
        self.append(
            actor,
            ApprovalEventPayload::TaskSubmitted {
                task,
                idempotency_key: key.map(str::to_string),
            },
        )
        .await?;
        Ok((task_id, false))
    }

    async fn decide_task(
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
            .find(|stored| stored.id == task_id)
            .ok_or_else(|| GovernanceError::validation(format!("unknown task {task_id}")))?
            .clone();
        let definition = self
            .projection
            .definitions
            .iter()
            .find(|stored| stored.id == task.workflow_id && stored.version == task.workflow_version)
            .ok_or_else(|| {
                GovernanceError::validation(format!("task {task_id} references unknown workflow"))
            })?
            .clone();
        let decision =
            ekp_domain::workflow::decide_task(&mut task, &definition, action, actor, now_unix)
                .map_err(|error| GovernanceError::validation(error.to_string()))?;
        let escalated_to = task.escalated_to.clone();
        self.append(
            actor,
            ApprovalEventPayload::TaskTransitioned {
                decision: decision.clone(),
                escalated_to,
                note: note.trim().to_string(),
                new_due_at: Some(task.due_at),
            },
        )
        .await?;
        Ok(decision)
    }

    /// SLA cleanup: for every overdue open task, fulfil its stage's
    /// `on_timeout` action (`Escalate` opens a fresh SLA window to avoid
    /// re-triggering the same expired window, `AutoReject`, `Cancel`), falling
    /// back to `Expire` when no escalation path exists. `AutoApprove` is
    /// refused at definition-validation time and is unreachable here.
    async fn cleanup_expired(&mut self, now_unix: i64) -> Result<u64, GovernanceError> {
        let overdue_ids: Vec<String> = self
            .projection
            .tasks
            .iter()
            .filter(|task| task.is_overdue(now_unix))
            .map(|task| task.id.clone())
            .collect();
        if overdue_ids.is_empty() {
            return Ok(0);
        }
        let count = overdue_ids.len() as u64;
        for task_id in overdue_ids {
            let mut task = self
                .projection
                .tasks
                .iter()
                .find(|stored| stored.id == task_id)
                .ok_or_else(|| GovernanceError::validation(format!("unknown task {task_id}")))?
                .clone();
            let definition = self
                .projection
                .definitions
                .iter()
                .find(|stored| {
                    stored.id == task.workflow_id && stored.version == task.workflow_version
                })
                .ok_or_else(|| {
                    GovernanceError::validation(format!(
                        "task {task_id} references unknown workflow"
                    ))
                })?
                .clone();
            let action = ekp_domain::workflow::timeout_transition_for(&task, &definition);
            let decision = ekp_domain::workflow::decide_task(
                &mut task,
                &definition,
                action,
                "approval-cleanup",
                now_unix,
            )
            .or_else(|error| {
                if matches!(
                    error,
                    ekp_domain::workflow::WorkflowError::NoEscalationPath { .. }
                ) {
                    ekp_domain::workflow::decide_task(
                        &mut task,
                        &definition,
                        ekp_domain::workflow::TaskAction::Expire,
                        "approval-cleanup",
                        now_unix,
                    )
                } else {
                    Err(error)
                }
            })
            .map_err(|error| GovernanceError::validation(error.to_string()))?;
            let escalated_to = task.escalated_to.clone();
            let new_due_at = if decision.action == TaskAction::Escalate {
                let sla = definition
                    .stages
                    .iter()
                    .find(|stage| stage.id == decision.from_stage)
                    .map(|stage| stage.sla_seconds as i64)
                    .unwrap_or(3600);
                Some(now_unix + sla)
            } else {
                None
            };
            self.append(
                "approval-cleanup",
                ApprovalEventPayload::TaskTransitioned {
                    decision: decision.clone(),
                    escalated_to,
                    note: "SLA timeout (cleanup)".into(),
                    new_due_at,
                },
            )
            .await?;
        }
        Ok(count)
    }

    async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>, GovernanceError> {
        Ok(self.projection.definitions.clone())
    }

    async fn list_tasks(&self) -> Result<Vec<ekp_domain::workflow::ApprovalTask>, GovernanceError> {
        Ok(self.projection.tasks.clone())
    }

    fn verify(&self) -> Result<(), GovernanceError> {
        PgApprovalRepository::verify(self)
    }
}
