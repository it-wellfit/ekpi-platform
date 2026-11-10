// C6 同位性紅報告（refactor-criteria C6）：`platform::governance::EventPayload`
// 與 `ekp_domain::governance::EventPayload` 是兩個獨立的同名 enum（S11 發現）。
// 本測試以**邏輯相同**的承載在兩個型別間比對 serde 序列化輸出——
// 預期**紅**（欄位差異實測），作為合併前置的量化證據。
//
// 執行方式：`cargo test -p ekp_platform --test c6_parity -- --ignored --nocapture`
// 合併實作完成後：移除 `#[ignore]`，測試轉綠即為合併安全的證明。
//
// 為何 `#[ignore]` 而非常規失敗：canonical 門檻要求全綠；預期紅的測試
// 若常規執行會破壞「測試全綠」的機械判定。ignore 標注讓差異報告
// 可按需產生，同時不污染 canonical。

use ekp_domain::governance as domain_gov;
use serde_json::Value;

/// 從 platform 側建構樣本 payload（與 jsonl_ledger apply 邏輯同構）。
fn platform_samples() -> Vec<(String, Value)> {
    let mut samples = Vec::new();
    let project = ekp_platform::governance::Project {
        id: "PRJ-000001".into(),
        name: "parity".into(),
        department: "PMO".into(),
        owner: "tester".into(),
        risk_tier: "low".into(),
        created_at: 1_000_000,
    };
    samples.push((
        "project_created".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::ProjectCreated(
            project,
        ))
        .expect("serialize"),
    ));
    let evidence = ekp_platform::governance::Evidence {
        id: "EVD-000001".into(),
        project_id: "PRJ-000001".into(),
        title: "parity".into(),
        source_system: "drill".into(),
        source_url: "http://example/parity".into(),
        source_version: "1".into(),
        evidence_type: "doc".into(),
        owner: "tester".into(),
        created_at: 1_000_001,
    };
    samples.push((
        "evidence_linked".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::EvidenceLinked(
            evidence,
        ))
        .expect("serialize"),
    ));
    let review = ekp_platform::governance::EvidenceReview {
        evidence_id: "EVD-000001".into(),
        result: "accepted".into(),
        reviewer: "tester".into(),
        note: String::new(),
        reviewed_at: 1_000_002,
    };
    samples.push((
        "evidence_reviewed".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::EvidenceReviewed(
            review,
        ))
        .expect("serialize"),
    ));
    let workflow = ekp_platform::governance::Workflow {
        id: "WF-000001".into(),
        version: 1,
        title: "parity".into(),
        department: "PMO".into(),
        owner: "tester".into(),
        description: String::new(),
        stages: vec!["s1".into()],
        status: "draft".into(),
        created_at: 1_000_003,
        published_at: None,
        published_by: None,
    };
    samples.push((
        "workflow_saved".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::WorkflowSaved(
            workflow,
        ))
        .expect("serialize"),
    ));
    samples.push((
        "workflow_published".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::WorkflowPublished(
            ekp_platform::governance::WorkflowPublication {
                id: "WF-000001".into(),
                version: 1,
                reviewer: "tester".into(),
                published_at: 1_000_004,
            },
        ))
        .expect("serialize"),
    ));
    let case = ekp_platform::governance::QualityCase {
        id: "QMS-000001".into(),
        project_id: "PRJ-000001".into(),
        kind: "risk".into(),
        title: "parity".into(),
        description: String::new(),
        owner: "tester".into(),
        due_date: "2026-12-31".into(),
        status: "open".into(),
        created_at: 1_000_005,
        updated_at: 1_000_005,
        closure_note: None,
    };
    samples.push((
        "quality_case_created".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::QualityCaseCreated(
            case,
        ))
        .expect("serialize"),
    ));
    samples.push((
        "quality_case_updated".into(),
        serde_json::to_value(&ekp_platform::governance::EventPayload::QualityCaseUpdated(
            ekp_platform::governance::QualityCaseUpdate {
                id: "QMS-000001".into(),
                status: "in_progress".into(),
                actor: "tester".into(),
                note: String::new(),
                updated_at: 1_000_006,
            },
        ))
        .expect("serialize"),
    ));
    samples.push((
        "quality_inspection_recorded".into(),
        serde_json::to_value(
            &ekp_platform::governance::EventPayload::QualityInspectionRecorded(
                ekp_platform::governance::QualityInspection {
                    id: "INS-000001".into(),
                    project_id: "PRJ-000001".into(),
                    reviewer: "tester".into(),
                    created_at: 1_000_007,
                    results: vec![],
                },
            ),
        )
        .expect("serialize"),
    ));
    samples.push((
        "gate_decision_recorded".into(),
        serde_json::to_value(
            &ekp_platform::governance::EventPayload::GateDecisionRecorded(
                ekp_platform::governance::GateDecision {
                    id: "GATE-000001".into(),
                    project_id: "PRJ-000001".into(),
                    gate_id: "G1".into(),
                    decision: "go".into(),
                    actor: "tester".into(),
                    reason: String::new(),
                    evidence_reference: String::new(),
                    risk_acceptor: String::new(),
                    remediation_due: String::new(),
                    non_waivable: false,
                    created_at: 1_000_008,
                },
            ),
        )
        .expect("serialize"),
    ));
    samples
}

