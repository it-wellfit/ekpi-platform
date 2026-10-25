#![forbid(unsafe_code)]

//! CSS-first, server-rendered local pilot workbench.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Form, Path, Request, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;
use maud::{html, Markup, DOCTYPE};
use serde::Deserialize;

use crate::governance::{
    GovernanceSnapshot, Ledger, Project, QualityCheckResult, QualityInspectionInput, Workflow,
    WorkflowDraftInput, QUALITY_CHECKLIST,
};

type SharedLedger = Arc<Mutex<Ledger>>;

const CSS: &str = include_str!("../assets/governance.css");

pub fn router(ledger: SharedLedger) -> Router {
    Router::new()
        .route("/", get(dashboard))
        .route("/projects", get(projects_page).post(create_project))
        .route("/evidence", get(evidence_page).post(link_evidence))
        .route("/evidence/{id}/review", post(review_evidence))
        .route(
            "/inspections",
            get(inspections_page).post(record_inspection),
        )
        .route("/workflows", get(workflows_page).post(save_workflow))
        .route("/workflows/{id}/{version}/publish", post(publish_workflow))
        .route("/controls", get(controls_page).post(create_quality_case))
        .route("/controls/{id}/status", post(update_quality_case))
        .route("/lifecycle", get(lifecycle_page).post(record_gate_decision))
        .route("/audit", get(audit_page))
        .route("/assets/governance.css", get(stylesheet))
        .with_state(ledger)
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn(same_origin_guard))
}

async fn stylesheet() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], CSS)
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; style-src 'self'; img-src 'self' data:; form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    response
}

async fn same_origin_guard(request: Request, next: Next) -> Response {
    let is_mutation = request.method() != Method::GET && request.method() != Method::HEAD;
    if is_mutation {
        if let (Some(origin), Some(host)) = (
            request
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok()),
            request
                .headers()
                .get(header::HOST)
                .and_then(|v| v.to_str().ok()),
        ) {
            let expected = format!("http://{host}");
            if origin != expected {
                return (StatusCode::FORBIDDEN, "跨來源表單提交已拒絕").into_response();
            }
        }
    }
    next.run(request).await
}

async fn dashboard(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("工作總覽", "overview", dashboard_content(&data)),
        Err(error) => error_response("overview", &error),
    }
}

async fn projects_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("專案登錄", "projects", projects_content(&data, None)),
        Err(error) => error_response("projects", &error),
    }
}

async fn create_project(
    State(ledger): State<SharedLedger>,
    Form(form): Form<ProjectForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.record_project(
                &form.name,
                &form.department,
                &form.owner,
                &form.risk_tier,
                &form.actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/projects").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "專案登錄",
                "projects",
                projects_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("projects", &error),
        },
    }
}

async fn evidence_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("文件與證據", "evidence", evidence_content(&data, None)),
        Err(error) => error_response("evidence", &error),
    }
}

async fn link_evidence(
    State(ledger): State<SharedLedger>,
    Form(form): Form<EvidenceForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.link_evidence(
                &form.project_id,
                &form.title,
                &form.source_system,
                &form.source_url,
                &form.source_version,
                &form.evidence_type,
                &form.owner,
                &form.actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/evidence").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "文件與證據",
                "evidence",
                evidence_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("evidence", &error),
        },
    }
}

async fn review_evidence(
    State(ledger): State<SharedLedger>,
    Path(id): Path<String>,
    Form(form): Form<ReviewForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.review_evidence(&id, &form.result, &form.actor, &form.note));
    match result {
        Ok(()) => Redirect::to("/evidence").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "文件與證據",
                "evidence",
                evidence_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("evidence", &error),
        },
    }
}

async fn workflows_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("流程範本", "workflows", workflows_content(&data, None)),
        Err(error) => error_response("workflows", &error),
    }
}

async fn save_workflow(
    State(ledger): State<SharedLedger>,
    Form(form): Form<WorkflowForm>,
) -> Response {
    let stages: Vec<String> = form
        .stages
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    let base_id = (!form.base_id.trim().is_empty()).then_some(form.base_id);
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.save_workflow(WorkflowDraftInput {
                base_id,
                title: form.title,
                department: form.department,
                owner: form.owner,
                description: form.description,
                stages,
                actor: form.actor,
            })
        });
    match result {
        Ok(_) => Redirect::to("/workflows").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "流程範本",
                "workflows",
                workflows_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("workflows", &error),
        },
    }
}

async fn publish_workflow(
    State(ledger): State<SharedLedger>,
    Path((id, version)): Path<(String, u32)>,
    Form(form): Form<PublishForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.publish_workflow(&id, version, &form.actor));
    match result {
        Ok(()) => Redirect::to("/workflows").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "流程範本",
                "workflows",
                workflows_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("workflows", &error),
        },
    }
}

async fn controls_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("風險與品質事件", "controls", controls_content(&data, None)),
        Err(error) => error_response("controls", &error),
    }
}

async fn create_quality_case(
    State(ledger): State<SharedLedger>,
    Form(form): Form<QualityCaseForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.create_quality_case(
                &form.project_id,
                &form.kind,
                &form.title,
                &form.description,
                &form.owner,
                &form.due_date,
                &form.actor,
            )
        });
    match result {
        Ok(_) => Redirect::to("/controls").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "風險與品質事件",
                "controls",
                controls_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("controls", &error),
        },
    }
}

async fn update_quality_case(
    State(ledger): State<SharedLedger>,
    Path(id): Path<String>,
    Form(form): Form<QualityCaseUpdateForm>,
) -> Response {
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.update_quality_case(&id, &form.status, &form.actor, &form.note));
    match result {
        Ok(()) => Redirect::to("/controls").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "風險與品質事件",
                "controls",
                controls_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("controls", &error),
        },
    }
}

