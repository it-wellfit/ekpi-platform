#![forbid(unsafe_code)]

//! CSS-first, server-rendered local pilot workbench.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Request, State};
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

    let core = Router::new()
        .route("/ready", get(ready_check))
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .route("/", get(dashboard))
        .route("/login", get(login_page).post(login_post))
        .route("/logout", post(logout_post))
        .route("/projects", get(projects_page).post(create_project))
        .route("/evidence", get(evidence_page).post(link_evidence))
        .route("/evidence/{id}/review", post(review_evidence))
        .route(
            "/inspections",
            get(inspections_page).post(record_inspection),
        )
        .route("/workflows", get(workflows_page).post(save_workflow))
        .route("/workflows/{id}/{version}/publish", post(publish_workflow))
        .route("/controls", get(controls_page).post(create_quality_case))
        .route("/controls/{id}/status", post(update_quality_case))
        .route("/lifecycle", get(lifecycle_page).post(record_gate_decision))
        .route("/audit", get(audit_page))
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

    core.merge(approval_routes)
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
        Ok((_, workflows, _)) => workflows,
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
        Ok((tasks, _, _)) => tasks,
        Err(error) => {
            return crate::errors::error_response(error, StatusCode::INTERNAL_SERVER_ERROR, None)
                .into_response();
        }
    };
    let entries: Vec<serde_json::Value> = tasks
        .iter()
        .map(|t| serde_json::json!({
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
        }))
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

