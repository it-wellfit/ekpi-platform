// C3 route-inventory characterization.
//
// This suite locks the served front door before the `governance_web` module
// split: every production path is registered, unsupported methods are a 405
// (not a fallback 404), every mutation requires a session and a CSRF token,
// the public surface is exactly the documented allowlist, and the API v1
// boundary keeps its honest 200/400/501 contract.
//
// It is deliberately behavioral, not a source scan: after the split the same
// paths must respond identically from different files.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use ekp_platform::governance::Ledger;
use ekp_platform::governance_web::{self, PAGE_REGISTRY};
use ekp_platform::local_auth::{AuthStore, Role};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

const HOST: &str = "127.0.0.1:8080";
const ORIGIN: &str = "http://127.0.0.1:8080";
const PASSWORD: &str = "correct-horse-battery-staple";

/// The 47 production route declarations. Methods are the ones declared on the
/// router; `DELETE` is used as the unsupported-method probe for every path.
const ROUTES: &[(&str, &[&str])] = &[
    ("/ready", &["GET"]),
    ("/healthz", &["GET"]),
    ("/metrics", &["GET"]),
    ("/", &["GET"]),
    ("/login", &["GET", "POST"]),
    ("/logout", &["POST"]),
    ("/projects", &["GET", "POST"]),
    ("/projects/{id}", &["GET"]),
    ("/evidence", &["GET", "POST"]),
    ("/evidence/{id}", &["GET"]),
    ("/evidence/{id}/review", &["POST"]),
    ("/inspections", &["GET", "POST"]),
    ("/workflows", &["GET", "POST"]),
    ("/workflows/{id}/{version}/publish", &["POST"]),
    ("/controls", &["GET", "POST"]),
    ("/controls/{id}", &["GET"]),
    ("/controls/{id}/status", &["POST"]),
    ("/lifecycle", &["GET", "POST"]),
    ("/audit", &["GET"]),
    ("/events", &["GET"]),
    ("/search", &["GET"]),
    ("/settings", &["GET"]),
    ("/api-docs", &["GET"]),
    ("/assets/governance.css", &["GET"]),
    ("/api/v1/events/replay", &["POST"]),
    ("/api/v1/channels", &["GET"]),
    ("/api/v1/channels/health", &["GET"]),
    ("/api/v1/users", &["GET"]),
    ("/api/v1/settings", &["GET"]),
    ("/api/v1/search", &["GET"]),
    ("/approvals", &["GET"]),
    ("/approvals/calendar", &["GET"]),
    ("/approvals/workflows", &["POST"]),
    ("/approvals/workflows/{id}/{version}/publish", &["POST"]),
    ("/approvals/submit", &["POST"]),
    ("/approvals/{id}/decide", &["POST"]),
    ("/api/v1/workflows", &["GET", "POST"]),
    ("/api/v1/workflows/{id}/publish", &["POST"]),
    ("/api/v1/approvals", &["GET", "POST"]),
    ("/api/v1/approvals/{id}/decide", &["POST"]),
    ("/api/v1/approvals/{id}/escalate", &["POST"]),
    ("/data-map", &["GET"]),
    ("/audit/chains", &["GET"]),
    ("/role-matrix", &["GET"]),
    ("/glossary", &["GET"]),
    ("/help", &["GET"]),
    ("/about", &["GET"]),
];

/// GET paths that are not `PAGE_REGISTRY` pages: probes, static asset, API
/// reads and the three detail routes.
const NON_PAGE_GET: &[&str] = &[
    "/ready",
    "/healthz",
    "/metrics",
    "/assets/governance.css",
    "/api/v1/channels",
    "/api/v1/channels/health",
    "/api/v1/users",
    "/api/v1/settings",
    "/api/v1/search",
    "/api/v1/workflows",
    "/api/v1/approvals",
    "/projects/{id}",
    "/evidence/{id}",
    "/controls/{id}",
];

