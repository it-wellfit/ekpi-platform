#![forbid(unsafe_code)]

//! API layer (Master Prompt Phase 7a).
//!
//! axum REST endpoints for document, rule, case, and process operations.

use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::engine::Platform;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub platform: Arc<Platform>,
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Case adjudication request.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdjudicateRequest {
    pub case_id: String,
}

/// Case adjudication response.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdjudicateResponse {
    pub case_id: String,
    pub verdict: String,
    pub citations: Vec<String>,
    pub rule_ids_used: Vec<String>,
    pub notes: Vec<String>,
}

/// Build the axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/cases/:id/decide", post(adjudicate_case))
        .route("/api/v1/cases/:id/snapshot", get(get_snapshot))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn adjudicate_case(
    Path(id): Path<String>,
    axum::extract::State(_state): axum::extract::State<AppState>,
) -> Result<Json<AdjudicateResponse>, StatusCode> {
    // Simplified: return a placeholder response
    Ok(Json(AdjudicateResponse {
        case_id: id,
        verdict: "comply".to_string(),
        citations: Vec::new(),
        rule_ids_used: Vec::new(),
        notes: Vec::new(),
    }))
}

async fn get_snapshot(
    Path(id): Path<String>,
    axum::extract::State(_state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(
        serde_json::json!({"case_id": id, "status": "not_found"}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_serializes() {
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("ok"));
    }

    #[test]
    fn adjudicate_response_serializes() {
        let resp = AdjudicateResponse {
            case_id: "C1".to_string(),
            verdict: "comply".to_string(),
            citations: vec!["doc@1 §1".to_string()],
            rule_ids_used: vec!["RULE-1".to_string()],
            notes: Vec::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("C1"));
    }
}
