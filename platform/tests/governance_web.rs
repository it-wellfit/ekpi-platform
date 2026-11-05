use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use ekp_platform::governance::{Ledger, QUALITY_CHECKLIST};
use ekp_platform::governance_web;
use ekp_platform::local_auth::{AuthStore, Role};
use http_body_util::BodyExt;
use tower::ServiceExt;

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);
// The guard must stay alive across awaits so the environment variable is
// visible to the handler, so this lock is async-aware rather than std.
static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const ADMIN_PASSWORD: &str = "correct-horse-battery-staple";
const VIEWER_PASSWORD: &str = "viewer-password";

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

fn test_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "ekpi-governance-web-{}-{}.jsonl",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ))
}

fn test_app(path: &PathBuf) -> axum::Router {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let ledger = Ledger::open(path).expect("test ledger opens");
    let auth = Arc::new(Mutex::new(AuthStore::with_iterations(3600, 1_000)));
    {
        let mut store = auth.lock().expect("auth store lock");
        store
            .register("tester", ADMIN_PASSWORD, Role::Admin)
            .expect("test admin registers");
        store
            .register("viewer", VIEWER_PASSWORD, Role::Viewer)
            .expect("test viewer registers");
    }
    governance_web::router(Arc::new(Mutex::new(ledger)), auth)
}

async fn post_form_with_cookie(
    app: &axum::Router,
    uri: &str,
    body: &str,
    cookie: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::HOST, "127.0.0.1:3000")
        .header(header::ORIGIN, "http://127.0.0.1:3000")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder
        .body(Body::from(body.to_string()))
        .expect("form request builds");
    app.clone().oneshot(request).await.expect("router responds")
}

async fn post_form(app: &axum::Router, uri: &str, body: &str) -> axum::response::Response {
    post_form_with_cookie(app, uri, body, None).await
}

async fn post_authed(
    app: &axum::Router,
    cookie: &str,
    uri: &str,
    body: &str,
) -> axum::response::Response {
    post_form_with_cookie(app, uri, body, Some(cookie)).await
}

