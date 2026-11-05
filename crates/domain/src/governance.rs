#![forbid(unsafe_code)]

//! Governance domain model and business rules.
//!
//! The types describe what a governance record *is*; the rules describe what
//! makes a record valid. Nothing here touches a database, a filesystem or an
//! HTTP framework, so the same rules apply to every storage adapter.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::hashing::sha256_hex;

pub const GENESIS_HASH: &str = "GENESIS";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventPayload {
    ProjectCreated(Project),
    EvidenceLinked(Evidence),
    EvidenceReviewed(EvidenceReview),
    WorkflowSaved(Workflow),
    WorkflowPublished(WorkflowPublication),
    QualityCaseCreated(QualityCase),
    QualityCaseUpdated(QualityCaseUpdate),
    QualityInspectionRecorded(QualityInspection),
    GateDecisionRecorded(GateDecision),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualityCheckDefinition {
    pub id: &'static str,
    pub category: &'static str,
    pub criterion: &'static str,
}

pub const QUALITY_CHECKLIST: [QualityCheckDefinition; 29] = [
    QualityCheckDefinition {
        id: "QC-01",
        category: "目標與需求",
        criterion: "專案目標、範圍與成功條件已核准",
    },
    QualityCheckDefinition {
        id: "QC-02",
        category: "目標與需求",
        criterion: "需求具有唯一識別碼及明確來源",
    },
    QualityCheckDefinition {
        id: "QC-03",
        category: "目標與需求",
        criterion: "需求有可驗證的驗收標準",
    },
    QualityCheckDefinition {
        id: "QC-04",
        category: "目標與需求",
        criterion: "需求變更已完成影響評估",
    },
    QualityCheckDefinition {
        id: "QC-05",
        category: "計畫與責任",
        criterion: "工作、交付物、責任人及期限已明確定義",
    },
    QualityCheckDefinition {
        id: "QC-06",
        category: "計畫與責任",
        criterion: "時程、資源、預算及依賴關係已審查",
    },
    QualityCheckDefinition {
        id: "QC-07",
        category: "計畫與責任",
        criterion: "品質計畫及檢查頻率已核准",
    },
    QualityCheckDefinition {
        id: "QC-08",
        category: "計畫與責任",
        criterion: "重大風險及應變措施已有責任人",
    },
    QualityCheckDefinition {
        id: "QC-09",
        category: "文件與證據",
        criterion: "文件編號、版本、狀態及核准資訊正確",
    },
    QualityCheckDefinition {
        id: "QC-10",
        category: "文件與證據",
        criterion: "使用中的文件為有效版本",
    },
    QualityCheckDefinition {
        id: "QC-11",
        category: "文件與證據",
        criterion: "必要附件、引用依據及簽核完整",
    },
    QualityCheckDefinition {
        id: "QC-12",
        category: "文件與證據",
        criterion: "文件內容與實際執行結果一致",
    },
    QualityCheckDefinition {
        id: "QC-13",
        category: "文件與證據",
        criterion: "證據可讀、可追溯且有適當來源",
    },
    QualityCheckDefinition {
        id: "QC-14",
        category: "文件與證據",
        criterion: "權限、保存期限及封存規則已設定",
    },
    QualityCheckDefinition {
        id: "QC-15",
        category: "執行與成果",
        criterion: "工作依核准的程序及基準執行",
    },
    QualityCheckDefinition {
        id: "QC-16",
        category: "執行與成果",
        criterion: "必要檢查、測試及獨立覆核已完成",
    },
    QualityCheckDefinition {
        id: "QC-17",
        category: "執行與成果",
        criterion: "數據與規格符合核准的驗收標準",
    },
    QualityCheckDefinition {
        id: "QC-18",
        category: "執行與成果",
        criterion: "關鍵交付物已對應需求及測試結果",
    },
    QualityCheckDefinition {
        id: "QC-19",
        category: "執行與成果",
        criterion: "偏差、例外與未達標項目均已登錄",
    },
    QualityCheckDefinition {
        id: "QC-20",
        category: "缺失與變更",
        criterion: "缺失已分級並指定責任人及期限",
    },
    QualityCheckDefinition {
        id: "QC-21",
        category: "缺失與變更",
        criterion: "重大問題已完成根因分析",
    },
    QualityCheckDefinition {
        id: "QC-22",
        category: "缺失與變更",
        criterion: "矯正措施已執行並有複驗證據",
    },
    QualityCheckDefinition {
        id: "QC-23",
        category: "缺失與變更",
        criterion: "必要的改善有效性已確認",
    },
    QualityCheckDefinition {
        id: "QC-24",
        category: "缺失與變更",
        criterion: "變更已核准並更新所有受影響文件",
    },
    QualityCheckDefinition {
        id: "QC-25",
        category: "交付與治理",
        criterion: "必要驗收條件已逐項確認",
    },
    QualityCheckDefinition {
        id: "QC-26",
        category: "交付與治理",
        criterion: "未結事項已明確記錄責任、期限及接受者",
    },
    QualityCheckDefinition {
        id: "QC-27",
        category: "交付與治理",
        criterion: "最終交付版本及移交內容正確",
    },
    QualityCheckDefinition {
        id: "QC-28",
        category: "交付與治理",
        criterion: "文件封存、存取權限與備份已驗證",
    },
    QualityCheckDefinition {
        id: "QC-29",
        category: "交付與治理",
        criterion: "經驗回饋、效益追蹤及改善計畫已安排",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerEvent {
    pub sequence: u64,
    pub timestamp_unix: u64,
    pub actor: String,
    pub payload: EventPayload,
    pub previous_sha256: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub department: String,
    pub owner: String,
    pub risk_tier: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evidence {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub source_system: String,
    pub source_url: String,
    pub source_version: String,
    pub evidence_type: String,
    pub owner: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceReview {
    pub evidence_id: String,
    pub result: String,
    pub reviewer: String,
    pub note: String,
    pub reviewed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Workflow {
    pub id: String,
    pub version: u32,
    pub title: String,
    pub department: String,
    pub owner: String,
    pub description: String,
    pub stages: Vec<String>,
    pub status: String,
    pub created_at: u64,
    pub published_at: Option<u64>,
    pub published_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowPublication {
    pub id: String,
    pub version: u32,
    pub reviewer: String,
    pub published_at: u64,
}

#[derive(Debug, Clone)]
pub struct WorkflowDraftInput {
    pub base_id: Option<String>,
    pub title: String,
    pub department: String,
    pub owner: String,
    pub description: String,
    pub stages: Vec<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityCase {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub owner: String,
    pub due_date: String,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub closure_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityCaseUpdate {
    pub id: String,
    pub status: String,
    pub actor: String,
    pub note: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityCheckResult {
    pub item_id: String,
    pub result: String,
    pub note: String,
    pub evidence_reference: String,
    pub issue_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityInspection {
    pub id: String,
    pub project_id: String,
    pub reviewer: String,
    pub created_at: u64,
    pub results: Vec<QualityCheckResult>,
}

#[derive(Debug, Clone)]
pub struct QualityInspectionInput {
    pub project_id: String,
    pub reviewer: String,
    pub results: Vec<QualityCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateDecision {
    pub id: String,
    pub project_id: String,
    pub gate_id: String,
    pub decision: String,
    pub actor: String,
    pub reason: String,
    pub evidence_reference: String,
    pub risk_acceptor: String,
    pub remediation_due: String,
    pub non_waivable: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GovernanceSnapshot {
    pub projects: Vec<Project>,
    pub evidence: Vec<Evidence>,
    pub reviews: BTreeMap<String, EvidenceReview>,
    pub workflows: Vec<Workflow>,
    pub cases: Vec<QualityCase>,
    pub inspections: Vec<QualityInspection>,
    pub gate_decisions: Vec<GateDecision>,
    pub events: Vec<LedgerEvent>,
}

#[derive(Serialize)]
struct EventBody<'a> {
    sequence: u64,
    timestamp_unix: u64,
    actor: &'a str,
    payload: &'a EventPayload,
    previous_sha256: &'a str,
}

pub fn previous_hash(snapshot: &GovernanceSnapshot) -> String {
    snapshot
        .events
        .last()
        .map(|event| event.sha256.clone())
        .unwrap_or_else(|| GENESIS_HASH.to_string())
}

pub fn apply_event(snapshot: &mut GovernanceSnapshot, event: &LedgerEvent) -> Result<(), String> {
    match &event.payload {
        EventPayload::ProjectCreated(project) => {
            validate_record_id("專案編號", &project.id)?;
            validate_text("專案名稱", &project.name, 120)?;
            validate_text("部門", &project.department, 80)?;
            validate_text("負責人", &project.owner, 80)?;
            if snapshot.projects.iter().any(|item| item.id == project.id) {
                return Err(format!("duplicate project id {}", project.id));
            }
            if !["low", "medium", "high", "critical"].contains(&project.risk_tier.as_str()) {
                return Err(format!("invalid risk tier for project {}", project.id));
            }
            snapshot.projects.push(project.clone());
        }
        EventPayload::EvidenceLinked(evidence) => {
            validate_record_id("證據編號", &evidence.id)?;
            validate_text("證據標題", &evidence.title, 160)?;
            validate_text("來源系統", &evidence.source_system, 80)?;
            validate_source_url(&evidence.source_url)?;
            validate_text("來源版本", &evidence.source_version, 80)?;
            validate_text("證據類型", &evidence.evidence_type, 80)?;
            validate_text("負責人", &evidence.owner, 80)?;
            if !snapshot
                .projects
                .iter()
                .any(|project| project.id == evidence.project_id)
            {
                return Err(format!(
                    "evidence {} references missing project {}",
                    evidence.id, evidence.project_id
                ));
            }
            if snapshot.evidence.iter().any(|item| item.id == evidence.id) {
                return Err(format!("duplicate evidence id {}", evidence.id));
            }
            snapshot.evidence.push(evidence.clone());
        }
        EventPayload::EvidenceReviewed(review) => {
            if !["accepted", "changes_requested"].contains(&review.result.as_str()) {
                return Err(format!(
                    "review of {} has an invalid result {}",
                    review.evidence_id, review.result
                ));
            }
            validate_actor(&review.reviewer)?;
            require_payload_actor(&event.actor, &review.reviewer, "evidence review")?;
            if review.result == "changes_requested" {
                validate_text("退回說明", &review.note, 1000)?;
            }
            if !review.note.trim().is_empty() {
                validate_text("審查備註", &review.note, 1000)?;
            }
            if !snapshot
                .evidence
                .iter()
                .any(|evidence| evidence.id == review.evidence_id)
            {
                return Err(format!(
                    "review references missing evidence {}",
                    review.evidence_id
                ));
            }
            snapshot
                .reviews
                .insert(review.evidence_id.clone(), review.clone());
        }
        EventPayload::WorkflowSaved(workflow) => {
            validate_record_id("流程編號", &workflow.id)?;
            if workflow.version == 0 {
                return Err(format!(
                    "workflow {} version 0 is invalid; use version 1 or later",
                    workflow.id
                ));
            }
            if workflow.status != "draft"
                || workflow.published_at.is_some()
                || workflow.published_by.is_some()
            {
                return Err(format!(
                    "workflow {}@{} must be saved as an unpublished draft",
                    workflow.id, workflow.version
                ));
            }
            validate_text("流程名稱", &workflow.title, 120)?;
            validate_text("部門", &workflow.department, 80)?;
            validate_text("流程負責人", &workflow.owner, 80)?;
            validate_multiline_text("流程目的", &workflow.description, 1500)?;
            if workflow.stages.is_empty() || workflow.stages.len() > 30 {
                return Err(format!(
                    "workflow {}@{} must have 1 to 30 stages",
                    workflow.id, workflow.version
                ));
            }
            for stage in &workflow.stages {
                validate_text("階段名稱", stage, 120)?;
            }
            if snapshot
                .workflows
                .iter()
                .any(|item| item.id == workflow.id && item.version == workflow.version)
            {
                return Err(format!(
                    "duplicate workflow version {}@{}",
                    workflow.id, workflow.version
                ));
            }
            if workflow.version == 1 {
                if snapshot.workflows.iter().any(|item| item.id == workflow.id) {
                    return Err(format!(
                        "workflow {} starts at version 1 after a later version exists",
                        workflow.id
                    ));
                }
            } else if !snapshot
                .workflows
                .iter()
                .any(|item| item.id == workflow.id && item.version == workflow.version - 1)
            {
                return Err(format!(
                    "workflow {}@{} has no predecessor version",
                    workflow.id, workflow.version
                ));
            }
            snapshot.workflows.push(workflow.clone());
        }
        EventPayload::WorkflowPublished(publication) => {
            validate_record_id("流程編號", &publication.id)?;
            validate_actor(&publication.reviewer)?;
            require_payload_actor(&event.actor, &publication.reviewer, "workflow publication")?;
            let workflow = snapshot
                .workflows
                .iter_mut()
                .find(|item| item.id == publication.id && item.version == publication.version)
                .ok_or_else(|| {
                    format!(
                        "publication references missing workflow {}@{}",
                        publication.id, publication.version
                    )
                })?;
            if workflow.status != "draft" {
                return Err(format!(
                    "workflow {}@{} is not a draft",
                    publication.id, publication.version
                ));
            }
            workflow.status = "published".to_string();
            workflow.published_at = Some(publication.published_at);
            workflow.published_by = Some(publication.reviewer.clone());
        }
        EventPayload::QualityCaseCreated(case) => {
            validate_record_id("品質事件編號", &case.id)?;
            if !["risk", "change", "finding", "capa"].contains(&case.kind.as_str()) {
                return Err(format!("invalid quality case kind {}", case.kind));
            }
            if case.status != "open" || case.closure_note.is_some() {
                return Err(format!(
                    "quality case {} must start open and unclosed",
                    case.id
                ));
            }
            validate_text("案件標題", &case.title, 160)?;
            validate_multiline_text("案件內容", &case.description, 2000)?;
            validate_text("負責人", &case.owner, 80)?;
            validate_date("期限", &case.due_date)?;
            if !snapshot
                .projects
                .iter()
                .any(|project| project.id == case.project_id)
            {
                return Err(format!(
                    "quality case {} references missing project {}",
                    case.id, case.project_id
                ));
            }
            if snapshot.cases.iter().any(|item| item.id == case.id) {
                return Err(format!("duplicate quality case id {}", case.id));
            }
            snapshot.cases.push(case.clone());
        }
        EventPayload::QualityCaseUpdated(update) => {
            validate_actor(&update.actor)?;
            require_payload_actor(&event.actor, &update.actor, "quality case update")?;
            if update.status == "closed" && update.note.trim().is_empty() {
                return Err(format!(
                    "quality case {} cannot close without a closure note",
                    update.id
                ));
            }
            if !update.note.trim().is_empty() {
                validate_text("處理備註", &update.note, 1200)?;
            }
            let case = snapshot
                .cases
                .iter_mut()
                .find(|item| item.id == update.id)
                .ok_or_else(|| format!("update references missing quality case {}", update.id))?;
            if !["open", "in_progress", "verification", "closed"].contains(&update.status.as_str())
            {
                return Err(format!("invalid quality case status {}", update.status));
            }
            if update.status == "closed" && case.kind == "capa" && case.status != "verification" {
                return Err(format!(
                    "CAPA {} cannot close before verification",
                    update.id
                ));
            }
            if !quality_case_transition_allowed(&case.status, &update.status) {
                return Err(format!(
                    "quality case {} cannot move from {} to {}",
                    update.id, case.status, update.status
                ));
            }
            case.status = update.status.clone();
            case.updated_at = update.updated_at;
            if update.status == "closed" {
                case.closure_note = Some(update.note.clone());
            }
        }
        EventPayload::QualityInspectionRecorded(inspection) => {
            validate_record_id("檢查編號", &inspection.id)?;
            validate_actor(&inspection.reviewer)?;
            require_payload_actor(&event.actor, &inspection.reviewer, "quality inspection")?;
            validate_inspection_results(&inspection.results)?;
            if !snapshot
                .projects
                .iter()
                .any(|project| project.id == inspection.project_id)
            {
                return Err(format!(
                    "inspection {} references missing project {}",
                    inspection.id, inspection.project_id
                ));
            }
            if snapshot
                .inspections
                .iter()
                .any(|item| item.id == inspection.id)
            {
                return Err(format!("duplicate inspection id {}", inspection.id));
            }
            snapshot.inspections.push(inspection.clone());
        }
        EventPayload::GateDecisionRecorded(decision) => {
            validate_record_id("關卡決議編號", &decision.id)?;
            validate_actor(&decision.actor)?;
            require_payload_actor(&event.actor, &decision.actor, "gate decision")?;
            if decision.decision != "go" {
                validate_multiline_text("決議原因", &decision.reason, 1200)?;
            }
            if decision.decision == "exception" {
                validate_text("風險接受者", &decision.risk_acceptor, 80)?;
                validate_date("補救期限", &decision.remediation_due)?;
            }
            if !decision.evidence_reference.trim().is_empty() {
                validate_text("證據參照", &decision.evidence_reference, 2048)?;
            }
            if !snapshot
                .projects
                .iter()
                .any(|project| project.id == decision.project_id)
            {
                return Err(format!(
                    "gate decision {} references missing project {}",
                    decision.id, decision.project_id
                ));
            }
            if !["G1", "G2", "G3", "G4", "G5"].contains(&decision.gate_id.as_str()) {
                return Err(format!("invalid gate id {}", decision.gate_id));
            }
            if !["go", "hold", "exception"].contains(&decision.decision.as_str()) {
                return Err(format!("invalid gate decision {}", decision.decision));
            }
            if decision.decision == "exception" && decision.non_waivable {
                return Err(format!(
                    "gate decision {} waives a non-waivable requirement",
                    decision.id
                ));
            }
            if snapshot
                .gate_decisions
                .iter()
                .any(|item| item.id == decision.id)
            {
                return Err(format!("duplicate gate decision id {}", decision.id));
            }
            snapshot.gate_decisions.push(decision.clone());
        }
    }
    Ok(())
}

pub fn validate_inspection_results(results: &[QualityCheckResult]) -> Result<(), String> {
    if results.len() != QUALITY_CHECKLIST.len() {
        return Err(format!(
            "品質檢查必須逐項判定全部 {} 個項目",
            QUALITY_CHECKLIST.len()
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for result in results {
        if !QUALITY_CHECKLIST
            .iter()
            .any(|item| item.id == result.item_id)
        {
            return Err(format!("未知的檢查項目 {}", result.item_id));
        }
        if !seen.insert(result.item_id.as_str()) {
            return Err(format!("檢查項目 {} 重複", result.item_id));
        }
        if !["pass", "fail", "not_applicable"].contains(&result.result.as_str()) {
            return Err(format!("{} 尚未選擇有效判定", result.item_id));
        }
        match result.result.as_str() {
            "pass" => validate_text("符合項目的證據參照", &result.evidence_reference, 2048)?,
            "fail" => {
                validate_text("不符合項目的說明", &result.note, 1000)?;
                validate_text("缺失編號", &result.issue_id, 80)?;
            }
            "not_applicable" => validate_text("不適用理由", &result.note, 1000)?,
            _ => unreachable!(),
        }
        if !result.note.trim().is_empty() {
            validate_text("檢查備註", &result.note, 1000)?;
        }
        if !result.evidence_reference.trim().is_empty() {
            validate_text("證據參照", &result.evidence_reference, 2048)?;
        }
        if !result.issue_id.trim().is_empty() {
            validate_text("缺失編號", &result.issue_id, 80)?;
        }
    }
    if seen.len() != QUALITY_CHECKLIST.len() {
        return Err("檢查項目不完整，請確認 QC-01 至 QC-29 都各出現一次".to_string());
    }
    Ok(())
}

pub fn event_hash(
    sequence: u64,
    timestamp_unix: u64,
    actor: &str,
    payload: &EventPayload,
    previous_sha256: &str,
) -> Result<String, String> {
    let body = EventBody {
        sequence,
        timestamp_unix,
        actor,
        payload,
        previous_sha256,
    };
    let encoded = serde_json::to_vec(&body).map_err(|e| format!("encode event hash body: {e}"))?;
    Ok(sha256_hex(&encoded))
}

pub fn now_unix() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("system clock before Unix epoch: {e}"))
}

pub fn validate_actor(value: &str) -> Result<(), String> {
    validate_text("操作者姓名", value, 80)
}

/// A payload that repeats the acting identity must repeat the same value as
/// the event envelope. Otherwise a crafted event could show one actor in the
/// chain while attributing an approval, publication or decision to another.
pub fn require_payload_actor(
    event_actor: &str,
    payload_actor: &str,
    label: &str,
) -> Result<(), String> {
    if event_actor == payload_actor {
        Ok(())
    } else {
        Err(format!(
            "{label} payload actor {payload_actor} does not match event actor {event_actor}"
        ))
    }
}

/// Record identifiers are generated as ASCII `<PREFIX>-<digits>` and are used
/// as lookup keys and URL path segments. Replay must reject a crafted empty,
/// oversized or non-ASCII identifier before it enters the snapshot.
pub fn validate_record_id(label: &str, value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(format!("{label}格式無效"))
    }
}

/// Allowed quality-case state changes. Same-state updates are allowed so an
/// owner can append a progress note. A failed verification returns to
/// `in_progress`; reopening a closed case is deliberately unsupported until a
/// new due date and approval can be captured as a separate controlled event.
pub fn quality_case_transition_allowed(from: &str, to: &str) -> bool {
    from == to
        || matches!(
            (from, to),
            ("open", "in_progress")
                | ("in_progress", "verification")
                | ("verification", "in_progress")
                | ("verification", "closed")
        )
}

pub fn validate_text(label: &str, value: &str, max_chars: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不可空白"));
    }
    if trimmed.chars().count() > max_chars {
        return Err(format!("{label}超過 {max_chars} 字限制"));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(format!("{label}含有不可接受的控制字元"));
    }
    Ok(())
}

/// Validate a field that is rendered as a textarea. LF and CRLF are accepted;
/// bare CR and every other control character remain rejected.
pub fn validate_multiline_text(label: &str, value: &str, max_chars: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不可空白"));
    }
    if trimmed.chars().count() > max_chars {
        return Err(format!("{label}超過 {max_chars} 字限制"));
    }
    let mut characters = trimmed.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\n' {
            continue;
        }
        if character == '\r' && characters.peek() == Some(&'\n') {
            continue;
        }
        if character.is_control() {
            return Err(format!("{label}含有不可接受的控制字元"));
        }
    }
    Ok(())
}

pub fn validate_source_url(value: &str) -> Result<(), String> {
    validate_text("來源連結", value, 2048)?;
    let trimmed = value.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("來源連結必須使用 http:// 或 https://".to_string());
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err("來源連結不可含空白字元".to_string());
    }
    Ok(())
}

pub fn validate_date(label: &str, value: &str) -> Result<(), String> {
    if value.len() != 10
        || !value.is_ascii()
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || !value
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return Err(format!("{label}請使用 YYYY-MM-DD 格式"));
    }
    let year = value[0..4]
        .parse::<u32>()
        .map_err(|_| format!("{label}請使用有效日期"))?;
    let month = value[5..7]
        .parse::<u32>()
        .map_err(|_| format!("{label}請使用有效日期"))?;
    let day = value[8..10]
        .parse::<u32>()
        .map_err(|_| format!("{label}請使用有效日期"))?;
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => 0,
    };
    if day == 0 || day > days_in_month {
        return Err(format!("{label}請使用有效日期"));
    }
    Ok(())
}

pub fn clean(value: &str) -> String {
    value.trim().to_string()
}

pub fn clean_multiline(value: &str) -> String {
    value.trim().replace("\r\n", "\n")
}