async fn inspections_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("品質檢查", "inspections", inspections_content(&data, None)),
        Err(error) => error_response("inspections", &error),
    }
}

async fn record_inspection(
    State(ledger): State<SharedLedger>,
    Form(fields): Form<HashMap<String, String>>,
) -> Response {
    let get = |field: &str| fields.get(field).cloned().unwrap_or_default();
    let results = QUALITY_CHECKLIST
        .iter()
        .map(|item| {
            let suffix = item.id;
            QualityCheckResult {
                item_id: suffix.to_string(),
                result: get(&format!("result_{suffix}")),
                note: get(&format!("note_{suffix}")),
                evidence_reference: get(&format!("evidence_{suffix}")),
                issue_id: get(&format!("issue_{suffix}")),
            }
        })
        .collect();
    let input = QualityInspectionInput {
        project_id: get("project_id"),
        reviewer: get("reviewer"),
        results,
    };
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| db.record_quality_inspection(input));
    match result {
        Ok(_) => Redirect::to("/inspections").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "品質檢查",
                "inspections",
                inspections_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("inspections", &error),
        },
    }
}

async fn lifecycle_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => page_response("階段與關卡", "lifecycle", lifecycle_content(&data, None)),
        Err(error) => error_response("lifecycle", &error),
    }
}

async fn record_gate_decision(
    State(ledger): State<SharedLedger>,
    Form(form): Form<GateDecisionForm>,
) -> Response {
    if !["yes", "no"].contains(&form.non_waivable.as_str()) {
        return match snapshot(&ledger) {
            Ok(data) => page_response(
                "階段與關卡",
                "lifecycle",
                lifecycle_content(&data, Some("請明確選擇是否涉及不可豁免要求。")),
            ),
            Err(error) => error_response("lifecycle", &error),
        };
    }
    let result = ledger
        .lock()
        .map_err(|_| "資料鎖定失敗".to_string())
        .and_then(|mut db| {
            db.record_gate_decision(
                &form.project_id,
                &form.gate_id,
                &form.decision,
                &form.actor,
                &form.reason,
                &form.evidence_reference,
                &form.risk_acceptor,
                &form.remediation_due,
                form.non_waivable == "yes",
            )
        });
    match result {
        Ok(_) => Redirect::to("/lifecycle").into_response(),
        Err(error) => match snapshot(&ledger) {
            Ok(data) => page_response(
                "階段與關卡",
                "lifecycle",
                lifecycle_content(&data, Some(&error)),
            )
            .into_response(),
            Err(_) => error_response("lifecycle", &error),
        },
    }
}

async fn audit_page(State(ledger): State<SharedLedger>) -> Response {
    match snapshot(&ledger) {
        Ok(data) => {
            let integrity = ledger.lock().map(|db| db.verify().is_ok()).unwrap_or(false);
            page_response("稽核紀錄", "audit", audit_content(&data, integrity))
        }
        Err(error) => error_response("audit", &error),
    }
}

fn snapshot(ledger: &SharedLedger) -> Result<GovernanceSnapshot, String> {
    ledger
        .lock()
        .map(|db| db.snapshot())
        .map_err(|_| "資料鎖定失敗，請重新啟動本機原型".to_string())
}

fn page_response(title: &str, active: &str, content: Markup) -> Response {
    Html(layout(title, active, content).into_string()).into_response()
}

fn error_response(active: &str, message: &str) -> Response {
    let content = html! {
        section class="page-heading" { p class="eyebrow" { "系統資料無法使用" } h1 { "需要檢查稽核資料" } }
        div class="alert alert-error" role="alert" { (message) }
        p { "請停止寫入，保留資料檔並聯絡管理者。原型不會自動修補不完整的稽核鏈。" }
    };
    page_response("資料錯誤", active, content)
}

fn layout(title: &str, active: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="zh-Hant" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light";
                title { (title) " · EKPI 治理工作台" }
                link rel="stylesheet" href="/assets/governance.css";
            }
            body {
                a class="skip-link" href="#main-content" { "跳至主要內容" }
                header class="topbar" {
                    a class="brand" href="/" aria-label="EKPI 治理工作台首頁" {
                        span class="brand-mark" aria-hidden="true" { "E" }
                        span { strong { "EKPI" } small { "治理工作台" } }
                    }
                    div class="topbar-meta" {
                        span class="local-chip" { "本機試點" }
                        span class="avatar" aria-hidden="true" { "G" }
                    }
                }
                div class="prototype-banner" role="note" {
                    strong { "原型模式" }
                    span { "僅限本機與非敏感試用資料；沒有企業登入，也未連接 Microsoft、Google 或 AI。" }
                }
                div class="app-shell" {
                    aside class="sidebar" aria-label="主要導覽" {
                        p class="nav-label" { "工作區" }
                        nav {
                            (nav_link("/", "overview", active, "總覽", "◫"))
                            (nav_link("/projects", "projects", active, "專案", "▦"))
                            (nav_link("/evidence", "evidence", active, "文件與證據", "▤"))
                            (nav_link("/inspections", "inspections", active, "品質檢查", "✓"))
                            (nav_link("/workflows", "workflows", active, "流程範本", "⌘"))
                            (nav_link("/controls", "controls", active, "風險與品質", "◇"))
                            (nav_link("/lifecycle", "lifecycle", active, "階段與關卡", "▥"))
                            (nav_link("/audit", "audit", active, "稽核紀錄", "◷"))
                        }
                        div class="sidebar-note" {
                            span class="status-dot" aria-hidden="true" {}
                            div { strong { "資料保存在本機" } small { "JSONL 雜湊鏈 · 手動來源參照" } }
                        }
                    }
                    main id="main-content" class="main-content" tabindex="-1" { (content) }
                }
                footer class="app-footer" {
                    span { "EKPI · 跨專案治理試點" }
                    span { "雲端文件留在原始平台，工作台僅保存參照與治理紀錄。" }
                }
            }
        }
    }
}

