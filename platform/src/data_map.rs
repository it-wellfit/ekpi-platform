#![forbid(unsafe_code)]

//! 資料地圖（CTR-DATA-008）：欄位 → 事件 → 寫入面 → 呈現頁的血緣表。
//!
//! 設計（S11）：血緣以 **`'static` 常數表**表達（每個事件一列），並以兩個
//! **無萬用臂的窮舉 match**（`governance_lineage_for`／`approval_lineage_for`）
//! 鎖定覆蓋——新增事件變體時編譯失敗，強制更新血緣。這是型別系統驅動的
//! 血緣，不是文字掃描（不會漂移）。
//!
//! JSON 由常數表序列化：無時間戳、無路徑、陣列順序固定 → 同輸入重放
//! 逐位元組一致（與 CTR-EXP-002/003 同一紀律）。

use serde::Serialize;

use ekp_domain::workflow::ApprovalEventPayload;

use crate::governance::EventPayload;

/// 單一事件的資料血緣：欄位 → 寫入面 → 呈現頁。
#[derive(Debug, Clone, Serialize)]
pub struct LineageEntry {
    /// serde 事件標籤（snake_case），與帳鏈 JSON 的事件類型一致。
    pub event: &'static str,
    /// 事件承載的資料欄位（與 domain 結構體欄位同名）。
    pub fields: &'static [&'static str],
    /// 可產生此事件的寫入面（HTML 表單／API）。
    pub write_surfaces: &'static [&'static str],
    /// 呈現此事件的頁面。
    pub pages: &'static [&'static str],
}

const GOV_PROJECT_CREATED: LineageEntry = LineageEntry {
    event: "project_created",
    fields: &[
        "id",
        "name",
        "department",
        "owner",
        "risk_tier",
        "created_at",
    ],
    write_surfaces: &["表單 POST /projects"],
    pages: &["/projects", "/"],
};

const GOV_EVIDENCE_LINKED: LineageEntry = LineageEntry {
    event: "evidence_linked",
    fields: &[
        "id",
        "project_id",
        "title",
        "source_system",
        "source_url",
        "source_version",
        "evidence_type",
        "owner",
        "created_at",
    ],
    write_surfaces: &["表單 POST /evidence"],
    pages: &["/evidence"],
};

const GOV_EVIDENCE_REVIEWED: LineageEntry = LineageEntry {
    event: "evidence_reviewed",
    fields: &["evidence_id", "result", "reviewer", "note", "reviewed_at"],
    write_surfaces: &["表單 POST /evidence/{id}/review"],
    pages: &["/evidence"],
};

const GOV_WORKFLOW_SAVED: LineageEntry = LineageEntry {
    event: "workflow_saved",
    fields: &[
        "id",
        "version",
        "title",
        "department",
        "owner",
        "description",
        "stages",
        "status",
        "created_at",
        "published_at",
        "published_by",
    ],
    write_surfaces: &["表單 POST /workflows"],
    pages: &["/workflows"],
};

const GOV_WORKFLOW_PUBLISHED: LineageEntry = LineageEntry {
    event: "workflow_published",
    fields: &["id", "version", "reviewer", "published_at"],
    write_surfaces: &["表單 POST /workflows/{id}/{version}/publish"],
    pages: &["/workflows"],
};

const GOV_QUALITY_CASE_CREATED: LineageEntry = LineageEntry {
    event: "quality_case_created",
    fields: &[
        "id",
        "project_id",
        "kind",
        "title",
        "description",
        "owner",
        "due_date",
        "status",
        "created_at",
        "updated_at",
        "closure_note",
    ],
    write_surfaces: &["表單 POST /controls"],
    pages: &["/controls"],
};

const GOV_QUALITY_CASE_UPDATED: LineageEntry = LineageEntry {
    event: "quality_case_updated",
    fields: &["id", "status", "actor", "note", "updated_at"],
    write_surfaces: &["表單 POST /controls/{id}/status"],
    pages: &["/controls"],
};

