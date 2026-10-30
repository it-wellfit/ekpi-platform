#![forbid(unsafe_code)]

//! Local governance pilot ledger.
//!
//! The ledger stores metadata and source references, never external document
//! contents. It is append-only and hash chained for tamper detection. It is a
//! local prototype store, not an enterprise WORM or identity service.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::docs::sha256_hex;
use crate::local_io::open_for_append;
use crate::local_lock::LocalFileLock;

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
    _lock: LocalFileLock,
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
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create ledger directory {}: {e}", parent.display()))?;
        }
        let lock = LocalFileLock::acquire(&path)?;
        let mut snapshot = GovernanceSnapshot::default();
        if path.exists() {
            let contents = fs::read_to_string(&path)
                .map_err(|e| format!("read ledger {}: {e}", path.display()))?;
            if !contents.is_empty() && !contents.ends_with('\n') {
                return Err(format!(
                    "ledger {} has a truncated final line; refusing to load",
                    path.display()
                ));
            }
            for (index, line) in contents.lines().enumerate() {
                if line.trim().is_empty() {
                    return Err(format!("ledger line {} is empty", index + 1));
                }
                let event: LedgerEvent = serde_json::from_str(line)
                    .map_err(|e| format!("parse ledger line {}: {e}", index + 1))?;
                if event.sequence != index as u64 + 1 {
                    return Err(format!("ledger sequence mismatch at line {}", index + 1));
                }
                if event.previous_sha256 != previous_hash(&snapshot) {
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
                validate_actor(&event.actor)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                apply_event(&mut snapshot, &event)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                snapshot.events.push(event);
            }
        }
        Ok(Self {
            path,
            snapshot,
            _lock: lock,
        })
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
        validate_multiline_text("流程目的", &draft.description, 1500)?;
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
                let next_version = prior
                    .version
                    .checked_add(1)
                    .ok_or_else(|| "流程版本已達上限，無法建立新版本".to_string())?;
                (id.to_string(), next_version)
            }
            None => (self.next_id("TPL"), 1),
        };
        let workflow = Workflow {
            id: id.clone(),
            version,
            title: clean(&draft.title),
            department: clean(&draft.department),
            owner: clean(&draft.owner),
            description: clean_multiline(&draft.description),
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
        validate_multiline_text("事件內容", description, 2000)?;
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
                description: clean_multiline(description),
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
        // Give the CAPA-specific refusal precedence: it tells the reviewer the
        // required next step, while the generic transition message would only
        // restate the current state.
        if status == "closed" && case.kind == "capa" {
            if case.status != "verification" {
                return Err("CAPA 必須先進入待複驗狀態，複驗通過後才能結案".to_string());
            }
            validate_text("CAPA 成效驗證紀錄", note, 1200)?;
        }
        if !quality_case_transition_allowed(&case.status, status) {
            return Err(format!(
                "品質事件狀態不可由「{}」直接轉為「{}」",
                case.status, status
            ));
        }
        validate_actor(actor)?;
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
        validate_inspection_results(&inspection.results)?;
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
            validate_multiline_text("決議原因", reason, 1200)?;
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
                reason: clean_multiline(reason),
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
        let actor = clean(actor);
        let sequence = self.snapshot.events.len() as u64 + 1;
        let previous_sha256 = previous_hash(&self.snapshot);
        let timestamp_unix = now_unix()?;
        let hash = event_hash(sequence, timestamp_unix, &actor, &payload, &previous_sha256)?;
        let event = LedgerEvent {
            sequence,
            timestamp_unix,
            actor,
            payload,
            previous_sha256,
            sha256: hash,
        };
        let mut next_snapshot = self.snapshot.clone();
        apply_event(&mut next_snapshot, &event)?;
        next_snapshot.events.push(event.clone());

        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create ledger directory {}: {e}", parent.display()))?;
        }
        let mut line =
            serde_json::to_vec(&event).map_err(|e| format!("encode ledger event: {e}"))?;
        line.push(b'\n');
        let mut file = open_for_append(&self.path)?;
        let original_len = file
            .metadata()
            .map_err(|e| format!("inspect ledger {}: {e}", self.path.display()))?
            .len();
        if let Err(error) = file.write_all(&line).and_then(|_| file.sync_all()) {
            let rollback = file.set_len(original_len).and_then(|_| file.sync_all());
            return Err(match rollback {
                Ok(()) => format!(
                    "write ledger event to {}: {error}; partial write rolled back",
                    self.path.display()
                ),
                Err(rollback_error) => format!(
                    "write ledger event to {}: {error}; rollback failed: {rollback_error}",
                    self.path.display()
                ),
            });
        }
        self.snapshot = next_snapshot;
        Ok(())
    }
}

