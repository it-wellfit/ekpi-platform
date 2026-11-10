#![forbid(unsafe_code)]

//! CSS-first, server-rendered local pilot workbench.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use getrandom::getrandom;
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use serde_urlencoded;
use tokio::sync::Semaphore;

use crate::governance::{
    GovernanceSnapshot, Ledger, Project, QUALITY_CHECKLIST, QualityCheckResult,
    QualityInspectionInput, Workflow, WorkflowDraftInput,
};
use crate::local_auth::{AuthStore, Role};

type SharedLedger = Arc<Mutex<Ledger>>;
type SharedAuth = Arc<Mutex<AuthStore>>;

/// Shared approval store (separate bounded context from the governance
/// ledger; own JSONL hash chain in `ekp_adapters::jsonl_approval`).
pub type SharedApprovals = Arc<Mutex<ekp_adapters::jsonl_approval::ApprovalStore>>;

const MAX_FORM_BYTES: usize = 64 * 1024;
const MAX_UPLOAD_BYTES: usize = 1024 * 1024; // 1MB for file uploads
const MAX_LOGIN_BYTES: usize = 4 * 1024; // 4KB for login forms
const TOKEN_BYTES: usize = 32;
const HEX: &[u8; 16] = b"0123456789abcdef";

/// Generate a random hex string of the given length using the OS CSPRNG.
fn random_hex(length: usize) -> Result<String, String> {
    let mut bytes = vec![0u8; length];
    getrandom(&mut bytes).map_err(|e| format!("random source unavailable: {e}"))?;
    Ok(to_hex(&bytes))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

/// Get the maximum form body size for a given route path.
fn max_form_bytes_for_path(path: &str) -> usize {
    if path.starts_with("/login") {
        MAX_LOGIN_BYTES
    } else if path.contains("/upload") || path.contains("/import") {
        MAX_UPLOAD_BYTES
    } else {
        MAX_FORM_BYTES
    }
}
const RATE_LIMIT_PER_MINUTE: u32 = 30;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

const CSS: &str = include_str!("../assets/governance.css");
const REQUEST_ID_HEADER: &str = "x-request-id";
const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_CONCURRENT_REQUESTS: usize = 64;
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static RESPONSES_4XX_TOTAL: AtomicU64 = AtomicU64::new(0);
static RESPONSES_5XX_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUESTS_IN_FLIGHT: AtomicU64 = AtomicU64::new(0);
static REQUEST_DURATION_MICROS_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUEST_DURATION_COUNT_TOTAL: AtomicU64 = AtomicU64::new(0);
static RATE_LIMITED_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUEST_LIMIT: LazyLock<Semaphore> =
    LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_REQUESTS));

/// Simple in-memory rate limiter for IP-based request throttling.
#[derive(Debug)]
struct RateLimiter {
    requests: std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn check_rate_limit(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = std::time::Instant::now();
        let entry = requests.entry(ip.to_string()).or_insert((0, now));

        if now.duration_since(entry.1).as_secs() >= RATE_LIMIT_WINDOW_SECS {
            *entry = (1, now);
            true
        } else if entry.0 < RATE_LIMIT_PER_MINUTE {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

static RATE_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);

pub fn router(ledger: SharedLedger, auth: SharedAuth) -> Router {
    // Test convenience: an ephemeral (temp-file) approval store. The production
    // binary must open the configured path and call `router_with_approvals`.
    let approvals = std::sync::Arc::new(std::sync::Mutex::new(
        ekp_adapters::jsonl_approval::ApprovalStore::ephemeral()
            .expect("ephemeral approval store opens"),
    ));
    router_with_approvals(ledger, auth, approvals)
}

/// Full router with an explicit approval store. The approval routes form a
/// sub-router over `(SharedAuth, SharedApprovals)` — different state from the
/// governance core — merged before the shared middleware stack.
pub fn router_with_approvals(
    ledger: SharedLedger,
    auth: SharedAuth,
    approvals: SharedApprovals,
) -> Router {
    let auth_for_guard = auth.clone();
    let auth_for_csrf = auth.clone();
    let auth_for_approvals = auth.clone();
    let auth_for_map = auth.clone();
    let auth_for_pages = auth.clone();
    let ledger_for_map = ledger.clone();
    let approvals_for_map = approvals.clone();

    let core = Router::new()
        .route("/ready", get(ready_check))
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .route("/", get(dashboard))
        .route("/login", get(login_page).post(login_post))
        .route("/logout", post(logout_post))
        .route("/projects", get(projects_page).post(create_project))
        .route("/projects/{id}", get(project_detail_page))
        .route("/evidence", get(evidence_page).post(link_evidence))
        .route("/evidence/{id}", get(evidence_detail_page))
        .route("/evidence/{id}/review", post(review_evidence))
        .route(
            "/inspections",
            get(inspections_page).post(record_inspection),
        )
        .route("/workflows", get(workflows_page).post(save_workflow))
        .route("/workflows/{id}/{version}/publish", post(publish_workflow))
        .route("/controls", get(controls_page).post(create_quality_case))
        .route("/controls/{id}", get(controls_detail_page))
        .route("/controls/{id}/status", post(update_quality_case))
        .route("/lifecycle", get(lifecycle_page).post(record_gate_decision))
        .route("/audit", get(audit_page))
        .route("/events", get(events_page))
        .route("/search", get(search_page))
        .route("/settings", get(settings_page))
        .route("/api-docs", get(api_docs_page))
        .route("/assets/governance.css", get(stylesheet))
        // ── API v1：唯讀核准端點已接線（見 docs/contracts/approval-api.md），
        //    其餘未接線一律誠實 501 ──
        .route("/api/v1/events/replay", post(api_replay_event))
        .route("/api/v1/channels", get(api_list_channels))
        .route("/api/v1/channels/health", get(api_channel_health))
        .route("/api/v1/users", get(api_list_users))
        .route("/api/v1/settings", get(api_get_settings))
        .route("/api/v1/search", get(api_search))
        .with_state((ledger, auth));

    let approval_routes = Router::new()
        .route("/approvals", get(approvals_page))
        .route("/approvals/calendar", get(calendar_page))
        .route("/approvals/workflows", post(create_approval_workflow))
        .route(
            "/approvals/workflows/{id}/{version}/publish",
            post(publish_approval_workflow),
        )
        .route("/approvals/submit", post(submit_approval_task))
        .route("/approvals/{id}/decide", post(decide_approval_task))
        // API v1 唯讀端點（真資料；寫入仍 501）
        .route(
            "/api/v1/workflows",
            get(api_workflows_list).post(api_create_workflow),
        )
        .route("/api/v1/workflows/{id}/publish", post(api_publish_workflow))
        .route(
            "/api/v1/approvals",
            get(api_approvals_list).post(api_submit_task),
        )
        .route("/api/v1/approvals/{id}/decide", post(api_decide_task))
        .route("/api/v1/approvals/{id}/escalate", post(api_escalate_task))
        .with_state((auth_for_approvals, approvals));

    let data_map_routes = Router::new()
        .route("/data-map", get(data_map_page))
        .route("/audit/chains", get(chains_summary_page))
        .with_state((ledger_for_map, auth_for_map, approvals_for_map));

    let info_routes = Router::new()
        .route("/role-matrix", get(role_matrix_page))
        .route("/glossary", get(glossary_page))
        .route("/help", get(help_page))
        .route("/about", get(about_page))
        .with_state((auth_for_pages.clone(),));

    core.merge(approval_routes)
        .merge(data_map_routes)
        .merge(info_routes)
        .layer(middleware::from_fn_with_state(
            auth_for_guard,
            require_session,
        ))
        .layer(middleware::from_fn_with_state(
            auth_for_csrf,
            validate_csrf_token,
        ))
        .layer(middleware::from_fn(rate_limit_guard))
        .layer(middleware::from_fn(same_origin_guard))
        .layer(middleware::from_fn(request_context))
}

// ── API v1 handlers ──────────────────────────────────────────────────
//
// 誠實性原則：本組端點均未接線（無 use case、無儲存後端），因此一律回
// 501 NOT_IMPLEMENTED 與結構化錯誤，絕不回傳佔位假資料（空陣列/常數）。
// 真實契約（含認證機制）將於再設計總計畫 S4 定義後實作；在此之前，
// HTML 工作台是唯一已接線的變更介面。

fn api_not_implemented(endpoint: &str) -> Response {
    crate::errors::error_response(
        format!("endpoint not implemented: {endpoint}"),
        StatusCode::NOT_IMPLEMENTED,
        None,
    )
    .into_response()
}

async fn api_workflows_list(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
) -> Response {
    let workflows = match approvals_view(&approvals) {
        Ok((_, workflows, _, _)) => workflows,
        Err(error) => {
            return crate::errors::error_response(error, StatusCode::INTERNAL_SERVER_ERROR, None)
                .into_response();
        }
    };
    let entries: Vec<serde_json::Value> = workflows
        .iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "version": d.version,
                "name": d.name,
                "active": d.active,
                "stages": d.stages.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "sla_seconds": s.sla_seconds,
                    "urgency": s.urgency,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema": "ekpi-approval-workflows-v1",
            "workflows": entries,
        })),
    )
        .into_response()
}

async fn api_create_workflow() -> Response {
    api_not_implemented("POST /api/v1/workflows")
}

async fn api_publish_workflow() -> Response {
    api_not_implemented("POST /api/v1/workflows/{id}/publish")
}

/// GET /api/v1/approvals：真資料（已接線；契約 CTR-API-001）。
async fn api_approvals_list(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
) -> Response {
    let tasks = match approvals_view(&approvals) {
        Ok((tasks, _, _, _)) => tasks,
        Err(error) => {
            return crate::errors::error_response(error, StatusCode::INTERNAL_SERVER_ERROR, None)
                .into_response();
        }
    };
    let entries: Vec<serde_json::Value> = tasks
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "workflow_id": t.workflow_id,
                "workflow_version": t.workflow_version,
                "stage_id": t.stage_id,
                "state": t.state,
                "urgency": t.urgency,
                "amount_cents": t.amount_cents,
                "platform": t.platform,
                "classification": t.classification,
                "risk_score": t.risk_score,
                "created_at": t.created_at,
                "due_at": t.due_at,
                "escalated_to": t.escalated_to,
                "escalation_count": t.escalation_count,
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema": "ekpi-approval-tasks-v1",
            "tasks": entries,
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct ApiSubmitTaskBody {
    workflow_ref: String,
    amount_cents: i64,
    platform: String,
    classification: String,
    risk_score: i32,
    urgency: String,
}

/// POST /api/v1/approvals — JSON write path with optional idempotency
/// (`Idempotency-Key` header, 1–128 printable ASCII).
///
/// Responses: `201 Created` (new task, `Location` header), `200 OK` when the
/// key replays (`Idempotency-Replayed: true`, same task id, **no second
/// event**), structured `400` for malformed body/key, `403` for
/// role/CSRF refusals, `422` for domain refusals (e.g. unpublished
/// workflow). Session actor is the submitter; form-free by design.
async fn api_submit_task(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    request: Request,
) -> Response {
    let headers = request.headers().clone();
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok());
    let (username, _role) = match session_identity_full(&_auth, &headers, Role::Editor) {
        Ok(identity) => identity,
        Err(response) => return *response,
    };
    let body_bytes = match read_limited_body(request).await {
        Ok(bytes) => bytes,
        Err(response) => return *response,
    };
    let body: ApiSubmitTaskBody = match serde_json::from_slice(&body_bytes) {
        Ok(body) => body,
        Err(error) => {
            return crate::errors::error_response(
                format!("malformed JSON body: {error}"),
                StatusCode::BAD_REQUEST,
                request_id,
            )
            .into_response();
        }
    };
    let idempotency_key: Option<String> = match headers.get(IDEMPOTENCY_KEY_HEADER) {
        None => None,
        Some(value) => match value.to_str() {
            Ok(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else if trimmed.len() > 128
                    || !trimmed.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
                {
                    return crate::errors::error_response(
                        "idempotency key must be 1-128 printable ASCII characters",
                        StatusCode::BAD_REQUEST,
                        request_id,
                    )
                    .into_response();
                } else {
                    Some(trimmed.to_string())
                }
            }
            Err(_) => {
                return crate::errors::error_response(
                    "idempotency key must be ASCII",
                    StatusCode::BAD_REQUEST,
                    request_id,
                )
                .into_response();
            }
        },
    };
    let (workflow_id, version) = match body.workflow_ref.split_once('@') {
        Some((id, version)) => match version.trim().parse::<u32>() {
            Ok(version) => (id.trim().to_string(), version),
            Err(_) => {
                return crate::errors::error_response(
                    "workflow version must be a number",
                    StatusCode::BAD_REQUEST,
                    request_id,
                )
                .into_response();
            }
        },
        None => {
            return crate::errors::error_response(
                "workflow_ref must be id@version",
                StatusCode::BAD_REQUEST,
                request_id,
            )
            .into_response();
        }
    };
    let urgency = match parse_urgency(&body.urgency) {
        Some(urgency) => urgency,
        None => {
            return crate::errors::error_response(
                "urgency must be one of low, normal, high, critical",
                StatusCode::BAD_REQUEST,
                request_id,
            )
            .into_response();
        }
    };
    let now = match now_i64() {
        Ok(now) => now,
        Err(error) => {
            return crate::errors::error_response(
                error,
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
            )
            .into_response();
        }
    };
    let input = ekp_domain::workflow::TaskSubmission {
        workflow_id,
        workflow_version: version,
        amount_cents: body.amount_cents,
        platform: body.platform.trim().to_string(),
        classification: body.classification.trim().to_string(),
        risk_score: body.risk_score,
        urgency,
    };
    let outcome = match approvals.lock() {
        Ok(mut store) => store
            .submit_task(&input, &username, now, idempotency_key.as_deref())
            .map_err(|e| e.to_string()),
        Err(_) => Err("核准資料鎖定失敗".to_string()),
    };
    match outcome {
        Ok((task_id, replayed)) => {
            let status = if replayed {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            let mut response = (
                status,
                Json(serde_json::json!({
                    "schema": "ekpi-approval-submit-v1",
                    "task_id": task_id,
                    "replayed": replayed,
                })),
            )
                .into_response();
            if let Ok(location) = HeaderValue::from_str(&format!("/api/v1/approvals/{task_id}")) {
                response.headers_mut().insert(header::LOCATION, location);
            }
            if replayed {
                response.headers_mut().insert(
                    HeaderName::from_static("idempotency-replayed"),
                    HeaderValue::from_static("true"),
                );
            }
            response
        }
        Err(message) => {
            crate::errors::error_response(message, StatusCode::UNPROCESSABLE_ENTITY, request_id)
                .into_response()
        }
    }
}

async fn api_decide_task() -> Response {
    api_not_implemented("POST /api/v1/approvals/{id}/decide")
}

async fn api_escalate_task() -> Response {
    api_not_implemented("POST /api/v1/approvals/{id}/escalate")
}

async fn api_replay_event() -> Response {
    api_not_implemented("POST /api/v1/events/replay")
}

async fn api_list_channels() -> Response {
    api_not_implemented("GET /api/v1/channels")
}

async fn api_channel_health() -> Response {
    api_not_implemented("GET /api/v1/channels/health")
}

async fn api_list_users() -> Response {
    api_not_implemented("GET /api/v1/users")
}

async fn api_get_settings() -> Response {
    api_not_implemented("GET /api/v1/settings")
}

async fn api_search() -> Response {
    api_not_implemented("GET /api/v1/search")
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ekpi-governance-web",
        "version": env!("CARGO_PKG_VERSION"),
        "scope": "local-only",
        "readiness": "liveness-only",
    }))
}

async fn ready_check(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
) -> impl IntoResponse {
    let ready_status = crate::ready::check_readiness(&ledger, &auth);
    let status_code = match ready_status.status {
        crate::ready::ReadyStatusType::Ready => StatusCode::OK,
        crate::ready::ReadyStatusType::Degraded => StatusCode::OK,
        crate::ready::ReadyStatusType::NotReady => StatusCode::SERVICE_UNAVAILABLE,
        crate::ready::ReadyStatusType::Checking => StatusCode::OK,
    };
    (
        status_code,
        Json(serde_json::to_value(ready_status).unwrap_or_default()),
    )
}

async fn metrics() -> impl IntoResponse {
    let requests = REQUESTS_TOTAL.load(Ordering::Relaxed);
    let responses_4xx = RESPONSES_4XX_TOTAL.load(Ordering::Relaxed);
    let responses_5xx = RESPONSES_5XX_TOTAL.load(Ordering::Relaxed);
    let in_flight = REQUESTS_IN_FLIGHT.load(Ordering::Relaxed);
    let rate_limited = RATE_LIMITED_TOTAL.load(Ordering::Relaxed);
    let duration_seconds =
        REQUEST_DURATION_MICROS_TOTAL.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let duration_count = REQUEST_DURATION_COUNT_TOTAL.load(Ordering::Relaxed);
    let body = format!(
        "# HELP ekpi_http_requests_total Total HTTP requests handled by the local workbench.\n\
         # TYPE ekpi_http_requests_total counter\n\
         ekpi_http_requests_total {requests}\n\
         # HELP ekpi_http_responses_4xx_total Responses with a 4xx status.\n\
         # TYPE ekpi_http_responses_4xx_total counter\n\
         ekpi_http_responses_4xx_total {responses_4xx}\n\
         # HELP ekpi_http_responses_5xx_total Responses with a 5xx status.\n\
         # TYPE ekpi_http_responses_5xx_total counter\n\
         ekpi_http_responses_5xx_total {responses_5xx}\n\
         # HELP ekpi_http_requests_in_flight Requests currently being handled.\n\
         # TYPE ekpi_http_requests_in_flight gauge\n\
         ekpi_http_requests_in_flight {in_flight}\n\
         # HELP ekpi_http_rate_limited_total Requests refused by the concurrency bound.\n\
         # TYPE ekpi_http_rate_limited_total counter\n\
         ekpi_http_rate_limited_total {rate_limited}\n\
         # HELP ekpi_http_request_duration_seconds_sum Total request handling time.\n\
         # TYPE ekpi_http_request_duration_seconds_sum counter\n\
         ekpi_http_request_duration_seconds_sum {duration_seconds}\n\
         # HELP ekpi_http_request_duration_seconds_count Requests included in the duration sum.\n\
         # TYPE ekpi_http_request_duration_seconds_count counter\n\
         ekpi_http_request_duration_seconds_count {duration_count}\n"
    );
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

async fn stylesheet() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], CSS)
}

async fn request_context(request: Request, next: Next) -> Response {
    request_context_with(request, next, REQUEST_TIMEOUT, &REQUEST_LIMIT).await
}