/// Every POST path. `/login` is the one CSRF-exempt mutation: it has no
/// session yet and enforces its own double-submit token.
const MUTATION_PATHS: &[&str] = &[
    "/login",
    "/logout",
    "/projects",
    "/evidence",
    "/evidence/{id}/review",
    "/inspections",
    "/workflows",
    "/workflows/{id}/{version}/publish",
    "/controls",
    "/controls/{id}/status",
    "/lifecycle",
    "/api/v1/events/replay",
    "/api/v1/workflows",
    "/api/v1/workflows/{id}/publish",
    "/api/v1/approvals",
    "/api/v1/approvals/{id}/decide",
    "/api/v1/approvals/{id}/escalate",
    "/approvals/workflows",
    "/approvals/workflows/{id}/{version}/publish",
    "/approvals/submit",
    "/approvals/{id}/decide",
];

struct Session {
    session_cookie: String,
    with_csrf: String,
}

fn app() -> axum::Router {
    // Keep the route-contract fixture inside the 15-second request timeout:
    // production PBKDF2 cost is covered by `local_auth` unit tests.
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "ekpi-route-inventory-{}-{}.jsonl",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(&path).expect("route-inventory ledger opens"),
    ));
    let mut store = AuthStore::with_iterations(3600, 1_000);
    store
        .register("tester", PASSWORD, Role::Admin)
        .expect("route-inventory admin registers");
    let auth = Arc::new(std::sync::Mutex::new(store));
    let approvals = Arc::new(std::sync::Mutex::new(
        ekp_adapters::jsonl_approval::ApprovalStore::ephemeral()
            .expect("route-inventory approval store opens"),
    ));
    governance_web::router_with_approvals(ledger, auth, approvals)
}

fn probe(path: &str) -> String {
    path.replace("{id}", "probe-id").replace("{version}", "1")
}

async fn send(
    app: &axum::Router,
    method: Method,
    uri: &str,
    cookies: Option<&str>,
    body: &str,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, HOST)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(cookies) = cookies {
        builder = builder.header(header::COOKIE, cookies);
    }
    let request = builder
        .body(Body::from(body.to_string()))
        .expect("route-inventory request builds");
    app.clone()
        .oneshot(request)
        .await
        .expect("route-inventory router responds")
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("route-inventory body reads");
    String::from_utf8_lossy(&bytes).to_string()
}

async fn login(app: &axum::Router) -> Session {
    let page = send(app, Method::GET, "/login", None, "").await;
    assert_eq!(page.status(), StatusCode::OK);
    let html = body_text(page).await;
    let marker = "name=\"csrf_token\" value=\"";
    let start = html.find(marker).expect("login form carries CSRF") + marker.len();
    let token = html[start..]
        .split('"')
        .next()
        .expect("CSRF value terminates")
        .to_string();

    let csrf_cookie = format!("ekpi_csrf={token}");
    let body = format!("csrf_token={token}&username=tester&password={PASSWORD}");
    let response = send(
        app,
        Method::POST,
        "/login",
        Some(&csrf_cookie),
        body.as_str(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let session_cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value
                .starts_with("ekpi_session=")
                .then(|| value.split(';').next().expect("cookie pair").to_string())
        })
        .expect("login sets a session cookie");
    let rotated = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            value
                .starts_with("ekpi_csrf=")
                .then(|| value.split(';').next().expect("cookie pair").to_string())
        })
        .expect("login rotates the CSRF cookie");
    Session {
        with_csrf: format!("{session_cookie}; {rotated}"),
        session_cookie,
    }
}

#[tokio::test]
async fn every_registered_path_rejects_an_unsupported_method() {
    let app = app();
    let session = login(&app).await;
    for (path, allowed) in ROUTES {
        let uri = probe(path);
        let response = send(&app, Method::DELETE, &uri, Some(&session.with_csrf), "").await;
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "DELETE {uri} must be a method mismatch, not a fallback or handler response"
        );
        let allow = response
            .headers()
            .get(header::ALLOW)
            .expect("405 must carry Allow")
            .to_str()
            .expect("Allow is visible ASCII");
        for method in *allowed {
            assert!(
                allow
                    .split(',')
                    .any(|value| value.trim().eq_ignore_ascii_case(method)),
                "Allow for {uri} must list {method}; got `{allow}`"
            );
        }
    }
}

