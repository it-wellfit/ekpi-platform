#![forbid(unsafe_code)]

//! Health, correlation-id and security-header contract for the local workbench.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{HeaderName, Request, StatusCode};
use ekp_platform::governance::Ledger;
use ekp_platform::governance_web;
use ekp_platform::local_auth::AuthStore;
use http_body_util::BodyExt;
use tower::ServiceExt;

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

fn test_app() -> (axum::Router, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "ekpi-web-health-{}-{}.jsonl",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ));
    let ledger = Ledger::open(&path).expect("test ledger opens");
    let auth = Arc::new(Mutex::new(AuthStore::new(3600)));
    let app = governance_web::router(Arc::new(Mutex::new(ledger)), auth);
    (app, path)
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body collects")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("response body is UTF-8")
}

fn request_id_header() -> HeaderName {
    HeaderName::from_static("x-request-id")
}

fn header_value<'a>(response: &'a axum::response::Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .unwrap_or_else(|| panic!("missing header {name}"))
        .to_str()
        .expect("header is ASCII")
}

#[tokio::test]
async fn health_endpoint_is_a_liveness_probe_with_no_store() {
    let (app, path) = test_app();
    let response = app
        .clone()
        .oneshot(
            Request::get("/healthz")
                .body(Body::empty())
                .expect("health GET builds"),
        )
        .await
        .expect("router responds");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(header_value(&response, "content-type").starts_with("application/json"));
    assert_eq!(header_value(&response, "cache-control"), "no-store");
    assert!(response.headers().get(request_id_header()).is_some());
    let body = body_text(response).await;
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"scope\":\"local-only\""));
    assert!(body.contains("\"readiness\":\"liveness-only\""));

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn metrics_endpoint_exposes_aggregate_counters_only() {
    let (app, path) = test_app();
    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::get("/metrics")
                .body(Body::empty())
                .expect("metrics GET builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(header_value(&response, "content-type").starts_with("text/plain"));
    assert_eq!(header_value(&response, "cache-control"), "no-store");
    assert!(response.headers().get(request_id_header()).is_some());

    let body = body_text(response).await;
    assert!(body.contains("ekpi_http_requests_total "));
    assert!(body.contains("ekpi_http_responses_4xx_total "));
    assert!(body.contains("ekpi_http_responses_5xx_total "));
    assert!(body.contains("ekpi_http_requests_in_flight "));
    assert!(body.contains("ekpi_http_rate_limited_total "));
    assert!(body.contains("ekpi_http_request_duration_seconds_sum "));
    assert!(body.contains("ekpi_http_request_duration_seconds_count "));
    assert!(!body.contains("actor"));
    assert!(!body.contains("stephanie"));

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn request_id_is_echoed_only_when_safe() {
    let (app, path) = test_app();

    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .header(request_id_header(), "pilot-123_alpha")
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");
    assert_eq!(header_value(&response, "x-request-id"), "pilot-123_alpha");

    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .header(request_id_header(), "bad id with spaces")
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");
    let generated = header_value(&response, "x-request-id").to_string();
    assert_ne!(generated, "bad id with spaces");
    assert!(generated.starts_with("ekpi-"));

    let too_long = "a".repeat(129);
    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .header(request_id_header(), too_long.clone())
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");
    assert_ne!(header_value(&response, "x-request-id"), too_long);

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}

#[tokio::test]
async fn dynamic_pages_carry_defence_in_depth_headers() {
    let (app, path) = test_app();
    let response = app
        .clone()
        .oneshot(
            Request::get("/login")
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");

    assert_eq!(response.status(), StatusCode::OK);
    let csp = header_value(&response, "content-security-policy");
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("frame-ancestors 'none'"));
    assert_eq!(header_value(&response, "x-content-type-options"), "nosniff");
    assert_eq!(header_value(&response, "x-frame-options"), "DENY");
    assert_eq!(
        header_value(&response, "referrer-policy"),
        "strict-origin-when-cross-origin"
    );
    assert_eq!(header_value(&response, "cache-control"), "no-store");
    assert!(header_value(&response, "permissions-policy").contains("camera=()"));
    assert_eq!(
        header_value(&response, "cross-origin-opener-policy"),
        "same-origin"
    );
    assert_eq!(
        header_value(&response, "cross-origin-resource-policy"),
        "same-origin"
    );

    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}
