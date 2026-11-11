#![forbid(unsafe_code)]

//! PostgreSQL approval repository integration tests (PG-A).
//!
//! These tests run against a live PostgreSQL when `DATABASE_URL` points at a
//! loopback server; otherwise they print a skip notice and pass vacuously
//! (the honest record is the canonical run output). Each test provisions a
//! **fresh schema** (`ekpi_pga_<tag>`), runs the real migration runner over
//! it (`sqlx::migrate!` via `ekp_adapters::postgres::migrate`), exercises the
//! repository, and leaves the schema for the next run to drop and recreate.
//! The `public` schema and any existing data are never touched.

use std::str::FromStr;

use ekp_adapters::jsonl_approval::ApprovalStore;
use ekp_adapters::postgres::approval::PgApprovalRepository;
use ekp_adapters::postgres::{PoolSettings, connect, migrate};
use ekp_application::ports::ApprovalRepository;
use ekp_domain::workflow::{
    Stage, TaskAction, TaskDecision, TaskSubmission, TimeoutAction, Urgency, WorkflowDefinition,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

struct Harness {
    repo: PgApprovalRepository,
}

/// Provision a fresh schema and open a repository over it. `None` means the
/// environment has no loopback PostgreSQL; the test then skips with a note.
async fn setup(tag: &str) -> Option<Harness> {
    let url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let options = PgConnectOptions::from_str(&url).expect("DATABASE_URL must parse");
    let host = options.get_host().to_string();
    if !matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1") {
        eprintln!(
            "skip: DATABASE_URL host {host:?} is not loopback; PostgreSQL integration tests not run"
        );
        return None;
    }
    let schema = format!("ekpi_pga_{tag}");
    assert!(
        schema
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        "test schema names must be plain identifiers"
    );
    let admin = connect(&url, &PoolSettings::default())
        .await
        .expect("DATABASE_URL is set but PostgreSQL is unreachable");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "DROP SCHEMA IF EXISTS {schema} CASCADE"
    )))
    .execute(&admin)
    .await
    .expect("drop stale test schema");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
        .execute(&admin)
        .await
        .expect("create test schema");
    let scoped = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options.options([("search_path", schema.as_str())]))
        .await
        .expect("connect schema-scoped pool");
    migrate(&scoped)
        .await
        .expect("fresh-schema migration must succeed");
    let repo = PgApprovalRepository::open(scoped)
        .await
        .expect("repository must open over a fresh schema");
    Some(Harness { repo })
}

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
                escalation_path: vec!["s2".into()],
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

async fn seed(repo: &mut PgApprovalRepository) {
    repo.create_workflow(&two_stage_definition(), "editor-a")
        .await
        .expect("create workflow");
    repo.publish_workflow("wf-two", 1, "editor-a")
        .await
        .expect("publish workflow");
}

async fn event_count(repo: &PgApprovalRepository) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM approval_events")
        .fetch_one(repo.pool())
        .await
        .expect("count events")
}

#[tokio::test]
async fn fresh_schema_migrates_and_empty_store_verifies() {
    let Some(harness) = setup("empty").await else {
        return;
    };
    let repo = &harness.repo;
    repo.verify().expect("empty chain verifies");
    assert!(repo.list_workflows().await.expect("list").is_empty());
    assert!(repo.list_tasks().await.expect("list").is_empty());
    assert_eq!(event_count(repo).await, 0);
}