const GOV_QUALITY_INSPECTION_RECORDED: LineageEntry = LineageEntry {
    event: "quality_inspection_recorded",
    fields: &[
        "id",
        "project_id",
        "reviewer",
        "created_at",
        "results(item_id, result, note, evidence_reference, issue_id)",
    ],
    write_surfaces: &["表單 POST /inspections"],
    pages: &["/inspections"],
};

const GOV_GATE_DECISION_RECORDED: LineageEntry = LineageEntry {
    event: "gate_decision_recorded",
    fields: &[
        "id",
        "project_id",
        "gate_id",
        "decision",
        "actor",
        "reason",
        "evidence_reference",
        "risk_acceptor",
        "remediation_due",
        "non_waivable",
        "created_at",
    ],
    write_surfaces: &["表單 POST /lifecycle"],
    pages: &["/lifecycle"],
};

/// 治理帳鏈（`EventPayload`）全部 9 個事件變體的血緣。
pub const GOVERNANCE_LINEAGE: &[LineageEntry] = &[
    GOV_PROJECT_CREATED,
    GOV_EVIDENCE_LINKED,
    GOV_EVIDENCE_REVIEWED,
    GOV_WORKFLOW_SAVED,
    GOV_WORKFLOW_PUBLISHED,
    GOV_QUALITY_CASE_CREATED,
    GOV_QUALITY_CASE_UPDATED,
    GOV_QUALITY_INSPECTION_RECORDED,
    GOV_GATE_DECISION_RECORDED,
];

const APR_WORKFLOW_CREATED: LineageEntry = LineageEntry {
    event: "workflow_created",
    fields: &[
        "definition.id",
        "definition.name",
        "stages[].id",
        "stages[].sla_seconds",
        "stages[].urgency",
        "stages[].on_timeout",
    ],
    write_surfaces: &["表單 POST /approvals/workflows"],
    pages: &["/approvals"],
};

const APR_WORKFLOW_PUBLISHED: LineageEntry = LineageEntry {
    event: "workflow_published",
    fields: &["workflow_id", "version"],
    write_surfaces: &["表單 POST /approvals/workflows/{id}/{version}/publish"],
    pages: &["/approvals"],
};

const APR_TASK_SUBMITTED: LineageEntry = LineageEntry {
    event: "task_submitted",
    fields: &[
        "task.id",
        "task.workflow_id@version",
        "task.stage_id",
        "task.urgency",
        "task.amount_cents",
        "task.platform",
        "task.classification",
        "task.risk_score",
        "task.due_at",
        "idempotency_key(選填)",
    ],
    write_surfaces: &[
        "表單 POST /approvals/submit",
        "API POST /api/v1/approvals（Idempotency-Key 可選）",
    ],
    pages: &["/approvals"],
};

const APR_TASK_TRANSITIONED: LineageEntry = LineageEntry {
    event: "task_transitioned",
    fields: &[
        "decision.task_id",
        "decision.action",
        "decision.from_state→to_state",
        "decision.from_stage→to_stage",
        "escalated_to",
        "note",
    ],
    write_surfaces: &["表單 POST /approvals/{id}/decide"],
    pages: &["/approvals"],
};

const APR_TASKS_EXPIRED: LineageEntry = LineageEntry {
    event: "tasks_expired",
    fields: &["task_ids"],
    write_surfaces: &["cleanup_expired（自動排程未建，IMP-020(b)）"],
    pages: &["—（無頁面；帳內狀態變更）"],
};

/// 核准帳鏈（`ApprovalEventPayload`）全部 5 個事件變體的血緣。
pub const APPROVAL_LINEAGE: &[LineageEntry] = &[
    APR_WORKFLOW_CREATED,
    APR_WORKFLOW_PUBLISHED,
    APR_TASK_SUBMITTED,
    APR_TASK_TRANSITIONED,
    APR_TASKS_EXPIRED,
];