fn nav_link(href: &str, key: &str, active: &str, label: &str, icon: &str) -> Markup {
    let selected = key == active;
    html! {
        a class={(if selected { "nav-link is-active" } else { "nav-link" })}
          href=(href) aria-current=(if selected { "page" } else { "false" }) {
            span class="nav-icon" aria-hidden="true" { (icon) }
            span { (label) }
        }
    }
}

fn dashboard_content(data: &GovernanceSnapshot) -> Markup {
    let reviewed = data.reviews.len();
    let waiting_review = data.evidence.len().saturating_sub(reviewed);
    let open_cases = data
        .cases
        .iter()
        .filter(|case| case.status != "closed")
        .count();
    let draft_workflows = data
        .workflows
        .iter()
        .filter(|workflow| workflow.status == "draft")
        .count();
    html! {
        section class="page-heading" {
            div { p class="eyebrow" { "治理概況" } h1 { "工作總覽" } p class="page-subtitle" { "集中查看專案進度、文件追溯與需要人工確認的工作。" } }
            a class="button button-primary" href="/projects" { "＋ 新增專案" }
        }
        div class="metric-grid" {
            (metric("專案", data.projects.len(), "已登錄專案"))
            (metric("待審證據", waiting_review, "尚未留下人工審查紀錄"))
            (metric("開放事項", open_cases, "風險、變更、缺失與 CAPA"))
            (metric("流程草稿", draft_workflows, "等待負責人審查發布"))
        }
        section class="content-grid" {
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "最近專案" } p { "所有數字來自本機實際登錄資料。" } } a href="/projects" class="text-link" { "查看專案 →" } }
                (project_table(&data.projects, 5))
            }
            div class="panel" {
                div class="panel-heading" { div { h2 { "待辦焦點" } p { "需要負責人確認的項目" } } }
                ul class="focus-list" {
                    li { span class="focus-icon focus-amber" { "▤" } div { strong { (waiting_review) " 筆證據待審查" } small { "核對來源版本與證據目的" } } }
                    li { span class="focus-icon focus-blue" { "◇" } div { strong { (open_cases) " 項風險或品質事項開啟" } small { "追蹤負責人、期限與結案證據" } } }
                    li { span class="focus-icon focus-green" { "⌘" } div { strong { (draft_workflows) " 份流程草稿" } small { "發布前需由流程負責人核准" } } }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "最近活動" } p { "追加式記錄；摘要不代表來源系統已完成同步。" } } a href="/audit" class="text-link" { "開啟稽核紀錄 →" } }
                (recent_events(&data.events, 5))
            }
        }
        p class="data-note" { "試點指標尚未建立基線；此處只顯示登錄筆數，不代表證據完整度或流程改善成效。" }
    }
}

fn metric(label: &str, value: usize, note: &str) -> Markup {
    html! { article class="metric-card" { p { (label) } strong class="metric-value" { (value) } small { (note) } } }
}

fn projects_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 專案" } h1 { "專案登錄" } p class="page-subtitle" { "使用共同識別碼串連流程、證據、決策與改善事項。" } } }
        (alert(error))
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "建立專案" } p { "第一版先記錄責任、部門與風險層級。" } } }
                form class="form-stack" method="post" action="/projects" {
                    (text_field("name", "專案名稱", "例如：跨部門文件治理試點", true, ""))
                    (text_field("department", "主要部門", "", true, ""))
                    (text_field("owner", "專案負責人", "", true, ""))
                    label class="field" { span { "初始風險等級" } select name="risk_tier" required { option value="medium" selected { "中" } option value="low" { "低" } option value="high" { "高" } option value="critical" { "重大" } } }
                    (text_field("actor", "登記人姓名", "僅供原型稽核標籤，尚未驗證身份", true, ""))
                    button class="button button-primary" type="submit" { "建立專案" }
                    p class="form-hint" { "請勿在本機原型輸入機密或受管制資訊。" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "已登錄專案" } p { (data.projects.len()) " 個專案" } } }
                (project_table(&data.projects, usize::MAX))
            }
        }
    }
}

fn project_table(projects: &[Project], limit: usize) -> Markup {
    html! {
        @if projects.is_empty() {
            (empty_state("尚無專案資料", "建立一筆專案記錄後，才能串連文件證據與治理事件。", "/projects", "建立專案"))
        } @else {
            div class="table-wrap" {
                table {
                    caption class="visually-hidden" { "已登錄專案清單" }
                    thead { tr { th scope="col" { "專案" } th scope="col" { "部門" } th scope="col" { "負責人" } th scope="col" { "風險" } } }
                    tbody {
                        @for project in projects.iter().rev().take(limit) {
                            tr { th scope="row" { strong { (project.name) } small class="row-id" { (project.id) } } td { (project.department) } td { (project.owner) } td { (risk_badge(&project.risk_tier)) } }
                        }
                    }
                }
            }
        }
    }
}