#[tokio::test]
async fn full_lifecycle_persists_and_replays() {
    let Some(mut harness) = setup("lifecycle").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    let (task_id, replayed) = repo
        .submit_task(&submission(), "editor-a", 1_000_000, None)
        .await
        .expect("submit");
    assert_eq!((task_id.as_str(), replayed), ("TSK-000001", false));
    let first = repo
        .decide_task(
            "TSK-000001",
            TaskAction::Approve,
            "approver-a",
            "",
            1_000_100,
        )
        .await
        .expect("approve s1");
    assert_eq!(first.from_stage, "s1");
    assert_eq!(first.to_stage.as_deref(), Some("s2"));
    let second = repo
        .decide_task(
            "TSK-000001",
            TaskAction::Approve,
            "approver-a",
            "",
            1_000_200,
        )
        .await
        .expect("approve s2");
    assert_eq!(second.to_state, ekp_domain::workflow::TaskState::Approved);
    repo.verify().expect("chain verifies");
    assert_eq!(
        event_count(repo).await,
        5,
        "created, published, submitted, two transitions"
    );

    // Reopen from the same database: the projection must replay identically.
    let reopened = PgApprovalRepository::open(repo.pool().clone())
        .await
        .expect("reopen");
    reopened.verify().expect("reopened chain verifies");
    let before = serde_json::to_value(repo.snapshot().tasks).expect("serialize");
    let after = serde_json::to_value(reopened.snapshot().tasks).expect("serialize");
    assert_eq!(before, after, "tasks replay identically after reopen");
    let definitions_before = serde_json::to_value(repo.snapshot().definitions).expect("serialize");
    let definitions_after =
        serde_json::to_value(reopened.snapshot().definitions).expect("serialize");
    assert_eq!(definitions_before, definitions_after);

    // The approval_tasks projection row matches the replayed state.
    let state: String = sqlx::query_scalar("SELECT state FROM approval_tasks WHERE id = $1")
        .bind("TSK-000001")
        .fetch_one(repo.pool())
        .await
        .expect("projection row");
    assert_eq!(state, "approved");
}

#[tokio::test]
async fn idempotent_submission_replays_without_second_event() {
    let Some(mut harness) = setup("idem").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    let (first_id, first_replayed) = repo
        .submit_task(&submission(), "editor-a", 1_000_000, Some("key-1"))
        .await
        .expect("first submit");
    assert_eq!((first_id.as_str(), first_replayed), ("TSK-000001", false));
    let (repeat_id, repeat_replayed) = repo
        .submit_task(&submission(), "editor-a", 1_000_500, Some("key-1"))
        .await
        .expect("repeat submit");
    assert_eq!((repeat_id.as_str(), repeat_replayed), ("TSK-000001", true));
    let (trimmed_id, trimmed_replayed) = repo
        .submit_task(&submission(), "editor-a", 1_000_600, Some(" key-1 "))
        .await
        .expect("trimmed repeat");
    assert_eq!(
        (trimmed_id.as_str(), trimmed_replayed),
        ("TSK-000001", true),
        "keys are compared after trimming"
    );
    let (second_id, second_replayed) = repo
        .submit_task(&submission(), "editor-a", 1_001_000, Some("key-2"))
        .await
        .expect("second key");
    assert_eq!((second_id.as_str(), second_replayed), ("TSK-000002", false));
    let submitted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM approval_events WHERE payload_jsonb->>'type' = 'task_submitted'",
    )
    .fetch_one(repo.pool())
    .await
    .expect("count submitted events");
    assert_eq!(submitted, 2, "replays must not append a second event");
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM approval_tasks")
        .fetch_one(repo.pool())
        .await
        .expect("count task rows");
    assert_eq!(rows, 2);
    let key: Option<String> =
        sqlx::query_scalar("SELECT idempotency_key FROM approval_tasks WHERE id = $1")
            .bind("TSK-000001")
            .fetch_one(repo.pool())
            .await
            .expect("key row");
    assert_eq!(key.as_deref(), Some("key-1"));
    let rejected = repo
        .submit_task(&submission(), "editor-a", 1_002_000, Some("bad\nkey"))
        .await
        .expect_err("non-printable keys are refused");
    assert!(rejected.to_string().contains("printable ASCII"));
}

