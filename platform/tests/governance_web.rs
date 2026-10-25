#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use ekp_platform::governance::{Ledger, QUALITY_CHECKLIST};
use ekp_platform::governance_web;
use http_body_util::BodyExt;
use tower::ServiceExt;

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

fn test_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "ekpi-governance-web-{}-{}.jsonl",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ))
}

fn test_app(path: &PathBuf) -> axum::Router {
    let ledger = Ledger::open(path).expect("test ledger opens");
    governance_web::router(Arc::new(Mutex::new(ledger)))
}

async fn post_form(app: &axum::Router, uri: &str, body: &str) -> axum::response::Response {
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::HOST, "127.0.0.1:3000")
        .header(header::ORIGIN, "http://127.0.0.1:3000")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .expect("form request builds");
    app.clone().oneshot(request).await.expect("router responds")
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
    let response = app
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).expect("GET builds"))
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response
        .headers()
        .get(header::CONTENT_SECURITY_POLICY)
        .is_some());
    let html = response_text(response).await;
    assert!(html.contains("跳至主要內容"));
    assert!(html.contains("文件與證據"));
    assert!(!html.contains("<script"));

    let response = app
        .clone()
        .oneshot(
            Request::get("/assets/governance.css")
                .body(Body::empty())
                .expect("stylesheet GET builds"),
        )
        .await
        .expect("router responds");
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
        let response = app
            .clone()
            .oneshot(
                Request::get(route)
                    .body(Body::empty())
                    .expect("page GET builds"),
            )
            .await
            .expect("page route responds");
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

    let response = post_form(
        &app,
        "/projects",
        "name=Pilot+project&department=PMO&owner=Alex&risk_tier=medium&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/projects"
    );

    let response = post_form(
        &app,
        "/evidence",
        "project_id=PRJ-000001&title=Approval+record&source_system=Microsoft+365&source_url=https%3A%2F%2Fexample.test%2Fapproval&source_version=v1&evidence_type=approval&owner=Alex&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_form(
        &app,
        "/evidence/EVD-000002/review",
        "result=accepted&actor=QA&note=Version+confirmed",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_form(
        &app,
        "/workflows",
        "base_id=&title=Change+control&department=Cross-functional&owner=Process+owner&description=Review+and+approve+changes&stages=Intake%0ARisk+review%0AApproval%0AExecution%0AClose&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_form(
        &app,
        "/workflows/TPL-000004/1/publish",
        "actor=Process+owner",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_form(
        &app,
        "/controls",
        "project_id=PRJ-000001&kind=capa&title=Corrective+action&description=Verify+root+cause+and+effectiveness&owner=Alex&due_date=2026-12-31&actor=Jordan",
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = post_form(
        &app,
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
        let response = post_form(&app, "/controls/QMS-000006/status", &body).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
    }

    let mut inspection_body = "project_id=PRJ-000001&reviewer=QA".to_string();
    for item in QUALITY_CHECKLIST {
        inspection_body.push_str(&format!(
            "&result_{}=pass&evidence_{}=REF-{}",
            item.id, item.id, item.id
        ));
    }
    let response = post_form(&app, "/inspections", &inspection_body).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let response = app
        .clone()
        .oneshot(
            Request::get("/inspections")
                .body(Body::empty())
                .expect("inspection GET builds"),
        )
        .await
        .expect("inspection page responds");
    let inspection_html = response_text(response).await;
    assert!(inspection_html.contains("1 份完整檢查紀錄"));
    assert!(inspection_html.contains("REF-QC-01"));

    let response = post_form(
        &app,
        "/lifecycle",
        "project_id=PRJ-000001&gate_id=G4&decision=exception&non_waivable=yes&reason=Missing+mandatory+evidence&evidence_reference=&risk_acceptor=Sponsor&remediation_due=2026-12-31&actor=Reviewer",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response_text(response)
        .await
        .contains("不可豁免要求不得以一般例外放行"));

    let response = app
        .clone()
        .oneshot(
            Request::get("/audit")
                .body(Body::empty())
                .expect("audit GET builds"),
        )
        .await
        .expect("router responds");
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
async fn cross_origin_mutating_request_is_rejected() {
    let path = test_path();
    let app = test_app(&path);
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
    let response = app
        .clone()
        .oneshot(
            Request::get("/audit")
                .body(Body::empty())
                .expect("GET builds"),
        )
        .await
        .expect("router responds");
    assert!(response_text(response).await.contains("0 筆事件"));
    drop(app);
    if path.exists() {
        std::fs::remove_file(path).expect("test ledger removed");
    }
}