fn evidence_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 文件與證據" } h1 { "文件與證據追溯" } p class="page-subtitle" { "記錄權威來源、版本與審查結果；文件本體保留在原平台。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再登錄其文件與證據來源。"))) }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "連結證據" } p { "這是來源參照，不會複製或同步雲端文件。" } } }
                form class="form-stack" method="post" action="/evidence" {
                    label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                    (text_field("title", "文件／證據標題", "", true, ""))
                    label class="field" { span { "來源平台" } select name="source_system" required { option value="Microsoft 365" { "Microsoft 365" } option value="Google Workspace" { "Google Workspace" } option value="Other" { "其他／混合工具" } } }
                    (text_field("source_url", "來源連結", "https://…", true, ""))
                    (text_field("source_version", "來源版本／修訂", "例如：v3.2 或平台版本 ID", true, ""))
                    (text_field("evidence_type", "證據類型", "例如：核准紀錄、測試報告", true, ""))
                    (text_field("owner", "文件負責人", "", true, ""))
                    (text_field("actor", "登記人姓名", "尚未驗證身份", true, ""))
                    button class="button button-primary" type="submit" { "登錄來源參照" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "證據索引" } p { (data.evidence.len()) " 筆來源參照" } } }
                (evidence_table(data))
            }
        }
    }
}

fn evidence_table(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.evidence.is_empty() {
            (empty_state("尚無證據參照", "先新增專案，再從來源平台登錄連結和文件版本。", "/projects", "建立專案"))
        } @else {
            div class="table-wrap" {
                table {
                    caption class="visually-hidden" { "文件與證據來源索引" }
                    thead { tr { th scope="col" { "文件與來源" } th scope="col" { "專案" } th scope="col" { "負責人" } th scope="col" { "審查狀態" } th scope="col" { "人工審查" } } }
                    tbody {
                        @for item in data.evidence.iter().rev() {
                            @let project = data.projects.iter().find(|p| p.id == item.project_id);
                            @let review = data.reviews.get(&item.id);
                            tr {
                                th scope="row" {
                                    strong { (item.title) }
                                    small class="row-id" { (item.id) " · " (item.source_system) " · " (item.source_version) }
                                    a class="source-link" href=(item.source_url) target="_blank" rel="noopener noreferrer" { "開啟來源連結 ↗" }
                                }
                                td { @if let Some(project) = project { (project.id) } @else { "專案缺失" } }
                                td { (item.owner) }
                                td { @if let Some(review) = review { (review_badge(&review.result)) } @else { (status_badge("pending", "待審查")) } }
                                td {
                                    @if let Some(review) = review { small { (review.reviewer) " · " (format_timestamp(review.reviewed_at)) } }
                                    form class="inline-review" method="post" action=(format!("/evidence/{}/review", item.id)) {
                                        select name="result" aria-label="審查結果" { option value="accepted" { "確認有效" } option value="changes_requested" { "退回補充" } }
                                        input type="text" name="actor" aria-label="審查人姓名" placeholder="審查人" required maxlength="80";
                                        input type="text" name="note" aria-label="審查備註或退回說明" placeholder="備註；退回必填" maxlength="1000";
                                        button class="button button-small" type="submit" { "記錄" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn workflows_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 流程範本" } h1 { "可配置流程範本" } p class="page-subtitle" { "以步驟清單快速建立部門流程；每個草稿必須經流程負責人審查才可發布。" } } }
        (alert(error))
        div class="notice notice-blue" { strong { "AI 連線尚未配置" } span { "本原型不會把需求傳給外部模型。請以人工整理的草稿建立範本；AI 服務需經資料與供應者審查後再接入。" } }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "建立流程草稿" } p { "先定義流程目的、負責人與可驗證階段。" } } }
                form class="form-stack" method="post" action="/workflows" {
                    label class="field" { span { "基於既有範本（選填）" } select name="base_id" { option value="" { "建立新範本" } @for workflow in latest_workflows(&data.workflows) { option value=(workflow.id) { (workflow.id) " · " (workflow.title) " v" (workflow.version) } } } }
                    (text_field("title", "流程名稱", "", true, ""))
                    (text_field("department", "適用部門／跨部門範圍", "", true, ""))
                    (text_field("owner", "流程負責人", "", true, ""))
                    label class="field" { span { "流程目的" } textarea name="description" required maxlength="1500" rows="3" placeholder="這條流程要解決的業務問題與完成條件" {} }
                    label class="field" { span { "流程階段" } textarea name="stages" required rows="7" placeholder="每行一個階段，例如：\n提出需求\n風險評估\n審查核准\n執行與驗證\n結案封存" {} }
                    small class="field-help" { "每行一個階段，最多 30 階段。五個強制關卡在生命週期頁面管理。" }
                    (text_field("actor", "草稿建立人", "尚未驗證身份", true, ""))
                    button class="button button-primary" type="submit" { "儲存為草稿" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "已登錄範本" } p { (data.workflows.len()) " 個版本" } } }
                (workflow_list(data))
            }
        }
    }
}

fn workflow_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.workflows.is_empty() { (empty_state("尚無流程範本", "建立一份草稿並由流程負責人審查發布。", "/workflows", "建立流程草稿")) }
        @else {
            div class="workflow-list" {
                @for workflow in latest_workflows(&data.workflows) {
                    article class="workflow-card" {
                        div class="workflow-title-row" { div { p class="row-id" { (workflow.id) " · v" (workflow.version) } h3 { (workflow.title) } } (workflow_badge(&workflow.status)) }
                        p class="muted" { (workflow.department) " · 負責人 " (workflow.owner) }
                        p { (workflow.description) }
                        ol class="stage-chips" { @for stage in &workflow.stages { li { (stage) } } }
                        @if workflow.status == "draft" {
                            form class="publish-form" method="post" action=(format!("/workflows/{}/{}/publish", workflow.id, workflow.version)) {
                                label class="compact-field" { span { "流程負責人審查者" } input type="text" name="actor" required maxlength="80" placeholder="姓名"; }
                                button class="button button-small button-secondary" type="submit" { "核准並發布此版本" }
                            }
                        } @else {
                            p class="success-line" { "已由 " (workflow.published_by.as_deref().unwrap_or("未記錄")) " 於 " (format_timestamp(workflow.published_at.unwrap_or(workflow.created_at))) " 發布" }
                        }
                    }
                }
            }
        }
    }
}