#[tokio::test]
async fn cleanup_honors_on_timeout_actions() {
    let Some(mut harness) = setup("cleanup").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    // Third workflow: Escalate configured, but no escalation path → fallback.
    let mut no_path = two_stage_definition();
    no_path.id = "wf-noesc".into();
    no_path.stages[0].escalation_path = vec![];
    repo.create_workflow(&no_path, "editor-a")
        .await
        .expect("create wf-noesc");
    repo.publish_workflow("wf-noesc", 1, "editor-a")
        .await
        .expect("publish wf-noesc");

    let now = 1_000_000;
    // Task 1: overdue on s1 (Escalate with path).
    repo.submit_task(&submission(), "editor-a", now - 7_200, None)
        .await
        .expect("task 1");
    // Task 2: advanced to s2, overdue there (AutoReject).
    repo.submit_task(&submission(), "editor-a", now - 200_000, None)
        .await
        .expect("task 2");
    repo.decide_task(
        "TSK-000002",
        TaskAction::Approve,
        "approver-a",
        "",
        now - 199_000,
    )
    .await
    .expect("task 2 to s2");
    // Task 3: overdue on wf-noesc s1 (Escalate without path → Expire).
    let mut input = submission();
    input.workflow_id = "wf-noesc".into();
    repo.submit_task(&input, "editor-a", now - 7_200, None)
        .await
        .expect("task 3");

    let acted = repo.cleanup_expired(now).await.expect("cleanup");
    assert_eq!(acted, 3);
    let tasks = repo.list_tasks().await.expect("tasks");
    let task1 = tasks.iter().find(|t| t.id == "TSK-000001").expect("task 1");
    assert_eq!(task1.state, ekp_domain::workflow::TaskState::Escalated);
    assert_eq!(task1.escalated_to.as_deref(), Some("s2"));
    assert_eq!(task1.escalation_count, 1);
    assert_eq!(
        task1.due_at,
        now + 3600,
        "escalation opens a fresh SLA window"
    );
    let task2 = tasks.iter().find(|t| t.id == "TSK-000002").expect("task 2");
    assert_eq!(task2.state, ekp_domain::workflow::TaskState::Rejected);
    let task3 = tasks.iter().find(|t| t.id == "TSK-000003").expect("task 3");
    assert_eq!(
        task3.state,
        ekp_domain::workflow::TaskState::Expired,
        "Escalate without a path falls back to Expire"
    );
    // Seconds-level stability of the TIMESTAMPTZ projection round-trip.
    let due_epoch: i64 = sqlx::query_scalar(
        "SELECT (EXTRACT(EPOCH FROM due_at))::bigint FROM approval_tasks WHERE id = $1",
    )
    .bind("TSK-000001")
    .fetch_one(repo.pool())
    .await
    .expect("due epoch");
    assert_eq!(due_epoch, now + 3600);
    assert_eq!(
        repo.cleanup_expired(now + 10).await.expect("idempotent"),
        0,
        "a fulfilled window must not re-trigger"
    );
}

#[tokio::test]
async fn tampered_event_refuses_to_open() {
    let Some(mut harness) = setup("tamper").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    repo.submit_task(&submission(), "editor-a", 1_000_000, None)
        .await
        .expect("submit");

    // The append-only trigger refuses in-place edits.
    let blocked = sqlx::query("UPDATE approval_events SET occurred_at = 0 WHERE sequence = 1")
        .execute(repo.pool())
        .await
        .expect_err("append-only trigger must refuse UPDATE");
    assert!(blocked.to_string().contains("append-only"), "{blocked}");

    // A tamper that bypasses the trigger is still caught by the hash chain.
    sqlx::query("ALTER TABLE approval_events DISABLE TRIGGER approval_events_append_only")
        .execute(repo.pool())
        .await
        .expect("disable trigger");
    sqlx::query("UPDATE approval_events SET occurred_at = occurred_at + 1 WHERE sequence = 2")
        .execute(repo.pool())
        .await
        .expect("tamper");
    sqlx::query("ALTER TABLE approval_events ENABLE TRIGGER approval_events_append_only")
        .execute(repo.pool())
        .await
        .expect("re-enable trigger");
    let refused = PgApprovalRepository::open(repo.pool().clone())
        .await
        .expect_err("tampered chain must refuse to open");
    assert!(refused.to_string().contains("hash mismatch"), "{refused}");
}

#[tokio::test]
async fn deleted_event_refuses_to_open() {
    let Some(mut harness) = setup("gap").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    repo.submit_task(&submission(), "editor-a", 1_000_000, None)
        .await
        .expect("submit");
    sqlx::query("ALTER TABLE approval_events DISABLE TRIGGER approval_events_append_only")
        .execute(repo.pool())
        .await
        .expect("disable trigger");
    sqlx::query("DELETE FROM approval_events WHERE sequence = 2")
        .execute(repo.pool())
        .await
        .expect("delete middle event");
    sqlx::query("ALTER TABLE approval_events ENABLE TRIGGER approval_events_append_only")
        .execute(repo.pool())
        .await
        .expect("re-enable trigger");
    let refused = PgApprovalRepository::open(repo.pool().clone())
        .await
        .expect_err("sequence gap must refuse to open");
    assert!(
        refused.to_string().contains("sequence mismatch"),
        "{refused}"
    );
}