fn previous_hash(snapshot: &GovernanceSnapshot) -> String {
    snapshot
        .events
        .last()
        .map(|event| event.sha256.clone())
        .unwrap_or_else(|| GENESIS_HASH.to_string())
}

fn apply_event(snapshot: &mut GovernanceSnapshot, event: &LedgerEvent) -> Result<(), String> {
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

fn validate_inspection_results(results: &[QualityCheckResult]) -> Result<(), String> {
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

/// A payload that repeats the acting identity must repeat the same value as
/// the event envelope. Otherwise a crafted event could show one actor in the
/// chain while attributing an approval, publication or decision to another.
fn require_payload_actor(
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
fn validate_record_id(label: &str, value: &str) -> Result<(), String> {
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
fn quality_case_transition_allowed(from: &str, to: &str) -> bool {
    from == to
        || matches!(
            (from, to),
            ("open", "in_progress")
                | ("in_progress", "verification")
                | ("verification", "in_progress")
                | ("verification", "closed")
        )
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

/// Validate a field that is rendered as a textarea. LF and CRLF are accepted;
/// bare CR and every other control character remain rejected.
fn validate_multiline_text(label: &str, value: &str, max_chars: usize) -> Result<(), String> {
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

fn clean_multiline(value: &str) -> String {
    value.trim().replace("\r\n", "\n")
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
    fn multiline_text_accepts_lf_and_crlf_but_rejects_bare_cr_and_other_controls() {
        assert!(validate_multiline_text("內容", "第一行\n第二行", 100).is_ok());
        assert!(validate_multiline_text("內容", "第一行\r\n第二行", 100).is_ok());
        assert!(validate_multiline_text("內容", "第一行\r第二行", 100).is_err());
        assert!(validate_multiline_text("內容", "第一行\t第二行", 100).is_err());
        assert!(
            validate_text("名稱", "第一行\n第二行", 100).is_err(),
            "single-line fields must still reject line breaks"
        );
    }

    #[test]
    fn multiline_fields_persist_and_replay_for_workflow_quality_case_and_gate() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("多行內容", "QA", "Alex", "medium", "Jordan")
            .expect("project records");
        let (workflow_id, _version) = ledger
            .save_workflow(WorkflowDraftInput {
                base_id: None,
                title: "多行流程".to_string(),
                department: "QA".to_string(),
                owner: "Alex".to_string(),
                description: "第一行\r\n第二行".to_string(),
                stages: vec!["提出".to_string(), "審查".to_string()],
                actor: "Jordan".to_string(),
            })
            .expect("multiline workflow records");
        let workflow = ledger
            .snapshot()
            .workflows
            .into_iter()
            .find(|workflow| workflow.id == workflow_id)
            .expect("workflow exists");
        assert_eq!(workflow.description, "第一行\n第二行");

        let case_id = ledger
            .create_quality_case(
                &project_id,
                "risk",
                "多行品質事件",
                "根因\r\n影響",
                "Alex",
                "2026-12-31",
                "Jordan",
            )
            .expect("multiline quality case records");
        let quality_case = ledger
            .snapshot()
            .cases
            .into_iter()
            .find(|case| case.id == case_id)
            .expect("quality case exists");
        assert_eq!(quality_case.description, "根因\n影響");

        ledger
            .record_gate_decision(
                &project_id,
                "G2",
                "hold",
                "Reviewer",
                "等待\r\n複核",
                "",
                "",
                "",
                false,
            )
            .expect("multiline gate reason records");
        assert_eq!(ledger.snapshot().gate_decisions[0].reason, "等待\n複核");

        ledger.verify().expect("multiline chain verifies");
        drop(ledger);
        let reloaded = Ledger::open(&path).expect("multiline ledger reloads");
        assert_eq!(
            reloaded
                .snapshot()
                .workflows
                .into_iter()
                .find(|workflow| workflow.id == workflow_id)
                .expect("workflow survives replay")
                .description,
            "第一行\n第二行"
        );
        assert_eq!(
            reloaded
                .snapshot()
                .cases
                .into_iter()
                .find(|case| case.id == case_id)
                .expect("quality case survives replay")
                .description,
            "根因\n影響"
        );
        assert_eq!(reloaded.snapshot().gate_decisions[0].reason, "等待\n複核");
        reloaded
            .verify()
            .expect("reloaded multiline chain verifies");
        drop(reloaded);
        fs::remove_file(path).expect("test ledger removed");
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

    fn hashed_event(
        sequence: u64,
        timestamp_unix: u64,
        actor: &str,
        payload: EventPayload,
        previous: &str,
    ) -> LedgerEvent {
        let sha256 =
            event_hash(sequence, timestamp_unix, actor, &payload, previous).expect("event hashes");
        LedgerEvent {
            sequence,
            timestamp_unix,
            actor: actor.to_string(),
            payload,
            previous_sha256: previous.to_string(),
            sha256,
        }
    }

    #[test]
    fn actor_is_trimmed_before_hashing_and_reloads() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        ledger
            .record_project("空白操作者", "PMO", "Alex", "low", " Jordan ")
            .expect("project records");
        drop(ledger);

        let reloaded = Ledger::open(&path).expect("trimmed actor must reload and verify");
        assert_eq!(reloaded.snapshot().events[0].actor, "Jordan");
        reloaded.verify().expect("reloaded chain verifies");
        drop(reloaded);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_an_event_with_an_invalid_actor() {
        let path = test_path();
        let payload = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "文件治理試點".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "medium".to_string(),
            created_at: 1_700_000_000,
        });
        let event = hashed_event(1, 1_700_000_000, "   ", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("an invalid event actor is refused");
        assert!(error.contains("操作者姓名"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_payload_actors_that_differ_from_the_event_actor() {
        let project = Project {
            id: "PRJ-000001".to_string(),
            name: "文件治理試點".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "medium".to_string(),
            created_at: 1_700_000_000,
        };
        let evidence = Evidence {
            id: "EVD-000002".to_string(),
            project_id: project.id.clone(),
            title: "來源文件".to_string(),
            source_system: "DMS".to_string(),
            source_url: "https://example.test/doc".to_string(),
            source_version: "v1".to_string(),
            evidence_type: "document".to_string(),
            owner: "Alex".to_string(),
            created_at: 1_700_000_000,
        };
        let workflow = Workflow {
            id: "TPL-000002".to_string(),
            version: 1,
            title: "變更管理".to_string(),
            department: "跨部門".to_string(),
            owner: "流程負責人".to_string(),
            description: "記錄評估、核准和結案".to_string(),
            stages: vec!["提出".to_string()],
            status: "draft".to_string(),
            created_at: 1_700_000_000,
            published_at: None,
            published_by: None,
        };
        let case = QualityCase {
            id: "QMS-000002".to_string(),
            project_id: project.id.clone(),
            kind: "finding".to_string(),
            title: "缺失".to_string(),
            description: "確認根因並完成矯正".to_string(),
            owner: "Alex".to_string(),
            due_date: "2026-12-31".to_string(),
            status: "open".to_string(),
            created_at: 1_700_000_000,
            updated_at: 1_700_000_000,
            closure_note: None,
        };
        let inspection_results = QUALITY_CHECKLIST
            .iter()
            .map(|item| QualityCheckResult {
                item_id: item.id.to_string(),
                result: "not_applicable".to_string(),
                note: "本試點不適用".to_string(),
                evidence_reference: String::new(),
                issue_id: String::new(),
            })
            .collect::<Vec<_>>();

        let base_snapshot = || {
            let mut snapshot = GovernanceSnapshot::default();
            snapshot.projects.push(project.clone());
            snapshot.evidence.push(evidence.clone());
            snapshot.workflows.push(workflow.clone());
            snapshot.cases.push(case.clone());
            snapshot
        };

        let payloads = [
            (
                "evidence review",
                EventPayload::EvidenceReviewed(EvidenceReview {
                    evidence_id: evidence.id.clone(),
                    result: "accepted".to_string(),
                    reviewer: "Mallory".to_string(),
                    note: String::new(),
                    reviewed_at: 1_700_000_001,
                }),
            ),
            (
                "workflow publication",
                EventPayload::WorkflowPublished(WorkflowPublication {
                    id: workflow.id.clone(),
                    version: workflow.version,
                    reviewer: "Mallory".to_string(),
                    published_at: 1_700_000_001,
                }),
            ),
            (
                "quality case update",
                EventPayload::QualityCaseUpdated(QualityCaseUpdate {
                    id: case.id.clone(),
                    status: "in_progress".to_string(),
                    actor: "Mallory".to_string(),
                    note: "處理中".to_string(),
                    updated_at: 1_700_000_001,
                }),
            ),
            (
                "quality inspection",
                EventPayload::QualityInspectionRecorded(QualityInspection {
                    id: "INSP-000001".to_string(),
                    project_id: project.id.clone(),
                    reviewer: "Mallory".to_string(),
                    created_at: 1_700_000_001,
                    results: inspection_results.clone(),
                }),
            ),
            (
                "gate decision",
                EventPayload::GateDecisionRecorded(GateDecision {
                    id: "GATE-000001".to_string(),
                    project_id: project.id.clone(),
                    gate_id: "G1".to_string(),
                    decision: "go".to_string(),
                    actor: "Mallory".to_string(),
                    reason: String::new(),
                    evidence_reference: String::new(),
                    risk_acceptor: String::new(),
                    remediation_due: String::new(),
                    non_waivable: false,
                    created_at: 1_700_000_001,
                }),
            ),
        ];

        for (label, payload) in payloads {
            let mut snapshot = base_snapshot();
            let event = hashed_event(1, 1_700_000_000, "Jordan", payload, GENESIS_HASH);
            let error = apply_event(&mut snapshot, &event)
                .expect_err("a payload actor that differs from the event actor is refused");
            assert!(
                error.contains("does not match"),
                "{label} must be refused: {error}"
            );
        }
    }

    #[test]
    fn quality_case_rejects_skipped_transitions_and_reopening() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        let project_id = ledger
            .record_project("品質改善", "QA", "Alex", "medium", "Jordan")
            .expect("project records");
        let case_id = ledger
            .create_quality_case(
                &project_id,
                "finding",
                "缺失",
                "確認根因並完成矯正",
                "Alex",
                "2026-12-31",
                "Jordan",
            )
            .expect("finding records");

        assert!(
            ledger
                .update_quality_case(&case_id, "closed", "Reviewer", "直接結案")
                .is_err(),
            "a case must not skip verification"
        );
        assert!(
            ledger
                .update_quality_case(&case_id, "verification", "Owner", "跳過處理")
                .is_err(),
            "a case must not move straight from open to verification"
        );
        ledger
            .update_quality_case(&case_id, "in_progress", "Owner", "已開始處理")
            .expect("open to in_progress is allowed");
        assert!(
            ledger
                .update_quality_case(&case_id, "closed", "Reviewer", "尚未複驗")
                .is_err(),
            "in_progress to closed must be refused"
        );
        ledger
            .update_quality_case(&case_id, "verification", "Owner", "提出複驗")
            .expect("in_progress to verification is allowed");
        assert!(
            ledger
                .update_quality_case(&case_id, "open", "Owner", "退回待處理")
                .is_err(),
            "verification must not move back to open"
        );
        ledger
            .update_quality_case(&case_id, "closed", "Reviewer", "有效性已確認")
            .expect("verification to closed is allowed");
        assert!(
            ledger
                .update_quality_case(&case_id, "in_progress", "Owner", "直接重開")
                .is_err(),
            "a closed case must not move directly back to in_progress"
        );
        assert!(
            ledger
                .update_quality_case(&case_id, "open", "Owner", "重開但無新期限")
                .is_err(),
            "reopening without a new due date is not supported"
        );
        drop(ledger);
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_a_quality_case_that_skips_verification() {
        let path = test_path();
        let project = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "品質改善".to_string(),
            department: "QA".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "medium".to_string(),
            created_at: 1_700_000_000,
        });
        let first = hashed_event(1, 1_700_000_000, "Jordan", project, GENESIS_HASH);
        let case = EventPayload::QualityCaseCreated(QualityCase {
            id: "QMS-000002".to_string(),
            project_id: "PRJ-000001".to_string(),
            kind: "finding".to_string(),
            title: "缺失".to_string(),
            description: "確認根因並完成矯正".to_string(),
            owner: "Alex".to_string(),
            due_date: "2026-12-31".to_string(),
            status: "open".to_string(),
            created_at: 1_700_000_000,
            updated_at: 1_700_000_000,
            closure_note: None,
        });
        let second = hashed_event(2, 1_700_000_001, "Jordan", case, &first.sha256);
        let closed = EventPayload::QualityCaseUpdated(QualityCaseUpdate {
            id: "QMS-000002".to_string(),
            status: "closed".to_string(),
            actor: "Reviewer".to_string(),
            note: "直接結案".to_string(),
            updated_at: 1_700_000_002,
        });
        let third = hashed_event(3, 1_700_000_002, "Reviewer", closed, &second.sha256);
        let lines = [first, second, third]
            .iter()
            .map(|event| serde_json::to_string(event).expect("event encodes"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, format!("{lines}\n")).expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("an invalid transition is refused");
        assert!(error.contains("cannot move from open to closed"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_an_empty_record_id() {
        let path = test_path();
        let payload = EventPayload::ProjectCreated(Project {
            id: String::new(),
            name: "文件治理試點".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "medium".to_string(),
            created_at: 1_700_000_000,
        });
        let event = hashed_event(1, 1_700_000_000, "Jordan", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("an empty record id is refused");
        assert!(error.contains("專案編號"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_workflow_version_zero() {
        let path = test_path();
        let payload = EventPayload::WorkflowSaved(Workflow {
            id: "TPL-999999".to_string(),
            version: 0,
            title: "無效版本".to_string(),
            department: "QA".to_string(),
            owner: "Alex".to_string(),
            description: "版本必須從 1 開始".to_string(),
            stages: vec!["提出".to_string()],
            status: "draft".to_string(),
            created_at: 1_700_000_000,
            published_at: None,
            published_by: None,
        });
        let event = hashed_event(1, 1_700_000_000, "Creator", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("version zero is refused");
        assert!(error.contains("version 1 or later"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_a_publication_for_a_missing_workflow() {
        let path = test_path();
        let payload = EventPayload::WorkflowPublished(WorkflowPublication {
            id: "TPL-999999".to_string(),
            version: 1,
            reviewer: "Reviewer".to_string(),
            published_at: 1_700_000_000,
        });
        let event = hashed_event(1, 1_700_000_000, "Reviewer", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("dangling publication is refused");
        assert!(
            error.contains("流程") || error.contains("workflow"),
            "{error}"
        );
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_evidence_for_a_missing_project() {
        let path = test_path();
        let payload = EventPayload::EvidenceLinked(Evidence {
            id: "EVD-999999".to_string(),
            project_id: "PRJ-999999".to_string(),
            title: "Orphan evidence".to_string(),
            source_system: "System".to_string(),
            source_url: "https://example.test/evidence".to_string(),
            source_version: "v1".to_string(),
            evidence_type: "approval".to_string(),
            owner: "Alex".to_string(),
            created_at: 1_700_000_000,
        });
        let event = hashed_event(1, 1_700_000_000, "Jordan", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("orphan evidence is refused");
        assert!(
            error.contains("專案") || error.contains("project"),
            "{error}"
        );
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_duplicate_project_ids() {
        let path = test_path();
        let first_payload = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "First".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "low".to_string(),
            created_at: 1_700_000_000,
        });
        let first = hashed_event(1, 1_700_000_000, "Jordan", first_payload, GENESIS_HASH);
        let second_payload = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "Duplicate".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "low".to_string(),
            created_at: 1_700_000_001,
        });
        let second = hashed_event(2, 1_700_000_001, "Jordan", second_payload, &first.sha256);
        fs::write(
            &path,
            format!(
                "{}\n{}\n",
                serde_json::to_string(&first).expect("first encodes"),
                serde_json::to_string(&second).expect("second encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("duplicate project id is refused");
        assert!(error.contains("duplicate project id"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_a_workflow_without_its_predecessor() {
        let path = test_path();
        let payload = EventPayload::WorkflowSaved(Workflow {
            id: "TPL-000001".to_string(),
            version: 2,
            title: "Orphan version".to_string(),
            department: "PMO".to_string(),
            owner: "Alex".to_string(),
            description: "No version 1 exists".to_string(),
            stages: vec!["Intake".to_string()],
            status: "draft".to_string(),
            created_at: 1_700_000_000,
            published_at: None,
            published_by: None,
        });
        let event = hashed_event(1, 1_700_000_000, "Jordan", payload, GENESIS_HASH);
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("event encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("orphan workflow version is refused");
        assert!(error.contains("predecessor"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn ledger_refuses_a_truncated_final_line() {
        let path = test_path();
        let mut ledger = Ledger::open(&path).expect("new ledger opens");
        ledger
            .record_project("截斷測試", "PMO", "Alex", "low", "Jordan")
            .expect("project records");
        drop(ledger);

        let text = fs::read_to_string(&path).expect("ledger reads");
        fs::write(&path, text.trim_end_matches('\n')).expect("truncate final newline");
        let error = Ledger::open(&path).expect_err("truncated final line is refused");
        assert!(error.contains("truncated"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_an_inspection_with_an_unknown_checklist_item() {
        let path = test_path();
        let project = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "Inspection project".to_string(),
            department: "QA".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "high".to_string(),
            created_at: 1_700_000_000,
        });
        let first = hashed_event(1, 1_700_000_000, "Jordan", project, GENESIS_HASH);
        let mut results: Vec<QualityCheckResult> = QUALITY_CHECKLIST
            .iter()
            .map(|item| QualityCheckResult {
                item_id: item.id.to_string(),
                result: "not_applicable".to_string(),
                note: "not used in this project".to_string(),
                evidence_reference: String::new(),
                issue_id: String::new(),
            })
            .collect();
        results[0].item_id = "QC-99".to_string();
        let inspection = EventPayload::QualityInspectionRecorded(QualityInspection {
            id: "INSP-000002".to_string(),
            project_id: "PRJ-000001".to_string(),
            reviewer: "Reviewer".to_string(),
            created_at: 1_700_000_001,
            results,
        });
        let second = hashed_event(2, 1_700_000_001, "Reviewer", inspection, &first.sha256);
        fs::write(
            &path,
            format!(
                "{}\n{}\n",
                serde_json::to_string(&first).expect("first encodes"),
                serde_json::to_string(&second).expect("second encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("invalid checklist item is refused");
        assert!(error.contains("QC-99"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }

    #[test]
    fn replay_refuses_a_quality_case_with_an_invalid_kind() {
        let path = test_path();
        let project = EventPayload::ProjectCreated(Project {
            id: "PRJ-000001".to_string(),
            name: "Case project".to_string(),
            department: "QA".to_string(),
            owner: "Alex".to_string(),
            risk_tier: "medium".to_string(),
            created_at: 1_700_000_000,
        });
        let first = hashed_event(1, 1_700_000_000, "Jordan", project, GENESIS_HASH);
        let case = EventPayload::QualityCaseCreated(QualityCase {
            id: "QC-000002".to_string(),
            project_id: "PRJ-000001".to_string(),
            kind: "arbitrary".to_string(),
            title: "Crafted case".to_string(),
            description: "Invalid kind".to_string(),
            owner: "Alex".to_string(),
            due_date: "2026-12-31".to_string(),
            status: "open".to_string(),
            created_at: 1_700_000_001,
            updated_at: 1_700_000_001,
            closure_note: None,
        });
        let second = hashed_event(2, 1_700_000_001, "Jordan", case, &first.sha256);
        fs::write(
            &path,
            format!(
                "{}\n{}\n",
                serde_json::to_string(&first).expect("first encodes"),
                serde_json::to_string(&second).expect("second encodes")
            ),
        )
        .expect("crafted ledger writes");

        let error = Ledger::open(&path).expect_err("invalid quality case kind is refused");
        assert!(error.contains("invalid quality case kind"), "{error}");
        fs::remove_file(path).expect("test ledger removed");
    }
}