async fn request_context_with(
    mut request: Request,
    next: Next,
    timeout: Duration,
    limit: &Semaphore,
) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_request_id(value))
        .map(str::to_owned)
        .unwrap_or_else(generate_request_id);
    // Propagate the resolved id so inner middleware layers can correlate
    // their structured error bodies with the same request.
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        request
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started = Instant::now();

    REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    REQUESTS_IN_FLIGHT.fetch_add(1, Ordering::Relaxed);
    let permit = limit.try_acquire();

    let mut response = if permit.is_err() {
        RATE_LIMITED_TOTAL.fetch_add(1, Ordering::Relaxed);
        let mut response = crate::errors::error_response_with_context(
            "too many concurrent requests",
            StatusCode::TOO_MANY_REQUESTS,
            Some(serde_json::json!({
                "limit": REQUEST_LIMIT.available_permits(),
                "recommendation": "Retry after the in-flight request completes.",
            })),
            Some(&request_id),
        )
        .into_response();
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        response
    } else {
        match tokio::time::timeout(timeout, next.run(request)).await {
            Ok(response) => response,
            Err(_) => {
                let mut response = crate::errors::error_response_with_context(
                    "request timed out",
                    StatusCode::GATEWAY_TIMEOUT,
                    Some(serde_json::json!({
                        "timeout_ms": timeout.as_millis() as u64,
                        "recommendation": "Reduce the workload or raise the configured timeout.",
                    })),
                    Some(&request_id),
                )
                .into_response();
                response
                    .headers_mut()
                    .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
                response
            }
        }
    };
    drop(permit);

    let elapsed = started.elapsed();
    let micros = elapsed.as_micros().min(u64::MAX as u128) as u64;
    REQUEST_DURATION_MICROS_TOTAL.fetch_add(micros, Ordering::Relaxed);
    REQUEST_DURATION_COUNT_TOTAL.fetch_add(1, Ordering::Relaxed);
    REQUESTS_IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
    if response.status().is_server_error() {
        RESPONSES_5XX_TOTAL.fetch_add(1, Ordering::Relaxed);
    } else if response.status().is_client_error() {
        RESPONSES_4XX_TOTAL.fetch_add(1, Ordering::Relaxed);
    }

    apply_security_headers(response.headers_mut(), &request_id);
    println!(
        "{}",
        serde_json::json!({
            "event": "http_request",
            "request_id": request_id,
            "method": method.as_str(),
            "path": path,
            "status": response.status().as_u16(),
            "duration_ms": elapsed.as_secs_f64() * 1000.0,
        })
    );
    response
}

fn session_token(headers: &HeaderMap) -> Option<String> {
    // Duplicate Cookie header lines are ambiguous: a proxy or client may have
    // appended a second line, and selecting a session from either one would
    // hide the ambiguity. Refuse the whole request instead.
    let mut header_values = headers.get_all(header::COOKIE).iter();
    let header = header_values.next()?;
    if header_values.next().is_some() {
        return None;
    }
    let cookie = header.to_str().ok()?;
    let mut token: Option<String> = None;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("ekpi_session=") {
            if value.is_empty() || token.is_some() {
                return None;
            }
            token = Some(value.to_string());
        }
    }
    token
}

fn unauthorized_with_id(request_id: &str) -> Response {
    crate::errors::error_response(
        "authentication required",
        StatusCode::UNAUTHORIZED,
        Some(request_id),
    )
    .into_response()
}

/// Shared 400 for unreadable form bodies (C5: one branch, three callers).
fn malformed_form(error: impl std::fmt::Display) -> Response {
    (StatusCode::BAD_REQUEST, format!("無法解析表單：{error}")).into_response()
}

/// Structured 403 used by middleware layers that know the request id.
fn forbidden_with_id(request_id: &str, message: &str) -> Response {
    crate::errors::error_response(message, StatusCode::FORBIDDEN, Some(request_id)).into_response()
}

/// Extract the `ekpi_csrf` cookie with the same fail-closed ambiguity policy
/// as `session_token`: duplicate Cookie header lines, a duplicate pair or an
/// empty value refuse the whole request instead of picking one silently.
fn csrf_cookie_value(headers: &HeaderMap) -> Option<String> {
    let mut header_values = headers.get_all(header::COOKIE).iter();
    let header = header_values.next()?;
    if header_values.next().is_some() {
        return None;
    }
    let cookie = header.to_str().ok()?;
    let mut token: Option<String> = None;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("ekpi_csrf=") {
            if value.is_empty() || token.is_some() {
                return None;
            }
            token = Some(value.to_string());
        }
    }
    token
}

/// Validate the session cookie, require at least the given role, and return
/// the local session username. Mutating handlers use this value as the ledger
/// actor instead of trusting a form field.
const CSRF_TOKEN_NAME: &str = "X-CSRF-Token";

async fn validate_csrf_token(
    State(auth): State<SharedAuth>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method();
    if method == Method::GET || method == Method::HEAD {
        return next.run(request).await;
    }
    // Login is exempt here because it has no session to bind a token to yet;
    // it enforces its own double-submit check (fresh token in the hidden form
    // field mirrored by the `ekpi_csrf` cookie) inside `login_post`.
    if method == Method::POST && request.uri().path() == "/login" {
        return next.run(request).await;
    }

    let headers = request.headers();
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");
    let session_token = match session_token(headers) {
        Some(token) => token,
        None => {
            return unauthorized_with_id(request_id);
        }
    };

    let session = {
        let auth_store = auth.lock().map_err(|_| {
            crate::errors::error_response(
                "auth store locked",
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(request_id),
            )
            .into_response()
        });
        let Some(auth_store) = auth_store.ok() else {
            return crate::errors::error_response(
                "auth store error",
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(request_id),
            )
            .into_response();
        };
        match auth_store.validate_session(&session_token).cloned() {
            Some(session) => session,
            None => {
                return unauthorized_with_id(request_id);
            }
        }
    };

    // Zero-JS HTML forms cannot set custom headers, so the session CSRF token
    // is accepted from the `X-CSRF-Token` header (programmatic clients) or the
    // `ekpi_csrf` cookie (server-rendered forms). The browser attaches the
    // cookie to same-site POSTs automatically, and `SameSite=Strict` keeps
    // cross-site requests from sending it at all.
    let csrf_header = headers
        .get(CSRF_TOKEN_NAME)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let csrf_cookie = csrf_cookie_value(headers).unwrap_or_default();

    if csrf_header != session.csrf_token && csrf_cookie != session.csrf_token {
        return forbidden_with_id(request_id, "Invalid CSRF token");
    }

    next.run(request).await
}

/// Validate the session cookie and require at least the given role.
/// Returns `Some(Response)` when the request must be rejected.
fn check_session_with_role(
    auth: &SharedAuth,
    headers: &HeaderMap,
    required: Role,
) -> Option<Response> {
    session_identity(auth, headers, required)
        .err()
        .map(|response| *response)
}

fn is_public_path(path: &str) -> bool {
    matches!(path, "/healthz" | "/metrics" | "/login") || path.starts_with("/assets/")
}

/// Validate the session cookie, require at least the given role, and return
/// the local session username. Mutating handlers use this value as the ledger
/// actor instead of trusting a form field.
fn session_identity(
    auth: &SharedAuth,
    headers: &HeaderMap,
    required: Role,
) -> std::result::Result<String, Box<Response>> {
    session_identity_full(auth, headers, required).map(|(username, _)| username)
}

/// Like [`session_identity`] but also returns the validated role, for the few
/// handlers that must map the local role into another vocabulary (e.g. the
/// retrieval permission context).
fn session_identity_full(
    auth: &SharedAuth,
    headers: &HeaderMap,
    required: Role,
) -> std::result::Result<(String, Role), Box<Response>> {
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok());
    let token = session_token(headers)
        .ok_or_else(|| Box::new(unauthorized_with_id(request_id.unwrap_or("unknown"))))?;
    let store = auth.lock().map_err(|_| {
        Box::new(
            crate::errors::error_response(
                "auth store locked",
                StatusCode::INTERNAL_SERVER_ERROR,
                request_id,
            )
            .into_response(),
        )
    })?;
    match store.validate_session(&token) {
        Some(session) if session.role >= required => Ok((session.username.clone(), session.role)),
        Some(_) => Err(Box::new(
            crate::errors::error_response(
                "insufficient role for this operation",
                StatusCode::FORBIDDEN,
                request_id,
            )
            .into_response(),
        )),
        None => Err(Box::new(unauthorized_with_id(
            request_id.unwrap_or("unknown"),
        ))),
    }
}

/// Require a valid session for every data page. The liveness probe, aggregate
/// metrics, login page and stylesheet remain reachable so an unauthenticated
/// operator can log in and monitoring can run.
async fn require_session(State(auth): State<SharedAuth>, request: Request, next: Next) -> Response {
    if is_public_path(request.uri().path()) {
        return next.run(request).await;
    }
    if let Some(response) = check_session_with_role(&auth, request.headers(), Role::Viewer) {
        return response;
    }
    next.run(request).await
}

async fn rate_limit_guard(request: Request, next: Next) -> Response {
    // Skip rate limiting in test mode
    if std::env::var("EKPI_TEST_MODE").is_ok() {
        return next.run(request).await;
    }
    // Extract client IP from headers or use a default
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1")
        .to_string();

    if !RATE_LIMITER.check_rate_limit(&client_ip) {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok());
        return crate::errors::rate_limit_error_response(
            RATE_LIMIT_PER_MINUTE,
            0,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() + RATE_LIMIT_WINDOW_SECS)
                .unwrap_or(0),
            RATE_LIMIT_WINDOW_SECS,
            request_id,
        )
        .into_response();
    }

    next.run(request).await
}

async fn login_page(State((_ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    // Pre-auth login CSRF uses a fresh double-submit token: the value is
    // rendered into the hidden form field and mirrored in the `ekpi_csrf`
    // cookie; `login_post` refuses the POST unless both match. The token is
    // generated per page load and is never read from an existing session, so
    // one visitor's token can never be served to another.
    let csrf_token = match random_hex(TOKEN_BYTES) {
        Ok(token) => token,
        Err(_) => {
            return crate::errors::error_response(
                "csrf token generation failed",
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
            )
            .into_response();
        }
    };
    let html = html! {
        (DOCTYPE)
        html lang="zh-Hant" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "登入 · EKPI 治理工作台" }
                link rel="stylesheet" href="/assets/governance.css";
            }
            body {
                div class="auth-page" {
                    h2 { "本機登入" }
                    p class="form-hint" { "僅限本機試點使用；尚未連接企業身分平台。" }
                    form class="form-stack" method="post" action="/login" {
                        input type="hidden" name="csrf_token" value=(csrf_token.clone());
                        label class="field" { span { "使用者名稱" } input type="text" name="username" required maxlength="80" placeholder="姓名"; }
                        label class="field" { span { "密碼" } input type="password" name="password" required maxlength="128" placeholder="密碼"; }
                        button class="button button-primary" type="submit" { "登入" }
                    }
                }
            }
        }
    };
    let mut response = Html(html.into_string()).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "ekpi_csrf={csrf_token}; Path=/; HttpOnly; SameSite=Strict{}",
            configured_secure_cookie_suffix()
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("ekpi_csrf=; Path=/; Max-Age=0")),
    );
    response
}

fn secure_cookie_suffix(value: Option<&str>) -> &'static str {
    if value == Some("1") { "; Secure" } else { "" }
}

fn configured_secure_cookie_suffix() -> &'static str {
    secure_cookie_suffix(std::env::var("EKPI_COOKIE_SECURE").ok().as_deref())
}

async fn login_post(
    State((_ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    // Headers must be captured before the body is consumed by `to_bytes`.
    let headers = request.headers().clone();
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok());
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(body) => body,
        Err(error) => {
            return crate::errors::error_response(
                format!("cannot read login request body: {error}"),
                StatusCode::BAD_REQUEST,
                request_id,
            )
            .into_response();
        }
    };
    let form: LoginForm = match serde_urlencoded::from_bytes(&body) {
        Ok(form) => form,
        Err(_) => {
            return crate::errors::error_response(
                "malformed login form",
                StatusCode::BAD_REQUEST,
                request_id,
            )
            .into_response();
        }
    };
    // Double-submit login CSRF: the hidden field must match the `ekpi_csrf`
    // cookie that `login_page` set. A refusal is generic on purpose — it must
    // not reveal whether the cookie or the field was missing.
    let csrf_cookie = csrf_cookie_value(&headers).unwrap_or_default();
    if form.csrf_token.is_empty() || csrf_cookie != form.csrf_token {
        return crate::errors::error_response(
            "invalid login request",
            StatusCode::FORBIDDEN,
            request_id,
        )
        .into_response();
    }
    let username = form.username;
    let password = form.password;
    let auth_for_task = auth.clone();
    let login_result = tokio::task::spawn_blocking(move || {
        let mut store = auth_for_task
            .lock()
            .map_err(|_| "auth store locked".to_string())?;
        store.login(&username, &password)
    })
    .await;

    match login_result {
        Ok(Ok(session)) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let max_age = session.expires_at.saturating_sub(now).max(1);
            let secure = configured_secure_cookie_suffix();
            let cookie = format!(
                "ekpi_session={}; Path=/; HttpOnly; SameSite=Strict{secure}; Max-Age={max_age}",
                session.token
            );
            let csrf_cookie = format!(
                "ekpi_csrf={}; Path=/; SameSite=Strict{secure}; Max-Age={max_age}",
                session.csrf_token
            );
            let mut response = Redirect::to("/").into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie).unwrap_or_else(|_| {
                    HeaderValue::from_static("ekpi_session=; Path=/; Max-Age=0")
                }),
            );
            response.headers_mut().append(
                header::SET_COOKIE,
                HeaderValue::from_str(&csrf_cookie)
                    .unwrap_or_else(|_| HeaderValue::from_static("ekpi_csrf=; Path=/; Max-Age=0")),
            );
            response
        }
        Ok(Err(_)) => {
            let request_id = headers
                .get(REQUEST_ID_HEADER)
                .and_then(|value| value.to_str().ok());
            crate::errors::auth_error_response(
                "invalid_credentials",
                "invalid username or password".to_string(),
                None,
                0,
                None,
                request_id,
            )
            .into_response()
        }
        Err(_) => crate::errors::error_response(
            "login task failed",
            StatusCode::INTERNAL_SERVER_ERROR,
            headers
                .get(REQUEST_ID_HEADER)
                .and_then(|value| value.to_str().ok()),
        )
        .into_response(),
    }
}

async fn logout_post(
    State((_ledger, auth)): State<(SharedLedger, SharedAuth)>,
    headers: HeaderMap,
) -> Response {
    if let Some(token) = session_token(&headers)
        && let Ok(mut store) = auth.lock()
    {
        store.logout(&token);
    }

    let secure = configured_secure_cookie_suffix();
    let cookie = format!("ekpi_session=; Path=/; HttpOnly; SameSite=Strict{secure}; Max-Age=0");
    let mut response = Redirect::to("/login").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .unwrap_or_else(|_| HeaderValue::from_static("ekpi_session=; Path=/; Max-Age=0")),
    );
    response
}
#[derive(Deserialize)]
struct LoginForm {
    #[serde(default)]
    csrf_token: String,
    username: String,
    password: String,
}

fn apply_security_headers(headers: &mut axum::http::HeaderMap, request_id: &str) {
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; style-src 'self'; img-src 'self' data:; form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    if let Ok(value) = HeaderValue::from_str(request_id) {
        headers.insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }
}

fn generate_request_id() -> String {
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("ekpi-{}-{nanos:x}-{sequence:x}", std::process::id())
}

fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Accept only loopback authorities: `localhost`, IPv4 loopback addresses and
/// `[::1]`, optionally followed by a numeric port. The Host header is compared
/// byte-for-byte against an allowlist after lowercasing; it is never resolved
/// through DNS, so a hostile resolver cannot redirect the check.
fn is_loopback_host(value: &str) -> bool {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    let (host, port) = if let Some(rest) = lower.strip_prefix('[') {
        let Some(end) = rest.find(']') else {
            return false;
        };
        let host = &rest[..end];
        let after = &rest[end + 1..];
        if after.is_empty() {
            (host, None)
        } else if let Some(port) = after.strip_prefix(':') {
            (host, Some(port))
        } else {
            return false;
        }
    } else {
        match lower.rsplit_once(':') {
            Some((host, port)) => {
                // A bare IPv6 address is not a valid Host value; IPv6 must be
                // bracketed. Reject any leftover colon instead of guessing.
                if host.contains(':') {
                    return false;
                }
                (host, Some(port))
            }
            None => (lower.as_str(), None),
        }
    };
    if let Some(port) = port
        && (port.is_empty()
            || port.len() > 5
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || !matches!(port.parse::<u16>(), Ok(1..=u16::MAX)))
    {
        return false;
    }
    if host == "localhost" || host == "::1" {
        return true;
    }
    host.parse::<std::net::Ipv4Addr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}

fn forbidden_cross_origin() -> Response {
    (
        StatusCode::FORBIDDEN,
        "cross-origin form submission rejected",
    )
        .into_response()
}

/// Return a header's single value. A repeated header, or a value that is not
/// valid visible ASCII, is rejected: different intermediaries may pick
/// different copies, so treating either one as authoritative is ambiguous.
fn single_header_value<'a>(
    headers: &'a HeaderMap,
    name: &HeaderName,
) -> Result<Option<&'a str>, ()> {
    let mut values = headers.get_all(name).iter();
    match values.next() {
        None => Ok(None),
        Some(value) => {
            if values.next().is_some() {
                return Err(());
            }
            value.to_str().map(Some).map_err(|_| ())
        }
    }
}

async fn same_origin_guard(request: Request, next: Next) -> Response {
    let headers = request.headers();
    let host = match single_header_value(headers, &header::HOST) {
        Ok(Some(value)) if is_loopback_host(value) => Some(value),
        Ok(None) => None,
        _ => return forbidden_cross_origin(),
    };
    let is_mutation = request.method() != Method::GET && request.method() != Method::HEAD;
    if is_mutation {
        let origin = match single_header_value(headers, &header::ORIGIN) {
            Ok(value) => value,
            Err(()) => return forbidden_cross_origin(),
        };
        let fetch_site =
            match single_header_value(headers, &HeaderName::from_static("sec-fetch-site")) {
                Ok(value) => value,
                Err(()) => return forbidden_cross_origin(),
            };
        let allowed = match origin {
            Some(origin) => match host {
                Some(host) => {
                    origin.eq_ignore_ascii_case(&format!("http://{host}"))
                        || origin.eq_ignore_ascii_case(&format!("https://{host}"))
                }
                None => false,
            },
            None => matches!(fetch_site, Some("same-origin") | Some("none")),
        };
        if !allowed {
            return forbidden_cross_origin();
        }
    }
    next.run(request).await
}
async fn dashboard(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("工作總覽", "overview", dashboard_content(&data)),
        Err(error) => page_error_response("overview", &error),
    }
}

async fn projects_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("專案", "projects", projects_content(&data, None)),
        Err(error) => page_error_response("projects", &error),
    }
}

/// GET /projects/{id} — 專案詳情：欄位＋該專案的證據/個案/檢驗/關卡
/// （全部自 snapshot 依 project_id 過濾；零虛構）。查無 → 誠實 404。
async fn project_detail_page(
    State((ledger, _auth)): State<(SharedLedger, SharedAuth)>,
    Path(project_id): Path<String>,
) -> Response {
    let data = match snapshot(&ledger) {
        Ok(data) => data,
        Err(error) => return page_error_response("projects", &error),
    };
    let project = match data
        .projects
        .iter()
        .find(|project| project.id == project_id)
    {
        Some(project) => project,
        None => return page_not_found("projects", &format!("找不到專案 {project_id}")),
    };
    let evidence: Vec<&crate::governance::Evidence> = data
        .evidence
        .iter()
        .filter(|item| item.project_id == project_id)
        .collect();
    let cases: Vec<&crate::governance::QualityCase> = data
        .cases
        .iter()
        .filter(|case| case.project_id == project_id)
        .collect();
    let inspections: Vec<&crate::governance::QualityInspection> = data
        .inspections
        .iter()
        .filter(|inspection| inspection.project_id == project_id)
        .collect();
    let gates: Vec<&crate::governance::GateDecision> = data
        .gate_decisions
        .iter()
        .filter(|decision| decision.project_id == project_id)
        .collect();
    page_response(
        "專案詳情",
        "projects",
        project_detail_content(project, &evidence, &cases, &inspections, &gates),
    )
}