#[tokio::test]
async fn unknown_paths_are_a_styled_404_not_a_method_mismatch() {
    let app = app();
    let session = login(&app).await;

    let unauth = send(&app, Method::GET, "/not-a-route", None, "").await;
    assert_eq!(
        unauth.status(),
        StatusCode::UNAUTHORIZED,
        "unknown paths must not enumerate routes before login"
    );

    let get = send(
        &app,
        Method::GET,
        "/not-a-route",
        Some(&session.with_csrf),
        "",
    )
    .await;
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
    let html = body_text(get).await;
    assert!(html.contains("找不到頁面 /not-a-route"));
    assert!(!html.contains("<script"));

    let delete = send(
        &app,
        Method::DELETE,
        "/not-a-route",
        Some(&session.with_csrf),
        "",
    )
    .await;
    assert_eq!(
        delete.status(),
        StatusCode::NOT_FOUND,
        "the fallback must not turn an unknown path into a 405"
    );
}

#[tokio::test]
async fn mutation_paths_require_a_session_then_a_csrf_token() {
    let app = app();
    let session = login(&app).await;
    for path in MUTATION_PATHS {
        let uri = probe(path);
        let unauth = send(&app, Method::POST, &uri, None, "").await;
        let expected_unauth = if *path == "/login" {
            // `/login` has no session to bind to; it rejects the empty form
            // before checking credentials.
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::UNAUTHORIZED
        };
        assert_eq!(
            unauth.status(),
            expected_unauth,
            "unauthenticated POST {uri} must not reach a handler"
        );

        let no_csrf = send(&app, Method::POST, &uri, Some(&session.session_cookie), "").await;
        let expected_no_csrf = if *path == "/login" {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::FORBIDDEN
        };
        assert_eq!(
            no_csrf.status(),
            expected_no_csrf,
            "session-only POST {uri} must be refused by the CSRF gate"
        );
    }
}

#[tokio::test]
async fn mutation_handlers_reject_empty_inputs_without_server_errors() {
    let app = app();
    let session = login(&app).await;
    let cases: &[(&str, StatusCode)] = &[
        ("/login", StatusCode::BAD_REQUEST),
        ("/projects", StatusCode::BAD_REQUEST),
        ("/evidence", StatusCode::BAD_REQUEST),
        ("/evidence/{id}/review", StatusCode::BAD_REQUEST),
        ("/inspections", StatusCode::OK),
        ("/workflows", StatusCode::BAD_REQUEST),
        ("/workflows/{id}/{version}/publish", StatusCode::OK),
        ("/controls", StatusCode::BAD_REQUEST),
        ("/controls/{id}/status", StatusCode::BAD_REQUEST),
        ("/lifecycle", StatusCode::BAD_REQUEST),
        ("/api/v1/events/replay", StatusCode::NOT_IMPLEMENTED),
        ("/api/v1/workflows", StatusCode::NOT_IMPLEMENTED),
        (
            "/api/v1/workflows/{id}/publish",
            StatusCode::NOT_IMPLEMENTED,
        ),
        ("/api/v1/approvals", StatusCode::BAD_REQUEST),
        ("/api/v1/approvals/{id}/decide", StatusCode::NOT_IMPLEMENTED),
        (
            "/api/v1/approvals/{id}/escalate",
            StatusCode::NOT_IMPLEMENTED,
        ),
        ("/approvals/workflows", StatusCode::BAD_REQUEST),
        (
            "/approvals/workflows/{id}/{version}/publish",
            StatusCode::OK,
        ),
        ("/approvals/submit", StatusCode::BAD_REQUEST),
        ("/approvals/{id}/decide", StatusCode::BAD_REQUEST),
        // `/logout` invalidates the session; it must stay last so the other
        // rows exercise an authenticated request.
        ("/logout", StatusCode::SEE_OTHER),
    ];
    for (path, expected) in cases {
        let response = send(
            &app,
            Method::POST,
            &probe(path),
            Some(&session.with_csrf),
            "",
        )
        .await;
        assert_eq!(
            response.status(),
            *expected,
            "empty POST {path} changed its first-validation contract"
        );
        assert_ne!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "empty POST {path} must never be an internal server error"
        );
        if *expected == StatusCode::NOT_IMPLEMENTED {
            let content_type = response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            assert!(
                content_type.starts_with("application/json"),
                "unwired API {path} must stay a structured JSON 501"
            );
            let json: serde_json::Value =
                serde_json::from_str(&body_text(response).await).expect("501 body is JSON");
            assert_eq!(json["error"]["code"], "501");
            assert!(
                json["error"]["message"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("endpoint not implemented")
            );
        }
    }
}

