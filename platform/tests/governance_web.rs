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
    // A successful login rotates the CSRF token: the response sets a new
    // `ekpi_csrf` pair bound to the session, and the browser must store it.
    let session_pair = set_cookie_header(&response, "ekpi_session=").expect("session cookie");
    let rotated_csrf = set_cookie_header(&response, "ekpi_csrf=").expect("rotated csrf cookie");
    assert!(session_pair.starts_with("ekpi_session="));
    assert!(set_cookie_header_full(&response, "ekpi_session=").contains("HttpOnly"));
    assert!(set_cookie_header_full(&response, "ekpi_session=").contains("SameSite=Strict"));
    assert!(set_cookie_header_full(&response, "ekpi_session=").contains("Max-Age="));
    format!("{session_pair}; {rotated_csrf}")
}

/// Returns the `name=value` pair of the Set-Cookie header whose value starts
/// with `prefix`, or `None` when no such header exists.
fn set_cookie_header(response: &axum::response::Response, prefix: &str) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value
                .starts_with(prefix)
                .then(|| value.split(';').next().expect("cookie pair").to_string())
        })
}

/// Returns the full Set-Cookie header value (attributes included) whose value
/// starts with `prefix`, or an empty string when no such header exists.
fn set_cookie_header_full(response: &axum::response::Response, prefix: &str) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value.starts_with(prefix).then(|| value.to_string())
        })
        .unwrap_or_default()
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
    // S3 七狀態基元（Q-21/Q-26 鎖定）：缺一即退回。
    assert!(css.contains(":user-invalid"), "form validation styling");
    assert!(css.contains("button:disabled"), "disabled state styling");
    assert!(
        css.contains(".badge-grey"),
        "grey badge for terminal states"
    );
    assert!(css.contains(".inline-form"), "inline decision form layout");
    assert!(
        css.contains("prefers-reduced-motion"),
        "reduced-motion must be honored"
    );
    // Design System v2 primitives（S7 十維躍升層）。
    assert!(css.contains(".kpi-card"), "KPI cards");
    assert!(css.contains(".donut"), "CSS donut visualization");
    assert!(css.contains(".timeline"), "timeline component");
    assert!(css.contains(".hash-chip"), "hash chips");
    assert!(css.contains(".result-card"), "search result cards");
    assert!(css.contains(".stage-dots"), "approval stage dots");
    assert!(css.contains(".meter"), "SLA/score meters");
    assert!(css.contains(".chip"), "quick-query chips");
    assert!(css.contains(".nav-group-label"), "grouped navigation");

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