#[tokio::test]
async fn stale_second_writer_is_refused() {
    let Some(mut harness) = setup("stale").await else {
        return;
    };
    let repo = &mut harness.repo;
    seed(repo).await;
    let mut stale = PgApprovalRepository::open(repo.pool().clone())
        .await
        .expect("second store");
    repo.submit_task(&submission(), "editor-a", 1_000_000, None)
        .await
        .expect("first writer appends");
    let refused = stale
        .submit_task(&submission(), "editor-b", 1_000_500, None)
        .await
        .expect_err("stale writer must be refused");
    assert!(refused.to_string().contains("head changed"), "{refused}");
    // Reopening refreshes the mirror and the writer recovers.
    let mut refreshed = PgApprovalRepository::open(repo.pool().clone())
        .await
        .expect("reopen");
    let (id, _) = refreshed
        .submit_task(&submission(), "editor-b", 1_001_000, None)
        .await
        .expect("refreshed writer appends");
    assert_eq!(id, "TSK-000002");
}

#[tokio::test]
async fn jsonl_parity_on_identical_scenarios() {
    let Some(mut harness) = setup("parity").await else {
        return;
    };
    let pg = &mut harness.repo;
    let mut jsonl = ApprovalStore::ephemeral().expect("jsonl store");
    let definition = two_stage_definition();

    // Identical operation sequence against both stores, same time inputs.
    pg.create_workflow(&definition, "editor-a")
        .await
        .expect("pg create");
    jsonl
        .create_workflow(definition.clone(), "editor-a")
        .expect("jsonl create");
    pg.publish_workflow("wf-two", 1, "editor-a")
        .await
        .expect("pg publish");
    jsonl
        .publish_workflow("wf-two", 1, "editor-a")
        .expect("jsonl publish");
    let (pg_id, pg_replay) = pg
        .submit_task(&submission(), "editor-a", 1_000_000, Some("par-1"))
        .await
        .expect("pg submit");
    let (jsonl_id, jsonl_replay) = jsonl
        .submit_task(&submission(), "editor-a", 1_000_000, Some("par-1"))
        .expect("jsonl submit");
    assert_eq!(
        (pg_id.as_str(), pg_replay),
        (jsonl_id.as_str(), jsonl_replay)
    );
    let (pg_replay_id, pg_replayed) = pg
        .submit_task(&submission(), "editor-a", 1_000_400, Some("par-1"))
        .await
        .expect("pg replay");
    let (jsonl_replay_id, jsonl_replayed) = jsonl
        .submit_task(&submission(), "editor-a", 1_000_400, Some("par-1"))
        .expect("jsonl replay");
    assert_eq!(
        (pg_replay_id.as_str(), pg_replayed),
        (jsonl_replay_id.as_str(), jsonl_replayed)
    );
    let pg_decision: TaskDecision = pg
        .decide_task(
            "TSK-000001",
            TaskAction::Approve,
            "approver-a",
            "ok",
            1_000_100,
        )
        .await
        .expect("pg approve");
    let jsonl_decision = jsonl
        .decide_task(
            "TSK-000001",
            TaskAction::Approve,
            "approver-a",
            "ok",
            1_000_100,
        )
        .expect("jsonl approve");
    assert_eq!(pg_decision, jsonl_decision);
    // A second, overdue task exercised through cleanup on both stores.
    pg.submit_task(&submission(), "editor-a", 1_000_000, None)
        .await
        .expect("pg task 2");
    jsonl
        .submit_task(&submission(), "editor-a", 1_000_000, None)
        .expect("jsonl task 2");
    let pg_acted = pg.cleanup_expired(1_007_200).await.expect("pg cleanup");
    let jsonl_acted = jsonl.cleanup_expired(1_007_200).expect("jsonl cleanup");
    assert_eq!(pg_acted, jsonl_acted);

    let pg_tasks = serde_json::to_value(pg.snapshot().tasks).expect("serialize");
    let jsonl_tasks = serde_json::to_value(jsonl.snapshot().tasks).expect("serialize");
    assert_eq!(pg_tasks, jsonl_tasks, "task projections must be identical");
    let pg_definitions = serde_json::to_value(pg.snapshot().definitions).expect("serialize");
    let jsonl_definitions = serde_json::to_value(jsonl.snapshot().definitions).expect("serialize");
    assert_eq!(pg_definitions, jsonl_definitions);
}