/// 窮舉 match（無萬用臂）：新增 `EventPayload` 變體時此函式編譯失敗，
/// 強制補上血緣列——型別系統驅動的血緣鎖。
pub fn governance_lineage_for(payload: &EventPayload) -> &'static LineageEntry {
    match payload {
        EventPayload::ProjectCreated(_) => &GOV_PROJECT_CREATED,
        EventPayload::EvidenceLinked(_) => &GOV_EVIDENCE_LINKED,
        EventPayload::EvidenceReviewed(_) => &GOV_EVIDENCE_REVIEWED,
        EventPayload::WorkflowSaved(_) => &GOV_WORKFLOW_SAVED,
        EventPayload::WorkflowPublished(_) => &GOV_WORKFLOW_PUBLISHED,
        EventPayload::QualityCaseCreated(_) => &GOV_QUALITY_CASE_CREATED,
        EventPayload::QualityCaseUpdated(_) => &GOV_QUALITY_CASE_UPDATED,
        EventPayload::QualityInspectionRecorded(_) => &GOV_QUALITY_INSPECTION_RECORDED,
        EventPayload::GateDecisionRecorded(_) => &GOV_GATE_DECISION_RECORDED,
    }
}

/// 窮舉 match（無萬用臂）：新增 `ApprovalEventPayload` 變體時編譯失敗。
pub fn approval_lineage_for(payload: &ApprovalEventPayload) -> &'static LineageEntry {
    match payload {
        ApprovalEventPayload::WorkflowCreated { .. } => &APR_WORKFLOW_CREATED,
        ApprovalEventPayload::WorkflowPublished { .. } => &APR_WORKFLOW_PUBLISHED,
        ApprovalEventPayload::TaskSubmitted { .. } => &APR_TASK_SUBMITTED,
        ApprovalEventPayload::TaskTransitioned { .. } => &APR_TASK_TRANSITIONED,
        ApprovalEventPayload::TasksExpired { .. } => &APR_TASKS_EXPIRED,
    }
}

/// 事件標籤（與血緣列同一來源，單一事實）。
pub fn governance_payload_tag(payload: &EventPayload) -> &'static str {
    governance_lineage_for(payload).event
}

/// 核准事件標籤。
pub fn approval_payload_tag(payload: &ApprovalEventPayload) -> &'static str {
    approval_lineage_for(payload).event
}

/// 建構資料地圖 JSON（`ekpi-data-map-v1`）：純常數表序列化——
/// 無時間戳、無路徑、無環境輸入 → 同建構器逐位元組一致。
pub fn build_data_map() -> serde_json::Value {
    serde_json::json!({
        "schema": "ekpi-data-map-v1",
        "governance": GOVERNANCE_LINEAGE,
        "approvals": APPROVAL_LINEAGE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// 血緣表規模鎖：治理 9 個事件變體、核准 5 個——數字漂移即提醒
    /// 「enum 變體與血緣表不同步」（配對窮舉 match 為編譯級主鎖）。
    #[test]
    fn lineage_table_sizes_match_the_event_enums() {
        assert_eq!(GOVERNANCE_LINEAGE.len(), 9);
        assert_eq!(APPROVAL_LINEAGE.len(), 5);
    }

    /// 標籤唯一且為 snake_case——帳鏈 JSON 的 type 欄位以此為準。
    #[test]
    fn lineage_tags_are_unique_and_snake_case() {
        for table in [GOVERNANCE_LINEAGE, APPROVAL_LINEAGE] {
            let mut tags = BTreeSet::new();
            for entry in table {
                assert!(!entry.event.is_empty());
                assert!(
                    entry
                        .event
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
                    "tag must be snake_case: {}",
                    entry.event
                );
                assert!(tags.insert(entry.event), "duplicate tag {}", entry.event);
                assert!(!entry.fields.is_empty(), "{} must list fields", entry.event);
                assert!(
                    !entry.write_surfaces.is_empty(),
                    "{} must list write surfaces",
                    entry.event
                );
                assert!(!entry.pages.is_empty(), "{} must list pages", entry.event);
            }
        }
    }

    /// 決定性：同建構器兩次產生逐位元組一致的地圖（契約要求）。
    #[test]
    fn build_data_map_is_deterministic() {
        let first = build_data_map();
        let second = build_data_map();
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert_eq!(first["schema"], "ekpi-data-map-v1");
        assert_eq!(first["governance"].as_array().map(|a| a.len()), Some(9));
        assert_eq!(first["approvals"].as_array().map(|a| a.len()), Some(5));
    }
}