fn project_detail_content(
    project: &crate::governance::Project,
    evidence: &[&crate::governance::Evidence],
    cases: &[&crate::governance::QualityCase],
    inspections: &[&crate::governance::QualityInspection],
    gates: &[&crate::governance::GateDecision],
) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "專案" } h1 { (project.name) } p class="page-subtitle" { (project.id) " · " (project.department) " · 負責人 " (project.owner) } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "專案欄位" } } }
            table class="api-table" {
                tbody {
                    tr { th { "識別碼" } td { (project.id) } }
                    tr { th { "名稱" } td { (project.name) } }
                    tr { th { "部門" } td { (project.department) } }
                    tr { th { "負責人" } td { (project.owner) } }
                    tr { th { "風險等級" } td { (project.risk_tier) } }
                    tr { th { "建立時間" } td class="num" { (project.created_at) } }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "證據（" (evidence.len()) "）" } } }
            @if evidence.is_empty() {
                p class="empty-inline" { "此專案尚無證據參照。" }
            } @else {
                ul { @for item in evidence { li { (item.id) " · " (item.title) " · " (item.evidence_type) } } }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "品質個案（" (cases.len()) "）" } } }
            @if cases.is_empty() {
                p class="empty-inline" { "此專案尚無品質個案。" }
            } @else {
                ul { @for case in cases { li { (case.id) " · " (case.title) " · " (case.status) } } }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "檢驗（" (inspections.len()) "）／關卡決策（" (gates.len()) "）" } } }
            @if inspections.is_empty() && gates.is_empty() {
                p class="empty-inline" { "此專案尚無檢驗與關卡決策。" }
            } @else {
                ul {
                    @for inspection in inspections { li { "檢驗 " (inspection.id) " · " (inspection.reviewer) } }
                    @for decision in gates { li { "關卡 " (decision.gate_id) " · " (decision.decision) } }
                }
            }
        }
    }
}

async fn create_project(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Editor) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: ProjectForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.record_project(
                &form.name,
                &form.department,
                &form.owner,
                &form.risk_tier,
                &actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/projects").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "專案登錄",
                "projects",
                projects_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("projects", &error),
        },
    }
}

async fn evidence_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("文件與證據", "evidence", evidence_content(&data, None)),
        Err(error) => page_error_response("evidence", &error),
    }
}

/// GET /evidence/{id} — 證據詳情：欄位＋審查紀錄（查無 → 誠實 404）。
async fn evidence_detail_page(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Path(evidence_id): Path<String>,
    request: Request,
) -> Response {
    if let Err(response) = session_identity(&auth, request.headers(), Role::Viewer) {
        return *response;
    }
    let data = match snapshot(&ledger) {
        Ok(data) => data,
        Err(error) => return page_error_response("evidence", &error),
    };
    let evidence = match data.evidence.iter().find(|item| item.id == evidence_id) {
        Some(item) => item,
        None => return page_not_found("evidence", &format!("找不到證據 {evidence_id}")),
    };
    let review = data.reviews.get(&evidence_id);
    page_response(
        "證據詳情",
        "evidence",
        evidence_detail_content(evidence, review),
    )
}

fn evidence_detail_content(
    evidence: &crate::governance::Evidence,
    review: Option<&crate::governance::EvidenceReview>,
) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "文件與證據" } h1 { (evidence.title) } p class="page-subtitle" { (evidence.id) " · " (evidence.source_system) " " (evidence.source_version) } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "證據欄位" } } }
            table class="api-table" {
                tbody {
                    tr { th { "識別碼" } td { (evidence.id) } }
                    tr { th { "所屬專案" } td { (evidence.project_id) } }
                    tr { th { "標題" } td { (evidence.title) } }
                    tr { th { "來源系統" } td { (evidence.source_system) } }
                    tr { th { "來源 URL" } td { (evidence.source_url) } }
                    tr { th { "來源版本" } td { (evidence.source_version) } }
                    tr { th { "證據類型" } td { (evidence.evidence_type) } }
                    tr { th { "負責人" } td { (evidence.owner) } }
                    tr { th { "建立時間" } td class="num" { (evidence.created_at) } }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "審查紀錄" } } }
            @match review {
                Some(review) => {
                    ul class="timeline" {
                        li class="t-done" {
                            div class="t-head" { strong { (review.result) } time { (review.reviewed_at) } }
                            div class="t-body" { "審查者 " (review.reviewer) "；備註：" (review.note) }
                        }
                    }
                }
                None => p class="empty-inline" { "尚未留下人工審查紀錄。" }
            }
        }
    }
}

async fn link_evidence(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Editor) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: EvidenceForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.link_evidence(
                &form.project_id,
                &form.title,
                &form.source_system,
                &form.source_url,
                &form.source_version,
                &form.evidence_type,
                &form.owner,
                &actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/evidence").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "文件與證據",
                "evidence",
                evidence_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("evidence", &error),
        },
    }
}

async fn review_evidence(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: ReviewForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.review_evidence(&id, &form.result, &actor, &form.note));
    match result {
        Ok(()) => Redirect::to("/evidence").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "文件與證據",
                "evidence",
                evidence_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("evidence", &error),
        },
    }
}

async fn workflows_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("流程範本", "workflows", workflows_content(&data, None)),
        Err(error) => page_error_response("workflows", &error),
    }
}

async fn save_workflow(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Editor) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: WorkflowForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let stages: Vec<String> = form
        .stages
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    let base_id = (!form.base_id.trim().is_empty()).then_some(form.base_id);
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.save_workflow(WorkflowDraftInput {
                base_id,
                title: form.title,
                department: form.department,
                owner: form.owner,
                description: form.description,
                stages,
                actor,
            })
        });
    match result {
        Ok(_) => Redirect::to("/workflows").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "流程範本",
                "workflows",
                workflows_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("workflows", &error),
        },
    }
}

async fn publish_workflow(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Path((id, version)): Path<(String, u32)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.publish_workflow(&id, version, &actor));
    match result {
        Ok(()) => Redirect::to("/workflows").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "流程範本",
                "workflows",
                workflows_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("workflows", &error),
        },
    }
}

async fn controls_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("風險與品質", "controls", controls_content(&data, None)),
        Err(error) => page_error_response("controls", &error),
    }
}

/// GET /controls/{id} — 個案詳情：欄位＋更新事件（查無 → 誠實 404）。
async fn controls_detail_page(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Path(case_id): Path<String>,
    request: Request,
) -> Response {
    if let Err(response) = session_identity(&auth, request.headers(), Role::Viewer) {
        return *response;
    }
    let data = match snapshot(&ledger) {
        Ok(data) => data,
        Err(error) => return page_error_response("controls", &error),
    };
    let case = match data.cases.iter().find(|case| case.id == case_id) {
        Some(case) => case,
        None => return page_not_found("controls", &format!("找不到品質個案 {case_id}")),
    };
    let updates: Vec<&crate::governance::QualityCaseUpdate> = data
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            crate::governance::EventPayload::QualityCaseUpdated(update) if update.id == case_id => {
                Some(update)
            }
            _ => None,
        })
        .collect();
    page_response(
        "個案詳情",
        "controls",
        controls_detail_content(case, &updates),
    )
}

fn controls_detail_content(
    case: &crate::governance::QualityCase,
    updates: &[&crate::governance::QualityCaseUpdate],
) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "風險與品質" } h1 { (case.title) } p class="page-subtitle" { (case.id) " · " (case.kind) " · 負責人 " (case.owner) } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "個案欄位" } } }
            table class="api-table" {
                tbody {
                    tr { th { "識別碼" } td { (case.id) } }
                    tr { th { "所屬專案" } td { (case.project_id) } }
                    tr { th { "類型" } td { (case.kind) } }
                    tr { th { "描述" } td { (case.description) } }
                    tr { th { "到期日" } td { (case.due_date) } }
                    tr { th { "狀態" } td { (case.status) } }
                    tr { th { "結案說明" } td { (case.closure_note.clone().unwrap_or_else(|| "—".to_string())) } }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "更新歷史（" (updates.len()) "）" } } }
            @if updates.is_empty() {
                p class="empty-inline" { "尚無狀態更新。" }
            } @else {
                ul class="timeline" {
                    @for update in updates {
                        li {
                            div class="t-head" { strong { (update.status) } time { (update.updated_at) } }
                            div class="t-body" { "操作者 " (update.actor) "；備註：" (update.note) }
                        }
                    }
                }
            }
        }
    }
}

async fn create_quality_case(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Editor) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: QualityCaseForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.create_quality_case(
                &form.project_id,
                &form.kind,
                &form.title,
                &form.description,
                &form.owner,
                &form.due_date,
                &actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/controls").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "風險與品質事件",
                "controls",
                controls_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("controls", &error),
        },
    }
}

async fn update_quality_case(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: QualityCaseUpdateForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.update_quality_case(&id, &form.status, &actor, &form.note));
    match result {
        Ok(()) => Redirect::to("/controls").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "風險與品質事件",
                "controls",
                controls_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("controls", &error),
        },
    }
}

async fn inspections_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("品質檢查", "inspections", inspections_content(&data, None)),
        Err(error) => page_error_response("inspections", &error),
    }
}

async fn record_inspection(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let reviewer = match session_identity(&auth, request.headers(), Role::Editor) {
        Ok(reviewer) => reviewer,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let fields: HashMap<String, String> = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    let get = |field: &str| fields.get(field).cloned().unwrap_or_default();
    let results = QUALITY_CHECKLIST
        .iter()
        .map(|item| {
            let suffix = item.id;
            QualityCheckResult {
                item_id: suffix.to_string(),
                result: get(&format!("result_{suffix}")),
                note: get(&format!("note_{suffix}")),
                evidence_reference: get(&format!("evidence_{suffix}")),
                issue_id: get(&format!("issue_{suffix}")),
            }
        })
        .collect();
    let input = QualityInspectionInput {
        project_id: get("project_id"),
        reviewer,
        results,
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.record_quality_inspection(input));
    match result {
        Ok(_) => Redirect::to("/inspections").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "品質檢查",
                "inspections",
                inspections_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("inspections", &error),
        },
    }
}

async fn lifecycle_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("階段與關卡", "lifecycle", lifecycle_content(&data, None)),
        Err(error) => page_error_response("lifecycle", &error),
    }
}

async fn record_gate_decision(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let form_limit = max_form_bytes_for_path(request.uri().path());
    let body = match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response();
        }
    };
    let form: GateDecisionForm = match serde_urlencoded::from_bytes(&body) {
        Ok(f) => f,
        Err(e) => return malformed_form(e),
    };
    if !["yes", "no"].contains(&form.non_waivable.as_str()) {
        return match snapshot(&ledger) {
            Ok(data) => page_response(
                "階段與關卡",
                "lifecycle",
                lifecycle_content(&data, Some("請明確選擇是否涉及不可豁免要求。")),
            ),
            Err(error) => page_error_response("lifecycle", &error),
        };
    }
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.record_gate_decision(
                &form.project_id,
                &form.gate_id,
                &form.decision,
                &actor,
                &form.reason,
                &form.evidence_reference,
                &form.risk_acceptor,
                &form.remediation_due,
                form.non_waivable == "yes",
            )
        });
    match result {
        Ok(_) => Redirect::to("/lifecycle").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "階段與關卡",
                "lifecycle",
                lifecycle_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => page_error_response("lifecycle", &error),
        },
    }
}

async fn audit_page(State((ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => {
            let integrity = ledger.lock().map(|db| db.verify().is_ok()).unwrap_or(false);
            page_response("稽核紀錄", "audit", audit_content(&data, integrity))
        }
        Err(error) => page_error_response("audit", &error),
    }
}

// ── 核准工作流（S2：真資料渲染 + 動作接線）────────────────────────

type ApprovalViewData = (
    Vec<ekp_domain::workflow::ApprovalTask>,
    Vec<ekp_domain::workflow::WorkflowDefinition>,
    bool,
    std::collections::BTreeMap<String, String>,
);

fn approvals_view(approvals: &SharedApprovals) -> Result<ApprovalViewData, String> {
    let store = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())?;
    let tasks = store.list_tasks();
    let workflows = store.list_workflows();
    let integrity = store.verify().is_ok();
    let submitters = store.task_submitters();
    Ok((tasks, workflows, integrity, submitters))
}

#[derive(Deserialize)]
struct ApprovalQuery {
    view: Option<String>,
}

/// GET /approvals[?view=mine|submitted|all] — role-adaptive queue views.
/// `mine` shows open tasks awaiting a decision (for Approvers), `submitted`
/// shows tasks whose submit event actor is the current session user.
async fn approvals_page(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    Query(params): Query<ApprovalQuery>,
    request: Request,
) -> Response {
    let (username, role) = match session_identity_full(&_auth, request.headers(), Role::Viewer) {
        Ok(identity) => identity,
        Err(response) => return *response,
    };
    let view = params.view.as_deref().unwrap_or("all");
    let can_decide = role >= Role::Approver;
    match approvals_view(&approvals) {
        Ok((tasks, workflows, integrity, submitters)) => page_response(
            "核准任務",
            "approvals",
            approvals_content(ApprovalsPageView {
                tasks: &tasks,
                workflows: &workflows,
                integrity,
                error: None,
                view,
                username: &username,
                can_decide,
                submitters: &submitters,
            }),
        ),
        Err(error) => page_error_response("approvals", &error),
    }
}

fn urgency_badge(urgency: ekp_domain::workflow::Urgency) -> &'static str {
    match urgency {
        ekp_domain::workflow::Urgency::Critical => "badge-red",
        ekp_domain::workflow::Urgency::High => "badge-amber",
        ekp_domain::workflow::Urgency::Normal => "badge-blue",
        ekp_domain::workflow::Urgency::Low => "badge-grey",
    }
}