fn latest_workflows(workflows: &[Workflow]) -> Vec<&Workflow> {
    let mut latest = std::collections::BTreeMap::<&str, &Workflow>::new();
    for workflow in workflows {
        match latest.get(workflow.id.as_str()) {
            Some(current) if current.version >= workflow.version => {}
            _ => {
                latest.insert(workflow.id.as_str(), workflow);
            }
        }
    }
    latest.into_values().collect()
}

fn controls_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 品質閉環" } h1 { "風險與品質事件" } p class="page-subtitle" { "集中追蹤風險、變更、缺失及矯正措施，保留責任人、期限和處理紀錄。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再登錄品質事件。"))) }
        section class="content-grid split-grid" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "新增事件" } p { "CAPA 關閉時須附複驗或有效性紀錄。" } } }
                form class="form-stack" method="post" action="/controls" {
                    label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                    label class="field" { span { "事件類型" } select name="kind" required { option value="risk" { "風險" } option value="change" { "變更" } option value="finding" { "缺失／問題" } option value="capa" { "CAPA 矯正預防" } } }
                    (text_field("title", "標題", "", true, ""))
                    label class="field" { span { "內容與影響" } textarea name="description" required maxlength="2000" rows="4" {} }
                    (text_field("owner", "負責人", "", true, ""))
                    label class="field" { span { "到期日" } input type="date" name="due_date" required; }
                    (text_field("actor", "登記人", "尚未驗證身份", true, ""))
                    button class="button button-primary" type="submit" { "新增追蹤事項" }
                }
            }
            div class="panel panel-wide" {
                div class="panel-heading" { div { h2 { "追蹤清單" } p { (data.cases.len()) " 項風險與品質事件" } } }
                (quality_case_list(data))
            }
        }
    }
}