async fn get_with_cookie(
    app: &axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::get(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = builder.body(Body::empty()).expect("GET request builds");
    app.clone().oneshot(request).await.expect("router responds")
}

async fn login(app: &axum::Router, username: &str, password: &str) -> String {
    // Browser-equivalent flow: fetch the login page, take the CSRF token from
    // the hidden field, send it back with the matching `ekpi_csrf` cookie.
    let (token, csrf_cookie) = login_page_csrf(app).await;
    let body = format!("csrf_token={token}&username={username}&password={password}");
    let response = post_form_with_cookie(app, "/login", &body, Some(&csrf_cookie)).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .expect("cookie is ASCII")
        .to_string();
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Max-Age="));
    let session_pair = set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string();
    format!("{session_pair}; {csrf_cookie}")
}

/// Fetches the login page and returns its fresh CSRF token plus the matching
/// `ekpi_csrf` cookie pair, mirroring what a zero-JS browser would hold.
async fn login_page_csrf(app: &axum::Router) -> (String, String) {
    let page = get_with_cookie(app, "/login", None).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    let token = extract_hidden_csrf(&html).expect("login form carries a CSRF token");
    (token.clone(), format!("ekpi_csrf={token}"))
}

/// Pulls the value of the hidden `csrf_token` input out of a login page.
fn extract_hidden_csrf(html: &str) -> Option<String> {
    let marker = "name=\"csrf_token\" value=\"";
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    let token = &rest[..end];
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// Keeps only the session pair from a combined cookie header.
fn session_only(combined: &str) -> String {
    combined
        .split("; ")
        .find(|pair| pair.starts_with("ekpi_session="))
        .expect("session pair present")
        .to_string()
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body collects")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("HTML is UTF-8")
}

#[tokio::test]
async fn workbench_renders_accessible_shell_and_local_stylesheet() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let response = get_with_cookie(&app, "/", Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .is_some()
    );
    let html = response_text(response).await;
    assert!(html.contains("跳至主要內容"));
    assert!(html.contains("文件與證據"));
    assert!(!html.contains("<script"));

    let response = get_with_cookie(&app, "/assets/governance.css", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/css; charset=utf-8"
    );
    let css = response_text(response).await;
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("@media (max-width: 760px)"));

    for (route, expected_heading) in [
        ("/projects", "專案登錄"),
        ("/evidence", "文件與證據追溯"),
        ("/inspections", "29 項品質檢查"),
        ("/workflows", "可配置流程範本"),
        ("/controls", "風險與品質事件"),
        ("/lifecycle", "生命週期與強制關卡"),
        ("/audit", "追加式稽核紀錄"),
    ] {
        let response = get_with_cookie(&app, route, Some(&cookie)).await;
        assert_eq!(response.status(), StatusCode::OK, "route {route}");
        assert!(
            response_text(response).await.contains(expected_heading),
            "route {route}"
        );
    }
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn governance_flow_persists_auditable_project_evidence_workflow_and_capa() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    let response = post_authed(
        &app,
        &cookie,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/projects"
    );

    let response = post_authed(
        &app,
        &cookie,
        "/evidence",
        "project_id=PRJ-000001&title=Approval+record&source_system=Microsoft+365&source_url=https%3A%2F%2Fexample.test%2Fapproval&source_version=v1&evidence_type=approval&owner=Alex&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/evidence/EVD-000002/review",
        "result=accepted&actor=QA&note=Version+confirmed",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/workflows",
        "base_id=&title=Change+control&department=Cross-functional&owner=Process+owner&description=Review+and+approve+changes&stages=Intake%0ARisk+review%0AApproval%0AExecution%0AClose&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/workflows/TPL-000004/1/publish",
        "actor=Process+owner",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/controls",
        "project_id=PRJ-000001&kind=capa&title=Corrective+action&description=Verify+root+cause+and+effectiveness&owner=Alex&due_date=2026-12-31&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/controls/QMS-000006/status",
        "status=closed&actor=Reviewer&note=Effectiveness+confirmed",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response_text(response).await.contains("待複驗狀態"));

    for (status, note) in [
        ("in_progress", "Corrective action performed"),
        ("verification", "Ready for independent verification"),
        ("closed", "Effectiveness confirmed"),
    ] {
        let body = format!("status={status}&actor=Reviewer&note={note}");
        let response = post_authed(&app, &cookie, "/controls/QMS-000006/status", &body).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    let mut inspection_body = "project_id=PRJ-000001&reviewer=QA".to_string();
    for item in QUALITY_CHECKLIST {
        inspection_body.push_str(&format!(
            "&result_{}=pass&evidence_{}=REF-{}",
            item.id, item.id, item.id
        ));
    }
    let response = post_authed(&app, &cookie, "/inspections", &inspection_body).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let response = get_with_cookie(&app, "/inspections", Some(&cookie)).await;
    let inspection_html = response_text(response).await;
    assert!(inspection_html.contains("1 份完整檢查紀錄"));
    assert!(inspection_html.contains("REF-QC-01"));

    let response = post_authed(
        &app,
        &cookie,
        "/lifecycle",
        "project_id=PRJ-000001&gate_id=G4&decision=exception&non_waivable=yes&reason=Missing+mandatory+evidence&evidence_reference=&risk_acceptor=Sponsor&remediation_due=2026-12-31&actor=Reviewer",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response_text(response)
            .await
            .contains("不可豁免要求不得以一般例外放行")
    );

    let response = get_with_cookie(&app, "/audit", Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let audit_html = response_text(response).await;
    assert!(audit_html.contains("本機雜湊鏈驗證通過"));
    assert!(audit_html.contains("Effectiveness confirmed"));
    assert!(audit_html.contains("完成品質檢查：PRJ-000001 · 29 項檢查"));

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn mutation_requires_authentication() {
    let path = test_path();
    let app = test_app(&path);
    let response = post_form(
        &app,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn data_pages_require_authentication() {
    let path = test_path();
    let app = test_app(&path);
    for route in ["/", "/projects", "/evidence", "/audit"] {
        let response = get_with_cookie(&app, route, None).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "route {route} must require a session"
        );
    }
    let response = get_with_cookie(&app, "/login", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = get_with_cookie(&app, "/healthz", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn viewer_role_cannot_mutate() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "viewer", VIEWER_PASSWORD).await;
    let response = post_authed(
        &app,
        &cookie,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn login_rejects_a_bad_password_without_a_cookie() {
    let path = test_path();
    let app = test_app(&path);
    let (token, csrf_cookie) = login_page_csrf(&app).await;
    let body = format!("csrf_token={token}&username=tester&password=wrong-password");
    let response = post_form_with_cookie(&app, "/login", &body, Some(&csrf_cookie)).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response.headers().get(header::SET_COOKIE).is_none());

    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .body(Body::empty())
                .expect("login GET builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response_text(response).await.contains("form"));
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn login_post_without_csrf_token_is_refused() {
    let path = test_path();
    let app = test_app(&path);
    let response = post_form(
        &app,
        "/login",
        "username=tester&password=correct-horse-battery-staple",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.headers().get(header::SET_COOKIE).is_none());
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn login_post_with_mismatched_csrf_token_is_refused() {
    let path = test_path();
    let app = test_app(&path);
    let (_, csrf_cookie) = login_page_csrf(&app).await;
    let response = post_form_with_cookie(
        &app,
        "/login",
        "csrf_token=deadbeef&username=tester&password=correct-horse-battery-staple",
        Some(&csrf_cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.headers().get(header::SET_COOKIE).is_none());
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn login_form_and_cookie_carry_the_same_csrf_token() {
    let path = test_path();
    let app = test_app(&path);
    let page = get_with_cookie(&app, "/login", None).await;
    let set_cookie = page
        .headers()
        .get(header::SET_COOKIE)
        .expect("login page sets the csrf cookie")
        .to_str()
        .expect("cookie is ASCII")
        .to_string();
    assert!(set_cookie.starts_with("ekpi_csrf="), "cookie is the csrf pair");
    assert!(set_cookie.contains("HttpOnly") && set_cookie.contains("SameSite=Strict"));
    let cookie_token = set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .strip_prefix("ekpi_csrf=")
        .expect("csrf prefix")
        .to_string();
    let html = response_text(page).await;
    let field_token = extract_hidden_csrf(&html).expect("hidden CSRF token present");
    assert_eq!(field_token.len(), 64, "token is 32 bytes of hex");
    assert_eq!(
        field_token, cookie_token,
        "hidden field and cookie must carry the same double-submit token"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn zero_js_mutation_passes_with_session_and_csrf_cookies_without_a_header() {
    let path = test_path();
    let app = test_app(&path);
    let cookies = login(&app, "tester", ADMIN_PASSWORD).await;
    let response = post_authed(
        &app,
        &cookies,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "a server-rendered form POST (cookies only, no header) must pass CSRF"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn mutation_with_session_but_without_csrf_cookie_is_refused() {
    let path = test_path();
    let app = test_app(&path);
    let cookies = login(&app, "tester", ADMIN_PASSWORD).await;
    let response = post_authed(
        &app,
        &session_only(&cookies),
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn oversized_login_body_is_rejected_before_parsing() {
    let path = test_path();
    let app = test_app(&path);
    let body = format!("username=tester&password={}", "a".repeat(70_000));
    let response = post_form(&app, "/login", &body).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(response.headers().get(header::SET_COOKIE).is_none());
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn logout_invalidates_the_session_cookie() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let response = post_authed(&app, &cookie, "/logout", "").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cleared = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("logout clears the cookie")
        .to_str()
        .expect("cookie is ASCII");
    assert!(cleared.contains("Max-Age=0"));
    assert!(cleared.contains("SameSite=Strict"));

    let response = post_authed(
        &app,
        &cookie,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn oversized_form_body_is_rejected_before_parsing() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let body = format!("name={}", "a".repeat(70_000));
    let response = post_authed(&app, &cookie, "/projects", &body).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn ledger_actor_is_the_authenticated_session_not_the_form_value() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let response = post_authed(
        &app,
        &cookie,
        "/projects",
        "name=Impersonation+check&department=PMO&owner=Alex&risk_tier=low&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let contents = std::fs::read_to_string(&path).expect("read ledger");
    let events: Vec<ekp_platform::governance::LedgerEvent> = contents
        .lines()
        .map(|line| serde_json::from_str(line).expect("ledger line is an event"))
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].actor, "tester",
        "the ledger must use the validated session username"
    );
    assert_ne!(
        events[0].actor, "Jordan",
        "a submitted actor field must not be trusted"
    );

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn cross_origin_mutating_request_is_rejected() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/projects")
        .header(header::HOST, "127.0.0.1:3000")
        .header(header::ORIGIN, "https://untrusted.example")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "name=Pilot&department=PMO&owner=Alex&risk_tier=low&actor=Jordan",
        ))
        .expect("cross-origin request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = get_with_cookie(&app, "/audit", Some(&cookie)).await;
    assert!(response_text(response).await.contains("0 筆事件"));
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn foreign_host_header_is_rejected_even_when_origin_matches() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/projects")
        .header(header::HOST, "attacker.example")
        .header(header::ORIGIN, "http://attacker.example")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie.as_str())
        .body(Body::from(
            "name=Pilot&department=PMO&owner=Alex&risk_tier=low",
        ))
        .expect("foreign-host request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = get_with_cookie(&app, "/audit", Some(&cookie)).await;
    assert!(response_text(response).await.contains("0 筆事件"));

    let request = Request::get("/")
        .header(header::HOST, "attacker.example")
        .body(Body::empty())
        .expect("foreign-host GET builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn loopback_alternative_hosts_are_allowed() {
    let path = test_path();
    let app = test_app(&path);
    for host in ["LOCALHOST:3000", "[::1]:3000", "127.0.0.2:3000"] {
        let request = Request::get("/healthz")
            .header(header::HOST, host)
            .body(Body::empty())
            .expect("request builds");
        let response = app.clone().oneshot(request).await.expect("router responds");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "loopback host {host} must be accepted"
        );
    }
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn duplicate_host_headers_are_rejected() {
    let path = test_path();
    let app = test_app(&path);
    let request = Request::get("/healthz")
        .header(header::HOST, "localhost:3000")
        .header(header::HOST, "localhost:3000")
        .body(Body::empty())
        .expect("request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn duplicate_origin_headers_are_rejected() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/projects")
        .header(header::HOST, "127.0.0.1:3000")
        .header(header::ORIGIN, "http://127.0.0.1:3000")
        .header(header::ORIGIN, "https://untrusted.example")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie.as_str())
        .body(Body::from(
            "name=Pilot&department=PMO&owner=Alex&risk_tier=low",
        ))
        .expect("duplicate-origin request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "an ambiguous Origin header must be rejected"
    );
    let response = get_with_cookie(&app, "/audit", Some(&cookie)).await;
    assert!(response_text(response).await.contains("0 筆事件"));
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn duplicate_sec_fetch_site_headers_are_rejected() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let request = Request::builder()
        .method(Method::POST)
        .uri("/projects")
        .header(header::HOST, "127.0.0.1:3000")
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-site", "cross-site")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie.as_str())
        .body(Body::from(
            "name=Pilot&department=PMO&owner=Alex&risk_tier=low",
        ))
        .expect("duplicate-fetch-site request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "an ambiguous Sec-Fetch-Site header must be rejected"
    );
    let response = get_with_cookie(&app, "/audit", Some(&cookie)).await;
    assert!(response_text(response).await.contains("0 筆事件"));
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn duplicate_session_cookies_are_rejected() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let duplicated = format!("{cookie}; {cookie}");
    let response = get_with_cookie(&app, "/", Some(&duplicated)).await;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "ambiguous duplicate session cookies must be rejected"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn duplicate_cookie_header_lines_are_rejected_even_when_only_one_has_a_session() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    // Two Cookie header lines are ambiguous even when only one of them carries
    // the session pair. The server must not scan the second line after seeing
    // an unrelated first line.
    let request = Request::get("/")
        .header(header::COOKIE, "theme=dark")
        .header(header::COOKIE, cookie.as_str())
        .body(Body::empty())
        .expect("request builds");
    let response = app.clone().oneshot(request).await.expect("router responds");
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "duplicate Cookie header lines must not be resolved to one session"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn multiline_textarea_content_is_accepted_and_rendered_with_preserved_breaks() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    let response = post_authed(
        &app,
        &cookie,
        "/projects",
        "name=Multiline+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_authed(
        &app,
        &cookie,
        "/workflows",
        "base_id=&title=Multiline+workflow&department=QA&owner=Alex&description=First+line%0D%0ASecond+line&stages=Intake%0D%0AReview&actor=Jordan",
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::SEE_OTHER,
        "a browser-style CRLF textarea value must be accepted"
    );
    let response = get_with_cookie(&app, "/workflows", Some(&cookie)).await;
    let html = response_text(response).await;
    assert!(html.contains("First line\nSecond line"));
    assert!(html.contains("class=\"multiline\""));

    let response = post_authed(
        &app,
        &cookie,
        "/controls",
        "project_id=PRJ-000001&kind=risk&title=Multiline+case&description=Root+cause%0D%0AEffect&owner=Alex&due_date=2026-12-31&actor=Jordan",
    )
    .await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::SEE_OTHER, "response body: {body}");
    let response = get_with_cookie(&app, "/controls", Some(&cookie)).await;
    let html = response_text(response).await;
    assert!(html.contains("Root cause\nEffect"));

    let response = post_authed(
        &app,
        &cookie,
        "/lifecycle",
        "project_id=PRJ-000001&gate_id=G2&decision=hold&non_waivable=no&reason=Waiting%0D%0Afor+review&evidence_reference=&risk_acceptor=&remediation_due=&actor=Reviewer",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let response = get_with_cookie(&app, "/lifecycle", Some(&cookie)).await;
    let html = response_text(response).await;
    assert!(html.contains("Waiting\nfor review"));

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn logout_mirrors_the_secure_cookie_attribute() {
    let _env_lock = ENV_LOCK.lock().await;
    let _secure = EnvGuard::set("EKPI_COOKIE_SECURE", "1");
    let path = test_path();
    let app = test_app(&path);

    let (token, csrf_cookie) = login_page_csrf(&app).await;
    let body = format!("csrf_token={token}&username=tester&password={ADMIN_PASSWORD}");
    let response = post_form_with_cookie(&app, "/login", &body, Some(&csrf_cookie)).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let login_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .expect("cookie is ASCII")
        .to_string();
    assert!(
        login_cookie.contains("; Secure"),
        "login must honor EKPI_COOKIE_SECURE"
    );
    let session = login_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .to_string();

    let response = post_authed(&app, &session, "/logout", "").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cleared = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("logout clears the cookie")
        .to_str()
        .expect("cookie is ASCII");
    assert!(
        cleared.contains("; Secure"),
        "logout must mirror the login cookie's Secure attribute"
    );

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}