/// Shortens a hex hash for display (hashes are ASCII hex, so byte slicing is
/// UTF-8 safe here).
fn short_hash(value: &str) -> String {
    if value.len() > 14 {
        format!("{}…{}", &value[..8], &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}

/// Stage-progress dots for an approval task: green = passed stages, blue
/// ring = current stage, grey = ahead. Approved tasks show all stages done.
fn stage_dots_view(
    task: &ekp_domain::workflow::ApprovalTask,
    workflows: &[ekp_domain::workflow::WorkflowDefinition],
) -> Markup {
    let definition = workflows
        .iter()
        .find(|d| d.id == task.workflow_id && d.version == task.workflow_version);
    let Some(definition) = definition else {
        return html! { span class="row-id" { (task.stage_id) } };
    };
    let total = definition.stages.len();
    let current = definition
        .stages
        .iter()
        .position(|stage| stage.id == task.stage_id)
        .unwrap_or(0);
    let done_all = task.state == ekp_domain::workflow::TaskState::Approved;
    html! {
        span class="stage-dots" {
            @for (index, _stage) in definition.stages.iter().enumerate().take(12) {
                @if done_all || index < current {
                    i class="done" {}
                } @else if index == current {
                    i class="current" {}
                } @else {
                    i {}
                }
            }
            span class="stage-dots-label" { (task.stage_id) " · 第 " (current + 1) "/" (total) " 階段" }
        }
    }
}

/// SLA budget meter for an open task: blue while >15% remains, amber under
/// 15%, red once overdue. Terminal tasks show a dash — not a fabricated 0.
fn sla_meter_view(task: &ekp_domain::workflow::ApprovalTask, now_secs: i64) -> Markup {
    if !task.is_open() {
        return html! { span class="meter-label" { "—" } };
    }
    let total = (task.due_at - task.created_at).max(1);
    let remain = task.due_at - now_secs;
    let pct = ((remain.max(0) * 100) / total).clamp(0, 100);
    let (class, label) = if remain <= 0 {
        ("meter meter-over", "已逾時".to_string())
    } else if pct < 15 {
        ("meter meter-warn", format!("剩 {} 分", remain / 60))
    } else {
        ("meter meter-info", format!("剩 {} 分", remain / 60))
    };
    html! {
        div {
            div class=(class) style=(format!("--w: {pct}")) { i {} }
            span class="meter-label" { (label) }
        }
    }
}

/// Render inputs for the approvals page. C4 refactor: one view struct
/// instead of eight positional parameters (the `too_many_arguments` allow is
/// gone), and the queue views / role adaptation read from it directly.
struct ApprovalsPageView<'a> {
    tasks: &'a [ekp_domain::workflow::ApprovalTask],
    workflows: &'a [ekp_domain::workflow::WorkflowDefinition],
    integrity: bool,
    error: Option<&'a str>,
    view: &'a str,
    username: &'a str,
    can_decide: bool,
    submitters: &'a std::collections::BTreeMap<String, String>,
}

fn approvals_content(view: ApprovalsPageView) -> Markup {
    let ApprovalsPageView {
        tasks,
        workflows,
        integrity,
        error,
        view,
        username,
        can_decide,
        submitters,
    } = view;
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let open_count = tasks.iter().filter(|task| task.is_open()).count();
    let submitted_count = tasks
        .iter()
        .filter(|task| submitters.get(&task.id).map(String::as_str) == Some(username))
        .count();
    let view = match view {
        "mine" | "submitted" => view,
        _ => "all",
    };
    let visible: Vec<&ekp_domain::workflow::ApprovalTask> = match view {
        "mine" => tasks.iter().filter(|task| task.is_open()).collect(),
        "submitted" => tasks
            .iter()
            .filter(|task| submitters.get(&task.id).map(String::as_str) == Some(username))
            .collect(),
        _ => tasks.iter().collect(),
    };
    html! {
        section class="page-heading" { div { p class="eyebrow" { "動態簽核工作流" } h1 { "核准任務佇列" } p class="page-subtitle" { "多急分級、階段推進、超時清理與 Agent 建議（advisory-only）。" } } }
        @if let Some(error) = error {
            div class="notice notice-red" role="alert" { strong { "操作失敗：" } span { (error) } }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "簽核任務" } p { "任務由工作流定義觸發；所有決策寫入核准事件鏈。" } }
                span class="badge" class=(if integrity { "badge-green" } else { "badge-red" }) { (if integrity { "鏈已驗證" } else { "鏈驗證失敗" }) }
            }
            div class="chip-row" {
                a class=(if view == "all" { "chip chip-active" } else { "chip" }) href="/approvals" { "全部 · " (tasks.len()) }
                a class=(if view == "mine" { "chip chip-active" } else { "chip" }) href="/approvals?view=mine" { "我的待簽核 · " (open_count) }
                a class=(if view == "submitted" { "chip chip-active" } else { "chip" }) href="/approvals?view=submitted" { "我送出的 · " (submitted_count) }
                a class="chip" href="/approvals/calendar" { "📅 行事曆" }
            }
            @if tasks.is_empty() {
                (empty_state("暫無簽核任務", "先建立工作流定義並發布，再提交簽核任務。核准決策會寫入核准事件鏈。", "/projects", "新增專案"))
            } @else if visible.is_empty() {
                @if view == "mine" {
                    (empty_state("目前沒有待你決策的任務", "所有開放任務已處理完畢，或任務尚在他人手上。", "/approvals", "查看全部任務"))
                } @else {
                    (empty_state("你尚未送出任何簽核任務", "以 Editor 角色提交任務後，會出現在這個檢視。", "/approvals", "查看全部任務"))
                }
            } @else {
                table class="api-table" {
                    thead { tr { th { "任務" } th { "工作流" } th { "階段" } th { "急分級" } th { "SLA" } th { "狀態" } th { "送出者" } th { "升級次數" } th { "決策" } } }
                    tbody {
                        @for task in &visible {
                            tr {
                                td { (task.id) }
                                td { (task.workflow_id) "@" (task.workflow_version) }
                                td { (stage_dots_view(task, workflows)) }
                                td { span class=(format!("badge {}", urgency_badge(task.urgency))) { (format!("{:?}", task.urgency)) } }
                                td { (sla_meter_view(task, now_secs)) }
                                td { span class=(if task.is_open() { "badge badge-blue" } else { "badge badge-grey" }) { (format!("{:?}", task.state)) } }
                                td { (submitters.get(&task.id).map(String::as_str).unwrap_or("—")) }
                                td { (task.escalation_count) }
                                td {
                                    @if task.is_open() && can_decide {
                                        form class="inline-form" method="post" action={ "/approvals/" (task.id) "/decide" } {
                                            select name="action" required {
                                                option value="approve" { "核准" }
                                                option value="reject" { "駁回" }
                                                option value="escalate" { "升級" }
                                            }
                                            input type="text" name="note" maxlength="300" placeholder="備註（可留空）";
                                            button class="button button-primary" type="submit" { "送出決策" }
                                        }
                                    } @else if task.is_open() {
                                        span class="meter-label" { "需 Approver 角色決策" }
                                    } @else {
                                        span { "—" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "工作流定義" } p { "建立後需發布才可被任務引用。" } } }
            @if workflows.is_empty() {
                p { "尚無定義。" }
            } @else {
                table class="api-table" {
                    thead { tr { th { "定義" } th { "名稱" } th { "階段" } th { "狀態" } } }
                    tbody {
                        @for definition in workflows {
                            tr {
                                td { (definition.id) "@" (definition.version) }
                                td { (definition.name) }
                                td { (definition.stages.len()) }
                                td { span class=(if definition.active { "badge badge-green" } else { "badge badge-grey" }) { (if definition.active { "已發布" } else { "未發布" }) } }
                            }
                        }
                    }
                }
            }
            form class="form-stack" method="post" action="/approvals/workflows" {
                label class="field" { span { "定義 id" } input type="text" name="id" required maxlength="60" placeholder="wf-purchase"; }
                label class="field" { span { "名稱" } input type="text" name="name" required maxlength="80"; }
                label class="field" { span { "階段 id（逗號分隔）" } input type="text" name="stages" required placeholder="s1,s2"; }
                label class="field" { span { "各階段 SLA 秒數（逗號分隔，數量須一致）" } input type="text" name="slas" required placeholder="3600,86400"; }
                label class="field" { span { "預設急分級" } select name="urgency" { option value="low" { "low" } option value="normal" selected { "normal" } option value="high" { "high" } option value="critical" { "critical" } } }
                button class="button button-primary" type="submit" { "建立定義（Approver）" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "提交簽核任務" } p { "引用已發布的定義版本；任務落在第一階段，期限依該階段 SLA。" } } }
            form class="form-stack" method="post" action="/approvals/submit" {
                label class="field" { span { "工作流（id@version，須已發布）" } input type="text" name="workflow_ref" required placeholder="wf-purchase@1"; }
                label class="field" { span { "金額（分）" } input type="number" name="amount_cents" required value="0"; }
                label class="field" { span { "平台" } input type="text" name="platform" required maxlength="60"; }
                label class="field" { span { "分類" } input type="text" name="classification" required maxlength="60"; }
                label class="field" { span { "風險分數" } input type="number" name="risk_score" required value="0"; }
                label class="field" { span { "急分級" } select name="urgency" { option value="low" { "low" } option value="normal" selected { "normal" } option value="high" { "high" } option value="critical" { "critical" } } }
                button class="button button-primary" type="submit" { "提交任務（Editor）" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "Agent 建議摘要" } p { "Agent 僅提供建議，不自動核准。所有決策由人工審核。" } } }
            div class="notice notice-blue" role="info" {
                strong { "範圍管制：" }
                span { "ekpi-platform 內 Agent 僅能 advisory（產生建議），不得自動放行。自動核准能力位於 platform-hub，須 DEC-15 決議後啟用。" }
            }
        }
    }
}

fn parse_urgency(text: &str) -> Option<ekp_domain::workflow::Urgency> {
    match text {
        "low" => Some(ekp_domain::workflow::Urgency::Low),
        "normal" => Some(ekp_domain::workflow::Urgency::Normal),
        "high" => Some(ekp_domain::workflow::Urgency::High),
        "critical" => Some(ekp_domain::workflow::Urgency::Critical),
        _ => None,
    }
}

fn now_i64() -> Result<i64, String> {
    let now = ekp_domain::governance::now_unix()?;
    i64::try_from(now).map_err(|_| "系統時鐘超出範圍".to_string())
}

#[derive(Deserialize)]
struct CreateApprovalWorkflowForm {
    id: String,
    name: String,
    stages: String,
    slas: String,
    urgency: String,
}

async fn create_approval_workflow(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&_auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let body = match read_limited_body(request).await {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let form: CreateApprovalWorkflowForm = match serde_urlencoded::from_bytes(&body) {
        Ok(form) => form,
        Err(e) => return malformed_form(e),
    };
    let stage_ids: Vec<&str> = form
        .stages
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let slas: Vec<&str> = form
        .slas
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if stage_ids.is_empty() || stage_ids.len() != slas.len() {
        return render_approval_error(&approvals, "階段與 SLA 數量須一致且不得為空").await;
    }
    let mut parsed_slas = Vec::with_capacity(slas.len());
    for sla in slas {
        match sla.parse::<u64>() {
            Ok(value) if value > 0 => parsed_slas.push(value),
            _ => return render_approval_error(&approvals, "SLA 必須為正整數秒").await,
        }
    }
    let urgency = match parse_urgency(&form.urgency) {
        Some(urgency) => urgency,
        None => return render_approval_error(&approvals, "急分級不正確").await,
    };
    let definition = ekp_domain::workflow::WorkflowDefinition {
        id: form.id.trim().to_string(),
        name: form.name.trim().to_string(),
        version: 1,
        stages: stage_ids
            .iter()
            .zip(parsed_slas.iter())
            .map(|(id, sla)| ekp_domain::workflow::Stage {
                id: (*id).to_string(),
                name: (*id).to_string(),
                approver_rule: "role:approver".to_string(),
                sla_seconds: *sla,
                urgency,
                conditions: vec![],
                escalation_path: vec![],
                on_timeout: ekp_domain::workflow::TimeoutAction::Escalate,
            })
            .collect(),
        active: false,
    };
    let result = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())
        .and_then(|mut store| {
            store
                .create_workflow(definition, &actor)
                .map_err(|e| e.to_string())
        });
    match result {
        Ok(_) => Redirect::to("/approvals").into_response(),
        Err(error) => render_approval_error(&approvals, &error).await,
    }
}

#[derive(Deserialize)]
struct SubmitApprovalTaskForm {
    workflow_ref: String,
    amount_cents: i64,
    platform: String,
    classification: String,
    risk_score: i32,
    urgency: String,
}

async fn submit_approval_task(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&_auth, request.headers(), Role::Editor) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let body = match read_limited_body(request).await {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let form: SubmitApprovalTaskForm = match serde_urlencoded::from_bytes(&body) {
        Ok(form) => form,
        Err(e) => return malformed_form(e),
    };
    let (workflow_id, version) = match form.workflow_ref.split_once('@') {
        Some((id, version)) => match version.trim().parse::<u32>() {
            Ok(version) => (id.trim().to_string(), version),
            Err(_) => return render_approval_error(&approvals, "工作流版本必須為數字").await,
        },
        None => return render_approval_error(&approvals, "工作流參照格式須為 id@version").await,
    };
    let urgency = match parse_urgency(&form.urgency) {
        Some(urgency) => urgency,
        None => return render_approval_error(&approvals, "急分級不正確").await,
    };
    let now = match now_i64() {
        Ok(now) => now,
        Err(error) => return render_approval_error(&approvals, &error).await,
    };
    let input = ekp_domain::workflow::TaskSubmission {
        workflow_id,
        workflow_version: version,
        amount_cents: form.amount_cents,
        platform: form.platform.trim().to_string(),
        classification: form.classification.trim().to_string(),
        risk_score: form.risk_score,
        urgency,
    };
    let result = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())
        .and_then(|mut store| {
            store
                .submit_task(&input, &actor, now, None)
                .map_err(|e| e.to_string())
        });
    match result {
        Ok(_) => Redirect::to("/approvals").into_response(),
        Err(error) => render_approval_error(&approvals, &error).await,
    }
}

#[derive(Deserialize)]
struct DecideApprovalTaskForm {
    action: String,
    note: String,
}

async fn decide_approval_task(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    Path(task_id): Path<String>,
    request: Request,
) -> Response {
    let actor = match session_identity(&_auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let body = match read_limited_body(request).await {
        Ok(body) => body,
        Err(response) => return *response,
    };
    let form: DecideApprovalTaskForm = match serde_urlencoded::from_bytes(&body) {
        Ok(form) => form,
        Err(e) => return malformed_form(e),
    };
    let action = match ekp_domain::workflow::TaskAction::parse(form.action.trim()) {
        Some(
            action @ (ekp_domain::workflow::TaskAction::Approve
            | ekp_domain::workflow::TaskAction::Reject
            | ekp_domain::workflow::TaskAction::Escalate),
        ) => action,
        _ => return render_approval_error(&approvals, "決策動作不正確").await,
    };
    let now = match now_i64() {
        Ok(now) => now,
        Err(error) => return render_approval_error(&approvals, &error).await,
    };
    let result = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())
        .and_then(|mut store| {
            store
                .decide_task(&task_id, action, &actor, &form.note, now)
                .map_err(|e| e.to_string())
        });
    match result {
        Ok(_) => Redirect::to("/approvals").into_response(),
        Err(error) => render_approval_error(&approvals, &error).await,
    }
}

/// Publishes a workflow definition version (Approver). Publication is the
/// gate between "definition exists" and "tasks may reference it" (B7).
async fn publish_approval_workflow(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    Path((workflow_id, version)): Path<(String, u32)>,
    request: Request,
) -> Response {
    let actor = match session_identity(&_auth, request.headers(), Role::Approver) {
        Ok(actor) => actor,
        Err(response) => return *response,
    };
    let result = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())
        .and_then(|mut store| {
            store
                .publish_workflow(&workflow_id, version, &actor)
                .map_err(|e| e.to_string())
        })
        .map_err(|e| e.to_string());
    match result {
        Ok(()) => Redirect::to("/approvals").into_response(),
        Err(error) => render_approval_error(&approvals, &error).await,
    }
}

async fn render_approval_error(approvals: &SharedApprovals, error: &str) -> Response {
    match approvals_view(approvals) {
        Ok((tasks, workflows, integrity, submitters)) => {
            // Error glance pages render read-only ("all" view, no decide
            // forms) to avoid role-ambiguous affordances after a refused POST.
            page_response(
                "核准任務",
                "approvals",
                approvals_content(ApprovalsPageView {
                    tasks: &tasks,
                    workflows: &workflows,
                    integrity,
                    error: Some(error),
                    view: "all",
                    username: "",
                    can_decide: false,
                    submitters: &submitters,
                }),
            )
            .into_response()
        }
        Err(view_error) => page_error_response("approvals", &view_error),
    }
}

async fn read_limited_body(request: Request) -> Result<Vec<u8>, Box<Response>> {
    let form_limit = max_form_bytes_for_path(request.uri().path());
    match axum::body::to_bytes(request.into_body(), form_limit).await {
        Ok(bytes) => Ok(bytes.to_vec()),
        Err(e) => Err(Box::new(
            (StatusCode::BAD_REQUEST, format!("無法讀取要求本文：{e}")).into_response(),
        )),
    }
}

// ── Phase C9：系統設定頁 ──────────────────────────────────────────

async fn settings_page(State((_ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    page_response("系統設定", "settings", settings_content())
}

fn settings_content() -> Markup {
    let version = env!("CARGO_PKG_VERSION");
    html! {
        section class="page-heading" { div { p class="eyebrow" { "營運設定" } h1 { "系統設定" } p class="page-subtitle" { "儲存模式、版本與健康檢查。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "儲存模式" } p { "JSONL（預設，本機 pilot）或 PostgreSQL 18.6（部署目標）。" } } }
            div class="setting-row" { label { "目前模式：" } span class="badge badge-blue" { "JSONL（本機）" } }
            div class="setting-row" { label { "PostgreSQL：" } span { "僅在 EKPI_STORAGE=postgres + DATABASE_URL 啟用" } }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "系統資訊" } } }
            div class="setting-row" { label { "版本：" } span { (version) } }
            div class="setting-row" { label { "部署目標：" } span { "本機 / loopback（跨機部署須擁有者核准）" } }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "健康檢查" } } }
            div class="setting-row" { label { "稽核鏈：" } span class="badge badge-green" { "本機可用" } }
            div class="setting-row" { label { "JSONL ledger：" } span { "append-only hash chain" } }
        }
    }
}

// ── Phase C9：API 文件頁 ──────────────────────────────────────────

async fn api_docs_page(State((_ledger, _auth)): State<(SharedLedger, SharedAuth)>) -> Response {
    page_response("API 文件", "api-docs", api_docs_content())
}

fn api_docs_content() -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "程式化存取" } h1 { "API 文件" } p class="page-subtitle" { "工作台為主介面；API 端點已註冊路由但尚未接線。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "端點狀態" } } }
            table class="api-table" {
                thead {
                    tr { th { "方法" } th { "端點" } th { "狀態" } }
                }
                tbody {
                    tr { td { "GET" } td { "/api/v1/workflows" } td class="status-wired" { "已實作（唯讀）" } }
                    tr { td { "POST" } td { "/api/v1/workflows" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/approvals" } td class="status-wired" { "已實作（唯讀）" } }
                    tr { td { "POST" } td { "/api/v1/approvals" } td class="status-wired" { "已實作（冪等鍵可選）" } }
                    tr { td { "POST" } td { "/api/v1/approvals/{id}/decide" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "POST" } td { "/api/v1/events/replay" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/channels" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/users" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/settings" } td class="status-planned" { "未實作（一律 501）" } }
                }
            }
            div class="notice notice-blue" role="info" {
                strong { "誠實聲明：" }
                span { "三個端點已接線（見綠色標示）：唯讀清單與冪等任務提交，契約見 approval-api.md。其餘端點一律回 501，不提供任何資料；本頁不宣稱任何認證機制。寫入端點的決策（decide）與其它模式將依契約逐項實作。" }
            }
        }
    }
}

fn snapshot(ledger: &SharedLedger) -> Result<GovernanceSnapshot, String> {
    ledger
        .lock()
        .map(|db| db.snapshot())
        .map_err(|_| "資料鎖定失敗，請重新啟動本機原型".to_string())
}

/// 頁面註冊表（單一事實來源）：GET 頁面的 route/標題/是否需登入。
/// 覆蓋測試（tests/page_registry.rs）由此逐頁衍生；nav 繫結列為 C3 延後項
/// （如實標注：本表尚未驅動 nav）。
pub struct PageSpec {
    pub path: &'static str,
    pub title: &'static str,
    pub requires_auth: bool,
    /// 是否出現在側邊導航（子檢視如行事曆為 false）。
    pub in_nav: bool,
}

macro_rules! page {
    ($path:literal, $title:literal, $auth:literal, $nav:literal) => {
        PageSpec {
            path: $path,
            title: $title,
            requires_auth: $auth,
            in_nav: $nav,
        }
    };
}

pub const PAGE_REGISTRY: &[PageSpec] = &[
    page!("/", "工作總覽", true, true),
    page!("/login", "登入", false, true),
    page!("/projects", "專案", true, true),
    page!("/evidence", "文件與證據", true, true),
    page!("/inspections", "品質檢查", true, true),
    page!("/workflows", "流程範本", true, true),
    page!("/controls", "風險與品質", true, true),
    page!("/lifecycle", "階段與關卡", true, true),
    page!("/audit", "稽核紀錄", true, true),
    page!("/events", "事件瀏覽器", true, true),
    page!("/audit/chains", "鏈狀態", true, true),
    page!("/approvals", "核准任務", true, true),
    page!("/approvals/calendar", "SLA 行事曆", true, false),
    page!("/search", "文件檢索", true, true),
    page!("/data-map", "資料地圖", true, true),
    page!("/role-matrix", "權限矩陣", true, true),
    page!("/glossary", "領域字彙", true, true),
    page!("/help", "操作說明", true, true),
    page!("/about", "關於", true, true),
    page!("/settings", "系統設定", true, true),
    page!("/api-docs", "API 文件", true, true),
];

fn page_response(title: &str, active: &str, content: Markup) -> Response {
    Html(layout(title, active, content).into_string()).into_response()
}

fn page_error_response(active: &str, message: &str) -> Response {
    let content = html! {
        section class="page-heading" { p class="eyebrow" { "系統資料無法使用" } h1 { "需要檢查稽核資料" } }
        div class="alert alert-error" role="alert" { (message) }
        p { "請停止寫入，保留資料檔並聯絡管理者。原型不會自動修補不完整的稽核鏈。" }
    };
    page_response("資料錯誤", active, content)
}

/// 詳細頁查無資料的誠實 404（沿用工作台外殼；非 JSON）。
fn page_not_found(active: &str, message: &str) -> Response {
    let content = html! {
        section class="page-heading" { p class="eyebrow" { "查無資料" } h1 { "找不到對應的記錄" } }
        div class="alert alert-error" role="alert" { (message) }
        p { "可能原因：識別碼輸入錯誤、資料尚未建立，或已被移除。請從清單頁重新導航。" }
    };
    (
        StatusCode::NOT_FOUND,
        Html(layout("查無資料", active, content).into_string()),
    )
        .into_response()
}

fn layout(title: &str, active: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="zh-Hant" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light";
                title { (title) " · EKPI 治理工作台" }
                link rel="stylesheet" href="/assets/governance.css";
            }
            body {
                a class="skip-link" href="#main-content" { "跳至主要內容" }
                header class="topbar" {
                    a class="brand" href="/" aria-label="EKPI 治理工作台首頁" {
                        span class="brand-mark" aria-hidden="true" { "E" }
                        span { strong { "EKPI" } small { "治理工作台" } }
                    }
                    div class="topbar-meta" {
                        span class="local-chip" { "本機試點" }
                        span class="avatar" aria-hidden="true" { "G" }
                    }
                }
                div class="prototype-banner" role="note" {
                    strong { "原型模式" }
                    span { "僅限本機與非敏感試用資料；沒有企業登入，也未連接 Microsoft、Google 或 AI。" }
                }
                div class="app-shell" {
                    aside class="sidebar" aria-label="主要導覽" {
                        p class="nav-label" { "工作區" }
                        nav {
                            (nav_link("/", "overview", active, "總覽", "◫"))
                            (nav_link("/search", "search", active, "文件檢索", "⌕"))
                            div class="nav-group-label" { "治理" }
                            (nav_link("/projects", "projects", active, "專案", "▦"))
                            (nav_link("/evidence", "evidence", active, "文件與證據", "▤"))
                            (nav_link("/inspections", "inspections", active, "品質檢查", "✓"))
                            (nav_link("/workflows", "workflows", active, "流程範本", "⌘"))
                            (nav_link("/approvals", "approvals", active, "核准任務", "◎"))
                            div class="nav-group-label" { "品質與稽核" }
                            (nav_link("/controls", "controls", active, "風險與品質", "◇"))
                            (nav_link("/lifecycle", "lifecycle", active, "階段與關卡", "▥"))
                            (nav_link("/audit", "audit", active, "稽核紀錄", "◷"))
                            (nav_link("/events", "events", active, "事件瀏覽器", "≡"))
                            (nav_link("/audit/chains", "chains", active, "鏈狀態", "⛓"))
                            (nav_link("/data-map", "data-map", active, "資料地圖", "⌗"))
                            (nav_link("/role-matrix", "role-matrix", active, "權限矩陣", "⛨"))
                            div class="nav-group-label" { "系統" }
                            (nav_link("/settings", "settings", active, "系統設定", "⚙"))
                            (nav_link("/api-docs", "api-docs", active, "API 文件", "❐"))
                            div class="nav-group-label" { "說明" }
                            (nav_link("/glossary", "glossary", active, "字彙", "〔"))
                            (nav_link("/help", "help", active, "操作說明", "?"))
                            (nav_link("/about", "about", active, "關於", "ⓘ"))
                            div class="nav-group-label" { "系統" }
                            (nav_link("/settings", "settings", active, "系統設定", "⚙"))
                            (nav_link("/api-docs", "api-docs", active, "API 文件", "❐"))
                        }
                        div class="sidebar-note" {
                            span class="status-dot" aria-hidden="true" {}
                            div { strong { "資料保存在本機" } small { "JSONL 雜湊鏈 · 手動來源參照" } }
                        }
                    }
                    main id="main-content" class="main-content" tabindex="-1" { (content) }
                }
                footer class="app-footer" {
                    span { "EKPI · 跨專案治理試點" }
                    span { "雲端文件留在原始平台，工作台僅保存參照與治理紀錄。" }
                }
            }
        }
    }
}

