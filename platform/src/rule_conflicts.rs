#![forbid(unsafe_code)]

//! 規則衝突分析（rule-conflict analysis，library-only）——免費 SOTA 之
//! 專家系統衝突檢查：**參照完整性 ＋ 宣告一致性 ＋ 保守語義重疊**。
//!
//! 與既有 `RuleDatabase::find_direct_conflicts` 互補而非重複：
//! 後者服務引擎（只用於「已解析案件引用」的執行期直接衝突）；
//! 本模組服務**規則庫作者與審閱者**（全庫、載入期/CI 期），
//! 專抓「宣告漏洞」與「介於衝突與已解決之間」的靜默縫隙。
//!
//! 檢查碼（穩定識別）：
//! - `RC-01` **Error** 自指：關係清單（conflicts_with／exception_of／overrides／
//!   overridden_by／supersedes／superseded_by）指向自己。
//! - `RC-02` **Error** 懸空參照：關係清單指向庫中不存在的 rule_id（跨全部記錄，
//!   含非 active——參照完整性不分生命週期）。
//! - `RC-03` **Warning** 衝突宣告不對稱：A.conflicts_with 含 B，但 B 未回指。
//! - `RC-04` **Warning** 覆寫鏈不對稱：A.overrides 含 B 但 B.overridden_by 缺 A（雙向檢查）。
//! - `RC-05` **Warning** 取代鏈不對稱：A.superseded_by=B 但 B.supersedes 缺 A（雙向檢查）。
//! - `RC-06` **Warning** 取代時間倒錯：被取代者的生效日晚於取代者。
//! - `RC-07` **Warning** 循環覆寫宣告：互相 conflicts_with **且互相 overrides**
//!   （雙方皆宣稱勝出）——引擎的全空判定會把它從直接衝突排除，卻沒有任何
//!   裁決來源，落入靜默縫隙（正確模式：敗方 overridden_by ＋勝方 overrides）。
//! - `RC-08` **Warning** 未宣告的禁制對立：雙方 active、同 subject（entity_type＋
//!   actor_role＋scope 全等）且同 condition（結構等值），modality 一為
//!   prohibition、另一非 prohibition，且雙方無任何關係宣告。
//! - `RC-09` **Info** 同觸發不同執行（**群組級**，每群一筆）：同 subject 同
//!   condition 同 modality 的 active 群組 ≥2 條，且 `on_violation` 值 ≥2 種
//!   （審閱者確認是否刻意並存；群組化避免兩兩組合爆炸）。
//!
//! 反模擬鐵律：語義檢查一律**保守**——只做結構可判定的比較（等值／集合歸屬／
//! 日期字串序），不做條件樹的可滿足性推理；無法判定者不生成發現（寧缺勿濫）。
//! 一致性掃描僅涵蓋 `review.status == "active"` 的記錄；RC-01／RC-02 掃描全部。

use serde_json::Value;

use crate::rules::{ConditionNode, RuleRecord, RuleSubject};

/// 發現嚴重度（Error > Warning > Info）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 一筆衝突分析發現（`rules` 為 1–2 個 `id@version` 標籤，依檢查碼排序後固定）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub severity: Severity,
    pub rules: Vec<String>,
    pub detail: String,
}

fn label(rule: &RuleRecord) -> String {
    format!("{}@{}", rule.rule_id, rule.rule_version)
}

fn is_active(rule: &RuleRecord) -> bool {
    rule.provenance.review.status == "active"
}

fn condition_value(condition: &Option<ConditionNode>) -> Value {
    serde_json::to_value(condition).expect("condition serialization cannot fail")
}

fn subjects_equal(a: &RuleSubject, b: &RuleSubject) -> bool {
    a.entity_type == b.entity_type && a.actor_role == b.actor_role && a.scope == b.scope
}

fn conditions_equal(a: &Option<ConditionNode>, b: &Option<ConditionNode>) -> bool {
    condition_value(a) == condition_value(b)
}

