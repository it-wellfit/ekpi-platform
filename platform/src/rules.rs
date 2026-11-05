#![forbid(unsafe_code)]

//! R2-2：active 規則層。
//!
//! 生命周期防線（addendum A1/A8）：`review.status != active` 的規則
//! （`draft_ai`/`expert_returned`/…）在載入時被拒收並登記；
//! 判定引擎只消費 active 規則——draft 規則即使引用真實條款，也不至於影響裁决。
//!
//! 案件規則引用格式（addendum A7 綁定文件版本）：
//!
//! - `RULE-CHG-001`     → 於案件可用版本中解析綁定（必須唯一）
//! - `RULE-CHG-001@2.1` → 以 `@` 釘住來源文件版本（綁定必須存在）
//!
//! 解析失敗（規則所綁文件不在案件文件集）按規則 `decision.missing_evidence_result`
//! 處理（保守拒答，非錯誤中止）。
//! engine-spec v0.1.2 D-02 追加：`(rule_id, rule_version)` 重複即載入失敗；
//! 多筆 active 紀錄同時符合一件 binding → Ambiguous（保守拒答，不靜默取首筆）。

use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RuleDatabase {
    pub rules: Vec<RuleRecord>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleRecord {
    pub rule_id: String,
    pub rule_version: String,
    pub deontic: RuleDeontic,
    #[serde(default)]
    pub subject: RuleSubject,
    #[serde(default)]
    pub condition: Option<ConditionNode>,
    #[serde(default)]
    pub sequencing: RuleSequencing,
    #[serde(default)]
    pub exceptions: RuleExceptions,
    pub temporal: RuleTemporal,
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirement>,
    pub decision: RuleDecision,
    pub source: RuleSource,
    pub provenance: RuleProvenance,
    #[serde(default)]
    pub validation: Option<Validation>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleDeontic {
    pub modality: String,
    pub strength: String,
}

#[derive(Debug, Default, Clone, Deserialize, serde::Serialize)]
pub struct RuleSubject {
    pub entity_type: String,
    #[serde(default)]
    pub actor_role: Option<String>,
    #[serde(default)]
    pub scope: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default, serde::Serialize)]
pub struct RuleSequencing {
    #[serde(default)]
    pub required_before: Vec<String>,
    #[serde(default)]
    pub forbidden_after: Vec<String>,
    #[serde(default)]
    pub deadline: Option<serde_json::Value>,
    #[serde(default)]
    pub on_violation: Option<String>,
    #[serde(default)]
    pub loop_policy: Option<LoopPolicy>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct LoopPolicy {
    pub failures_cap: u32,
    pub on_exceeded: String,
    /// 超限時行為的條款引用（§ 形式，含前綴）
    pub doc_ref: String,
    #[serde(default)]
    pub first_iteration_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, serde::Serialize)]
pub struct RuleExceptions {
    #[serde(default)]
    pub exception_of: Vec<String>,
    #[serde(default)]
    pub overridden_by: Vec<String>,
    #[serde(default)]
    pub overrides: Vec<String>,
    #[serde(default)]
    pub conflicts_with: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleTemporal {
    pub effective_from: String,
    #[serde(default)]
    pub effective_to: Option<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    #[serde(default)]
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct EvidenceRequirement {
    pub evidence_type: String,
    pub min_count: u32,
    #[serde(default)]
    pub metadata_constraints: Vec<Predicate>,
    pub missing_evidence_result: String,
}

#[derive(Debug, Deserialize, Clone, serde::Serialize)]
pub struct Predicate {
    pub field: String,
    pub op: String,
    #[serde(default)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleDecision {
    pub method: String,
    pub missing_evidence_result: String,
    #[serde(default)]
    pub abstain_threshold: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleSource {
    pub document_id: String,
    pub document_version: String,
    #[serde(default)]
    pub clause_refs: Vec<ClauseRef>,
    pub verbatim_text: String,
    #[serde(default)]
    pub selectors: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ClauseRef {
    pub section: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub paragraph: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct RuleProvenance {
    pub origin: String,
    #[serde(default)]
    pub extraction: Option<serde_json::Value>,
    pub review: ReviewInfo,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ReviewInfo {
    pub status: String,
    #[serde(default)]
    pub reviewer: Option<String>,
    #[serde(default)]
    pub reviewed_at: Option<String>,
    #[serde(default)]
    pub change_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Validation {
    #[serde(default)]
    pub test_case_refs: Vec<String>,
    #[serde(default)]
    pub last_run: Option<serde_json::Value>,
}

/// 條件樹（schema v0.1.1 的 conditionNode）。
#[derive(Debug, Deserialize, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum ConditionNode {
    Leaf { predicate: Predicate },
    All { all: Vec<ConditionNode> },
    Any { any: Vec<ConditionNode> },
    Not { not: Box<ConditionNode> },
}

/// 案件規則引用的解析结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// 綁定命中山案文件集（可判定）。
    Bound,
    /// 案件引用帶 @pin 但語料庫無此綁定 → 版本釘住與可用文件集不一致。
    PinnedMismatch,
    /// 案件引用（無 pin）在案件文件集中找不到綁定 → 證據不可定位。
    Unlocatable,
    /// 多筆 active 紀錄同時符合此綁定（同 rule_id 不同 rule_version 綁同一
    /// 文件版本）→ 語義歧義；引擎保守拒答，不靜默取首筆。
    Ambiguous,
}

#[derive(Debug)]
pub struct ResolvedRule<'a> {
    /// 僅 `Resolution::Bound` 時為 Some；未解析綁定時引擎不得存取。
    pub record: Option<&'a RuleRecord>,
    pub declared_ref: String,
    pub resolution: Resolution,
}

impl RuleDatabase {
    pub fn load(path: &Path) -> Result<(RuleDatabase, Vec<String>), String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_str_checked(&text)
    }

    /// 從字串載入並執行形狀門禁（`load` 與單元測試共用的唯一定義點）。
    pub fn from_str_checked(text: &str) -> Result<(RuleDatabase, Vec<String>), String> {
        let db: RuleDatabase =
            serde_json::from_str(text).map_err(|e| format!("rules parse error: {e}"))?;
        let rejected = db.check_shape()?;
        Ok((db, rejected))
    }

    /// 資料層唯一性門禁（engine-spec v0.1.2 D-02）：`(rule_id, rule_version)`
    /// 重複即載入失敗——「同名不同義」被靜默後者覆蓋是判定不可重放的根因，
    /// 偏愛顯式版本命名而非 last-wins。
    /// 非 active 規則拒收並登記（投毒防線的審計痕跡）。
    fn check_shape(&self) -> Result<Vec<String>, String> {
        let mut seen = std::collections::BTreeSet::new();
        let mut notes = Vec::new();
        for r in &self.rules {
            let key = (r.rule_id.clone(), r.rule_version.clone());
            if !seen.insert(key) {
                return Err(format!(
                    "duplicate rule record {}@{} (last-wins overwrite forbidden — same id+version must be unique)",
                    r.rule_id, r.rule_version
                ));
            }
            if r.provenance.review.status != "active" {
                notes.push(format!(
                    "rejected: {}@{} review.status={} (non-active rules invisible to engine)",
                    r.rule_id, r.rule_version, r.provenance.review.status
                ));
            }
        }
        Ok(notes)
    }

    pub fn active_rules(&self) -> impl Iterator<Item = &RuleRecord> {
        self.rules
            .iter()
            .filter(|r| r.provenance.review.status == "active")
    }

    /// 規則庫是否登記過此 rule_id（不分 active/draft；供引擎 note 區分
    /// 「未登錄」與「已登錄但無可消費版本」，D-02 的顯式聲明主義）。
    pub fn known_rule_id(&self, rule_id: &str) -> bool {
        self.rules.iter().any(|r| r.rule_id == rule_id)
    }

    /// 解析案件 rules 引用（見模組註解的引用格式）。
    /// 多筆同時命中 → `Resolution::Ambiguous` 且 record 設 None（保守拒答）。
    pub fn resolve_declared<'a>(
        &'a self,
        declared_refs: &[String],
        case_doc_versions: &std::collections::BTreeMap<String, String>,
    ) -> Vec<ResolvedRule<'a>> {
        let mut out = Vec::new();
        let mut seen_refs = std::collections::BTreeSet::new();
        for reference in declared_refs {
            if !seen_refs.insert(reference.clone()) {
                continue; // 去重，保持聲明順序
            }
            let (rule_id, pin) = match reference.split_once('@') {
                Some((id, pin)) => (id.to_string(), Some(pin.to_string())),
                None => (reference.clone(), None),
            };
            let candidates: Vec<&RuleRecord> = self
                .active_rules()
                .filter(|r| r.rule_id == rule_id)
                .filter(
                    |r| match case_doc_versions.get(r.source.document_id.as_str()) {
                        Some(v) => *v == r.source.document_version,
                        None => false,
                    },
                )
                .filter(|r| match &pin {
                    Some(p) => *r.source.document_version == *p,
                    None => true,
                })
                .collect();
            let (resolution, record) = match candidates.len() {
                0 => (
                    if pin.is_some() {
                        Resolution::PinnedMismatch
                    } else {
                        Resolution::Unlocatable
                    },
                    None,
                ),
                1 => (Resolution::Bound, candidates.into_iter().next()),
                _ => (Resolution::Ambiguous, None),
            };
            out.push(ResolvedRule {
                record,
                declared_ref: reference.clone(),
                resolution,
            });
        }
        out
    }

    /// 直接衝突侦测：兩 active 規則互指 conflicts_with、且雙方無 override 鏈。
    /// 返回 (i, j) 索引對（於 resolved 列表內）。
    pub fn find_direct_conflicts<'a>(res: &'a [ResolvedRule<'a>]) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for i in 0..res.len() {
            for j in (i + 1)..res.len() {
                let (Some(ri), Some(rj)) = (res[i].record, res[j].record) else {
                    continue;
                };
                if res[i].resolution == Resolution::Bound
                    && res[j].resolution == Resolution::Bound
                    && ri.exceptions.conflicts_with.contains(&rj.rule_id)
                    && rj.exceptions.conflicts_with.contains(&ri.rule_id)
                    && ri.exceptions.overridden_by.is_empty()
                    && rj.exceptions.overridden_by.is_empty()
                    && ri.exceptions.overrides.is_empty()
                    && rj.exceptions.overrides.is_empty()
                {
                    pairs.push((i, j));
                }
            }
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> RuleDatabase {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("rules.json");
        let (db, rejected) = RuleDatabase::load(&path).unwrap();
        // 投毒防綫：ai_extracted draft 規則必被拒收。
        assert_eq!(
            rejected.len(),
            1,
            "expected exactly one rejected draft rule"
        );
        assert!(rejected[0].contains("draft_ai"));
        db
    }

    #[test]
    fn loads_and_rejects_draft_rules() {
        let db = fixture();
        assert!(db.rules.iter().any(|r| r.rule_id == "RULE-CAPA-012"
            && r.sequencing.loop_policy.as_ref().unwrap().failures_cap == 3));
        assert!(
            db.active_rules()
                .all(|r| r.provenance.review.status == "active")
        );
    }

    #[test]
    fn pinned_and_unpinned_resolution() {
        let db = fixture();
        let mut pool = std::collections::BTreeMap::new();
        pool.insert("SOP-CHANGE".to_string(), "2.1".to_string());
        pool.insert("EX-7".to_string(), "2.0".to_string());
        let refs = vec![
            "RULE-CHG-001@2.1".to_string(),
            "RULE-EX-007@2.0".to_string(),
        ];
        let resolved = db.resolve_declared(&refs, &pool);
        assert_eq!(resolved.len(), 2);
        assert!(matches!(resolved[0].resolution, Resolution::Bound));
        assert!(matches!(resolved[1].resolution, Resolution::Bound));
        assert_eq!(resolved[0].record.as_ref().unwrap().rule_version, "2.0");

        // EVD-001 情境：pool 無 EX-7 → Unlocatable。
        let mut pool2 = std::collections::BTreeMap::new();
        pool2.insert("SOP-CHANGE".to_string(), "3.2".to_string());
        let refs2 = vec!["RULE-CHG-001".to_string(), "RULE-EX-007".to_string()];
        let resolved2 = db.resolve_declared(&refs2, &pool2);
        assert!(matches!(resolved2[0].resolution, Resolution::Bound));
        assert!(matches!(resolved2[1].resolution, Resolution::Unlocatable));
    }

    #[test]
    fn conflict_pair_detected() {
        let db = fixture();
        let mut pool = std::collections::BTreeMap::new();
        pool.insert("QM-01".to_string(), "5.0".to_string());
        pool.insert("SOP-PURCH".to_string(), "4.4".to_string());
        let refs = vec!["RULE-PUR-002".to_string(), "RULE-QM-014".to_string()];
        let resolved = db.resolve_declared(&refs, &pool);
        let conflicts = RuleDatabase::find_direct_conflicts(&resolved);
        assert_eq!(conflicts.len(), 1, "expected one direct conflict pair");
    }

    #[test]
    fn duplicate_rule_id_version_is_load_failure() {
        let db = fixture();
        let dup = db.rules[0].clone();
        let poisoned = RuleDatabase {
            rules: vec![dup.clone(), dup],
        };
        let err = poisoned
            .check_shape()
            .expect_err("duplicate (rule_id, rule_version) must fail load");
        assert!(err.contains("duplicate rule record"), "{err}");
    }

    #[test]
    fn ambiguous_binding_is_not_silently_first_wins() {
        // 同 rule_id、不同 rule_version、綁定同一文件版本 → Ambiguous（保守拒答）。
        let db = fixture();
        let mut second = db.rules[0].clone();
        second.rule_version = format!("clone-{}", second.rule_version);
        let ambiguous = RuleDatabase {
            rules: vec![db.rules[0].clone(), second],
        };
        let mut pool = std::collections::BTreeMap::new();
        pool.insert("SOP-CHANGE".to_string(), "2.1".to_string());
        let resolved = ambiguous.resolve_declared(&["RULE-CHG-001".to_string()], &pool);
        assert_eq!(resolved.len(), 1);
        assert!(matches!(resolved[0].resolution, Resolution::Ambiguous));
        assert!(resolved[0].record.is_none(), "ambiguous must not resolve");
    }

    #[test]
    fn known_rule_id_distinguishes_registration() {
        let db = fixture();
        assert!(db.known_rule_id("RULE-CHG-001"));
        assert!(db.known_rule_id("RULE-CHG-001-DRAFT"));
        assert!(!db.known_rule_id("RULE-NOSUCH"));
    }
}
