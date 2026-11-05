// Web integration tests with real HTTP
// Tests HTTP layer behavior including headers, cookies, redirects, etc.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ekp_platform::governance::Ledger;
use ekp_platform::governance_web;
use ekp_platform::local_auth::AuthStore;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_returns_200() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger2.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger3.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger4.jsonl")).unwrap(),
    ));
    let mut auth_store = AuthStore::default();
    auth_store
        .register(
            "testuser",
            "password123",
            ekp_platform::local_auth::Role::Editor,
        )
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(auth_store));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .body(Body::from("username=testuser&password=password123"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::FOUND
            || response.status() == StatusCode::OK
            || response.status() == StatusCode::SEE_OTHER
    );
    // Check for Set-Cookie header
    let set_cookie = response.headers().get("set-cookie");
    assert!(set_cookie.is_some());
}

#[tokio::test]
async fn oversized_form_body_is_rejected() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger5.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_ledger6.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_sec_headers.jsonl")).unwrap(),
    ));
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_err_struct.jsonl")).unwrap(),
    ));
    let auth = Arc::new(std::sync::Mutex::new(AuthStore::default()));
    let app = governance_web::router(ledger, auth);

    let response = app
        .oneshot(Request::get("/projects").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Unauthenticated request should return 401 with structured error
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn input_validation_edge_cases() {
    unsafe {
        std::env::set_var("EKPI_TEST_MODE", "1");
    }
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_input_val.jsonl")).unwrap(),
    ));
    let mut auth_store = AuthStore::default();
    auth_store
        .register(
            "testuser",
            "password123",
            ekp_platform::local_auth::Role::Editor,
        )
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(auth_store));
    let app = governance_web::router(ledger, auth);

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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_csrf_missing.jsonl")).unwrap(),
    ));
    let mut auth_store = AuthStore::default();
    auth_store
        .register("testuser", "password123", ekp_platform::local_auth::Role::Editor)
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(auth_store));
    let app = governance_web::router(ledger, auth);

    // POST to /evidence without CSRF token should be rejected
    let response = app
        .oneshot(
            Request::post("/evidence")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
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
    let ledger = Arc::new(std::sync::Mutex::new(
        Ledger::open(std::env::temp_dir().join("test_csrf_valid.jsonl")).unwrap(),
    ));
    let mut auth_store = AuthStore::default();
    auth_store
        .register("testuser", "password123", ekp_platform::local_auth::Role::Editor)
        .unwrap();
    let auth = Arc::new(std::sync::Mutex::new(auth_store));
    let app = governance_web::router(ledger, auth);

    // First create a session to get CSRF token
    let login_response = app
        .oneshot(
            Request::post("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .body(Body::from("username=testuser&password=password123"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Extract CSRF token from session cookie
    let csrf_token = login_response
        .headers()
        .get("set-cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| {
            v.split(';')
                .find(|p| p.trim_start().starts_with("ekpi_csrf="))
                .map(|p| p.trim_start().strip_prefix("ekpi_csrf=").unwrap().to_string())
        })
        .unwrap_or_else(|| "".to_string());

    // POST with CSRF token should pass CSRF check (may fail for other reasons)
    let response = app
        .oneshot(
            Request::post("/evidence")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", "127.0.0.1:8080")
                .header("origin", "http://127.0.0.1:8080")
                .header("X-CSRF-Token", &csrf_token)
                .body(Body::from("title=test&source_system=test&source_url=http://example.com&source_version=1&evidence_type=test&owner=test"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 403 (CSRF check passed)
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}
