// Web integration tests with real HTTP
// Tests HTTP layer behavior including headers, cookies, redirects, etc.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ekp_platform::governance::Ledger;
use ekp_platform::governance_web;
use ekp_platform::local_auth::AuthStore;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

/// Unique per-run ledger path: a shared fixed name would collide across
/// parallel test runs and could pick up events from a previous run.
fn test_ledger_path(label: &str) -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "ekpi-integration-web-{}-{}-{label}.jsonl",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ))
}

fn app_with_user(
    username: &str,
    password: &str,
    label: &str,
) -> (axum::Router, std::path::PathBuf) {
    let path = test_ledger_path(label);
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let mut store = AuthStore::default();
    store
        .register(username, password, ekp_platform::local_auth::Role::Editor)
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(store));
    (governance_web::router(ledger, auth), path)
}

/// Fetches the login page and returns its fresh CSRF token plus the matching
/// `ekpi_csrf` cookie pair; asserts the hidden field equals the cookie value.
async fn login_page_csrf(app: &axum::Router) -> (String, String) {
    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .header("host", "127.0.0.1:8080")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .expect("login page sets the csrf cookie")
        .to_str()
        .unwrap()
        .to_string();
    assert!(set_cookie.starts_with("ekpi_csrf="));
    let token = set_cookie
        .split(';')
        .next()
        .expect("cookie pair")
        .strip_prefix("ekpi_csrf=")
        .expect("csrf prefix")
        .to_string();
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    let marker = "name=\"csrf_token\" value=\"";
    let start = html.find(marker).expect("hidden csrf field") + marker.len();
    let rest = &html[start..];
    let end = rest.find('"').expect("closing quote");
    let field_token = rest[..end].to_string();
    assert_eq!(
        field_token, token,
        "double-submit token must match the cookie"
    );
    (token.clone(), format!("ekpi_csrf={token}"))
}

/// Browser-equivalent login: page token + page cookie in, rotated session and
/// csrf cookie pairs out, joined into a single Cookie header value.
async fn login(app: &axum::Router, username: &str, password: &str) -> String {
    let (token, csrf_cookie) = login_page_csrf(app).await;
    let body = format!("csrf_token={token}&username={username}&password={password}");
    let response = app
        .clone()
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .header("cookie", &csrf_cookie)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let session = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let v = v.to_str().ok()?;
            v.starts_with("ekpi_session=")
                .then(|| v.split(';').next().unwrap().to_string())
        })
        .expect("session cookie");
    let rotated = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let v = v.to_str().ok()?;
            v.starts_with("ekpi_csrf=")
                .then(|| v.split(';').next().unwrap().to_string())
        })
        .expect("rotated csrf cookie");
    format!("{session}; {rotated}")
}

#[tokio::test]
async fn health_endpoint_returns_200() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("health");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn metrics_endpoint_returns_prometheus_format() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("metrics");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}