async fn api_submit_task() -> Response {
    api_not_implemented("POST /api/v1/approvals")
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

/// Structured 401 used by middleware layers that know the request id.
fn unauthorized_with_id(request_id: &str) -> Response {
    crate::errors::error_response(
        "authentication required",
        StatusCode::UNAUTHORIZED,
        Some(request_id),
    )
    .into_response()
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
        Some(session) if session.role >= required => Ok(session.username.clone()),
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
        Ok(data) => page_response("專案登錄", "projects", projects_content(&data, None)),
        Err(error) => page_error_response("projects", &error),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Ok(data) => page_response("風險與品質事件", "controls", controls_content(&data, None)),
        Err(error) => page_error_response("controls", &error),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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

fn approvals_view(
    approvals: &SharedApprovals,
) -> Result<
    (
        Vec<ekp_domain::workflow::ApprovalTask>,
        Vec<ekp_domain::workflow::WorkflowDefinition>,
        bool,
    ),
    String,
> {
    let store = approvals
        .lock()
        .map_err(|_| "核准資料鎖定失敗".to_string())?;
    let tasks = store.list_tasks();
    let workflows = store.list_workflows();
    let integrity = store.verify().is_ok();
    Ok((tasks, workflows, integrity))
}

async fn approvals_page(
    State((_auth, approvals)): State<(SharedAuth, SharedApprovals)>,
) -> Response {
    match approvals_view(&approvals) {
        Ok((tasks, workflows, integrity)) => page_response(
            "核准任務",
            "approvals",
            approvals_content(&tasks, &workflows, integrity, None),
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

fn approvals_content(
    tasks: &[ekp_domain::workflow::ApprovalTask],
    workflows: &[ekp_domain::workflow::WorkflowDefinition],
    integrity: bool,
    error: Option<&str>,
) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "動態簽核工作流" } h1 { "核准任務佇列" } p class="page-subtitle" { "多急分級、階段推進、超時清理與 Agent 建議（advisory-only）。" } } }
        @if let Some(error) = error {
            div class="notice notice-red" role="alert" { strong { "操作失敗：" } span { (error) } }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "簽核任務" } p { "任務由工作流定義觸發；所有決策寫入核准事件鏈。" } }
                span class="badge" class=(if integrity { "badge-green" } else { "badge-red" }) { (if integrity { "鏈已驗證" } else { "鏈驗證失敗" }) }
            }
            @if tasks.is_empty() {
                (empty_state("暫無簽核任務", "先建立工作流定義並發布，再提交簽核任務。核准決策會寫入核准事件鏈。", "/projects", "新增專案"))
            } @else {
                table class="api-table" {
                    thead { tr { th { "任務" } th { "工作流" } th { "階段" } th { "急分級" } th { "狀態" } th { "升級次數" } th { "決策" } } }
                    tbody {
                        @for task in tasks {
                            tr {
                                td { (task.id) }
                                td { (task.workflow_id) "@" (task.workflow_version) }
                                td { (task.stage_id) }
                                td { span class=(format!("badge {}", urgency_badge(task.urgency))) { (format!("{:?}", task.urgency)) } }
                                td { span class=(if task.is_open() { "badge badge-blue" } else { "badge badge-grey" }) { (format!("{:?}", task.state)) } }
                                td { (task.escalation_count) }
                                td {
                                    @if task.is_open() {
                                        form class="inline-form" method="post" action={ "/approvals/" (task.id) "/decide" } {
                                            select name="action" required {
                                                option value="approve" { "核准" }
                                                option value="reject" { "駁回" }
                                                option value="escalate" { "升級" }
                                            }
                                            input type="text" name="note" maxlength="300" placeholder="備註（可留空）";
                                            button class="button button-primary" type="submit" { "送出決策" }
                                        }
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
                .submit_task(&input, &actor, now)
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
        Err(e) => return (StatusCode::BAD_REQUEST, format!("無法解析表單：{e}")).into_response(),
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
        Ok((tasks, workflows, integrity)) => page_response(
            "核准任務",
            "approvals",
            approvals_content(&tasks, &workflows, integrity, Some(error)),
        )
        .into_response(),
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
                    tr { td { "GET" } td { "/api/v1/workflows" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "POST" } td { "/api/v1/workflows" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/approvals" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "POST" } td { "/api/v1/approvals/{id}/decide" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "POST" } td { "/api/v1/events/replay" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/channels" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/users" } td class="status-planned" { "未實作（一律 501）" } }
                    tr { td { "GET" } td { "/api/v1/settings" } td class="status-planned" { "未實作（一律 501）" } }
                }
            }
            div class="notice notice-blue" role="info" {
                strong { "誠實聲明：" }
                span { "所有 API 端點目前一律回 501，不提供任何資料；本頁不宣稱任何認證機制。契約與認證設計將於再設計總計畫 S4 完成後實作，屆時本頁才會更新為可用狀態。目前唯一的變更介面是 HTML 工作台。" }
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
                            (nav_link("/projects", "projects", active, "專案", "▦"))
                            (nav_link("/evidence", "evidence", active, "文件與證據", "▤"))
                            (nav_link("/inspections", "inspections", active, "品質檢查", "✓"))
                            (nav_link("/workflows", "workflows", active, "流程範本", "⌘"))
                            (nav_link("/controls", "controls", active, "風險與品質", "◇"))
                            (nav_link("/lifecycle", "lifecycle", active, "階段與關卡", "▥"))
                            (nav_link("/audit", "audit", active, "稽核紀錄", "◷"))
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
    html! {
        section class="page-heading" {
            div { p class="eyebrow" { "治理概況" } h1 { "工作總覽" } p class="page-subtitle" { "集中查看專案進度、文件追溯與需要人工確認的工作。" } }
            a class="button button-primary" href="/projects" { "＋ 新增專案" }
        }
        div class="metric-grid" {
            (metric("專案", data.projects.len(), "已登錄專案"))
            (metric("待審證據", waiting_review, "尚未留下人工審查紀錄"))
            (metric("開放事項", open_cases, "風險、變更、缺失與 CAPA"))
            (metric("流程草稿", draft_workflows, "等待負責人審查發布"))
        }
        section class="content-grid" {
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
                (recent_events(&data.events, 5))
            }
        }
        p class="data-note" { "試點指標尚未建立基線；此處只顯示登錄筆數，不代表證據完整度或流程改善成效。" }
    }
}

fn metric(label: &str, value: usize, note: &str) -> Markup {
    html! { article class="metric-card" { p { (label) } strong class="metric-value" { (value) } small { (note) } } }
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
                                p class="row-id" { "前序雜湊 " (event.previous_sha256) }
                                p class="hash-value" { "SHA-256 " (event.sha256) }
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

fn event_label(payload: &crate::governance::EventPayload) -> String {
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