// ── S12：SLA 行事曆日期演算法（Hinnant civil-date，純整數、零依賴） ──

/// Unix 秒 → (年, 月, 日)（UTC）。Howard Hinnant 的 `civil_from_days` 演算法：
/// 純整數運算、無浮點、對所有 64 位輸入決定論。
pub fn civil_from_unix(secs: i64) -> (i64, u32, u32) {
    let days = secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// (年, 月, 日) → Unix 天數（`civil_from_unix` 的反函數）。
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + u64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // Hinnant 公式以 1970-03-01 為紀元內部原點，需減回 Unix 紀元
    // （civil_from_unix 的 +719_468 與此對稱——缺此行即整體平移錯誤，
    // 已由 epoch_round_trip 紅測試抓出）。
    era * 146_097 + doe as i64 - 719_468
}

/// 指定月份天數（閏年規則：4 年一閏、百年不閏、四百年再閏）。
pub fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// 月初的 Unix 天數 → 週幾（0=一 … 6=日）。1970-01-01 為週四。
fn weekday_of_first(y: i64, m: u32) -> u32 {
    (days_from_civil(y, m, 1) + 3).rem_euclid(7) as u32
}

fn calendar_caption(y: i64, m: u32) -> String {
    format!("{y} 年 {m:02} 月")
}

fn shift_month(y: i64, m: u32, delta: i32) -> (i64, u32) {
    let total = (y * 12 + i64::from(m) - 1) + i64::from(delta);
    (total.div_euclid(12), (total.rem_euclid(12) + 1) as u32)
}

#[derive(Deserialize)]
struct CalendarQuery {
    month: Option<String>,
}

async fn calendar_page(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
    Query(params): Query<CalendarQuery>,
) -> Response {
    let (tasks, _workflows, _integrity, _submitters) = match approvals_view(&approvals) {
        Ok(view) => view,
        Err(error) => return page_error_response("approvals", &error),
    };
    let now_secs = match now_i64() {
        Ok(now) => now,
        Err(error) => return page_error_response("approvals", &error),
    };
    let (year, month) = match params.month.as_deref() {
        None | Some("") => {
            let (y, m, _d) = civil_from_unix(now_secs);
            (y, m)
        }
        Some(text) => match parse_calendar_month(text) {
            Some(ym) => ym,
            None => {
                return crate::errors::error_response(
                    format!("month must be YYYY-MM (got `{text}`)"),
                    StatusCode::BAD_REQUEST,
                    None,
                )
                .into_response();
            }
        },
    };
    let tasks_ref: Vec<&ekp_domain::workflow::ApprovalTask> = tasks.iter().collect();
    page_response(
        "SLA 行事曆",
        "approvals",
        calendar_content(&tasks_ref, year, month, now_secs),
    )
}

/// 解析 `YYYY-MM`；年份限 1970–9999，月份 1–12（fail-closed，不靜默回退）。
fn parse_calendar_month(text: &str) -> Option<(i64, u32)> {
    let (y, m) = text.split_once('-')?;
    let y = y.trim().parse::<i64>().ok()?;
    let m = m.trim().parse::<u32>().ok()?;
    if !(1970..=9999).contains(&y) || !(1..=12).contains(&m) {
        return None;
    }
    Some((y, m))
}

fn calendar_content(
    tasks: &[&ekp_domain::workflow::ApprovalTask],
    year: i64,
    month: u32,
    now_secs: i64,
) -> Markup {
    let days = days_in_month(year, month);
    let leading = weekday_of_first(year, month);
    let (prev_y, prev_m) = shift_month(year, month, -1);
    let (next_y, next_m) = shift_month(year, month, 1);
    // 到期日落在本月的任務（依 due_at 的 UTC 日）。
    let mut due_by_day: BTreeMap<u32, Vec<&ekp_domain::workflow::ApprovalTask>> = BTreeMap::new();
    let mut outside = 0usize;
    for task in tasks {
        let (due_y, due_m, due_d) = civil_from_unix(task.due_at);
        if due_y == year && due_m == month {
            due_by_day.entry(due_d).or_default().push(task);
        } else {
            outside += 1;
        }
    }
    html! {
        section class="page-heading" {
            div {
                p class="eyebrow" { "動態簽核工作流" }
                h1 { "SLA 行事曆" }
                p class="page-subtitle" { "任務到期日分佈（UTC）；逾期以紅色與文字雙重標注，不以顏色為唯一傳達。" }
            }
            div class="chip-row" {
                a class="chip" href={ "/approvals/calendar?month=" (format!("{prev_y:04}-{prev_m:02}")) } { "← 上一月" }
                span class="calendar-caption" { (calendar_caption(year, month)) }
                a class="chip" href={ "/approvals/calendar?month=" (format!("{next_y:04}-{next_m:02}")) } { "下一月 →" }
                a class="chip" href="/approvals" { "返回佇列" }
            }
        }
        section class="panel" {
            table class="calendar-table" {
                caption class="visually-hidden" { (calendar_caption(year, month)) " 到期日分佈" }
                thead { tr { th { "一" } th { "二" } th { "三" } th { "四" } th { "五" } th { "六" } th { "日" } } }
                tbody {
                    @for week in 0..(leading + days).div_ceil(7) {
                        tr {
                            @for weekday in 0..7 {
                                @let slot = week * 7 + weekday;
                                @if slot < leading || slot >= leading + days {
                                    td class="calendar-cell calendar-blank" {}
                                } @else {
                                    @let day = slot - leading + 1;
                                    @let due_tasks = due_by_day.get(&day);
                                    @let has_overdue = due_tasks.map(|list| list.iter().any(|task| task.is_overdue(now_secs))).unwrap_or(false);
                                    td class=(if has_overdue { "calendar-cell calendar-overdue" } else { "calendar-cell" }) data-date=(format!("{year:04}-{month:02}-{day:02}")) {
                                        span class="calendar-day-num" { (day) }
                                        @if let Some(list) = due_tasks {
                                            @for task in list {
                                                @if task.is_overdue(now_secs) {
                                                    a class="calendar-task" href="/approvals" { span class="calendar-overdue-mark" { "已逾時 " } (task.id) }
                                                } @else {
                                                    a class="calendar-task" href="/approvals" { (task.id) }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p class="form-hint" {
                "本月到期任務 " (due_by_day.values().map(Vec::len).sum::<usize>()) " 筆；到期日在本月任務之外 " (outside) " 筆（見佇列檢視）。時區語義：UTC。"
            }
        }
    }
}

// ── S12 頁面批次：事件瀏覽器／鏈狀態／權限矩陣 ──
// ── 批次 3：glossary／help／about ──

const EVENTS_PAGE_SIZE: usize = 50;

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(rename = "type")]
    event_type: Option<String>,
    page: Option<usize>,
}

/// GET /events — 治理帳鏈瀏覽器：全部事件（新→舊），`?type=` 過濾、
/// `?page=` 分頁（50/頁）。標籤由血緣表窮舉 match 提供（單一事實）。
async fn events_page(
    State((ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Query(params): Query<EventsQuery>,
    request: Request,
) -> Response {
    if let Err(response) = session_identity(&auth, request.headers(), Role::Viewer) {
        return *response;
    }
    let snapshot = match ledger.lock() {
        Ok(ledger) => ledger.snapshot(),
        Err(_) => return page_error_response("data-map", "治理資料鎖定失敗"),
    };
    let filter = params.event_type.as_deref().unwrap_or("all");
    let mut matching: Vec<&crate::governance::LedgerEvent> = snapshot
        .events
        .iter()
        .filter(|event| {
            filter == "all" || crate::data_map::governance_payload_tag(&event.payload) == filter
        })
        .collect();
    matching.reverse(); // 新→舊
    let total_pages = (matching.len() + EVENTS_PAGE_SIZE)
        .div_ceil(EVENTS_PAGE_SIZE)
        .max(1);
    let page_index = params.page.unwrap_or(1).max(1);
    if page_index > total_pages && !matching.is_empty() {
        return crate::errors::error_response(
            format!("page {page_index} 超出範圍（共 {total_pages} 頁）"),
            StatusCode::BAD_REQUEST,
            None,
        )
        .into_response();
    }
    let start = (page_index - 1) * EVENTS_PAGE_SIZE;
    let page_slice: Vec<&crate::governance::LedgerEvent> = matching
        .iter()
        .skip(start)
        .take(EVENTS_PAGE_SIZE)
        .copied()
        .collect();
    page_response(
        "事件瀏覽器",
        "data-map",
        events_content(
            &page_slice,
            filter,
            page_index,
            total_pages,
            matching.len(),
            snapshot.events.len(),
        ),
    )
}

fn events_content(
    events: &[&crate::governance::LedgerEvent],
    filter: &str,
    page_index: usize,
    total_pages: usize,
    matching_count: usize,
    total_events: usize,
) -> Markup {
    let tags: Vec<&str> = crate::data_map::GOVERNANCE_LINEAGE
        .iter()
        .map(|entry| entry.event)
        .collect();
    html! {
        section class="page-heading" {
            div { p class="eyebrow" { "治理稽核" } h1 { "事件瀏覽器" } p class="page-subtitle" { "治理帳鏈全部事件（新→舊）；可依事件類型過濾。" } }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "事件類型過濾" } } }
            div class="chip-row" {
                a class=(if filter == "all" { "chip chip-active" } else { "chip" }) href="/events" { "全部 · " (total_events) }
                @for tag in tags {
                    a class=(if filter == tag { "chip chip-active" } else { "chip" }) href={ "/events?type=" (tag) } { (tag) }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "事件（" (matching_count) " 筆符合）" } p { "第 " (page_index) " / " (total_pages) " 頁" } } }
            @if events.is_empty() {
                (empty_state("沒有符合的事件", "過濾條件下無事件，或帳鏈尚為空。", "/events", "清除過濾"))
            } @else {
                table class="api-table" {
                    thead { tr { th { "序號" } th { "事件" } th { "操作者" } th { "時間 (Unix)" } th { "雜湊" } } }
                    tbody {
                        @for event in events {
                            tr {
                                td class="num" { (event.sequence) }
                                td { span class="result-cite" { (crate::data_map::governance_payload_tag(&event.payload)) } }
                                td { (event.actor) }
                                td class="num" { (event.timestamp_unix) }
                                td { span class="hash-chip" { (short_hash(&event.sha256)) } }
                            }
                        }
                    }
                }
                div class="chip-row" {
                    @if page_index > 1 {
                        a class="chip" href={ "/events?type=" (filter) "&page=" (page_index - 1) } { "← 上一頁" }
                    }
                    @if page_index < total_pages {
                        a class="chip" href={ "/events?type=" (filter) "&page=" (page_index + 1) } { "下一頁 →" }
                    }
                }
            }
        }
    }
}

/// GET /audit/chains — 雙鏈狀態摘要：長度、鏈頭、驗證、各類事件計數。
async fn chains_summary_page(
    State((ledger, auth, approvals)): State<(SharedLedger, SharedAuth, SharedApprovals)>,
    request: Request,
) -> Response {
    if let Err(response) = session_identity(&auth, request.headers(), Role::Viewer) {
        return *response;
    }
    let (gov_verified, gov_count, gov_head, gov_counts) = match ledger.lock() {
        Ok(ledger) => {
            let verified = ledger.verify().is_ok();
            let snapshot = ledger.snapshot();
            let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
            for event in &snapshot.events {
                let tag = crate::data_map::governance_payload_tag(&event.payload);
                *counts.entry(tag).or_insert(0) += 1;
            }
            (
                verified,
                snapshot.events.len(),
                snapshot
                    .events
                    .last()
                    .map(|event| event.sha256.clone())
                    .unwrap_or_default(),
                counts,
            )
        }
        Err(_) => return page_error_response("data-map", "治理資料鎖定失敗"),
    };
    let (appr_verified, appr_count, appr_head, appr_counts) = match approvals.lock() {
        Ok(store) => {
            let verified = store.verify().is_ok();
            let string_counts = store.event_type_counts();
            let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
            for entry in crate::data_map::APPROVAL_LINEAGE {
                counts.insert(
                    entry.event,
                    string_counts.get(entry.event).copied().unwrap_or(0),
                );
            }
            (
                verified,
                store.event_count(),
                store.head_sha256().unwrap_or_default(),
                counts,
            )
        }
        Err(_) => return page_error_response("data-map", "核准資料鎖定失敗"),
    };
    page_response(
        "鏈狀態",
        "audit",
        chains_content(ChainsView {
            gov_verified,
            gov_count,
            gov_head: &gov_head,
            gov_counts: &gov_counts,
            appr_verified,
            appr_count,
            appr_head: &appr_head,
            appr_counts: &appr_counts,
        }),
    )
}

fn chain_panel(
    title: &str,
    verified: bool,
    count: usize,
    head: &str,
    counts: &BTreeMap<&'static str, usize>,
) -> Markup {
    html! {
        div class="panel" {
            div class="panel-heading" { div { h2 { (title) } }
                span class="badge" class=(if verified { "badge-green" } else { "badge-red" }) { (if verified { "鏈已驗證" } else { "鏈驗證失敗" }) }
            }
            p { "事件數 " (count) "；鏈頭 " span class="hash-chip" { (short_hash(head)) } }
            @if counts.is_empty() {
                p class="empty-inline" { "尚無事件。" }
            } @else {
                table class="api-table" {
                    thead { tr { th { "事件類型" } th { "數量" } } }
                    tbody {
                        @for (tag, count) in counts {
                            tr { td { (tag) } td class="num" { (count) } }
                        }
                    }
                }
            }
        }
    }
}

fn chains_content(view: ChainsView) -> Markup {
    let ChainsView {
        gov_verified,
        gov_count,
        gov_head,
        gov_counts,
        appr_verified,
        appr_count,
        appr_head,
        appr_counts,
    } = view;
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理稽核" } h1 { "雙鏈狀態" } p class="page-subtitle" { "治理帳鏈與核准帳鏈為兩個獨立的雜湊鏈；本頁彙整長度、鏈頭與驗證狀態。" } } }
        div class="kpi-grid" {
            (kpi("治理事件", &gov_count.to_string(), if gov_verified { "鏈已驗證" } else { "鏈驗證失敗" }, if gov_verified { "good" } else { "alert" }))
            (kpi("核准事件", &appr_count.to_string(), if appr_verified { "鏈已驗證" } else { "鏈驗證失敗" }, if appr_verified { "good" } else { "alert" }))
        }
        section class="content-grid" {
            (chain_panel("治理帳鏈", gov_verified, gov_count, gov_head, gov_counts))
            (chain_panel("核准帳鏈", appr_verified, appr_count, appr_head, appr_counts))
        }
        p class="data-note" { "兩條鏈皆為追加式 JSONL 雜湊鏈（非 WORM）；驗證為全鏈重算。任何不符即本頁紅標，且啟動時 fail-closed。" }
    }
}

/// 權限矩陣列（與路由/中介層行為對照；變更程式碼時須同步本表——
/// 由 page_registry 覆蓋測試與角色測試把關）。
const ROLE_MATRIX: &[RoleMatrixRow] = &[
    RoleMatrixRow {
        area: "健康探針（/healthz /ready /metrics）",
        viewer: "✓",
        editor: "✓",
        approver: "✓",
        note: "公開；無敏感資料",
    },
    RoleMatrixRow {
        area: "登入 / 靜態資產",
        viewer: "✓",
        editor: "✓",
        approver: "✓",
        note: "登入本身不需 session",
    },
    RoleMatrixRow {
        area: "資料頁（總覽/專案/證據/檢驗/流程/控制/關卡/稽核/核准/搜尋/資料地圖/系統設定/說明/事件瀏覽器/鏈狀態/權限矩陣/字彙）",
        viewer: "✓ 唯讀",
        editor: "✓ 唯讀＋可建立",
        approver: "✓",
        note: "GET 唯讀",
    },
    RoleMatrixRow {
        area: "建立與更新（專案/證據/檢驗/流程/個案/關卡/送出簽核）",
        viewer: "✗",
        editor: "✓",
        approver: "✓",
        note: "POST 需 Editor 以上；actor 取自 session",
    },
    RoleMatrixRow {
        area: "決策（核准/駁回/升級）",
        viewer: "✗",
        editor: "✗",
        approver: "✓",
        note: "Approver 專屬；workflow_published 同級",
    },
    RoleMatrixRow {
        area: "API v1 唯讀端點（workflows/approvals 清單）",
        viewer: "✓ (session)",
        editor: "✓",
        approver: "✓",
        note: "GET 需 session",
    },
    RoleMatrixRow {
        area: "API v1 寫入端點",
        viewer: "501",
        editor: "501",
        approver: "501",
        note: "未接線，誠實 501（非 403）",
    },
];

#[derive(Debug, Clone, Copy)]
/// 雙鏈狀態摘要的視圖輸入（C4 同款：避免長參數列）。
struct ChainsView<'a> {
    gov_verified: bool,
    gov_count: usize,
    gov_head: &'a str,
    gov_counts: &'a BTreeMap<&'static str, usize>,
    appr_verified: bool,
    appr_count: usize,
    appr_head: &'a str,
    appr_counts: &'a BTreeMap<&'static str, usize>,
}
struct RoleMatrixRow {
    area: &'static str,
    viewer: &'static str,
    editor: &'static str,
    approver: &'static str,
    note: &'static str,
}

async fn role_matrix_page(State((_auth,)): State<(SharedAuth,)>, request: Request) -> Response {
    if let Err(response) = session_identity(&_auth, request.headers(), Role::Viewer) {
        return *response;
    }
    page_response("權限矩陣", "role-matrix", role_matrix_content())
}

fn role_matrix_content() -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理稽核" } h1 { "權限矩陣" } p class="page-subtitle" { "角色 × 能力對照；本表鏡像程式碼行為（is_public_path／session_identity 角色檢查），變更行為時須同步本表。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "角色 × 能力" } } }
            table class="api-table" {
                thead { tr { th { "能力區塊" } th { "Viewer" } th { "Editor" } th { "Approver" } th { "備註" } } }
                tbody {
                    @for row in ROLE_MATRIX {
                        tr {
                            td { (row.area) }
                            td { (row.viewer) }
                            td { (row.editor) }
                            td { (row.approver) }
                            td { (row.note) }
                        }
                    }
                }
            }
        }
        p class="data-note" { "來源：is_public_path／session_identity／approval 子路由的角色檢查。企業 IdP/SSO/MFA 未接線（DEC-12 未決）。" }
    }
}

