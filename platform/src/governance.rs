#![forbid(unsafe_code)]

//! Local governance pilot ledger.
//!
//! The ledger stores metadata and source references, never external document
//! contents. It is append-only and hash chained for tamper detection. It is a
//! local prototype store, not an enterprise WORM or identity service.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::docs::sha256_hex;

const GENESIS_HASH: &str = "GENESIS";

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

#[derive(Debug)]
pub struct Ledger {
    path: PathBuf,
    snapshot: GovernanceSnapshot,
}

#[derive(Serialize)]
struct EventBody<'a> {
    sequence: u64,
    timestamp_unix: u64,
    actor: &'a str,
    payload: &'a EventPayload,
    previous_sha256: &'a str,
}

impl Ledger {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let mut snapshot = GovernanceSnapshot::default();
        if path.exists() {
            let file = fs::File::open(&path)
                .map_err(|e| format!("open ledger {}: {e}", path.display()))?;
            let reader = BufReader::new(file);
            let mut previous = GENESIS_HASH.to_string();
            for (index, line) in reader.lines().enumerate() {
                let line = line.map_err(|e| format!("read ledger line {}: {e}", index + 1))?;
                if line.trim().is_empty() {
                    return Err(format!("ledger line {} is empty", index + 1));
                }
                let event: LedgerEvent = serde_json::from_str(&line)
                    .map_err(|e| format!("parse ledger line {}: {e}", index + 1))?;
                if event.sequence != index as u64 + 1 {
                    return Err(format!("ledger sequence mismatch at line {}", index + 1));
                }
                if event.previous_sha256 != previous {
                    return Err(format!("ledger predecessor mismatch at line {}", index + 1));
                }
                let expected = event_hash(
                    event.sequence,
                    event.timestamp_unix,
                    &event.actor,
                    &event.payload,
                    &event.previous_sha256,
                )?;
                if event.sha256 != expected {
                    return Err(format!("ledger hash mismatch at line {}", index + 1));
                }
                previous = event.sha256.clone();
                apply_event(&mut snapshot, &event);
                snapshot.events.push(event);
            }
        }
        Ok(Self { path, snapshot })
    }

    pub fn snapshot(&self) -> GovernanceSnapshot {
        self.snapshot.clone()
    }

    pub fn record_project(
        &mut self,
        name: &str,
        department: &str,
        owner: &str,
        risk_tier: &str,
        actor: &str,
    ) -> Result<String, String> {
        validate_text("專案名稱", name, 120)?;
        validate_text("部門", department, 80)?;
        validate_text("負責人", owner, 80)?;
        validate_actor(actor)?;
        if !["low", "medium", "high", "critical"].contains(&risk_tier) {
            return Err("請選擇有效的風險等級".to_string());
        }
        let created_at = now_unix()?;
        let id = self.next_id("PRJ");
        let project = Project {
            id: id.clone(),
            name: clean(name),
            department: clean(department),
            owner: clean(owner),
            risk_tier: risk_tier.to_string(),
            created_at,
        };
        self.append(actor, EventPayload::ProjectCreated(project))?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn link_evidence(
        &mut self,
        project_id: &str,
        title: &str,
        source_system: &str,
        source_url: &str,
        source_version: &str,
        evidence_type: &str,
        owner: &str,
        actor: &str,
    ) -> Result<String, String> {
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err("請先選擇有效的專案".to_string());
        }
        validate_text("證據標題", title, 160)?;
        validate_text("來源系統", source_system, 80)?;
        validate_source_url(source_url)?;
        validate_text("來源版本", source_version, 80)?;
        validate_text("證據類型", evidence_type, 80)?;
        validate_text("負責人", owner, 80)?;
        validate_actor(actor)?;
        let id = self.next_id("EVD");
        let evidence = Evidence {
            id: id.clone(),
            project_id: project_id.to_string(),
            title: clean(title),
            source_system: clean(source_system),
            source_url: clean(source_url),
            source_version: clean(source_version),
            evidence_type: clean(evidence_type),
            owner: clean(owner),
            created_at: now_unix()?,
        };
        self.append(actor, EventPayload::EvidenceLinked(evidence))?;
        Ok(id)
    }

    pub fn review_evidence(
        &mut self,
        evidence_id: &str,
        result: &str,
        reviewer: &str,
        note: &str,
    ) -> Result<(), String> {
        if !self.snapshot.evidence.iter().any(|e| e.id == evidence_id) {
            return Err("找不到此證據項目".to_string());
        }
        if !["accepted", "changes_requested"].contains(&result) {
            return Err("請選擇有效的審查結果".to_string());
        }
        validate_actor(reviewer)?;
        if result == "changes_requested" {
            validate_text("退回說明", note, 1000)?;
        }
        if !note.trim().is_empty() {
            validate_text("審查備註", note, 1000)?;
        }
        self.append(
            reviewer,
            EventPayload::EvidenceReviewed(EvidenceReview {
                evidence_id: evidence_id.to_string(),
                result: result.to_string(),
                reviewer: clean(reviewer),
                note: clean(note),
                reviewed_at: now_unix()?,
            }),
        )
    }

    pub fn save_workflow(&mut self, draft: WorkflowDraftInput) -> Result<(String, u32), String> {
        validate_text("流程名稱", &draft.title, 120)?;
        validate_text("部門", &draft.department, 80)?;
        validate_text("流程負責人", &draft.owner, 80)?;
        validate_text("流程目的", &draft.description, 1500)?;
        validate_actor(&draft.actor)?;
        if draft.stages.is_empty() || draft.stages.len() > 30 {
            return Err("流程需有 1 至 30 個階段".to_string());
        }
        for stage in &draft.stages {
            validate_text("階段名稱", stage, 120)?;
        }
        let (id, version) = match draft.base_id.as_deref() {
            Some(id) => {
                let prior = self
                    .snapshot
                    .workflows
                    .iter()
                    .filter(|w| w.id == id)
                    .max_by_key(|w| w.version)
                    .ok_or_else(|| "找不到來源流程模板".to_string())?;
                (id.to_string(), prior.version + 1)
            }
            None => (self.next_id("TPL"), 1),
        };
        let workflow = Workflow {
            id: id.clone(),
            version,
            title: clean(&draft.title),
            department: clean(&draft.department),
            owner: clean(&draft.owner),
            description: clean(&draft.description),
            stages: draft.stages.into_iter().map(|s| clean(&s)).collect(),
            status: "draft".to_string(),
            created_at: now_unix()?,
            published_at: None,
            published_by: None,
        };
        self.append(&draft.actor, EventPayload::WorkflowSaved(workflow))?;
        Ok((id, version))
    }

    pub fn publish_workflow(
        &mut self,
        id: &str,
        version: u32,
        reviewer: &str,
    ) -> Result<(), String> {
        validate_actor(reviewer)?;
        let workflow = self
            .snapshot
            .workflows
            .iter()
            .find(|w| w.id == id && w.version == version)
            .ok_or_else(|| "找不到此流程版本".to_string())?;
        if workflow.status != "draft" {
            return Err("只有草稿版本可以發布".to_string());
        }
        self.append(
            reviewer,
            EventPayload::WorkflowPublished(WorkflowPublication {
                id: id.to_string(),
                version,
                reviewer: clean(reviewer),
                published_at: now_unix()?,
            }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_quality_case(
        &mut self,
        project_id: &str,
        kind: &str,
        title: &str,
        description: &str,
        owner: &str,
        due_date: &str,
        actor: &str,
    ) -> Result<String, String> {
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err("請先選擇有效的專案".to_string());
        }
        if !["risk", "change", "finding", "capa"].contains(&kind) {
            return Err("請選擇有效的品質事件類型".to_string());
        }
        validate_text("事件標題", title, 160)?;
        validate_text("事件內容", description, 2000)?;
        validate_text("負責人", owner, 80)?;
        validate_date("到期日", due_date)?;
        validate_actor(actor)?;
        let id = self.next_id("QMS");
        let now = now_unix()?;
        self.append(
            actor,
            EventPayload::QualityCaseCreated(QualityCase {
                id: id.clone(),
                project_id: project_id.to_string(),
                kind: kind.to_string(),
                title: clean(title),
                description: clean(description),
                owner: clean(owner),
                due_date: due_date.to_string(),
                status: "open".to_string(),
                created_at: now,
                updated_at: now,
                closure_note: None,
            }),
        )?;
        Ok(id)
    }

    pub fn update_quality_case(
        &mut self,
        id: &str,
        status: &str,
        actor: &str,
        note: &str,
    ) -> Result<(), String> {
        let case = self
            .snapshot
            .cases
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| "找不到此品質事件".to_string())?;
        if !["open", "in_progress", "verification", "closed"].contains(&status) {
            return Err("請選擇有效的事件狀態".to_string());
        }
        validate_actor(actor)?;
        if status == "closed" && case.kind == "capa" {
            if case.status != "verification" {
                return Err("CAPA 必須先進入待複驗狀態，複驗通過後才能結案".to_string());
            }
            validate_text("CAPA 成效驗證紀錄", note, 1200)?;
        }
        if status == "closed" && note.trim().is_empty() {
            return Err("結案前必須記錄複驗或結案依據".to_string());
        }
        if !note.trim().is_empty() {
            validate_text("處理紀錄", note, 1200)?;
        }
        self.append(
            actor,
            EventPayload::QualityCaseUpdated(QualityCaseUpdate {
                id: id.to_string(),
                status: status.to_string(),
                actor: clean(actor),
                note: clean(note),
                updated_at: now_unix()?,
            }),
        )
    }

    pub fn record_quality_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, String> {
        if !self
            .snapshot
            .projects
            .iter()
            .any(|project| project.id == inspection.project_id)
        {
            return Err("請先選擇有效的專案".to_string());
        }
        validate_actor(&inspection.reviewer)?;
        if inspection.results.len() != QUALITY_CHECKLIST.len() {
            return Err(format!(
                "品質檢查必須逐項判定全部 {} 個項目",
                QUALITY_CHECKLIST.len()
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for result in &inspection.results {
            let definition = QUALITY_CHECKLIST
                .iter()
                .find(|item| item.id == result.item_id)
                .ok_or_else(|| format!("未知的檢查項目 {}", result.item_id))?;
            let _ = definition;
            if !seen.insert(result.item_id.as_str()) {
                return Err(format!("檢查項目 {} 重複", result.item_id));
            }
            if !["pass", "fail", "not_applicable"].contains(&result.result.as_str()) {
                return Err(format!("{} 尚未選擇有效判定", result.item_id));
            }
            if result.result == "pass" {
                validate_text("符合項目的證據參照", &result.evidence_reference, 2048)?;
            }
            if result.result == "fail" {
                validate_text("不符合項目的說明", &result.note, 1000)?;
                validate_text("缺失編號", &result.issue_id, 80)?;
            }
            if result.result == "not_applicable" {
                validate_text("不適用理由", &result.note, 1000)?;
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
        let id = self.next_id("INSP");
        self.append(
            &inspection.reviewer,
            EventPayload::QualityInspectionRecorded(QualityInspection {
                id: id.clone(),
                project_id: inspection.project_id,
                reviewer: clean(&inspection.reviewer),
                created_at: now_unix()?,
                results: inspection.results,
            }),
        )?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_gate_decision(
        &mut self,
        project_id: &str,
        gate_id: &str,
        decision: &str,
        actor: &str,
        reason: &str,
        evidence_reference: &str,
        risk_acceptor: &str,
        remediation_due: &str,
        non_waivable: bool,
    ) -> Result<String, String> {
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err("請先選擇有效的專案".to_string());
        }
        if !["G1", "G2", "G3", "G4", "G5"].contains(&gate_id) {
            return Err("請選擇有效的關卡".to_string());
        }
        if !["go", "hold", "exception"].contains(&decision) {
            return Err("請選擇有效的關卡決議".to_string());
        }
        if decision == "exception" && non_waivable {
            return Err("法律、安全或其他不可豁免要求不得以一般例外放行".to_string());
        }
        validate_actor(actor)?;
        if decision != "go" {
            validate_text("決議原因", reason, 1200)?;
        }
        if decision == "exception" {
            validate_text("風險接受者", risk_acceptor, 80)?;
            validate_date("補救期限", remediation_due)?;
        }
        if !evidence_reference.trim().is_empty() {
            validate_text("證據參照", evidence_reference, 2048)?;
        }
        let id = self.next_id("GTE");
        self.append(
            actor,
            EventPayload::GateDecisionRecorded(GateDecision {
                id: id.clone(),
                project_id: project_id.to_string(),
                gate_id: gate_id.to_string(),
                decision: decision.to_string(),
                actor: clean(actor),
                reason: clean(reason),
                evidence_reference: clean(evidence_reference),
                risk_acceptor: clean(risk_acceptor),
                remediation_due: clean(remediation_due),
                non_waivable,
                created_at: now_unix()?,
            }),
        )?;
        Ok(id)
    }

    pub fn verify(&self) -> Result<(), String> {
        let mut previous = GENESIS_HASH.to_string();
        for (index, event) in self.snapshot.events.iter().enumerate() {
            if event.sequence != index as u64 + 1 || event.previous_sha256 != previous {
                return Err(format!("稽核鏈在第 {} 筆事件失去連續性", index + 1));
            }
            let expected = event_hash(
                event.sequence,
                event.timestamp_unix,
                &event.actor,
                &event.payload,
                &event.previous_sha256,
            )?;
            if event.sha256 != expected {
                return Err(format!("稽核鏈第 {} 筆事件雜湊不符", index + 1));
            }
            previous = event.sha256.clone();
        }
        Ok(())
    }

    fn next_id(&self, prefix: &str) -> String {
        format!("{prefix}-{:06}", self.snapshot.events.len() + 1)
    }

    fn append(&mut self, actor: &str, payload: EventPayload) -> Result<(), String> {
        validate_actor(actor)?;
        let sequence = self.snapshot.events.len() as u64 + 1;
        let previous_sha256 = self
            .snapshot
            .events
            .last()
            .map(|e| e.sha256.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let timestamp_unix = now_unix()?;
        let hash = event_hash(sequence, timestamp_unix, actor, &payload, &previous_sha256)?;
        let event = LedgerEvent {
            sequence,
            timestamp_unix,
            actor: clean(actor),
            payload,
            previous_sha256,
            sha256: hash,
        };
        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create ledger directory {}: {e}", parent.display()))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| format!("append ledger {}: {e}", self.path.display()))?;
        serde_json::to_writer(&mut file, &event)
            .map_err(|e| format!("encode ledger event: {e}"))?;
        file.write_all(b"\n")
            .map_err(|e| format!("write ledger line ending: {e}"))?;
        file.sync_data()
            .map_err(|e| format!("flush ledger event: {e}"))?;
        apply_event(&mut self.snapshot, &event);
        self.snapshot.events.push(event);
        Ok(())
    }
}

fn apply_event(snapshot: &mut GovernanceSnapshot, event: &LedgerEvent) {
    match &event.payload {
        EventPayload::ProjectCreated(project) => snapshot.projects.push(project.clone()),
        EventPayload::EvidenceLinked(evidence) => snapshot.evidence.push(evidence.clone()),
        EventPayload::EvidenceReviewed(review) => {
            snapshot
                .reviews
                .insert(review.evidence_id.clone(), review.clone());
        }
        EventPayload::WorkflowSaved(workflow) => snapshot.workflows.push(workflow.clone()),
        EventPayload::WorkflowPublished(publication) => {
            if let Some(workflow) = snapshot
                .workflows
                .iter_mut()
                .find(|w| w.id == publication.id && w.version == publication.version)
            {
                workflow.status = "published".to_string();
                workflow.published_at = Some(publication.published_at);
                workflow.published_by = Some(publication.reviewer.clone());
            }
        }
        EventPayload::QualityCaseCreated(case) => snapshot.cases.push(case.clone()),
        EventPayload::QualityCaseUpdated(update) => {
            if let Some(case) = snapshot.cases.iter_mut().find(|c| c.id == update.id) {
                case.status = update.status.clone();
                case.updated_at = update.updated_at;
                if update.status == "closed" {
                    case.closure_note = Some(update.note.clone());
                }
            }
        }
        EventPayload::QualityInspectionRecorded(inspection) => {
            snapshot.inspections.push(inspection.clone())
        }
        EventPayload::GateDecisionRecorded(decision) => {
            snapshot.gate_decisions.push(decision.clone())
        }
    }
}

fn event_hash(
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

pub fn default_ledger_path() -> PathBuf {
    if let Some(path) = std::env::var_os("EKP_GOVERNANCE_DATA") {
        return PathBuf::from(path);
    }
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return Path::new(&root)
            .join("EKPI")
            .join("governance-ledger.jsonl");
    }
    std::env::temp_dir()
        .join("ekpi")
        .join("governance-ledger.jsonl")
}

fn now_unix() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("system clock before Unix epoch: {e}"))
}

fn validate_actor(value: &str) -> Result<(), String> {
    validate_text("操作者姓名", value, 80)
}

fn validate_text(label: &str, value: &str, max_chars: usize) -> Result<(), String> {
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

fn validate_source_url(value: &str) -> Result<(), String> {
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

fn validate_date(label: &str, value: &str) -> Result<(), String> {
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

fn clean(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "ekpi-governance-{}-{}.jsonl",
            std::process::id(),
            NEXT_PATH.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn records_projects_and_reloads_a_valid_chain() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("文件治理試點", "PMO", "Alex", "medium", "Jordan")
            .expect("project records");
        assert_eq!(project_id, "PRJ-000001");
        ledger.verify().expect("new chain verifies");
        drop(ledger);
        let reloaded = Ledger::open(&path).expect("ledger reloads");
        assert_eq!(reloaded.snapshot().projects.len(), 1);
        reloaded.verify().expect("reloaded chain verifies");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn evidence_requires_http_source_reference_and_existing_project() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        assert!(ledger
            .link_evidence(
                "PRJ-000001",
                "核准記錄",
                "Microsoft 365",
                "javascript:alert(1)",
                "v1",
                "approval",
                "Alex",
                "Jordan",
            )
            .is_err());
        let project_id = ledger
            .record_project("文件治理試點", "PMO", "Alex", "medium", "Jordan")
            .expect("project records");
        let evidence_id = ledger
            .link_evidence(
                &project_id,
                "核准記錄",
                "Microsoft 365",
                "https://tenant.example/doc/1",
                "v1",
                "approval",
                "Alex",
                "Jordan",
            )
            .expect("valid source link records");
        assert_eq!(evidence_id, "EVD-000002");
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn non_waivable_gate_cannot_be_exception_released() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("安全流程", "QA", "Alex", "high", "Jordan")
            .expect("project records");
        let result = ledger.record_gate_decision(
            &project_id,
            "G4",
            "exception",
            "Reviewer",
            "缺少法規證據",
            "",
            "Sponsor",
            "2026-11-30",
            true,
        );
        assert!(result.is_err());
        assert_eq!(ledger.snapshot().gate_decisions.len(), 0);
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn date_validation_rejects_impossible_dates_and_accepts_leap_day() {
        assert!(validate_date("期限", "2026-02-29").is_err());
        assert!(validate_date("期限", "2024-02-29").is_ok());
        assert!(validate_date("期限", "2026-13-01").is_err());
    }

    #[test]
    fn capa_must_pass_through_verification_before_closure() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("品質改善", "QA", "Alex", "medium", "Jordan")
            .expect("project records");
        let case_id = ledger
            .create_quality_case(
                &project_id,
                "capa",
                "缺失改善",
                "確認根因並完成矯正",
                "Alex",
                "2026-12-31",
                "Jordan",
            )
            .expect("CAPA records");
        assert!(ledger
            .update_quality_case(&case_id, "closed", "Reviewer", "有效")
            .is_err());
        ledger
            .update_quality_case(&case_id, "in_progress", "Owner", "已執行矯正")
            .expect("case moves to work");
        ledger
            .update_quality_case(&case_id, "verification", "Owner", "提出複驗")
            .expect("case moves to verification");
        ledger
            .update_quality_case(&case_id, "closed", "Reviewer", "有效性已確認")
            .expect("verified CAPA closes");
        assert_eq!(ledger.snapshot().cases[0].status, "closed");
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn workflow_versions_are_immutable_and_publish_requires_a_draft() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let (workflow_id, version) = ledger
            .save_workflow(WorkflowDraftInput {
                base_id: None,
                title: "變更管理".to_string(),
                department: "跨部門".to_string(),
                owner: "流程負責人".to_string(),
                description: "記錄評估、核准和結案".to_string(),
                stages: vec!["提出".to_string(), "評估".to_string(), "核准".to_string()],
                actor: "建立者".to_string(),
            })
            .expect("draft created");
        assert_eq!(version, 1);
        ledger
            .publish_workflow(&workflow_id, version, "審查者")
            .expect("draft can be reviewed and published");
        assert!(ledger
            .publish_workflow(&workflow_id, version, "審查者")
            .is_err());
        let (same_id, next_version) = ledger
            .save_workflow(WorkflowDraftInput {
                base_id: Some(workflow_id.clone()),
                title: "變更管理".to_string(),
                department: "跨部門".to_string(),
                owner: "流程負責人".to_string(),
                description: "增加影響分析".to_string(),
                stages: vec![
                    "提出".to_string(),
                    "影響分析".to_string(),
                    "核准".to_string(),
                ],
                actor: "建立者".to_string(),
            })
            .expect("new version created");
        assert_eq!(same_id, workflow_id);
        assert_eq!(next_version, 2);
        let latest = ledger
            .snapshot()
            .workflows
            .into_iter()
            .find(|workflow| workflow.version == next_version)
            .expect("new version exists");
        assert_eq!(latest.status, "draft");
        ledger.verify().expect("workflow audit chain verifies");
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn quality_inspection_requires_all_29_decisions_and_evidence() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("品質檢查", "QA", "Alex", "medium", "Jordan")
            .expect("project records");
        assert!(ledger
            .record_quality_inspection(QualityInspectionInput {
                project_id: project_id.clone(),
                reviewer: "Reviewer".to_string(),
                results: Vec::new(),
            })
            .is_err());
        let mut results: Vec<QualityCheckResult> = QUALITY_CHECKLIST
            .iter()
            .map(|item| QualityCheckResult {
                item_id: item.id.to_string(),
                result: "pass".to_string(),
                note: String::new(),
                evidence_reference: format!("REF-{}", item.id),
                issue_id: String::new(),
            })
            .collect();
        results[0].evidence_reference.clear();
        let invalid = QualityInspectionInput {
            project_id: project_id.clone(),
            reviewer: "Reviewer".to_string(),
            results: results.clone(),
        };
        assert!(ledger.record_quality_inspection(invalid).is_err());
        results[0].evidence_reference = "REF-QC-01".to_string();
        let id = ledger
            .record_quality_inspection(QualityInspectionInput {
                project_id,
                reviewer: "Reviewer".to_string(),
                results,
            })
            .expect("complete evidence-backed checklist records");
        assert_eq!(id, "INSP-000002");
        assert_eq!(ledger.snapshot().inspections[0].results.len(), 29);
        ledger.verify().expect("inspection audit chain verifies");
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn tampering_with_a_stored_event_is_detected() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        ledger
            .record_project("原始名稱", "PMO", "Alex", "low", "Jordan")
            .expect("project records");
        drop(ledger);
        let text = fs::read_to_string(&path).expect("ledger reads");
        fs::write(&path, text.replace("原始名稱", "竄改名稱")).expect("tampered copy writes");
        assert!(Ledger::open(&path).is_err());
        fs::remove_file(path).expect("test ledger removed");
    }
}
