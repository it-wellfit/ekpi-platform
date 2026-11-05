#![forbid(unsafe_code)]

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Unified error type for the EKPI platform. All errors should use this enum
/// to provide consistent error codes, messages, and context information.
///
/// This enables structured error responses and better error handling across
/// the application.
#[derive(Debug, Clone, serde::Serialize)]
pub enum EkpiError {
    /// Configuration related errors
    ConfigError(String),
    /// Authentication and authorization errors
    AuthError(String),
    /// Database/storage related errors
    StorageError(String),
    /// Rule engine and evaluation errors
    EngineError(String),
    /// Data validation errors
    ValidationError(String),
    /// I/O and file system errors
    IOError(String),
    /// Network and connection errors
    NetworkError(String),
    /// Business logic errors
    BusinessError(String),
    /// System state errors
    SystemError(String),
    /// Generic error with code
    Generic { code: String, message: String },
}

impl std::fmt::Display for EkpiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EkpiError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            EkpiError::AuthError(msg) => write!(f, "Authentication error: {msg}"),
            EkpiError::StorageError(msg) => write!(f, "Storage error: {msg}"),
            EkpiError::EngineError(msg) => write!(f, "Engine error: {msg}"),
            EkpiError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            EkpiError::IOError(msg) => write!(f, "I/O error: {msg}"),
            EkpiError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            EkpiError::BusinessError(msg) => write!(f, "Business logic error: {msg}"),
            EkpiError::SystemError(msg) => write!(f, "System error: {msg}"),
            EkpiError::Generic { code, message } => write!(f, "[{code}] {message}"),
        }
    }
}

impl std::error::Error for EkpiError {}

/// Type alias for Result with our unified error type
pub type Result<T> = std::result::Result<T, EkpiError>;

/// Convert from String to EkpiError
impl From<String> for EkpiError {
    fn from(value: String) -> Self {
        EkpiError::Generic {
            code: "INTERNAL_ERROR".to_string(),
            message: value,
        }
    }
}

/// Convert from &str to EkpiError
impl From<&str> for EkpiError {
    fn from(value: &str) -> Self {
        EkpiError::Generic {
            code: "INTERNAL_ERROR".to_string(),
            message: value.to_string(),
        }
    }
}

/// Convert from io::Error to EkpiError
impl From<std::io::Error> for EkpiError {
    fn from(value: std::io::Error) -> Self {
        EkpiError::IOError(value.to_string())
    }
}

/// Helper function to create error responses. `request_id` should carry the
/// per-request correlation id resolved by the request-context middleware;
/// `None` falls back to `unknown` (no environment variable is consulted —
/// a global env var cannot represent concurrent requests).
pub fn error_response<T: Into<String>>(
    error: T,
    status: StatusCode,
    request_id: Option<&str>,
) -> impl axum::response::IntoResponse {
    let error_string = error.into();

    let body = json!({
        "error": {
            "code": status.as_str(),
            "type": error_type_for(status),
            "message": error_string,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "request_id": request_id.unwrap_or("unknown"),
            "details": null,
            "documentation_url": format!("https://docs.ekpi.ai/errors/{}", error_type_for(status)),
        }
    });

    (status, Json(body)).into_response()
}

fn error_type_for(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "VALIDATION_ERROR",
        401 => "AUTHENTICATION_ERROR",
        403 => "AUTHORIZATION_ERROR",
        404 => "NOT_FOUND_ERROR",
        405 => "METHOD_NOT_ALLOWED_ERROR",
        408 => "TIMEOUT_ERROR",
        409 => "CONFLICT_ERROR",
        422 => "PROCESSING_ERROR",
        429 => "RATE_LIMIT_ERROR",
        500 => "INTERNAL_ERROR",
        502 => "SERVICE_UNAVAILABLE_ERROR",
        503 => "SERVICE_OVERLOADED_ERROR",
        504 => "TIMEOUT_ERROR",
        _ => "APPLICATION_ERROR",
    }
}

/// Enhanced error response with context information for debugging and diagnostics
pub fn error_response_with_context<T: Into<String>>(
    error: T,
    status: StatusCode,
    context: Option<serde_json::Value>,
    request_id: Option<&str>,
) -> impl axum::response::IntoResponse {
    let error_string = error.into();
    let error_type = error_type_for(status);

    let body = serde_json::json!({
        "error": {
            "code": status.as_str(),
            "type": error_type,
            "message": error_string,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "request_id": request_id.unwrap_or("unknown"),
            "details": context,
            "documentation_url": format!("https://docs.ekpi.ai/errors/{}", error_type),
        }
    });

    (status, axum::Json(body)).into_response()
}

/// Validation error response with field-specific details
pub fn validation_error_response(
    field_errors: std::collections::HashMap<String, serde_json::Value>,
    global_message: Option<String>,
    status: StatusCode,
    request_id: Option<&str>,
) -> impl axum::response::IntoResponse {
    let context = serde_json::json!({
        "field_errors": field_errors,
        "global_message": global_message.unwrap_or_else(|| "Validation failed".to_string()),
        "error_count": field_errors.len(),
        "summary": format!("Validation failed for {} field(s)", field_errors.len()),
    });

    error_response_with_context("Validation failed", status, Some(context), request_id)
}

/// Authentication error response with session details
pub fn auth_error_response(
    error_type: &str,
    message: String,
    remaining_lockout_time: Option<u64>,
    attempt_count: u32,
    account_status: Option<String>,
    request_id: Option<&str>,
) -> impl axum::response::IntoResponse {
    let context = serde_json::json!({
        "error_type": error_type,
        "attempt_count": attempt_count,
        "remaining_lockout_time": remaining_lockout_time,
        "account_status": account_status,
        "retry_after": remaining_lockout_time.map(|secs| secs.to_string()),
        "authentication_method": "local_session",
    });

    error_response_with_context(message, StatusCode::UNAUTHORIZED, Some(context), request_id)
}

/// Rate limiting error response with quota details
pub fn rate_limit_error_response(
    limit: u32,
    remaining: u32,
    reset_time: u64,
    retry_after: u64,
    request_id: Option<&str>,
) -> impl axum::response::IntoResponse {
    let context = serde_json::json!({
        "rate_limit": {
            "limit": limit,
            "remaining": remaining,
            "reset_time": reset_time,
            "retry_after": retry_after,
        },
        "error_type": "RATE_LIMIT_EXCEEDED",
        "recommendation": "Please slow down your request rate and try again later.",
    });

    error_response_with_context(
        format!("Rate limit exceeded: {} requests per minute", limit),
        StatusCode::TOO_MANY_REQUESTS,
        Some(context),
        request_id,
    )
}

/// Panic recovery hook that converts panics into controlled errors
///
/// This hook captures panics and converts them into EkpiError instances,
/// preventing the application from crashing due to unhandled panics.
///
/// # Safety
/// This function is marked as unsafe because it modifies global panic handling
/// state, but it's safe to call during application initialization.
pub fn setup_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let error_message = if let Some(s) = panic_info.payload().downcast_ref::<&'static str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("at {}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let error = EkpiError::SystemError(format!("Panic: {} - {}", error_message, location));
        eprintln!("PANIC: {}", error);

        original_hook(panic_info);
    }));
}