#[test]
fn page_registry_covers_every_get_page_route() {
    let get_paths: BTreeSet<&str> = ROUTES
        .iter()
        .filter(|(_, methods)| methods.contains(&"GET"))
        .map(|(path, _)| *path)
        .collect();
    let non_page: BTreeSet<&str> = NON_PAGE_GET.iter().copied().collect();
    let page_routes: BTreeSet<&str> = get_paths.difference(&non_page).copied().collect();
    let registry: BTreeSet<&str> = PAGE_REGISTRY.iter().map(|page| page.path).collect();
    assert_eq!(
        registry, page_routes,
        "PAGE_REGISTRY must cover exactly the registered GET page routes"
    );
    assert_eq!(PAGE_REGISTRY.len(), 21);
}

#[tokio::test]
async fn public_surface_is_the_documented_allowlist() {
    let app = app();
    let session = login(&app).await;

    for path in ["/login", "/healthz", "/metrics", "/assets/governance.css"] {
        let response = send(&app, Method::GET, path, None, "").await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "`{path}` is part of the public allowlist"
        );
    }

    let health = send(&app, Method::GET, "/healthz", None, "").await;
    assert_eq!(health.status(), StatusCode::OK);
    let json: serde_json::Value =
        serde_json::from_str(&body_text(health).await).expect("healthz is JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["readiness"], "liveness-only");

    let metrics = send(&app, Method::GET, "/metrics", None, "").await;
    let metrics_type = metrics
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(metrics_type.starts_with("text/plain"));
    assert!(
        body_text(metrics)
            .await
            .contains("ekpi_http_requests_total")
    );

    let css = send(&app, Method::GET, "/assets/governance.css", None, "").await;
    let css_type = css
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(css_type.starts_with("text/css"));

    // Observed at this revision: the readiness probe is session-gated while
    // liveness/metrics are public. C3 preserves that behavior; changing it is a
    // separate operational decision, not part of the module split.
    let ready_unauth = send(&app, Method::GET, "/ready", None, "").await;
    assert_eq!(ready_unauth.status(), StatusCode::UNAUTHORIZED);
    let ready = send(&app, Method::GET, "/ready", Some(&session.with_csrf), "").await;
    assert_eq!(ready.status(), StatusCode::OK);
    let json: serde_json::Value =
        serde_json::from_str(&body_text(ready).await).expect("ready is JSON");
    assert!(json.get("status").is_some(), "ready must report a status");

    let dashboard = send(&app, Method::GET, "/", None, "").await;
    assert_eq!(dashboard.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_v1_status_contract_is_locked() {
    let app = app();
    let session = login(&app).await;

    let reads: &[(&str, StatusCode, Option<&str>)] = &[
        (
            "/api/v1/workflows",
            StatusCode::OK,
            Some("ekpi-approval-workflows-v1"),
        ),
        (
            "/api/v1/approvals",
            StatusCode::OK,
            Some("ekpi-approval-tasks-v1"),
        ),
        ("/api/v1/channels", StatusCode::NOT_IMPLEMENTED, None),
        ("/api/v1/channels/health", StatusCode::NOT_IMPLEMENTED, None),
        ("/api/v1/users", StatusCode::NOT_IMPLEMENTED, None),
        ("/api/v1/settings", StatusCode::NOT_IMPLEMENTED, None),
        ("/api/v1/search", StatusCode::NOT_IMPLEMENTED, None),
    ];
    for (path, expected, schema) in reads {
        let response = send(&app, Method::GET, path, Some(&session.with_csrf), "").await;
        assert_eq!(response.status(), *expected, "GET {path} status changed");
        let json: serde_json::Value =
            serde_json::from_str(&body_text(response).await).expect("API body is JSON");
        match schema {
            Some(schema) => assert_eq!(json["schema"], *schema),
            None => {
                assert_eq!(json["error"]["code"], "501");
                assert!(
                    json["error"]["message"]
                        .as_str()
                        .unwrap_or_default()
                        .contains("endpoint not implemented")
                );
            }
        }
    }

    let unauth = send(&app, Method::GET, "/api/v1/workflows", None, "").await;
    assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);
    let json: serde_json::Value =
        serde_json::from_str(&body_text(unauth).await).expect("401 body is JSON");
    assert_eq!(json["error"]["code"], "401");
}