/// 關係清單（欄位名，id 清單）——供自指與懸空掃描共用。
fn relationship_lists(rule: &RuleRecord) -> Vec<(&'static str, Vec<&String>)> {
    let mut lists: Vec<(&'static str, Vec<&String>)> = vec![
        (
            "conflicts_with",
            rule.exceptions.conflicts_with.iter().collect(),
        ),
        (
            "exception_of",
            rule.exceptions.exception_of.iter().collect(),
        ),
        ("overrides", rule.exceptions.overrides.iter().collect()),
        (
            "overridden_by",
            rule.exceptions.overridden_by.iter().collect(),
        ),
        ("supersedes", rule.temporal.supersedes.iter().collect()),
    ];
    if let Some(superseded_by) = &rule.temporal.superseded_by {
        lists.push(("superseded_by", vec![superseded_by]));
    }
    lists
}

/// 全庫分析（決定性：輸出以 (code, rules) 排序）。
pub fn analyze(rules: &[RuleRecord]) -> Vec<Finding> {
    let mut findings = Vec::new();
    let known_ids: std::collections::BTreeSet<&str> =
        rules.iter().map(|r| r.rule_id.as_str()).collect();

    // RC-01／RC-02：參照完整性（全部記錄）。
    for rule in rules {
        for (field, targets) in relationship_lists(rule) {
            for target in targets {
                if target == &rule.rule_id {
                    findings.push(Finding {
                        code: "RC-01",
                        severity: Severity::Error,
                        rules: vec![label(rule)],
                        detail: format!("{field} references the rule itself"),
                    });
                } else if !known_ids.contains(target.as_str()) {
                    findings.push(Finding {
                        code: "RC-02",
                        severity: Severity::Error,
                        rules: vec![label(rule), target.clone()],
                        detail: format!("{field} references unknown rule `{target}`"),
                    });
                }
            }
        }
    }

    let active: Vec<&RuleRecord> = rules.iter().filter(|r| is_active(r)).collect();

    // 對稱性與縫隙檢查（僅 active）。
    for i in 0..active.len() {
        for j in (i + 1)..active.len() {
            let (a, b) = (active[i], active[j]);
            let a_conflicts_b = a.exceptions.conflicts_with.contains(&b.rule_id);
            let b_conflicts_a = b.exceptions.conflicts_with.contains(&a.rule_id);
            let a_overrides_b = a.exceptions.overrides.contains(&b.rule_id);
            let b_overridden_by_a = b.exceptions.overridden_by.contains(&a.rule_id);
            let a_overridden_by_b = a.exceptions.overridden_by.contains(&b.rule_id);
            let b_overrides_a = b.exceptions.overrides.contains(&a.rule_id);

            if a_conflicts_b && !b_conflicts_a {
                findings.push(Finding {
                    code: "RC-03",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "conflicts_with is declared one-way only".to_string(),
                });
            }
            if a_overrides_b && !b_overridden_by_a {
                findings.push(Finding {
                    code: "RC-04",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "overrides is declared without the reciprocal overridden_by"
                        .to_string(),
                });
            }
            if a_overridden_by_b && !b_overrides_a {
                findings.push(Finding {
                    code: "RC-04",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "overridden_by is declared without the reciprocal overrides"
                        .to_string(),
                });
            }
            if a_conflicts_b && b_conflicts_a && a_overrides_b && b_overrides_a {
                findings.push(Finding {
                    code: "RC-07",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "mutual conflict where both sides declare overrides (circular; no arbitration source)"
                        .to_string(),
                });
            }

            // 語義重疊（保守）：同 subject 同 condition。
            if subjects_equal(&a.subject, &b.subject)
                && conditions_equal(&a.condition, &b.condition)
                && !a_conflicts_b
                && !b_conflicts_a
                && !a_overrides_b
                && !b_overrides_a
                && !a_overridden_by_b
                && !b_overridden_by_a
            {
                let a_proh = a.deontic.modality == "prohibition";
                let b_proh = b.deontic.modality == "prohibition";
                if a_proh != b_proh {
                    findings.push(Finding {
                        code: "RC-08",
                        severity: Severity::Warning,
                        rules: vec![label(a), label(b)],
                        detail: "identical trigger with opposing deontic force (prohibition vs non-prohibition), no relationship declared".to_string(),
                    });
                }
            }
        }
    }

    // RC-09（群組級，避免兩兩爆炸）：同一 (subject, condition, modality) 群組內
    // active 規則 ≥2 且 `on_violation` 出現 ≥2 種不同值 → 一筆 Info。
    let mut groups: std::collections::BTreeMap<String, Vec<&RuleRecord>> =
        std::collections::BTreeMap::new();
    for rule in &active {
        let key = serde_json::to_string(&serde_json::json!([
            rule.subject.entity_type,
            rule.subject.actor_role,
            rule.subject.scope,
            condition_value(&rule.condition),
            rule.deontic.modality,
        ]))
        .expect("group key serialization cannot fail");
        groups.entry(key).or_default().push(rule);
    }
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let mut effects: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for member in members {
            *effects
                .entry(
                    member
                        .sequencing
                        .on_violation
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                )
                .or_insert(0) += 1;
        }
        if effects.len() < 2 {
            continue;
        }
        let mut rules: Vec<String> = members.iter().map(|m| label(m)).collect();
        rules.sort();
        let breakdown = effects
            .iter()
            .map(|(effect, count)| format!("{effect}×{count}"))
            .collect::<Vec<_>>()
            .join(", ");
        findings.push(Finding {
            code: "RC-09",
            severity: Severity::Info,
            rules,
            detail: format!(
                "{} rules share the same trigger; on_violation diverges ({breakdown})",
                members.len()
            ),
        });
    }

    // 取代鏈（僅 active）：對稱性與時間序。
    for a in &active {
        // 懸空參照（None）已由 RC-02 全記錄掃描處理。
        if let Some(target_id) = &a.temporal.superseded_by
            && let Some(b) = active.iter().find(|r| &r.rule_id == target_id)
        {
            if !b.temporal.supersedes.contains(&a.rule_id) {
                findings.push(Finding {
                    code: "RC-05",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "superseded_by is declared without the reciprocal supersedes"
                        .to_string(),
                });
            }
            if b.temporal.effective_from < a.temporal.effective_from {
                findings.push(Finding {
                    code: "RC-06",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: format!(
                        "superseder effective_from {} precedes superseded effective_from {}",
                        b.temporal.effective_from, a.temporal.effective_from
                    ),
                });
            }
        }
        for target_id in &a.temporal.supersedes {
            if let Some(b) = active.iter().find(|r| &r.rule_id == target_id)
                && b.temporal.superseded_by.as_deref() != Some(a.rule_id.as_str())
            {
                findings.push(Finding {
                    code: "RC-05",
                    severity: Severity::Warning,
                    rules: vec![label(a), label(b)],
                    detail: "supersedes is declared without the reciprocal superseded_by"
                        .to_string(),
                });
            }
        }
    }

    findings.sort_by(|x, y| {
        x.code
            .cmp(y.code)
            .then_with(|| x.rules.cmp(&y.rules))
            .then_with(|| x.detail.cmp(&y.detail))
    });
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(id: &str, modality: &str, status: &str) -> Value {
        serde_json::json!({
            "rule_id": id,
            "rule_version": "1.0",
            "deontic": {"modality": modality, "strength": "mandatory"},
            "temporal": {"effective_from": "2026-01-01"},
            "decision": {"method": "deterministic_rule_check",
                          "missing_evidence_result": "insufficient_evidence"},
            "source": {"document_id": "D1", "document_version": "1.0", "verbatim_text": "x"},
            "provenance": {"origin": "authoritative_text",
                            "review": {"status": status, "reviewer": null,
                                        "reviewed_at": null, "change_note": null}}
        })
    }

    fn decode(value: Value) -> RuleRecord {
        serde_json::from_value(value).expect("test rule must decode")
    }

    #[test]
    fn clean_pair_produces_no_findings() {
        let rules = vec![
            decode(base("R-1", "obligation", "active")),
            decode(base("R-2", "permission", "active")),
        ];
        assert!(analyze(&rules).is_empty());
    }

    #[test]
    fn self_reference_is_an_error() {
        let mut value = base("R-1", "obligation", "active");
        value["exceptions"]["conflicts_with"] = serde_json::json!(["R-1"]);
        let findings = analyze(&[decode(value)]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-01");
        assert_eq!(findings[0].severity, Severity::Error);
    }

    #[test]
    fn dangling_reference_is_an_error_even_for_non_active() {
        let mut value = base("R-1", "obligation", "draft_ai");
        value["exceptions"]["exception_of"] = serde_json::json!(["R-NOPE"]);
        let findings = analyze(&[decode(value)]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-02");
        assert_eq!(findings[0].rules, vec!["R-1@1.0", "R-NOPE"]);
    }

    #[test]
    fn one_way_conflict_is_a_warning() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["conflicts_with"] = serde_json::json!(["R-2"]);
        let findings = analyze(&[decode(a), decode(base("R-2", "obligation", "active"))]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-03");
    }

    #[test]
    fn one_way_override_chain_is_a_warning() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["overrides"] = serde_json::json!(["R-2"]);
        let findings = analyze(&[decode(a), decode(base("R-2", "obligation", "active"))]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-04");
    }

    #[test]
    fn mutual_conflict_with_two_sided_override_is_resolved() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["conflicts_with"] = serde_json::json!(["R-2"]);
        a["exceptions"]["overridden_by"] = serde_json::json!(["R-2"]);
        let mut b = base("R-2", "obligation", "active");
        b["exceptions"]["conflicts_with"] = serde_json::json!(["R-1"]);
        b["exceptions"]["overrides"] = serde_json::json!(["R-1"]);
        let findings = analyze(&[decode(a), decode(b)]);
        assert!(
            findings.is_empty(),
            "fully declared override chain must not be flagged: {findings:?}"
        );
    }

    #[test]
    fn one_sided_override_on_mutual_conflict_flags_asymmetry() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["conflicts_with"] = serde_json::json!(["R-2"]);
        a["exceptions"]["overridden_by"] = serde_json::json!(["R-2"]);
        let mut b = base("R-2", "obligation", "active");
        b["exceptions"]["conflicts_with"] = serde_json::json!(["R-1"]);
        let findings = analyze(&[decode(a), decode(b)]);
        // 單側覆寫宣告的不對稱由 RC-04 專責；RC-07（循環）不應誤報。
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].code, "RC-04");
    }

    #[test]
    fn circular_override_claim_is_flagged() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["conflicts_with"] = serde_json::json!(["R-2"]);
        a["exceptions"]["overrides"] = serde_json::json!(["R-2"]);
        let mut b = base("R-2", "obligation", "active");
        b["exceptions"]["conflicts_with"] = serde_json::json!(["R-1"]);
        b["exceptions"]["overrides"] = serde_json::json!(["R-1"]);
        let findings = analyze(&[decode(a), decode(b)]);
        assert!(findings.iter().any(|f| f.code == "RC-07"), "{findings:?}");
    }

    #[test]
    fn supersession_time_inversion_is_flagged() {
        let mut a = base("R-1", "obligation", "active");
        a["temporal"]["effective_from"] = serde_json::json!("2026-06-01");
        a["temporal"]["superseded_by"] = serde_json::json!("R-2");
        let mut b = base("R-2", "obligation", "active");
        b["temporal"]["effective_from"] = serde_json::json!("2026-01-01");
        b["temporal"]["supersedes"] = serde_json::json!(["R-1"]);
        let findings = analyze(&[decode(a), decode(b)]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-06");
    }

    #[test]
    fn reciprocal_supersession_with_later_start_is_clean() {
        let mut a = base("R-1", "obligation", "active");
        a["temporal"]["effective_from"] = serde_json::json!("2026-01-01");
        a["temporal"]["superseded_by"] = serde_json::json!("R-2");
        let mut b = base("R-2", "obligation", "active");
        b["temporal"]["effective_from"] = serde_json::json!("2026-06-01");
        b["temporal"]["supersedes"] = serde_json::json!(["R-1"]);
        assert!(analyze(&[decode(a), decode(b)]).is_empty());
    }

    #[test]
    fn opposing_force_on_identical_trigger_is_flagged() {
        let mut a = base("R-1", "obligation", "active");
        a["condition"] = serde_json::json!({"all": [{"predicate": {"field": "activity.name", "op": "eq", "value": "x"}}]});
        let mut b = base("R-2", "prohibition", "active");
        b["condition"] = a["condition"].clone();
        let findings = analyze(&[decode(a), decode(b)]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "RC-08");
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn effect_divergence_is_informational_and_effect_equal_is_clean() {
        let mut a = base("R-1", "obligation", "active");
        a["condition"] = serde_json::json!({"all": [{"predicate": {"field": "activity.name", "op": "eq", "value": "x"}}]});
        a["sequencing"]["on_violation"] = serde_json::json!("block");
        let mut b = base("R-2", "obligation", "active");
        b["condition"] = a["condition"].clone();
        b["sequencing"]["on_violation"] = serde_json::json!("warn");
        let divergent = analyze(&[decode(a.clone()), decode(b.clone())]);
        assert_eq!(divergent.len(), 1);
        assert_eq!(divergent[0].code, "RC-09");
        assert_eq!(divergent[0].severity, Severity::Info);
        let mut b_same = b;
        b_same["sequencing"]["on_violation"] = serde_json::json!("block");
        // 效果相同、無關係宣告 → 無發現（同義重複不濫報）。
        assert!(analyze(&[decode(a), decode(b_same)]).is_empty());
    }

    #[test]
    fn non_active_rules_are_excluded_from_consistency_checks() {
        let mut a = base("R-1", "obligation", "active");
        a["condition"] = serde_json::json!({"all": [{"predicate": {"field": "activity.name", "op": "eq", "value": "x"}}]});
        let mut b = base("R-2", "prohibition", "draft_ai");
        b["condition"] = a["condition"].clone();
        assert!(
            analyze(&[decode(a), decode(b)]).is_empty(),
            "draft rules cannot conflict operationally (engine never consumes them)"
        );
    }

    #[test]
    fn deterministic_output_order() {
        let mut a = base("R-1", "obligation", "active");
        a["exceptions"]["conflicts_with"] = serde_json::json!(["R-2"]);
        let rules = vec![decode(a), decode(base("R-2", "obligation", "active"))];
        let first = analyze(&rules);
        let second = analyze(&rules);
        assert_eq!(first, second);
    }

    #[test]
    fn real_corpus_yields_exactly_the_reviewer_set() {
        // 實務防護：對真實 data/rules.json 只允許一組已知、可解釋的發現；
        // 若語料變動，此測試會要求人工檢視（防止無聲語義漂移）。
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("rules.json");
        let (db, _rejected) = crate::rules::RuleDatabase::load(&path).expect("rules load");
        let findings = analyze(&db.rules);
        let codes: Vec<&str> = findings.iter().map(|f| f.code).collect();
        assert_eq!(
            codes,
            vec!["RC-08", "RC-09", "RC-09"],
            "unexpected findings on the real corpus: {findings:#?}"
        );
        let opposing = &findings[0];
        assert_eq!(
            opposing.rules,
            vec!["RULE-CHG-002@3.0", "RULE-CLOSE-GATE@1.0"]
        );
        // G1：execute_change 觸發群（7 條 active；含三種 on_violation）。
        assert_eq!(findings[1].rules.len(), 7);
        assert_eq!(findings[1].rules[0], "RULE-CHG-001@2.0");
        // G4a：purchase_requested 觸發群（obligation；5 條）。
        assert_eq!(findings[2].rules.len(), 5);
        assert!(findings[2].rules.contains(&"RULE-QM-014@5.0".to_string()));
    }
}