const GLOSSARY: &[(&str, &str)] = &[
    (
        "工作流定義（WorkflowDefinition）",
        "版本化的簽核流程；含有序階段，建立後不可變，發布後才可被任務引用",
    ),
    (
        "階段（Stage）",
        "簽核的一站；有識別碼、名稱、SLA 秒數、升級路徑與逾時動作",
    ),
    (
        "任務（ApprovalTask）",
        "一筆待簽核事項；綁定定義＋版本＋目前階段，狀態機為 Pending→Approved/Rejected/Escalated/Expired/Cancelled",
    ),
    (
        "升級（Escalate）",
        "任務停在同階段但記錄升級對象並累計次數；不推進階段",
    ),
    (
        "逾時（Expire）",
        "due_at 已過且未決策；由 cleanup 標記為 Expired（終態）",
    ),
    (
        "決策者（Approver）",
        "可核准/駁回/升級的角色；來自 session，不接受表單指定",
    ),
    (
        "冪等鍵（Idempotency-Key）",
        "送出簽核時的選填鍵；同鍵重送回原任務且不追加事件",
    ),
    (
        "治理帳鏈",
        "專案/證據/檢驗/流程/個案/關卡的追加式 JSONL 雜湊鏈",
    ),
    (
        "核准帳鏈",
        "工作流與簽核任務的追加式 JSONL 雜湊鏈（與治理帳鏈獨立）",
    ),
    (
        "血緣（Lineage）",
        "欄位 → 事件 → 寫入面 → 呈現頁的對照；由型別系統窮舉鎖定",
    ),
];

async fn glossary_page(State((_auth,)): State<(SharedAuth,)>, request: Request) -> Response {
    if let Err(response) = session_identity(&_auth, request.headers(), Role::Viewer) {
        return *response;
    }
    page_response("領域字彙", "glossary", glossary_content())
}

fn glossary_content() -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "說明" } h1 { "領域字彙" } p class="page-subtitle" { "Ubiquitous Language——本工作台全部頁面與 API 使用同一套術語。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "核准工作流" } } }
            table class="api-table" {
                thead { tr { th { "詞彙" } th { "定義" } } }
                tbody {
                    @for (term, definition) in GLOSSARY.iter().take(7) {
                        tr { td { (term) } td { (definition) } }
                    }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "資料與鏈" } } }
            table class="api-table" {
                tbody {
                    @for (term, definition) in GLOSSARY.iter().skip(7) {
                        tr { td { (term) } td { (definition) } }
                    }
                }
            }
        }
        p class="data-note" { "本表為單一事實來源；變更術語須同步 BDD 場景與本表。" }
    }
}

async fn help_page(State((_auth,)): State<(SharedAuth,)>, request: Request) -> Response {
    if let Err(response) = session_identity(&_auth, request.headers(), Role::Viewer) {
        return *response;
    }
    page_response("操作說明", "help", help_content())
}

fn help_content() -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "說明" } h1 { "操作說明" } p class="page-subtitle" { "本機工作台的日常操作與疑難排解；全部流程零 JavaScript。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "登入" } } }
            ul {
                li { "以環境變數 EKPI_LOCAL_AUTH_USER／EKPI_LOCAL_AUTH_PASSWORD 設定帳號後，於 /login 登入。" }
                li { "登入流程為雙重提交：先 GET 取得一次性 token（隱藏欄位＋ekpi_csrf cookie），再 POST 回帶。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "角色" } } }
            ul {
                li { strong { "Viewer" } "：唯讀所有資料頁。" }
                li { strong { "Editor" } "：可建立專案/證據/檢驗/流程/個案，並送出簽核。" }
                li { strong { "Approver" } "：可核准/駁回/升級簽核任務。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "匯出（CLI）" } } }
            ul {
                li { code { "--export-evidence-pack DIR" } "：雙鏈摘要證據包。" }
                li { code { "--export-compliance-report DIR" } "：合規狀態報告（JSON＋MD）。" }
                li { code { "--data-map DIR" } "：資料血緣地圖。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "疑難排解" } } }
            table class="api-table" {
                thead { tr { th { "癥狀" } th { "可能原因" } th { "處理" } } }
                tbody {
                    tr { td { "401" } td { "未登入或工作階段過期" } td { "重新登入；檢查 cookie 是否被清除" } }
                    tr { td { "403" } td { "角色不足或跨站請求" } td { "確認角色（見權限矩陣）；勿跨站張貼表單" } }
                    tr { td { "404" } td { "識別碼不存在" } td { "從清單頁重新導航" } }
                    tr { td { "鏈驗證失敗（紅徽章）" } td { "資料檔被改動或不完整" } td { "立即停止寫入、保留檔案並聯絡管理者；原型不會自動修補鏈" } }
                }
            }
        }
    }
}

async fn about_page(State((_auth,)): State<(SharedAuth,)>, request: Request) -> Response {
    if let Err(response) = session_identity(&_auth, request.headers(), Role::Viewer) {
        return *response;
    }
    page_response("關於", "help", about_content())
}

fn about_content() -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "說明" } h1 { "關於本工作台" } p class="page-subtitle" { "誠實邊界——本頁說明本系統「是什麼」與「明確不是什麼」。" } } }
        section class="panel" {
            div class="panel-heading" { div { h2 { "是什麼" } } }
            ul {
                li { "本機（127.0.0.1）治理與簽核試點工作台；資料為追加式 JSONL 雜湊鏈。" }
                li { "零 JavaScript：全部互動為伺服器渲染 HTML＋CSS。" }
                li { "全部寫入以 session 使用者歸因，並記錄於雜湊鏈。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "明確不是什麼" } } }
            ul {
                li { "不是企業身分系統：本機 session 認證，無 IdP/SSO/MFA（DEC-12 未決）。" }
                li { "不是 WORM 儲存：JSONL 鏈可被具權限者改動，驗證僅能偵測（Phase B）。" }
                li { "非多租戶、非高可用部署；尚未通過 Phase A 全部關卡。" }
                li { "Agent 僅能 advisory（建議），永不自動核准。" }
            }
        }
        p class="data-note" { "完整邊界與證據：docs/quality/phase-gates.md、docs/traceability/register.md。本頁與 AGENTS.md 的誠實規則同步。" }
    }
}

fn nav_link(href: &str, key: &str, active: &str, label: &str, icon: &str) -> Markup {
    let selected = key == active;
    html! {
        a class={(if selected { "nav-link is-active" } else { "nav-link" })}
          href=(href) aria-current=(if selected { "page" } else { "false" }) {
            span class="nav-icon" aria-hidden="true" { (icon) }
            span { (label) }
        }
    }
}

fn dashboard_content(data: &GovernanceSnapshot) -> Markup {
    let reviewed = data.reviews.len();
    let waiting_review = data.evidence.len().saturating_sub(reviewed);
    let open_cases = data
        .cases
        .iter()
        .filter(|case| case.status != "closed")
        .count();
    let draft_workflows = data
        .workflows
        .iter()
        .filter(|workflow| workflow.status == "draft")
        .count();
    let closed_cases = data
        .cases
        .iter()
        .filter(|case| case.status == "closed")
        .count();
    let closed_pct = (closed_cases * 100)
        .checked_div(data.cases.len())
        .unwrap_or(0) as i64;
    let gate_go = data
        .gate_decisions
        .iter()
        .filter(|decision| decision.decision == "go")
        .count();
    let gate_hold = data
        .gate_decisions
        .iter()
        .filter(|decision| decision.decision == "hold")
        .count();
    let gate_exception = data
        .gate_decisions
        .iter()
        .filter(|decision| decision.decision == "exception")
        .count();
    html! {
        section class="page-heading" {
            div { p class="eyebrow" { "治理概況" } h1 { "工作總覽" } p class="page-subtitle" { "集中查看專案進度、文件追溯與需要人工確認的工作。" } }
            div class="chip-row" {
                a class="button button-primary" href="/projects" { "＋ 新增專案" }
                a class="button button-secondary" href="/search" { "⌕ 文件檢索" }
            }
        }
        div class="kpi-grid" {
            (kpi("專案", &data.projects.len().to_string(), "已登錄專案", "default"))
            (kpi("證據", &data.evidence.len().to_string(), "來源參照總數", "default"))
            (kpi("待審證據", &waiting_review.to_string(), "尚未留下人工審查紀錄", if waiting_review > 0 { "alert" } else { "good" }))
            (kpi("開放事項", &open_cases.to_string(), "風險、變更、缺失與 CAPA", if open_cases > 0 { "alert" } else { "good" }))
            (kpi("流程草稿", &draft_workflows.to_string(), "等待負責人審查發布", "default"))
            (kpi("稽核事件", &data.events.len().to_string(), "追加式雜湊鏈事件", "default"))
        }
        section class="content-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "品質事項狀態" } p { (closed_cases) " / " (data.cases.len()) " 已結案" } } }
                @if data.cases.is_empty() {
                    p class="empty-inline" { "尚無品質事項。" }
                } @else {
                    div class="donut-wrap" {
                        div class="donut" style=(format!("--pct: {closed_pct}; --donut-a: #1d9a6c")) {
                            div class="donut-hole" { strong { (closed_pct) "%" } span { "已結案" } }
                        }
                        div class="donut-legend" {
                            (legend_row("已結案", closed_cases, data.cases.len(), "#1d9a6c"))
                            (legend_row("未結案", data.cases.len() - closed_cases, data.cases.len(), "#d99a24"))
                        }
                    }
                }
            }
            div class="panel" {
                div class="panel-heading" { div { h2 { "關卡決策分佈" } p { "G1–G5 人工決議統計" } } }
                @if data.gate_decisions.is_empty() {
                    p class="empty-inline" { "尚無關卡決策。" }
                } @else {
                    div class="donut-legend" {
                        (legend_row("放行 go", gate_go, data.gate_decisions.len(), "#1d9a6c"))
                        (legend_row("暫緩 hold", gate_hold, data.gate_decisions.len(), "#d99a24"))
                        (legend_row("例外 exception", gate_exception, data.gate_decisions.len(), "#c15461"))
                    }
                    p class="form-hint" { "例外決策需登錄風險接受人與補救期限；不可豁免關卡不得例外。" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "最近專案" } p { "所有數字來自本機實際登錄資料。" } } a href="/projects" class="text-link" { "查看專案 →" } }
                (project_table(&data.projects, 5))
            }
            div class="panel" {
                div class="panel-heading" { div { h2 { "待辦焦點" } p { "需要負責人確認的項目" } } }
                ul class="focus-list" {
                    li { span class="focus-icon focus-amber" { "▤" } div { strong { (waiting_review) " 筆證據待審查" } small { "核對來源版本與證據目的" } } }
                    li { span class="focus-icon focus-blue" { "◇" } div { strong { (open_cases) " 項風險或品質事項開啟" } small { "追蹤負責人、期限與結案證據" } } }
                    li { span class="focus-icon focus-green" { "⌘" } div { strong { (draft_workflows) " 份流程草稿" } small { "發布前需由流程負責人核准" } } }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "最近活動" } p { "追加式記錄；摘要不代表來源系統已完成同步。" } } a href="/audit" class="text-link" { "開啟稽核紀錄 →" } }
                (recent_events(&data.events, 6))
            }
        }
        p class="data-note" { "試點指標尚未建立基線；此處只顯示登錄筆數，不代表證據完整度或流程改善成效。" }
    }
}

fn kpi(label: &str, value: &str, foot: &str, tone: &str) -> Markup {
    let class = match tone {
        "alert" => "kpi-card kpi-alert",
        "good" => "kpi-card kpi-good",
        _ => "kpi-card",
    };
    html! {
        article class=(class) {
            p class="kpi-label" { (label) }
            p class="kpi-value" { (value) }
            p class="kpi-foot" { (foot) }
        }
    }
}

fn legend_row(label: &str, count: usize, total: usize, color: &str) -> Markup {
    let pct = (count * 100).checked_div(total).unwrap_or(0);
    html! {
        div class="legend-row" {
            span class="legend-dot" style=(format!("background: {color}")) {}
            span { (label) }
            span class="legend-bar" { i style=(format!("background: {color}; width: {pct}%")) {} }
            span class="legend-score" { (count) }
        }
    }
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

/// The bundled data directory (same resolution as the CLI: compile-time
/// manifest path). The workbench reads it read-only for retrieval.
fn platform_data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// GET /search — read-only ranked retrieval over the versioned document
/// sections, permission-filtered by the validated local session role.
/// Zero-JS: the query travels as a GET parameter and results are rendered
/// server-side with `doc_id@version§section` citations.
async fn search_page(
    State((_ledger, auth)): State<(SharedLedger, SharedAuth)>,
    Query(params): Query<SearchParams>,
    request: Request,
) -> Response {
    let (username, role) = match session_identity_full(&auth, request.headers(), Role::Viewer) {
        Ok(identity) => identity,
        Err(response) => return *response,
    };
    let query = params.q.unwrap_or_default().trim().to_string();
    let mut hits: Vec<(crate::retrieval::RetrievalHit, String)> = Vec::new();
    let mut section_count = 0usize;
    let mut doc_count = 0usize;
    let mut load_error: Option<String> = None;
    if !query.is_empty() {
        match crate::docs::DocDatabase::load(&platform_data_dir().join("docs.json")) {
            Ok(db) => {
                let ctx = crate::retrieval::PermissionContext::new(
                    "default",
                    &username,
                    vec![role.as_str().to_string()],
                );
                let mut pool = Vec::new();
                for doc in &db.documents {
                    if !ctx.can_access(&doc.permissions) {
                        continue;
                    }
                    doc_count += 1;
                    for version in &doc.versions {
                        for section in &version.sections {
                            pool.push(crate::retrieval::PoolSection {
                                doc_id: doc.doc_id.clone(),
                                version: version.version.clone(),
                                section: section.section.clone(),
                                verbatim_text: section.verbatim_text.clone(),
                            });
                        }
                    }
                }
                section_count = pool.len();
                let index = crate::retrieval::InvertedIndex::build(&pool);
                let mut raw = index.search(&query, 0.15);
                raw.truncate(10);
                hits = raw
                    .into_iter()
                    .map(|hit| {
                        let snippet = pool
                            .iter()
                            .find(|section| {
                                section.doc_id == hit.doc_id
                                    && section.version == hit.version
                                    && section.section == hit.section
                            })
                            .map(|section| section.verbatim_text.clone())
                            .unwrap_or_default();
                        (hit, snippet)
                    })
                    .collect();
            }
            Err(error) => load_error = Some(error),
        }
    }
    page_response(
        "文件檢索",
        "search",
        search_content(
            &query,
            &hits,
            doc_count,
            section_count,
            load_error.as_deref(),
        ),
    )
}

fn search_content(
    query: &str,
    hits: &[(crate::retrieval::RetrievalHit, String)],
    doc_count: usize,
    section_count: usize,
    load_error: Option<&str>,
) -> Markup {
    html! {
        section class="page-heading" {
            div {
                p class="eyebrow" { "治理 / 文件檢索" }
                h1 { "文件檢索與引用定位" }
                p class="page-subtitle" { "字面相似度檢索版本化條款；結果附 doc@version§section 引用，可直接核對來源。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "查詢" } p { "輸入關鍵詞或條款片段；檢索為唯讀操作（GET），不寫入稽核鏈。" } } }
            form class="inline-form" method="get" action="/search" {
                label class="field" { span { "關鍵詞" } input type="text" name="q" value=(query) maxlength="200" placeholder="例如：變更核准、版本綁定、CAPA"; }
                button class="button button-primary" type="submit" { "檢索" }
            }
            div class="chip-row" {
                a class="chip" href="/search?q=%E8%AE%8A%E6%9B%B4%E6%A0%B8%E5%87%86" { "變更核准" }
                a class="chip" href="/search?q=%E7%89%88%E6%9C%AC" { "版本" }
                a class="chip" href="/search?q=%E5%93%81%E8%B3%AA" { "品質" }
                a class="chip" href="/search?q=CAPA" { "CAPA" }
            }
        }
        @if let Some(error) = load_error {
            div class="notice notice-red" role="alert" { strong { "文件庫載入失敗：" } span { (error) } }
        }
        @if query.is_empty() {
            (empty_state("輸入關鍵詞開始檢索", "檢索涵蓋已載入的版本化條款；結果會標示版本與章節錨點，供人工核對。", "/search", "重新整理"))
        } @else if hits.is_empty() {
            div class="panel" {
                (empty_state("找不到相符條款", "沒有條款達到相似度門檻。這是誠實的空結果——可嘗試更短或更通用的關鍵詞。", "/search", "重新檢索"))
            }
        } @else {
            section class="panel" {
                div class="panel-heading" { div { h2 { "檢索結果" } p { (hits.len()) " 筆結果（最多顯示 10 筆）· 相似度門檻 0.15" } } span class="badge badge-blue" { "唯讀" } }
                div class="result-list" {
                    @for (hit, snippet) in hits {
                        article class="result-card" {
                            div class="result-head" {
                                span class="result-cite" { (hit.doc_id) "@" (hit.version) "§" (hit.section) }
                                span class="meter-label" { "相似度" }
                            }
                            p class="result-snippet" {
                                @if snippet.chars().count() > 180 {
                                    (snippet.chars().take(180).collect::<String>()) "…"
                                } @else {
                                    (snippet)
                                }
                            }
                            div class="result-foot" {
                                div class="meter meter-info" style=(format!("--w: {}", (hit.score * 100.0).round() as i64)) { i {} }
                                span class="meter-label" { (format!("{:.2}", hit.score)) }
                            }
                        }
                    }
                }
            }
        }
        p class="data-note" {
            "索引涵蓋 " (doc_count) " 份文件、" (section_count) " 個條款區段（依登入角色過濾權限）。檢索為字面相似度，不含語意模型；文件本體保留在來源平台。"
        }
    }
}

/// GET /data-map — 欄位 → 事件 → 寫入面 → 呈現頁的血緣（S11，CTR-DATA-008）。
/// 血緣本體為 `'static` 常數表（型別系統驅動、窮舉鎖定）；此頁另附雙鏈
/// 的**現況事件計數**，讓血緣與實際資料對照。唯讀（GET），Viewer 以上。
async fn data_map_page(
    State((ledger, auth, approvals)): State<(SharedLedger, SharedAuth, SharedApprovals)>,
    request: Request,
) -> Response {
    let (_username, _role) = match session_identity_full(&auth, request.headers(), Role::Viewer) {
        Ok(identity) => identity,
        Err(response) => return *response,
    };
    let (governance_counts, approvals_counts, approvals_integrity) =
        match approvals_view(&approvals) {
            Ok((_, _, integrity, _)) => {
                let snapshot = match ledger.lock() {
                    Ok(ledger) => ledger.snapshot(),
                    Err(_) => return page_error_response("data-map", "治理資料鎖定失敗"),
                };
                let mut governance_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
                for event in &snapshot.events {
                    let tag = crate::data_map::governance_payload_tag(&event.payload);
                    *governance_counts.entry(tag).or_insert(0) += 1;
                }
                let mut approvals_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
                for (tag, count) in approvals_view_counts(&approvals) {
                    approvals_counts.insert(tag, count);
                }
                (governance_counts, approvals_counts, integrity)
            }
            Err(error) => return page_error_response("data-map", &error),
        };
    page_response(
        "資料地圖",
        "data-map",
        data_map_content(&governance_counts, &approvals_counts, approvals_integrity),
    )
}

/// 核准鏈各事件標籤的現況計數（唯讀；借道 store 的摘要存取子）。
fn approvals_view_counts(approvals: &SharedApprovals) -> Vec<(&'static str, usize)> {
    let store = match approvals.lock() {
        Ok(store) => store,
        Err(_) => return Vec::new(),
    };
    let counts = store.event_type_counts();
    crate::data_map::APPROVAL_LINEAGE
        .iter()
        .map(|entry| (entry.event, counts.get(entry.event).copied().unwrap_or(0)))
        .collect()
}

fn data_map_content(
    governance_counts: &BTreeMap<&'static str, usize>,
    approval_counts: &BTreeMap<&'static str, usize>,
    approvals_integrity: bool,
) -> Markup {
    html! {
        section class="page-heading" {
            div {
                p class="eyebrow" { "治理 / 資料地圖" }
                h1 { "資料血緣地圖" }
                p class="page-subtitle" { "欄位 → 事件 → 寫入面 → 呈現頁；血緣由型別系統窮舉鎖定，新增事件變體時編譯失敗即強制更新。" }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "治理事件血緣" } p { "governance ledger（JSONL 雜湊鏈）全部事件變體" } }
                span class="meter-label" { (format!("已記錄 {} 筆", governance_counts.values().sum::<usize>())) }
            }
            table class="api-table" {
                thead { tr { th { "事件" } th { "欄位" } th { "寫入面" } th { "呈現頁" } th { "現有事件數" } } }
                tbody {
                    @for entry in crate::data_map::GOVERNANCE_LINEAGE {
                        tr {
                            td { span class="result-cite" { (entry.event) } }
                            td { (entry.fields.join("、")) }
                            td { (entry.write_surfaces.join("；")) }
                            td { (entry.pages.join(" ")) }
                            td class="num" { (governance_counts.get(entry.event).copied().unwrap_or(0)) }
                        }
                    }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "核准事件血緣" } p { "approval store（獨立 JSONL 雜湊鏈）全部事件變體" } }
                span class="badge" class=(if approvals_integrity { "badge-green" } else { "badge-red" }) { (if approvals_integrity { "鏈已驗證" } else { "鏈驗證失敗" }) }
            }
            table class="api-table" {
                thead { tr { th { "事件" } th { "欄位" } th { "寫入面" } th { "呈現頁" } th { "現有事件數" } } }
                tbody {
                    @for entry in crate::data_map::APPROVAL_LINEAGE {
                        tr {
                            td { span class="result-cite" { (entry.event) } }
                            td { (entry.fields.join("、")) }
                            td { (entry.write_surfaces.join("；")) }
                            td { (entry.pages.join(" ")) }
                            td class="num" { (approval_counts.get(entry.event).copied().unwrap_or(0)) }
                        }
                    }
                }
            }
        }
        p class="data-note" { "血緣為 'static 常數表＋窮舉 match 鎖定（編譯期保證覆蓋全部事件變體）；本頁為唯讀（GET），不寫入任何鏈。" }
    }
}

