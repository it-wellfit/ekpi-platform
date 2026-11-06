// ekpi-platform web crate
// Axum HTTP server, Askama HTML templates, handlers, routes, middleware

use axum::{
    Router,
    http::{StatusCode, header},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod templates;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("shutdown signal received");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/submit", post(handle_submit))
        .nest_service("/static", ServeDir::new("static"))
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let graceful = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    graceful.await.unwrap();
}

async fn home() -> Html<String> {
    let csrf_token = generate_csrf_token();
    let tmpl = templates::HomeTemplate {
        title: "EKPI Platform".to_string(),
        csrf_token,
    };
    Html(tmpl.to_string())
}

async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn handle_submit(
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Validate CSRF token
    let csrf_token = form.get("csrf_token").cloned().unwrap_or_default();
    if !validate_csrf_token(&csrf_token) {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    // Process the form data (PRG pattern: redirect after POST)
    tracing::info!("Form submitted successfully");

    // Redirect to home page (Post/Redirect/Get pattern)
    Redirect::to("/").into_response()
}

fn generate_csrf_token() -> String {
    // CSPRNG-based token (UUID v4). Session binding and single-use
    // enforcement require session infrastructure not yet present in
    // this skeleton app; tracked as follow-up (see plan C6-3).
    uuid::Uuid::new_v4().to_string()
}

fn validate_csrf_token(token: &str) -> bool {
    // Accept only well-formed UUID v4 tokens (unpredictable, CSPRNG).
    uuid::Uuid::parse_str(token)
        .map(|u| u.get_version_num() == 4)
        .unwrap_or(false)
}

async fn security_headers(
    request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'".parse().unwrap(),
    );
    headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(
        header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_is_uuid_v4_and_validates() {
        let token = generate_csrf_token();
        assert!(
            validate_csrf_token(&token),
            "a freshly generated token must validate: {token}"
        );
    }

    #[test]
    fn csrf_validation_rejects_legacy_and_malformed_tokens() {
        assert!(!validate_csrf_token(""), "empty token refused");
        assert!(
            !validate_csrf_token("csrf-12345"),
            "legacy time-based prefix must be refused"
        );
        assert!(!validate_csrf_token("not-a-uuid"));
        // A UUID v1/v3/v5 shape must also be refused; only v4 passes.
        assert!(
            !validate_csrf_token("00000000-0000-1000-8000-000000000000"),
            "non-v4 UUID versions must be refused"
        );
    }
}