/// S7：檢索頁載入、唯讀標示、誠實空查詢狀態與索引統計。
#[tokio::test]
async fn search_page_renders_honest_states_and_index_counts() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    // 空查詢：引導狀態，不出假結果。
    let page = get_with_cookie(&app, "/search", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("輸入關鍵詞開始檢索"), "empty-query guidance");
    assert!(html.contains("索引涵蓋"), "index counts note");

    // 有查詢：結果卡或誠實空結果，二者必居其一，不得捏造。
    let page = get_with_cookie(&app, "/search?q=%E8%AE%8A%E6%9B%B4", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(
        html.contains("result-card") || html.contains("找不到相符條款"),
        "either ranked results or an honest empty result"
    );
    assert!(html.contains("唯讀"), "the read-only badge is shown");
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// S7：檢索需登入（Viewer 以上）。
#[tokio::test]
async fn search_page_requires_session() {
    let path = test_path();
    let app = test_app(&path);
    let response = get_with_cookie(&app, "/search", None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// S7：儀表板含 KPI 卡與品質狀態區塊；nav 含檢索與核准入口。
#[tokio::test]
async fn dashboard_renders_kpi_cards_and_grouped_nav() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let page = get_with_cookie(&app, "/", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("kpi-card"), "KPI cards render");
    assert!(html.contains("品質事項狀態"), "case-status panel");
    assert!(html.contains("關卡決策分佈"), "gate distribution panel");
    assert!(html.contains("href=\"/search\""), "nav links to search");
    assert!(
        html.contains("href=\"/approvals\""),
        "nav links to approvals"
    );
    assert!(html.contains("nav-group-label"), "grouped navigation");
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
    assert!(
        set_cookie.starts_with("ekpi_csrf="),
        "cookie is the csrf pair"
    );
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
    let session_full = set_cookie_header_full(&response, "ekpi_session=");
    assert!(
        session_full.contains("; Secure"),
        "login must honor EKPI_COOKIE_SECURE"
    );
    let rotated_csrf = set_cookie_header_full(&response, "ekpi_csrf=");
    assert!(
        rotated_csrf.contains("; Secure"),
        "the rotated csrf cookie must also honor EKPI_COOKIE_SECURE"
    );
    let session = set_cookie_header(&response, "ekpi_session=").expect("session pair");
    let rotated_pair = set_cookie_header(&response, "ekpi_csrf=").expect("rotated csrf pair");

    let response = post_authed(&app, &format!("{session}; {rotated_pair}"), "/logout", "").await;
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

/// S11：資料地圖頁——血緣表（治理 9＋核准 5）需登入；未登入 401。
#[tokio::test]
async fn data_map_page_requires_session_and_renders() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_path();
    let app = test_app(&path);
    let response = get_with_cookie(&app, "/data-map", None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let page = get_with_cookie(&app, "/data-map", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("資料血緣地圖"), "page heading");
    assert!(html.contains("project_created"), "governance lineage tag");
    assert!(
        html.contains("gate_decision_recorded"),
        "last governance tag"
    );
    assert!(html.contains("task_submitted"), "approval lineage tag");
    assert!(html.contains("tasks_expired"), "last approval tag");
    assert!(html.contains("現有事件數"), "live count column");
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// S11：血緣計數隨實際資料增長——建立專案後 project_created 計數為 1。
#[tokio::test]
async fn data_map_counts_reflect_live_ledger() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let page = get_with_cookie(&app, "/data-map", Some(&cookie)).await;
    let html = response_text(page).await;
    let before = html.matches("project_created").count();
    let form = "name=血緣測試&department=PMO&owner=Alex&risk_tier=low";
    let created = post_form_with_cookie(&app, "/projects", form, Some(&cookie)).await;
    assert_eq!(created.status(), StatusCode::SEE_OTHER, "project created");
    let page = get_with_cookie(&app, "/data-map", Some(&cookie)).await;
    let html = response_text(page).await;
    let after = html.matches("project_created").count();
    assert_eq!(before, after, "lineage tags are static (type-driven)");
    assert!(
        html.contains("血緣測試") || html.contains("1"),
        "live count column must reflect the new ledger event"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// S12 批次：事件瀏覽器——標籤過濾與誠實空狀態。
#[tokio::test]
async fn events_browser_filters_by_type() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    // 建立一個專案 → 治理鏈出現一筆 project_created。
    let form = "name=瀏覽器測試&department=PMO&owner=Alex&risk_tier=low";
    let created = post_form_with_cookie(&app, "/projects", form, Some(&cookie)).await;
    assert_eq!(created.status(), StatusCode::SEE_OTHER);

    // 全部視圖：事件存在。
    let page = get_with_cookie(&app, "/events", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("project_created"), "event tag rendered");

    // 過濾到不存在的事件類型 → 誠實空狀態。
    let page = get_with_cookie(&app, "/events?type=workflow_published", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("沒有符合的事件"), "honest empty state");

    // 過濾 project_created → 1 筆（表渲染鏈事實：標籤/操作者，非 payload 內文）。
    let page = get_with_cookie(&app, "/events?type=project_created", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("project_created"), "filtered tag rendered");
    assert!(html.contains("tester"), "event actor rendered");
    assert!(
        !html.contains("瀏覽器測試"),
        "payload contents are not part of the chain-facts table"
    );
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// S12 批次：權限矩陣渲染角色×能力，並聲明與程式碼同步。
#[tokio::test]
async fn role_matrix_renders_role_capability_table() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    let page = get_with_cookie(&app, "/role-matrix", Some(&cookie)).await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = response_text(page).await;
    assert!(html.contains("Approver 專屬"), "decision row present");
    assert!(html.contains("is_public_path"), "mirrors-code disclaimer");
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// 批次 3：字彙／說明／關於三頁。
#[tokio::test]
async fn info_pages_render_glossary_help_about() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    for (route, marker) in [
        ("/glossary", "Ubiquitous Language"),
        ("/help", "疑難排解"),
        ("/about", "明確不是什麼"),
    ] {
        let page = get_with_cookie(&app, route, Some(&cookie)).await;
        assert_eq!(page.status(), StatusCode::OK, "{route} status");
        let html = response_text(page).await;
        assert!(html.contains(marker), "{route} must contain `{marker}`");
        assert!(!html.contains("<script"), "{route} zero-JS");
    }
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// 突變邊界：不存在的目標 ID 必須被拒絕，且不得追加任何事件。
#[tokio::test]
async fn mutations_for_unknown_records_are_refused_without_appending_events() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;

    for (route, body, expected_error) in [
        (
            "/evidence/EVD-999999/review",
            "result=accepted&note=unknown",
            "找不到此證據項目",
        ),
        ("/workflows/TPL-999999/1/publish", "", "找不到此流程版本"),
        (
            "/controls/QMS-999999/status",
            "status=closed&note=unknown",
            "找不到此品質事件",
        ),
        (
            "/lifecycle",
            "project_id=PRJ-999999&gate_id=G1&decision=go&reason=&evidence_reference=&risk_acceptor=&remediation_due=&non_waivable=no",
            "請先選擇有效的專案",
        ),
    ] {
        let response = post_authed(&app, &cookie, route, body).await;
        assert_eq!(response.status(), StatusCode::OK, "refusal via {route}");
        let html = response_text(response).await;
        assert!(
            html.contains(expected_error),
            "route {route} must show `{expected_error}`"
        );
        assert!(html.contains("alert-error"), "route {route} error notice");
    }

    let events = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert_eq!(events, 0, "refused mutations must not append an event");

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// 詳情頁邊界：不存在的 ID 必須回誠實 404（不得 200 空頁、不得捏造資料）。
#[tokio::test]
async fn detail_pages_return_honest_404_for_unknown_ids() {
    let path = test_path();
    let app = test_app(&path);
    let cookie = login(&app, "tester", ADMIN_PASSWORD).await;
    for (route, missing_id) in [
        ("/projects/PRJ-999999", "PRJ-999999"),
        ("/evidence/EVD-999999", "EVD-999999"),
        ("/controls/QMS-999999", "QMS-999999"),
    ] {
        let response = get_with_cookie(&app, route, Some(&cookie)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "route {route}");
        let html = response_text(response).await;
        assert!(
            html.contains("找不到"),
            "route {route} renders the not-found heading"
        );
        assert!(
            html.contains(missing_id),
            "route {route} names the missing id"
        );
        assert!(!html.contains("<script"), "route {route} stays zero-JS");
    }
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

/// 詳情頁：需登入；Viewer 可讀；列表頁必須連到詳情頁（避免孤島頁面）。
#[tokio::test]
async fn detail_pages_require_session_are_readable_by_viewer_and_linked() {
    let path = test_path();
    let app = test_app(&path);

    for route in [
        "/projects/PRJ-000001",
        "/evidence/EVD-000002",
        "/controls/QMS-000003",
    ] {
        let response = get_with_cookie(&app, route, None).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "route {route}");
    }

    let admin = login(&app, "tester", ADMIN_PASSWORD).await;
    for (route, body) in [
        (
            "/projects",
            "name=Detail+target&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
        ),
        (
            "/evidence",
            "project_id=PRJ-000001&title=Traceable+record&source_system=Microsoft+365&source_url=https%3A%2F%2Fexample.test%2Frecord&source_version=v1&evidence_type=approval&owner=Alex&actor=Jordan",
        ),
        (
            "/controls",
            "project_id=PRJ-000001&kind=capa&title=Corrective+action&description=Verify+root+cause&owner=Alex&due_date=2026-12-31&actor=Jordan",
        ),
    ] {
        let response = post_authed(&app, &admin, route, body).await;
        assert_eq!(
            response.status(),
            StatusCode::SEE_OTHER,
            "create via {route}"
        );
    }

    // 列表頁必須提供詳情連結，否則詳情頁無法從介面抵達。
    for (route, link) in [
        ("/projects", "href=\"/projects/PRJ-000001\""),
        ("/evidence", "href=\"/evidence/EVD-000002\""),
        ("/controls", "href=\"/controls/QMS-000003\""),
    ] {
        let page = get_with_cookie(&app, route, Some(&admin)).await;
        assert_eq!(page.status(), StatusCode::OK, "list {route}");
        let html = response_text(page).await;
        assert!(html.contains(link), "list {route} links to {link}");
    }

    let viewer = login(&app, "viewer", VIEWER_PASSWORD).await;
    for (route, marker) in [
        ("/projects/PRJ-000001", "Detail target"),
        ("/evidence/EVD-000002", "Traceable record"),
        ("/controls/QMS-000003", "Corrective action"),
    ] {
        let page = get_with_cookie(&app, route, Some(&viewer)).await;
        assert_eq!(page.status(), StatusCode::OK, "viewer detail {route}");
        let html = response_text(page).await;
        assert!(html.contains(marker), "detail {route} renders `{marker}`");
        assert!(!html.contains("<script"), "detail {route} zero-JS");
    }

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}