fn projects_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 專案" } h1 { "專案登錄" } p class="page-subtitle" { "使用共同識別碼串連流程、證據、決策與改善事項。" } } }
        (alert(error))
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "建立專案" } p { "第一版先記錄責任、部門與風險層級。" } } }
                form class="form-stack" method="post" action="/projects" {
                    (text_field("name", "專案名稱", "例如：跨部門文件治理試點", true, ""))
                    (text_field("department", "主要部門", "", true, ""))
                    (text_field("owner", "專案負責人", "", true, ""))
                    label class="field" { span { "初始風險等級" } select name="risk_tier" required { option value="medium" selected { "中" } option value="low" { "低" } option value="high" { "高" } option value="critical" { "重大" } } }
                    p class="form-hint" { "操作人由目前登入的本機帳號自動記錄，不採信表單輸入。" }
                    button class="button button-primary" type="submit" { "建立專案" }
                    p class="form-hint" { "請勿在本機原型輸入機密或受管制資訊。" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "已登錄專案" } p { (data.projects.len()) " 個專案" } } }
                (project_table(&data.projects, usize::MAX))
            }
        }
    }
}

fn project_table(projects: &[Project], limit: usize) -> Markup {
    html! {
        @if projects.is_empty() {
            (empty_state("尚無專案資料", "建立一筆專案記錄後，才能串連文件證據與治理事件。", "/projects", "建立專案"))
        } @else {
            div class="table-wrap" {
                table {
                    caption class="visually-hidden" { "已登錄專案清單" }
                    thead { tr { th scope="col" { "專案" } th scope="col" { "部門" } th scope="col" { "負責人" } th scope="col" { "風險" } } }
                    tbody {
                        @for project in projects.iter().rev().take(limit) {
                            tr { th scope="row" { strong { (project.name) } small class="row-id" { (project.id) } } td { (project.department) } td { (project.owner) } td { (risk_badge(&project.risk_tier)) } }
                        }
                    }
                }
            }
        }
    }
}

fn evidence_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 文件與證據" } h1 { "文件與證據追溯" } p class="page-subtitle" { "記錄權威來源、版本與審查結果；文件本體保留在原平台。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再登錄其文件與證據來源。"))) }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "連結證據" } p { "這是來源參照，不會複製或同步雲端文件。" } } }
                form class="form-stack" method="post" action="/evidence" {
                    label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                    (text_field("title", "文件／證據標題", "", true, ""))
                    label class="field" { span { "來源平台" } select name="source_system" required { option value="Microsoft 365" { "Microsoft 365" } option value="Google Workspace" { "Google Workspace" } option value="Other" { "其他／混合工具" } } }
                    (text_field("source_url", "來源連結", "https://…", true, ""))
                    (text_field("source_version", "來源版本／修訂", "例如：v3.2 或平台版本 ID", true, ""))
                    (text_field("evidence_type", "證據類型", "例如：核准紀錄、測試報告", true, ""))
                    (text_field("owner", "文件負責人", "", true, ""))
                    p class="form-hint" { "操作人由目前登入的本機帳號自動記錄，不採信表單輸入。" }
                    button class="button button-primary" type="submit" { "登錄來源參照" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "證據索引" } p { (data.evidence.len()) " 筆來源參照" } } }
                (evidence_table(data))
            }
        }
    }
}

fn evidence_table(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.evidence.is_empty() {
            (empty_state("尚無證據參照", "先新增專案，再從來源平台登錄連結和文件版本。", "/projects", "建立專案"))
        } @else {
            div class="table-wrap" {
                table {
                    caption class="visually-hidden" { "文件與證據來源索引" }
                    thead { tr { th scope="col" { "文件與來源" } th scope="col" { "專案" } th scope="col" { "負責人" } th scope="col" { "審查狀態" } th scope="col" { "人工審查" } } }
                    tbody {
                        @for item in data.evidence.iter().rev() {
                            @let project = data.projects.iter().find(|p| p.id == item.project_id);
                            @let review = data.reviews.get(&item.id);
                            tr {
                                th scope="row" {
                                    strong { (item.title) }
                                    small class="row-id" { (item.id) " · " (item.source_system) " · " (item.source_version) }
                                    a class="source-link" href=(item.source_url) target="_blank" rel="noopener noreferrer" { "開啟來源連結 ↗" }
                                }
                                td { @if let Some(project) = project { (project.id) } @else { "專案缺失" } }
                                td { (item.owner) }
                                td { @if let Some(review) = review { (review_badge(&review.result)) } @else { (status_badge("pending", "待審查")) } }
                                td {
                                    @if let Some(review) = review { small { (review.reviewer) " · " (format_timestamp(review.reviewed_at)) } }
                                    form class="inline-review" method="post" action=(format!("/evidence/{}/review", item.id)) {
                                        select name="result" aria-label="審查結果" { option value="accepted" { "確認有效" } option value="changes_requested" { "退回補充" } }
                                        input type="text" name="note" aria-label="審查備註或退回說明" placeholder="備註；退回必填" maxlength="1000";
                                        button class="button button-small" type="submit" { "記錄" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn workflows_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 流程範本" } h1 { "可配置流程範本" } p class="page-subtitle" { "以步驟清單快速建立部門流程；每個草稿必須經流程負責人審查才可發布。" } } }
        (alert(error))
        div class="notice notice-blue" { strong { "AI 連線尚未配置" } span { "本原型不會把需求傳給外部模型。請以人工整理的草稿建立範本；AI 服務需經資料與供應者審查後再接入。" } }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "建立流程草稿" } p { "先定義流程目的、負責人與可驗證階段。" } } }
                form class="form-stack" method="post" action="/workflows" {
                    label class="field" { span { "基於既有範本（選填）" } select name="base_id" { option value="" { "建立新範本" } @for workflow in latest_workflows(&data.workflows) { option value=(workflow.id) { (workflow.id) " · " (workflow.title) " v" (workflow.version) } } } }
                    (text_field("title", "流程名稱", "", true, ""))
                    (text_field("department", "適用部門／跨部門範圍", "", true, ""))
                    (text_field("owner", "流程負責人", "", true, ""))
                    label class="field" { span { "流程目的" } textarea name="description" required maxlength="1500" rows="3" placeholder="這條流程要解決的業務問題與完成條件" {} }
                    label class="field" { span { "流程階段" } textarea name="stages" required rows="7" placeholder="每行一個階段，例如：\n提出需求\n風險評估\n審查核准\n執行與驗證\n結案封存" {} }
                    small class="field-help" { "每行一個階段，最多 30 階段。五個強制關卡在生命週期頁面管理。" }
                    p class="form-hint" { "操作人由目前登入的本機帳號自動記錄，不採信表單輸入。" }
                    button class="button button-primary" type="submit" { "儲存為草稿" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "已登錄範本" } p { (data.workflows.len()) " 個版本" } } }
                (workflow_list(data))
            }
        }
    }
}

fn workflow_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.workflows.is_empty() { (empty_state("尚無流程範本", "建立一份草稿並由流程負責人審查發布。", "/workflows", "建立流程草稿")) }
        @else {
            div class="workflow-list" {
                @for workflow in latest_workflows(&data.workflows) {
                    article class="workflow-card" {
                        div class="workflow-title-row" { div { p class="row-id" { (workflow.id) " · v" (workflow.version) } h3 { (workflow.title) } } (workflow_badge(&workflow.status)) }
                        p class="muted" { (workflow.department) " · 負責人 " (workflow.owner) }
                        p class="multiline" { (workflow.description) }
                        ol class="stage-chips" { @for stage in &workflow.stages { li { (stage) } } }
                        @if workflow.status == "draft" {
                            form class="publish-form" method="post" action=(format!("/workflows/{}/{}/publish", workflow.id, workflow.version)) {
                                p class="form-hint" { "發布人由目前登入的本機帳號自動記錄。" }
                                button class="button button-small button-secondary" type="submit" { "核准並發布此版本" }
                            }
                        } @else {
                            p class="success-line" { "已由 " (workflow.published_by.as_deref().unwrap_or("未記錄")) " 於 " (format_timestamp(workflow.published_at.unwrap_or(workflow.created_at))) " 發布" }
                        }
                    }
                }
            }
        }
    }
}

fn latest_workflows(workflows: &[Workflow]) -> Vec<&Workflow> {
    let mut latest = std::collections::BTreeMap::<&str, &Workflow>::new();
    for workflow in workflows {
        match latest.get(workflow.id.as_str()) {
            Some(current) if current.version >= workflow.version => {}
            _ => {
                latest.insert(workflow.id.as_str(), workflow);
            }
        }
    }
    latest.into_values().collect()
}

fn controls_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 品質閉環" } h1 { "風險與品質事件" } p class="page-subtitle" { "集中追蹤風險、變更、缺失及矯正措施，保留責任人、期限和處理紀錄。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再登錄品質事件。"))) }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "新增事件" } p { "CAPA 關閉時須附複驗或有效性紀錄。" } } }
                form class="form-stack" method="post" action="/controls" {
                    label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                    label class="field" { span { "事件類型" } select name="kind" required { option value="risk" { "風險" } option value="change" { "變更" } option value="finding" { "缺失／問題" } option value="capa" { "CAPA 矯正預防" } } }
                    (text_field("title", "標題", "", true, ""))
                    label class="field" { span { "內容與影響" } textarea name="description" required maxlength="2000" rows="4" {} }
                    (text_field("owner", "負責人", "", true, ""))
                    label class="field" { span { "到期日" } input type="date" name="due_date" required; }
                    p class="form-hint" { "操作人由目前登入的本機帳號自動記錄，不採信表單輸入。" }
                    button class="button button-primary" type="submit" { "新增追蹤事項" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "追蹤清單" } p { (data.cases.len()) " 項風險與品質事件" } } }
                (quality_case_list(data))
            }
        }
    }
}

fn inspections_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 品質保證" } h1 { "29 項品質檢查" } p class="page-subtitle" { "逐項記錄判定與可追溯依據；不適用項目需說明，不符合項目需連到缺失。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再記錄品質檢查。"))) }
        section class="panel" {
            div class="panel-heading" { div { h2 { "建立檢查紀錄" } p { "符合須填證據參照；不符合須填說明及缺失編號；不適用須填理由。" } } }
            @if data.projects.is_empty() { p class="empty-inline" { "目前沒有可選專案。" } }
            @else {
                form class="form-stack" method="post" action="/inspections" {
                    div class="inspection-meta" {
                        label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                    }
                    div class="notice notice-blue" { strong { "判定規則" } span { "符合：填可查證的證據 ID／連結；不符合：填說明及缺失編號；不適用：填理由。檢查人由目前登入的本機帳號自動記錄。" } }
                    div class="table-wrap" {
                        table class="inspection-table" {
                            caption class="visually-hidden" { "29 項專案品質檢查表" }
                            thead { tr { th scope="col" { "檢查項目" } th scope="col" { "判定" } th scope="col" { "證據參照" } th scope="col" { "說明／不適用理由" } th scope="col" { "缺失編號" } } }
                            tbody { @for item in QUALITY_CHECKLIST {
                                @let result_name = format!("result_{}", item.id);
                                @let evidence_name = format!("evidence_{}", item.id);
                                @let note_name = format!("note_{}", item.id);
                                @let issue_name = format!("issue_{}", item.id);
                                tr {
                                    th scope="row" { small class="row-id" { (item.id) " · " (item.category) } strong class="check-criterion" { (item.criterion) } }
                                    td { select name=(result_name) required aria-label=(format!("{} 判定", item.id)) { option value="" selected disabled { "選擇" } option value="pass" { "符合" } option value="fail" { "不符合" } option value="not_applicable" { "不適用" } } }
                                    td { input type="text" name=(evidence_name) maxlength="2048" aria-label=(format!("{} 證據參照", item.id)) placeholder="證據 ID／連結"; }
                                    td { input type="text" name=(note_name) maxlength="1000" aria-label=(format!("{} 說明", item.id)) placeholder="說明／理由"; }
                                    td { input type="text" name=(issue_name) maxlength="80" aria-label=(format!("{} 缺失編號", item.id)) placeholder="不符合時必填"; }
                                }
                            } }
                        }
                    }
                    button class="button button-primary" type="submit" { "保存完整檢查紀錄" }
                }
            }
        }
        section class="panel inspection-history" {
            div class="panel-heading" { div { h2 { "已保存檢查" } p { (data.inspections.len()) " 份完整檢查紀錄" } } }
            @if data.inspections.is_empty() { p class="empty-inline" { "尚無檢查紀錄；每份紀錄都必須包含全部 29 個項目。" } }
            @else { div class="table-wrap" { table {
                caption class="visually-hidden" { "已保存品質檢查紀錄" }
                thead { tr { th scope="col" { "檢查識別碼" } th scope="col" { "專案" } th scope="col" { "檢查人" } th scope="col" { "符合" } th scope="col" { "不符合" } th scope="col" { "不適用" } th scope="col" { "日期" } th scope="col" { "逐項結果" } } }
                tbody { @for inspection in data.inspections.iter().rev() {
                    @let passed = inspection.results.iter().filter(|item| item.result == "pass").count();
                    @let failed = inspection.results.iter().filter(|item| item.result == "fail").count();
                    @let not_applicable = inspection.results.iter().filter(|item| item.result == "not_applicable").count();
                    tr {
                        th scope="row" { (inspection.id) }
                        td { (inspection.project_id) }
                        td { (inspection.reviewer) }
                        td { (passed) }
                        td { (failed) }
                        td { (not_applicable) }
                        td { (format_timestamp(inspection.created_at)) }
                        td {
                            details class="inspection-details" {
                                summary { "查看 29 項" }
                                ol class="inspection-result-list" {
                                    @for result in &inspection.results {
                                        @let definition = QUALITY_CHECKLIST.iter().find(|item| item.id == result.item_id);
                                        li {
                                            div class="inspection-result-heading" { strong { (result.item_id) } (check_result_badge(&result.result)) }
                                            @if let Some(definition) = definition { p { (definition.criterion) } }
                                            @if !result.evidence_reference.is_empty() { small { "證據：" (result.evidence_reference) } }
                                            @if !result.note.is_empty() { small { "說明：" (result.note) } }
                                            @if !result.issue_id.is_empty() { small { "缺失：" (result.issue_id) } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } }
            } } }
        }
    }
}

fn quality_case_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.cases.is_empty() { (empty_state("尚無追蹤事項", "新增風險、變更、缺失或 CAPA，並指定負責人與期限。", "/controls", "新增事項")) }
        @else {
            div class="case-list" {
                @for case in data.cases.iter().rev() {
                    article class="case-card" {
                        div class="workflow-title-row" { div { p class="row-id" { (case.id) " · " (kind_label(&case.kind)) " · " (case.project_id) } h3 { (case.title) } } (case_status_badge(&case.status)) }
                        p class="multiline" { (case.description) }
                        p class="muted" { "負責人 " (case.owner) " · 到期 " (case.due_date) }
                        @if case.status != "closed" {
                            form class="publish-form" method="post" action=(format!("/controls/{}/status", case.id)) {
                                label class="compact-field" { span { "更新狀態" } select name="status" { option value="open" selected[case.status == "open"] { "待處理" } option value="in_progress" selected[case.status == "in_progress"] { "處理中" } option value="verification" selected[case.status == "verification"] { "待複驗" } option value="closed" { "結案" } } }
                                label class="compact-field" { span { "處理／結案紀錄" } input type="text" name="note" maxlength="1200" required placeholder="複驗結果或更新說明"; }
                                p class="form-hint" { "更新人由目前登入的本機帳號自動記錄。" }
                                button class="button button-small button-secondary" type="submit" { "記錄更新" }
                            }
                        } @else if let Some(note) = &case.closure_note { p class="success-line" { "結案紀錄：" (note) } }
                    }
                }
            }
        }
    }
}

fn lifecycle_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    let stages = [
        "專案立案與分類",
        "需求與合規分析",
        "計畫與基準建立",
        "文件與系統設定",
        "執行與過程管制",
        "品質檢查與驗證",
        "變更與影響分析",
        "缺失與風險閉環",
        "階段審查與放行",
        "最終驗收與移交",
        "結案與封存",
        "效益追蹤與改善",
    ];
    let gates = [
        ("G1", "立案", "目的、贊助人、範圍與風險分級已確認。"),
        ("G2", "基準", "需求、驗收標準、計畫及責任已確認。"),
        ("G3", "執行", "關鍵品質檢查通過，或缺失已正式登錄。"),
        ("G4", "交付", "必要證據、適用要求與驗收條件已滿足。"),
        ("G5", "結案", "移交、未結責任、封存及改善計畫已完成。"),
    ];
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理模型" } h1 { "生命週期與強制關卡" } p class="page-subtitle" { "每階段定義責任與放行條件；關卡決定由授權人員記錄。" } } }
        (alert(error))
        section class="panel" {
            div class="panel-heading" { div { h2 { "12 個專案階段" } p { "來源：使用者提供的治理藍圖；試點時由流程擁有者確認適用性。" } } }
            ol class="lifecycle-grid" { @for (index, stage) in stages.iter().enumerate() { li { span class="stage-number" { (format!("{:02}", index + 1)) } strong { (stage) } } } }
        }
        section class="gate-layout" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "五個強制關卡" } p { "例外需有風險接受者、補救措施和期限。" } } }
                div class="gate-list" { @for (id, name, condition) in gates { article class="gate-item" { span class="gate-id" { (id) } div { strong { (name) } p { (condition) } } } } }
                p class="form-hint" { "法律、安全及其他不可豁免要求不可一般例外放行。原型要求登記人明確聲明是否涉及不可豁免事項。" }
            }
            div class="panel" {
                div class="panel-heading" { div { h2 { "記錄人工關卡決議" } p { "僅記錄人工決定，不自動推定通過。" } } }
                @if data.projects.is_empty() { p class="empty-inline" { "先建立專案後才能記錄決議。" } }
                @else {
                    form class="form-stack" method="post" action="/lifecycle" {
                        label class="field" { span { "專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                        label class="field" { span { "關卡" } select name="gate_id" required { @for (id, name, _) in gates { option value=(id) { (id) " · " (name) } } } }
                        label class="field" { span { "人工決議" } select name="decision" required { option value="hold" { "暫緩" } option value="go" { "放行" } option value="exception" { "一般例外" } } }
                        label class="field" { span { "是否涉及不可豁免要求" } select name="non_waivable" required { option value="yes" { "是" } option value="no" { "否" } } }
                        label class="field" { span { "原因／例外依據" } textarea name="reason" rows="2" maxlength="1200" placeholder="暫緩或例外時必填" {} }
                        (text_field("evidence_reference", "證據參照（選填）", "來源系統連結或證據 ID", false, ""))
                        (text_field("risk_acceptor", "風險接受者（例外必填）", "", false, ""))
                        label class="field" { span { "補救期限（例外必填）" } input type="date" name="remediation_due"; }
                        p class="form-hint" { "決議記錄人由目前登入的本機帳號自動記錄，不採信表單輸入。" }
                        button class="button button-primary" type="submit" { "保存決議" }
                    }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "決議歷程" } p { (data.gate_decisions.len()) " 筆人工關卡記錄" } } }
            (gate_decision_list(data))
        }
    }
}