#[tokio::test]
async fn unauthenticated_request_to_data_page_returns_401() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("unauth");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/projects").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_valid_credentials_sets_session_cookie() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let (app, _path) = app_with_user("testuser", "password123", "login-cookie");

    let response = app
        .clone()
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .body(Body::from(
                    "csrf_token=missing&username=testuser&password=password123",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    // The double-submit check runs before credential verification.
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let cookies = login(&app, "testuser", "password123").await;
    assert!(cookies.starts_with("ekpi_session="));
    assert!(cookies.contains("ekpi_csrf="));
}

#[tokio::test]
async fn oversized_form_body_is_rejected() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("oversized");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let large_body = "a".repeat(65 * 1024); // 65KB, exceeds 64KB limit
    let response = app
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .body(Body::from(large_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn security_headers_are_present() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("sec-headers-1");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Check for security headers
    assert!(
        response.headers().get("x-content-type-options").is_some()
            || response.headers().get("content-security-policy").is_some()
    );
}

#[tokio::test]
async fn security_headers_validation() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("sec-headers-2");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Verify security headers are present
    let headers = response.headers();
    // X-Content-Type-Options should be present
    assert!(
        headers.get("x-content-type-options").is_some()
            || headers.get("content-security-policy").is_some(),
        "Security headers should be present"
    );
}

#[tokio::test]
async fn error_response_structure_validation() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let path = test_ledger_path("err-struct");
    let ledger = Arc::new(std::sync::Mutex::new(Ledger::open(&path).unwrap()));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    // A client-supplied correlation id must come back in both the response
    // header and the structured error body.
    let response = app
        .oneshot(
            Request::get("/projects")
                .header("host", "127.0.0.1:8080")
                .header("x-request-id", "test-req-42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Unauthenticated request should return 401 with structured error
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("x-request-id")
            .map(|v| v.to_str().unwrap()),
        Some("test-req-42"),
        "the request id must round-trip"
    );
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).expect("error body must be structured JSON");
    let error = parsed
        .get("error")
        .expect("error envelope present")
        .as_object()
        .expect("error is an object");
    assert_eq!(error.get("code").and_then(|v| v.as_str()), Some("401"));
    assert_eq!(
        error.get("type").and_then(|v| v.as_str()),
        Some("AUTHENTICATION_ERROR")
    );
    assert!(
        error
            .get("message")
            .and_then(|v| v.as_str())
            .is_some_and(|m| !m.is_empty()),
        "message must be non-empty"
    );
    assert_eq!(
        error.get("request_id").and_then(|v| v.as_str()),
        Some("test-req-42"),
        "the body must carry the same correlation id"
    );
}

#[tokio::test]
async fn input_validation_edge_cases() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let (app, _path) = app_with_user("testuser", "password123", "input-val");

    // Test with empty body
    let response = app
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .body(Body::from(""))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle empty body gracefully (not crash)
    assert!(response.status().is_client_error() || response.status().is_success());
}

#[tokio::test]
async fn csrf_rejects_missing_token() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let (app, _path) = app_with_user("testuser", "password123", "csrf-missing");

    let session_only = {
        let cookies = login(&app, "testuser", "password123").await;
        cookies
            .split("; ")
            .find(|pair| pair.starts_with("ekpi_session="))
            .expect("session pair")
            .to_string()
    };

    // POST to /evidence with a valid session but no CSRF token must be
    // rejected by the CSRF layer, not the authentication layer.
    let response = app
        .oneshot(
            Request::post("/evidence")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .header("cookie", &session_only)
                .body(Body::from("title=test&source_system=test&source_url=http://example.com&source_version=1&evidence_type=test&owner=test"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn csrf_accepts_valid_token() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let (app, _path) = app_with_user("testuser", "password123", "csrf-valid");

    let cookies = login(&app, "testuser", "password123").await;

    // Zero-JS browser shape: the CSRF token arrives via the rotated
    // `ekpi_csrf` cookie; no custom header is set.
    let response = app
        .oneshot(
            Request::post("/evidence")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .header("cookie", &cookies)
                .body(Body::from("title=test&source_system=test&source_url=http://example.com&source_version=1&evidence_type=test&owner=test"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Neither the CSRF layer (403) nor the session layer (401) may reject.
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unwired_api_endpoints_return_honest_501_never_fabricated_data() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let (app, _path) = app_with_user("testuser", "password123", "api-honest");

    let cookies = login(&app, "testuser", "password123").await;

    // An authenticated GET against every unwired API endpoint must answer
    // 501 NOT IMPLEMENTED with the structured error body — never 200 with
    // placeholder data that would read as a working feature.
    for path in [
        "/api/v1/workflows",
        "/api/v1/approvals",
        "/api/v1/channels",
        "/api/v1/users",
        "/api/v1/settings",
        "/api/v1/search",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("host", "127.0.0.1:8080")
                    .header("cookie", cookies.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NOT_IMPLEMENTED,
            "{path} must fail honestly"
        );
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).expect("error body must be structured JSON");
        assert_eq!(
            parsed
                .get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_str()),
            Some("501"),
            "{path} must carry the structured 501 envelope"
        );
    }
}