/// 從 domain 側建構**邏輯相同**的樣本。欄位名與型別以 domain/governance.rs
/// 為準；兩側差異即本測試的報告內容。
fn domain_samples() -> Vec<(String, Value)> {
    let mut samples = Vec::new();
    let project = domain_gov::Project {
        id: "PRJ-000001".into(),
        name: "parity".into(),
        department: "PMO".into(),
        owner: "tester".into(),
        risk_tier: "low".into(),
        created_at: 1_000_000,
    };
    samples.push((
        "project_created".into(),
        serde_json::to_value(&domain_gov::EventPayload::ProjectCreated(project))
            .expect("serialize"),
    ));
    let evidence = domain_gov::Evidence {
        id: "EVD-000001".into(),
        project_id: "PRJ-000001".into(),
        title: "parity".into(),
        source_system: "drill".into(),
        source_url: "http://example/parity".into(),
        source_version: "1".into(),
        evidence_type: "doc".into(),
        owner: "tester".into(),
        created_at: 1_000_001,
    };
    samples.push((
        "evidence_linked".into(),
        serde_json::to_value(&domain_gov::EventPayload::EvidenceLinked(evidence))
            .expect("serialize"),
    ));
    let review = domain_gov::EvidenceReview {
        evidence_id: "EVD-000001".into(),
        result: "accepted".into(),
        reviewer: "tester".into(),
        note: String::new(),
        reviewed_at: 1_000_002,
    };
    samples.push((
        "evidence_reviewed".into(),
        serde_json::to_value(&domain_gov::EventPayload::EvidenceReviewed(review))
            .expect("serialize"),
    ));
    let workflow = domain_gov::Workflow {
        id: "WF-000001".into(),
        version: 1,
        title: "parity".into(),
        department: "PMO".into(),
        owner: "tester".into(),
        description: String::new(),
        stages: vec!["s1".into()],
        status: "draft".into(),
        created_at: 1_000_003,
        published_at: None,
        published_by: None,
    };
    samples.push((
        "workflow_saved".into(),
        serde_json::to_value(&domain_gov::EventPayload::WorkflowSaved(workflow))
            .expect("serialize"),
    ));
    samples.push((
        "workflow_published".into(),
        serde_json::to_value(&domain_gov::EventPayload::WorkflowPublished(
            domain_gov::WorkflowPublication {
                id: "WF-000001".into(),
                version: 1,
                reviewer: "tester".into(),
                published_at: 1_000_004,
            },
        ))
        .expect("serialize"),
    ));
    let case = domain_gov::QualityCase {
        id: "QMS-000001".into(),
        project_id: "PRJ-000001".into(),
        kind: "risk".into(),
        title: "parity".into(),
        description: String::new(),
        owner: "tester".into(),
        due_date: "2026-12-31".into(),
        status: "open".into(),
        created_at: 1_000_005,
        updated_at: 1_000_005,
        closure_note: None,
    };
    samples.push((
        "quality_case_created".into(),
        serde_json::to_value(&domain_gov::EventPayload::QualityCaseCreated(case))
            .expect("serialize"),
    ));
    samples.push((
        "quality_case_updated".into(),
        serde_json::to_value(&domain_gov::EventPayload::QualityCaseUpdated(
            domain_gov::QualityCaseUpdate {
                id: "QMS-000001".into(),
                status: "in_progress".into(),
                actor: "tester".into(),
                note: String::new(),
                updated_at: 1_000_006,
            },
        ))
        .expect("serialize"),
    ));
    samples.push((
        "quality_inspection_recorded".into(),
        serde_json::to_value(&domain_gov::EventPayload::QualityInspectionRecorded(
            domain_gov::QualityInspection {
                id: "INS-000001".into(),
                project_id: "PRJ-000001".into(),
                reviewer: "tester".into(),
                created_at: 1_000_007,
                results: vec![],
            },
        ))
        .expect("serialize"),
    ));
    samples.push((
        "gate_decision_recorded".into(),
        serde_json::to_value(&domain_gov::EventPayload::GateDecisionRecorded(
            domain_gov::GateDecision {
                id: "GATE-000001".into(),
                project_id: "PRJ-000001".into(),
                gate_id: "G1".into(),
                decision: "go".into(),
                actor: "tester".into(),
                reason: String::new(),
                evidence_reference: String::new(),
                risk_acceptor: String::new(),
                remediation_due: String::new(),
                non_waivable: false,
                created_at: 1_000_008,
            },
        ))
        .expect("serialize"),
    ));
    samples
}

/// 同位性主測試：九種事件逐一比對 platform 與 domain 的 serde 輸出。
/// 預期紅（兩 enum 獨立定義，欄位差異實測）；合併後轉綠即為合併安全證明。
#[test]
#[ignore = "C6 parity: expected-red until the dual EventPayload consolidation (refactor-criteria C6); run with --ignored to produce the diff report"]
fn event_payloads_parity_platform_vs_domain() {
    let platform = platform_samples();
    let domain = domain_samples();
    assert_eq!(
        platform.len(),
        domain.len(),
        "sample counts must match (9 governance event kinds)"
    );
    for ((p_tag, p_value), (d_tag, d_value)) in platform.iter().zip(domain.iter()) {
        assert_eq!(p_tag, d_tag, "event tags must align for {p_tag}");
        assert_eq!(
            p_value, d_value,
            "serde output differs between platform and domain for {p_tag}\nplatform: {p_value}\ndomain:   {d_value}"
        );
    }
}