fn gate_decision_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.gate_decisions.is_empty() { p class="empty-inline" { "尚無關卡決議。建立專案後，保存實際人工決策。" } }
        @else {
            div class="table-wrap" { table {
                caption class="visually-hidden" { "關卡決議歷程" }
                thead { tr { th scope="col" { "專案／關卡" } th scope="col" { "決議" } th scope="col" { "記錄人" } th scope="col" { "理由／補救" } th scope="col" { "時間" } } }
                tbody { @for decision in data.gate_decisions.iter().rev() { tr {
                    th scope="row" { (decision.project_id) " · " (decision.gate_id) }
                    td { (gate_badge(&decision.decision)) }
                    td { (decision.actor) }
                    td class="multiline" { (decision.reason) @if decision.decision == "exception" { small class="row-id" { "風險接受者：" (decision.risk_acceptor) "；期限：" (decision.remediation_due) } } }
                    td { (format_timestamp(decision.created_at)) }
                } } }
            } }
        }
    }
}

fn audit_content(data: &GovernanceSnapshot, integrity: bool) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理控制" } h1 { "追加式稽核紀錄" } p class="page-subtitle" { "每次新增、審查、發布與決議都新增事件，不覆寫先前事件。" } } }
        div class={(if integrity { "notice notice-green" } else { "notice notice-red" })} role="status" {
            strong { @if integrity { "本機雜湊鏈驗證通過" } @else { "稽核鏈驗證失敗" } }
            span { "雜湊鏈能偵測部分竄改；它不等同不可變 WORM 儲存或防竄改企業稽核服務。" }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "事件明細" } p { (data.events.len()) " 筆事件 · 由新至舊" } } }
            @if data.events.is_empty() { (empty_state("尚無稽核事件", "新增專案或治理記錄後，事件會出現在此處。", "/projects", "新增專案")) }
            @else {
                div class="audit-timeline" {
                    @for event in data.events.iter().rev() {
                        article class="audit-event" {
                            span class="audit-sequence" { (format!("{:04}", event.sequence)) }
                            div class="audit-event-body" {
                                div class="workflow-title-row" { strong { (event_label(&event.payload)) } small { (event.actor) " · " (format_timestamp(event.timestamp_unix)) } }
                                p class="row-id" { "前序雜湊 " span class="hash-chip" { (short_hash(&event.previous_sha256)) } }
                                p class="hash-value" { "SHA-256 " span class="hash-chip" { (short_hash(&event.sha256)) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn recent_events(events: &[crate::governance::LedgerEvent], limit: usize) -> Markup {
    html! {
        @if events.is_empty() { p class="empty-inline" { "尚無活動記錄。建立專案即可開始試用治理流程。" } }
        @else {
            ul class="activity-list" { @for event in events.iter().rev().take(limit) { li { span class="activity-marker" aria-hidden="true" {} div { strong { (event_label(&event.payload)) } small { (event.actor) " · " (format_timestamp(event.timestamp_unix)) } } } } }
        }
    }
}

fn alert(message: Option<&str>) -> Markup {
    html! { @if let Some(message) = message { div class="alert alert-error" role="alert" { (message) } } }
}

fn text_field(name: &str, label: &str, placeholder: &str, required: bool, value: &str) -> Markup {
    html! {
        label class="field" { span { (label) } input type="text" name=(name) value=(value) placeholder=(placeholder) maxlength="2048" required[required]; }
    }
}

fn empty_state(title: &str, description: &str, href: &str, action: &str) -> Markup {
    html! { div class="empty-state" { span class="empty-icon" aria-hidden="true" { "＋" } h3 { (title) } p { (description) } a class="text-link" href=(href) { (action) " →" } } }
}

fn risk_badge(value: &str) -> Markup {
    let (class, text) = match value {
        "low" => ("badge badge-green", "低"),
        "high" => ("badge badge-amber", "高"),
        "critical" => ("badge badge-red", "重大"),
        _ => ("badge badge-blue", "中"),
    };
    html! { span class=(class) { (text) } }
}

fn review_badge(value: &str) -> Markup {
    match value {
        "accepted" => html! { span class="badge badge-green" { "已確認" } },
        _ => html! { span class="badge badge-amber" { "退回補充" } },
    }
}

fn workflow_badge(value: &str) -> Markup {
    if value == "published" {
        html! { span class="badge badge-green" { "已發布" } }
    } else {
        html! { span class="badge badge-amber" { "草稿待審" } }
    }
}

fn case_status_badge(value: &str) -> Markup {
    let (class, label) = match value {
        "in_progress" => ("badge badge-blue", "處理中"),
        "verification" => ("badge badge-amber", "待複驗"),
        "closed" => ("badge badge-green", "已結案"),
        _ => ("badge badge-red", "待處理"),
    };
    html! { span class=(class) { (label) } }
}

fn gate_badge(value: &str) -> Markup {
    let (class, label) = match value {
        "go" => ("badge badge-green", "放行"),
        "exception" => ("badge badge-amber", "例外"),
        _ => ("badge badge-red", "暫緩"),
    };
    html! { span class=(class) { (label) } }
}

fn status_badge(_key: &str, label: &str) -> Markup {
    html! { span class="badge badge-amber" { (label) } }
}

fn check_result_badge(value: &str) -> Markup {
    match value {
        "pass" => html! { span class="badge badge-green" { "符合" } },
        "fail" => html! { span class="badge badge-red" { "不符合" } },
        _ => html! { span class="badge badge-blue" { "不適用" } },
    }
}

fn kind_label(value: &str) -> &'static str {
    match value {
        "risk" => "風險",
        "change" => "變更",
        "capa" => "CAPA",
        _ => "缺失",
    }
}

fn format_timestamp(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let remainder = seconds % 86_400;
    let hour = remainder / 3_600;
    let minute = (remainder % 3_600) / 60;
    let second = remainder % 60;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

/// Human-readable event label; shared with the CLI evidence-pack export.
pub fn event_label(payload: &crate::governance::EventPayload) -> String {
    use crate::governance::EventPayload;
    match payload {
        EventPayload::ProjectCreated(project) => format!("建立專案：{}", project.name),
        EventPayload::EvidenceLinked(item) => format!("連結證據：{}", item.title),
        EventPayload::EvidenceReviewed(review) => format!(
            "審查證據：{} · {} · {}",
            review.evidence_id,
            review_label_text(&review.result),
            review.note
        ),
        EventPayload::WorkflowSaved(workflow) => {
            format!("儲存流程草稿：{} v{}", workflow.title, workflow.version)
        }
        EventPayload::WorkflowPublished(publication) => {
            format!("發布流程：{} v{}", publication.id, publication.version)
        }
        EventPayload::QualityCaseCreated(case) => {
            format!("新增{}：{}", kind_label(&case.kind), case.title)
        }
        EventPayload::QualityCaseUpdated(update) => format!(
            "更新品質事件：{} → {} · {}",
            update.id,
            case_status_label(&update.status),
            update.note
        ),
        EventPayload::QualityInspectionRecorded(inspection) => {
            let failed = inspection
                .results
                .iter()
                .filter(|item| item.result == "fail")
                .count();
            format!(
                "完成品質檢查：{} · {} 項檢查 · {} 項不符合",
                inspection.project_id,
                inspection.results.len(),
                failed
            )
        }
        EventPayload::GateDecisionRecorded(decision) => format!(
            "{} 決議：{} · {}",
            decision.gate_id,
            gate_decision_label(&decision.decision),
            decision.reason
        ),
    }
}

fn case_status_label(value: &str) -> &'static str {
    match value {
        "in_progress" => "處理中",
        "verification" => "待複驗",
        "closed" => "結案",
        _ => "待處理",
    }
}

fn gate_decision_label(value: &str) -> &'static str {
    match value {
        "go" => "放行",
        "exception" => "一般例外",
        _ => "暫緩",
    }
}

fn review_label_text(value: &str) -> &'static str {
    match value {
        "accepted" => "已確認",
        _ => "退回補充",
    }
}

#[derive(Deserialize)]
struct ProjectForm {
    name: String,
    department: String,
    owner: String,
    risk_tier: String,
}
#[derive(Deserialize)]
struct EvidenceForm {
    project_id: String,
    title: String,
    source_system: String,
    source_url: String,
    source_version: String,
    evidence_type: String,
    owner: String,
}
#[derive(Deserialize)]
struct ReviewForm {
    result: String,
    note: String,
}
#[derive(Deserialize)]
struct WorkflowForm {
    base_id: String,
    title: String,
    department: String,
    owner: String,
    description: String,
    stages: String,
}
#[derive(Deserialize)]
struct QualityCaseForm {
    project_id: String,
    kind: String,
    title: String,
    description: String,
    owner: String,
    due_date: String,
}
#[derive(Deserialize)]
struct QualityCaseUpdateForm {
    status: String,
    note: String,
}
#[derive(Deserialize)]
struct GateDecisionForm {
    project_id: String,
    gate_id: String,
    decision: String,
    reason: String,
    evidence_reference: String,
    risk_acceptor: String,
    remediation_due: String,
    non_waivable: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_cookie_suffix_only_enables_for_value_one() {
        assert_eq!(secure_cookie_suffix(Some("1")), "; Secure");
        assert_eq!(secure_cookie_suffix(Some("0")), "");
        assert_eq!(secure_cookie_suffix(Some("true")), "");
        assert_eq!(secure_cookie_suffix(None), "");
    }

    #[test]
    fn date_formatter_is_utc_and_stable() {
        assert_eq!(format_timestamp(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_timestamp(1_798_000_000), "2026-12-23 04:26:40 UTC");
    }

    #[test]
    fn rendered_shell_uses_semantic_accessible_navigation_without_scripts() {
        let rendered = layout("測試", "overview", html! { main { "content" } }).into_string();
        assert!(rendered.contains("lang=\"zh-Hant\""));
        assert!(rendered.contains("跳至主要內容"));
        assert!(rendered.contains("aria-label=\"主要導覽\""));
        assert!(!rendered.contains("<script"));
    }

    #[test]
    fn latest_workflow_version_wins() {
        let old = Workflow {
            id: "TPL-1".to_string(),
            version: 1,
            title: "流程".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            description: "用途".to_string(),
            stages: vec!["提出".to_string()],
            status: "published".to_string(),
            created_at: 1,
            published_at: Some(1),
            published_by: Some("Alex".to_string()),
        };
        let new = Workflow {
            version: 2,
            status: "draft".to_string(),
            ..old.clone()
        };
        let versions = [old, new];
        let latest = latest_workflows(&versions);
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].version, 2);
    }

    #[test]
    fn only_loopback_authorities_are_accepted() {
        for host in [
            "localhost",
            "LOCALHOST",
            "localhost:3000",
            "127.0.0.1",
            "127.0.0.1:3000",
            "127.255.255.254:8080",
            "[::1]",
            "[::1]:3000",
        ] {
            assert!(is_loopback_host(host), "{host} must be accepted");
        }
        for host in [
            "",
            "localhost.",
            "localhost:",
            "localhost:abc",
            "localhost:3000:80",
            "localhost :3000",
            " attacker.example",
            "attacker.example",
            "127.0.0.1.attacker.example",
            "10.0.0.1",
            "192.168.1.1:3000",
            "::1",
            "::1:3000",
            "[::2]",
            "[::1]extra",
            "[::1]:",
            "[::1]:99999",
            "2130706433",
            "0x7f.0.0.1",
        ] {
            assert!(!is_loopback_host(host), "{host:?} must be rejected");
        }
    }

    #[test]
    fn malformed_cookie_header_fails_closed() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::COOKIE,
            HeaderValue::from_static("ekpi_session=valid-token"),
        );
        headers.append(
            header::COOKIE,
            HeaderValue::from_bytes(b"ekpi_session=\x80invalid").expect("non-ASCII header"),
        );
        assert_eq!(
            session_token(&headers),
            None,
            "an unparseable Cookie header must not be ignored"
        );
    }
}

#[cfg(test)]
mod request_limits_tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use tower::ServiceExt;

    static TEST_LIMIT: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

    #[tokio::test]
    async fn timeout_returns_gateway_timeout() {
        let app = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    "late"
                }),
            )
            .layer(middleware::from_fn(|request, next| async move {
                request_context_with(request, next, Duration::from_millis(25), &REQUEST_LIMIT).await
            }));

        let response = app
            .oneshot(
                Request::get("/slow")
                    .body(Body::empty())
                    .expect("request builds"),
            )
            .await
            .expect("router responds");

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(
            response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some("1")
        );
    }

    #[tokio::test]
    async fn concurrency_bound_returns_too_many_requests() {
        let permit = TEST_LIMIT.acquire().await.expect("test permit");
        let app = Router::new()
            .route("/ok", get(|| async { "ok" }))
            .layer(middleware::from_fn(|request, next| async move {
                request_context_with(request, next, Duration::from_secs(1), &TEST_LIMIT).await
            }));

        let response = app
            .oneshot(
                Request::get("/ok")
                    .body(Body::empty())
                    .expect("request builds"),
            )
            .await
            .expect("router responds");

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        drop(permit);
    }
}

#[cfg(test)]
mod civil_date_tests {
    use super::*;

    /// 已知日期：Unix 紀元 1970-01-01（週四）。
    #[test]
    fn epoch_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_unix(0), (1970, 1, 1));
        assert_eq!(weekday_of_first(1970, 1), 3, "1970-01-01 is Thursday");
    }

    /// 閏年規則：2000-02-29 存在（400 年再閏）、1900-02-29 不存在（百年不閏）。
    #[test]
    fn leap_year_rule() {
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
    }

    /// 月界往返：月底最後一日 → 次月一日（含跨年 12→1）。
    #[test]
    fn month_boundaries_round_trip() {
        assert_eq!(
            civil_from_unix(days_from_civil(2026, 11, 30) * 86_400),
            (2026, 11, 30)
        );
        assert_eq!(
            civil_from_unix(days_from_civil(2026, 12, 31) * 86_400),
            (2026, 12, 31)
        );
        assert_eq!(
            civil_from_unix(days_from_civil(2027, 1, 1) * 86_400),
            (2027, 1, 1)
        );
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
    }

    /// 不變式：次月一日的星期＝本月一日星期＋本月天數（mod 7）——
    /// 對 120 個連續月份驗證（涵蓋閏年與跨年）。
    #[test]
    fn month_start_weekday_advances_by_month_length() {
        let mut y = 2020;
        let mut m = 1u32;
        for _ in 0..120 {
            let w = weekday_of_first(y, m);
            let len = days_in_month(y, m);
            let (ny, nm) = shift_month(y, m, 1);
            let w2 = weekday_of_first(ny, nm);
            assert_eq!(
                w2,
                (w + len) % 7,
                "weekday must advance by month length mod 7 ({y}-{m:02})"
            );
            y = ny;
            m = nm;
        }
    }
}