fn inspections_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "工作區 / 品質保證" } h1 { "29 項品質檢查" } p class="page-subtitle" { "逐項記錄判定與可追溯依據；不適用項目需說明，不符合項目需連到缺失。" } } }
        (alert(error))
        @if data.projects.is_empty() { (alert(Some("請先建立專案，再記錄品質檢查。"))) }
        section class="panel" {
            div class="panel-heading" { div { h2 { "建立檢查紀錄" } p { "符合須填證據參照；不符合須填說明及缺失編號；不適用須填理由。" } } }
            @if data.projects.is_empty() { p class="empty-inline" { "目前沒有可選專案。" } }
            @else {
                form class="form-stack" method="post" action="/inspections" {
                    div class="inspection-meta" {
                        label class="field" { span { "關聯專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                        label class="field" { span { "檢查人姓名" } input type="text" name="reviewer" required maxlength="80" placeholder="尚未驗證身份"; }
                    }
                    div class="notice notice-blue" { strong { "判定規則" } span { "符合：填可查證的證據 ID／連結；不符合：填說明及缺失編號；不適用：填理由。此原型的姓名欄僅作登記，沒有身份驗證。" } }
                    div class="table-wrap" {
                        table class="inspection-table" {
                            caption class="visually-hidden" { "29 項專案品質檢查表" }
                            thead { tr { th scope="col" { "檢查項目" } th scope="col" { "判定" } th scope="col" { "證據參照" } th scope="col" { "說明／不適用理由" } th scope="col" { "缺失編號" } } }
                            tbody { @for item in QUALITY_CHECKLIST {
                                @let result_name = format!("result_{}", item.id);
                                @let evidence_name = format!("evidence_{}", item.id);
                                @let note_name = format!("note_{}", item.id);
                                @let issue_name = format!("issue_{}", item.id);
                                tr {
                                    th scope="row" { small class="row-id" { (item.id) " · " (item.category) } strong class="check-criterion" { (item.criterion) } }
                                    td { select name=(result_name) required aria-label=(format!("{} 判定", item.id)) { option value="" selected disabled { "選擇" } option value="pass" { "符合" } option value="fail" { "不符合" } option value="not_applicable" { "不適用" } } }
                                    td { input type="text" name=(evidence_name) maxlength="2048" aria-label=(format!("{} 證據參照", item.id)) placeholder="證據 ID／連結"; }
                                    td { input type="text" name=(note_name) maxlength="1000" aria-label=(format!("{} 說明", item.id)) placeholder="說明／理由"; }
                                    td { input type="text" name=(issue_name) maxlength="80" aria-label=(format!("{} 缺失編號", item.id)) placeholder="不符合時必填"; }
                                }
                            } }
                        }
                    }
                    button class="button button-primary" type="submit" { "保存完整檢查紀錄" }
                }
            }
        }
        section class="panel inspection-history" {
            div class="panel-heading" { div { h2 { "已保存檢查" } p { (data.inspections.len()) " 份完整檢查紀錄" } } }
            @if data.inspections.is_empty() { p class="empty-inline" { "尚無檢查紀錄；每份紀錄都必須包含全部 29 個項目。" } }
            @else { div class="table-wrap" { table {
                caption class="visually-hidden" { "已保存品質檢查紀錄" }
                thead { tr { th scope="col" { "檢查識別碼" } th scope="col" { "專案" } th scope="col" { "檢查人" } th scope="col" { "符合" } th scope="col" { "不符合" } th scope="col" { "不適用" } th scope="col" { "日期" } th scope="col" { "逐項結果" } } }
                tbody { @for inspection in data.inspections.iter().rev() {
                    @let passed = inspection.results.iter().filter(|item| item.result == "pass").count();
                    @let failed = inspection.results.iter().filter(|item| item.result == "fail").count();
                    @let not_applicable = inspection.results.iter().filter(|item| item.result == "not_applicable").count();
                    tr {
                        th scope="row" { (inspection.id) }
                        td { (inspection.project_id) }
                        td { (inspection.reviewer) }
                        td { (passed) }
                        td { (failed) }
                        td { (not_applicable) }
                        td { (format_timestamp(inspection.created_at)) }
                        td {
                            details class="inspection-details" {
                                summary { "查看 29 項" }
                                ol class="inspection-result-list" {
                                    @for result in &inspection.results {
                                        @let definition = QUALITY_CHECKLIST.iter().find(|item| item.id == result.item_id);
                                        li {
                                            div class="inspection-result-heading" { strong { (result.item_id) } (check_result_badge(&result.result)) }
                                            @if let Some(definition) = definition { p { (definition.criterion) } }
                                            @if !result.evidence_reference.is_empty() { small { "證據：" (result.evidence_reference) } }
                                            @if !result.note.is_empty() { small { "說明：" (result.note) } }
                                            @if !result.issue_id.is_empty() { small { "缺失：" (result.issue_id) } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } }
            } } }
        }
    }
}

fn quality_case_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.cases.is_empty() { (empty_state("尚無追蹤事項", "新增風險、變更、缺失或 CAPA，並指定負責人與期限。", "/controls", "新增事項")) }
        @else {
            div class="case-list" {
                @for case in data.cases.iter().rev() {
                    article class="case-card" {
                        div class="workflow-title-row" { div { p class="row-id" { (case.id) " · " (kind_label(&case.kind)) " · " (case.project_id) } h3 { (case.title) } } (case_status_badge(&case.status)) }
                        p { (case.description) }
                        p class="muted" { "負責人 " (case.owner) " · 到期 " (case.due_date) }
                        @if case.status != "closed" {
                            form class="publish-form" method="post" action=(format!("/controls/{}/status", case.id)) {
                                label class="compact-field" { span { "更新狀態" } select name="status" { option value="open" selected[case.status == "open"] { "待處理" } option value="in_progress" selected[case.status == "in_progress"] { "處理中" } option value="verification" selected[case.status == "verification"] { "待複驗" } option value="closed" { "結案" } } }
                                label class="compact-field" { span { "處理／結案紀錄" } input type="text" name="note" maxlength="1200" required placeholder="複驗結果或更新說明"; }
                                label class="compact-field" { span { "更新人" } input type="text" name="actor" maxlength="80" required placeholder="姓名"; }
                                button class="button button-small button-secondary" type="submit" { "記錄更新" }
                            }
                        } @else if let Some(note) = &case.closure_note { p class="success-line" { "結案紀錄：" (note) } }
                    }
                }
            }
        }
    }
}

fn lifecycle_content(data: &GovernanceSnapshot, error: Option<&str>) -> Markup {
    let stages = [
        "專案立案與分類",
        "需求與合規分析",
        "計畫與基準建立",
        "文件與系統設定",
        "執行與過程管制",
        "品質檢查與驗證",
        "變更與影響分析",
        "缺失與風險閉環",
        "階段審查與放行",
        "最終驗收與移交",
        "結案與封存",
        "效益追蹤與改善",
    ];
    let gates = [
        ("G1", "立案", "目的、贊助人、範圍與風險分級已確認。"),
        ("G2", "基準", "需求、驗收標準、計畫及責任已確認。"),
        ("G3", "執行", "關鍵品質檢查通過，或缺失已正式登錄。"),
        ("G4", "交付", "必要證據、適用要求與驗收條件已滿足。"),
        ("G5", "結案", "移交、未結責任、封存及改善計畫已完成。"),
    ];
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理模型" } h1 { "生命週期與強制關卡" } p class="page-subtitle" { "每階段定義責任與放行條件；關卡決定由授權人員記錄。" } } }
        (alert(error))
        section class="panel" {
            div class="panel-heading" { div { h2 { "12 個專案階段" } p { "來源：使用者提供的治理藍圖；試點時由流程擁有者確認適用性。" } } }
            ol class="lifecycle-grid" { @for (index, stage) in stages.iter().enumerate() { li { span class="stage-number" { (format!("{:02}", index + 1)) } strong { (stage) } } } }
        }
        section class="gate-layout" {
            div class="panel" {
                div class="panel-heading" { div { h2 { "五個強制關卡" } p { "例外需有風險接受者、補救措施和期限。" } } }
                div class="gate-list" { @for (id, name, condition) in gates { article class="gate-item" { span class="gate-id" { (id) } div { strong { (name) } p { (condition) } } } } }
                p class="form-hint" { "法律、安全及其他不可豁免要求不可一般例外放行。原型要求登記人明確聲明是否涉及不可豁免事項。" }
            }
            div class="panel" {
                div class="panel-heading" { div { h2 { "記錄人工關卡決議" } p { "僅記錄人工決定，不自動推定通過。" } } }
                @if data.projects.is_empty() { p class="empty-inline" { "先建立專案後才能記錄決議。" } }
                @else {
                    form class="form-stack" method="post" action="/lifecycle" {
                        label class="field" { span { "專案" } select name="project_id" required { option value="" selected disabled { "選擇專案" } @for project in &data.projects { option value=(project.id) { (project.id) " · " (project.name) } } } }
                        label class="field" { span { "關卡" } select name="gate_id" required { @for (id, name, _) in gates { option value=(id) { (id) " · " (name) } } } }
                        label class="field" { span { "人工決議" } select name="decision" required { option value="hold" { "暫緩" } option value="go" { "放行" } option value="exception" { "一般例外" } } }
                        label class="field" { span { "是否涉及不可豁免要求" } select name="non_waivable" required { option value="yes" { "是" } option value="no" { "否" } } }
                        label class="field" { span { "原因／例外依據" } textarea name="reason" rows="2" maxlength="1200" placeholder="暫緩或例外時必填" {} }
                        (text_field("evidence_reference", "證據參照（選填）", "來源系統連結或證據 ID", false, ""))
                        (text_field("risk_acceptor", "風險接受者（例外必填）", "", false, ""))
                        label class="field" { span { "補救期限（例外必填）" } input type="date" name="remediation_due"; }
                        (text_field("actor", "決議記錄人", "尚未驗證身份", true, ""))
                        button class="button button-primary" type="submit" { "保存決議" }
                    }
                }
            }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "決議歷程" } p { (data.gate_decisions.len()) " 筆人工關卡記錄" } } }
            (gate_decision_list(data))
        }
    }
}

fn gate_decision_list(data: &GovernanceSnapshot) -> Markup {
    html! {
        @if data.gate_decisions.is_empty() { p class="empty-inline" { "尚無關卡決議。建立專案後，保存實際人工決策。" } }
        @else {
            div class="table-wrap" { table {
                caption class="visually-hidden" { "關卡決議歷程" }
                thead { tr { th scope="col" { "專案／關卡" } th scope="col" { "決議" } th scope="col" { "記錄人" } th scope="col" { "理由／補救" } th scope="col" { "時間" } } }
                tbody { @for decision in data.gate_decisions.iter().rev() { tr {
                    th scope="row" { (decision.project_id) " · " (decision.gate_id) }
                    td { (gate_badge(&decision.decision)) }
                    td { (decision.actor) }
                    td { (decision.reason) @if decision.decision == "exception" { small class="row-id" { "風險接受者：" (decision.risk_acceptor) "；期限：" (decision.remediation_due) } } }
                    td { (format_timestamp(decision.created_at)) }
                } } }
            } }
        }
    }
}

fn audit_content(data: &GovernanceSnapshot, integrity: bool) -> Markup {
    html! {
        section class="page-heading" { div { p class="eyebrow" { "治理控制" } h1 { "追加式稽核紀錄" } p class="page-subtitle" { "每次新增、審查、發布與決議都新增事件，不覆寫先前事件。" } } }
        div class={(if integrity { "notice notice-green" } else { "notice notice-red" })} role="status" {
            strong { @if integrity { "本機雜湊鏈驗證通過" } @else { "稽核鏈驗證失敗" } }
            span { "雜湊鏈能偵測部分竄改；它不等同不可變 WORM 儲存或防竄改企業稽核服務。" }
        }
        section class="panel" {
            div class="panel-heading" { div { h2 { "事件明細" } p { (data.events.len()) " 筆事件 · 由新至舊" } } }
            @if data.events.is_empty() { (empty_state("尚無稽核事件", "新增專案或治理記錄後，事件會出現在此處。", "/projects", "新增專案")) }
            @else {
                div class="audit-timeline" {
                    @for event in data.events.iter().rev() {
                        article class="audit-event" {
                            span class="audit-sequence" { (format!("{:04}", event.sequence)) }
                            div class="audit-event-body" {
                                div class="workflow-title-row" { strong { (event_label(&event.payload)) } small { (event.actor) " · " (format_timestamp(event.timestamp_unix)) } }
                                p class="row-id" { "前序雜湊 " (event.previous_sha256) }
                                p class="hash-value" { "SHA-256 " (event.sha256) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn recent_events(events: &[crate::governance::LedgerEvent], limit: usize) -> Markup {
    html! {
        @if events.is_empty() { p class="empty-inline" { "尚無活動記錄。建立專案即可開始試用治理流程。" } }
        @else {
            ul class="activity-list" { @for event in events.iter().rev().take(limit) { li { span class="activity-marker" aria-hidden="true" {} div { strong { (event_label(&event.payload)) } small { (event.actor) " · " (format_timestamp(event.timestamp_unix)) } } } } }
        }
    }
}

fn alert(message: Option<&str>) -> Markup {
    html! { @if let Some(message) = message { div class="alert alert-error" role="alert" { (message) } } }
}

fn text_field(name: &str, label: &str, placeholder: &str, required: bool, value: &str) -> Markup {
    html! {
        label class="field" { span { (label) } input type="text" name=(name) value=(value) placeholder=(placeholder) maxlength="2048" required[required]; }
    }
}

fn empty_state(title: &str, description: &str, href: &str, action: &str) -> Markup {
    html! { div class="empty-state" { span class="empty-icon" aria-hidden="true" { "＋" } h3 { (title) } p { (description) } a class="text-link" href=(href) { (action) " →" } } }
}

fn risk_badge(value: &str) -> Markup {
    let (class, text) = match value {
        "low" => ("badge badge-green", "低"),
        "high" => ("badge badge-amber", "高"),
        "critical" => ("badge badge-red", "重大"),
        _ => ("badge badge-blue", "中"),
    };
    html! { span class=(class) { (text) } }
}

fn review_badge(value: &str) -> Markup {
    match value {
        "accepted" => html! { span class="badge badge-green" { "已確認" } },
        _ => html! { span class="badge badge-amber" { "退回補充" } },
    }
}

fn workflow_badge(value: &str) -> Markup {
    if value == "published" {
        html! { span class="badge badge-green" { "已發布" } }
    } else {
        html! { span class="badge badge-amber" { "草稿待審" } }
    }
}

fn case_status_badge(value: &str) -> Markup {
    let (class, label) = match value {
        "in_progress" => ("badge badge-blue", "處理中"),
        "verification" => ("badge badge-amber", "待複驗"),
        "closed" => ("badge badge-green", "已結案"),
        _ => ("badge badge-red", "待處理"),
    };
    html! { span class=(class) { (label) } }
}

fn gate_badge(value: &str) -> Markup {
    let (class, label) = match value {
        "go" => ("badge badge-green", "放行"),
        "exception" => ("badge badge-amber", "例外"),
        _ => ("badge badge-red", "暫緩"),
    };
    html! { span class=(class) { (label) } }
}

fn status_badge(_key: &str, label: &str) -> Markup {
    html! { span class="badge badge-amber" { (label) } }
}

fn check_result_badge(value: &str) -> Markup {
    match value {
        "pass" => html! { span class="badge badge-green" { "符合" } },
        "fail" => html! { span class="badge badge-red" { "不符合" } },
        _ => html! { span class="badge badge-blue" { "不適用" } },
    }
}

fn kind_label(value: &str) -> &'static str {
    match value {
        "risk" => "風險",
        "change" => "變更",
        "capa" => "CAPA",
        _ => "缺失",
    }
}

fn format_timestamp(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let remainder = seconds % 86_400;
    let hour = remainder / 3_600;
    let minute = (remainder % 3_600) / 60;
    let second = remainder % 60;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

fn event_label(payload: &crate::governance::EventPayload) -> String {
    use crate::governance::EventPayload;
    match payload {
        EventPayload::ProjectCreated(project) => format!("建立專案：{}", project.name),
        EventPayload::EvidenceLinked(item) => format!("連結證據：{}", item.title),
        EventPayload::EvidenceReviewed(review) => format!(
            "審查證據：{} · {} · {}",
            review.evidence_id,
            review_label_text(&review.result),
            review.note
        ),
        EventPayload::WorkflowSaved(workflow) => {
            format!("儲存流程草稿：{} v{}", workflow.title, workflow.version)
        }
        EventPayload::WorkflowPublished(publication) => {
            format!("發布流程：{} v{}", publication.id, publication.version)
        }
        EventPayload::QualityCaseCreated(case) => {
            format!("新增{}：{}", kind_label(&case.kind), case.title)
        }
        EventPayload::QualityCaseUpdated(update) => format!(
            "更新品質事件：{} → {} · {}",
            update.id,
            case_status_label(&update.status),
            update.note
        ),
        EventPayload::QualityInspectionRecorded(inspection) => {
            let failed = inspection
                .results
                .iter()
                .filter(|item| item.result == "fail")
                .count();
            format!(
                "完成品質檢查：{} · {} 項檢查 · {} 項不符合",
                inspection.project_id,
                inspection.results.len(),
                failed
            )
        }
        EventPayload::GateDecisionRecorded(decision) => format!(
            "{} 決議：{} · {}",
            decision.gate_id,
            gate_decision_label(&decision.decision),
            decision.reason
        ),
    }
}

fn case_status_label(value: &str) -> &'static str {
    match value {
        "in_progress" => "處理中",
        "verification" => "待複驗",
        "closed" => "結案",
        _ => "待處理",
    }
}

fn gate_decision_label(value: &str) -> &'static str {
    match value {
        "go" => "放行",
        "exception" => "一般例外",
        _ => "暫緩",
    }
}

fn review_label_text(value: &str) -> &'static str {
    match value {
        "accepted" => "已確認",
        _ => "退回補充",
    }
}

#[derive(Deserialize)]
struct ProjectForm {
    name: String,
    department: String,
    owner: String,
    risk_tier: String,
    actor: String,
}
#[derive(Deserialize)]
struct EvidenceForm {
    project_id: String,
    title: String,
    source_system: String,
    source_url: String,
    source_version: String,
    evidence_type: String,
    owner: String,
    actor: String,
}
#[derive(Deserialize)]
struct ReviewForm {
    result: String,
    actor: String,
    note: String,
}
#[derive(Deserialize)]
struct WorkflowForm {
    base_id: String,
    title: String,
    department: String,
    owner: String,
    description: String,
    stages: String,
    actor: String,
}
#[derive(Deserialize)]
struct PublishForm {
    actor: String,
}
#[derive(Deserialize)]
struct QualityCaseForm {
    project_id: String,
    kind: String,
    title: String,
    description: String,
    owner: String,
    due_date: String,
    actor: String,
}
#[derive(Deserialize)]
struct QualityCaseUpdateForm {
    status: String,
    actor: String,
    note: String,
}
#[derive(Deserialize)]
struct GateDecisionForm {
    project_id: String,
    gate_id: String,
    decision: String,
    actor: String,
    reason: String,
    evidence_reference: String,
    risk_acceptor: String,
    remediation_due: String,
    non_waivable: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_formatter_is_utc_and_stable() {
        assert_eq!(format_timestamp(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_timestamp(1_798_000_000), "2026-12-23 04:26:40 UTC");
    }

    #[test]
    fn rendered_shell_uses_semantic_accessible_navigation_without_scripts() {
        let rendered = layout("測試", "overview", html! { main { "content" } }).into_string();
        assert!(rendered.contains("lang=\"zh-Hant\""));
        assert!(rendered.contains("跳至主要內容"));
        assert!(rendered.contains("aria-label=\"主要導覽\""));
        assert!(!rendered.contains("<script"));
    }

    #[test]
    fn latest_workflow_version_wins() {
        let old = Workflow {
            id: "TPL-1".to_string(),
            version: 1,
            title: "流程".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            description: "用途".to_string(),
            stages: vec!["提出".to_string()],
            status: "published".to_string(),
            created_at: 1,
            published_at: Some(1),
            published_by: Some("Alex".to_string()),
        };
        let new = Workflow {
            version: 2,
            status: "draft".to_string(),
            ..old.clone()
        };
        let versions = [old, new];
        let latest = latest_workflows(&versions);
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].version, 2);
    }
}
