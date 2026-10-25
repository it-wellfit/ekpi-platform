#![forbid(unsafe_code)]

//! R2-3：流程模型 token replay → 四分類符合性報告（conformance MVP）。
//!
//! 分類（addendum A4）：`Conform` 符合 / `Deviation` 偏差 / `Missing` 缺失 /
//! `Unconfirmed` 無法確認。偏差含「禁止對出現」與「前置序逆轉」——
//! 後者在引擎中映射為硬門檻 H-1 的違規（放行即不可交付）。
//!
//! 注意（反模擬）：MVP 的 replay 只標記問題，不出成因結論；
//! 判定權仍在規則層 + 引擎合併邏輯。

use serde::Deserialize;

use crate::docs::sha256_hex;

#[derive(Debug, Deserialize)]
pub struct ProcessLibrary {
    pub processes: Vec<ProcessModel>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessModel {
    pub process_id: String,
    #[serde(default)]
    pub title: String,
    pub activities: Vec<String>,
    #[serde(default)]
    pub precedence: Vec<PrecedencePair>,
    #[serde(default)]
    pub parallel_pairs: Vec<[String; 2]>,
    #[serde(default)]
    pub forbidden_pairs: Vec<[String; 2]>,
    #[serde(default)]
    pub repeatable: bool,
    /// 允許多主體（不同 actor 各一次）重複觀測的活動（如雙簽 approve）。
    /// 未列出的活動出現多筆觀測即程序缺陷（D-18）。
    #[serde(default)]
    pub repeat_parties: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PrecedencePair {
    pub before: String,
    pub after: String,
}

/// 四分類（A4）。`items` 為人讀描述，不改變類別本身。
#[derive(Debug, Clone)]
pub enum ReplayFinding {
    Conform,
    Deviation(Vec<String>),
    Missing(Vec<String>),
    Unconfirmed(Vec<String>),
}

impl ReplayFinding {
    /// 多問題並存時取最高嚴重度類別；Deviation > Missing > Unconfirmed > Conform。
    pub fn severity_rank(&self) -> u8 {
        match self {
            ReplayFinding::Deviation(_) => 3,
            ReplayFinding::Missing(_) => 2,
            ReplayFinding::Unconfirmed(_) => 1,
            ReplayFinding::Conform => 0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ReplayFinding::Conform => "conform",
            ReplayFinding::Deviation(_) => "deviation",
            ReplayFinding::Missing(_) => "missing",
            ReplayFinding::Unconfirmed(_) => "unconfirmed",
        }
    }

    pub fn items(&self) -> Vec<String> {
        match self {
            ReplayFinding::Conform => Vec::new(),
            ReplayFinding::Deviation(xs)
            | ReplayFinding::Missing(xs)
            | ReplayFinding::Unconfirmed(xs) => xs.clone(),
        }
    }
}

/// 一筆觀察事件（與 eval cases 的 events 欄位同構）。
#[derive(Debug, Clone)]
pub struct ReplayEvent {
    pub activity: String,
    pub ts: String,
    /// 觀測主體（角色）；D-18 以「活動×主體」配對判定重複執行。
    pub actor: String,
}

impl ProcessLibrary {
    pub fn load(path: &std::path::Path) -> Result<ProcessLibrary, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let lib: ProcessLibrary =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        lib.check_shape()?;
        Ok(lib)
    }

    fn check_shape(&self) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        for p in &self.processes {
            if !seen.insert(p.process_id.clone()) {
                return Err(format!("processes.json: duplicate id {}", p.process_id));
            }
            for pair in &p.precedence {
                if pair.before == pair.after {
                    return Err(format!(
                        "processes.json: {} self-precedence {}",
                        p.process_id, pair.before
                    ));
                }
            }
        }
        Ok(())
    }

    /// 選模型的確定性規則：活動覆蓋率最高者；平手取 process_id 字典序較小者。
    pub fn match_for<'a>(&'a self, events: &[ReplayEvent]) -> Option<&'a ProcessModel> {
        let mut best: Option<(&ProcessModel, usize)> = None;
        for model in &self.processes {
            let covered = events
                .iter()
                .filter(|e| model.activities.contains(&e.activity))
                .count();
            best = match best {
                None => Some((model, covered)),
                Some((_, best_cov)) if covered > best_cov => Some((model, covered)),
                Some((m, best_cov)) if covered == best_cov && model.process_id < m.process_id => {
                    Some((model, covered))
                }
                other => other,
            };
        }
        best.map(|(m, _)| m)
    }
}

/// Token replay for one case's observed events against one model.
///
/// 配對語義：對每個 precedence (before, after)，將兩活動的觀察事件各自依
/// (ts, 原始序) 排序後一一配對；第 k 個 after 必須有第 k 個 before 且
/// before_ts ≤ after_ts。違反 → Deviation；before 缺席而 after 現身 → Missing。
/// forbidden (a, b)：觀察到 a 在 b 之前 → Deviation（H-1 保護帶）。
fn replay(model: &ProcessModel, events: &[ReplayEvent]) -> ReplayFinding {
    let mut deviations: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    // Sort per activity by (ts, original index) for stable pairing.
    let mut by_activity: std::collections::BTreeMap<&str, Vec<(String, usize)>> =
        std::collections::BTreeMap::new();
    for (idx, e) in events.iter().enumerate() {
        by_activity
            .entry(e.activity.as_str())
            .or_default()
            .push((e.ts.clone(), idx));
    }
    for seq in by_activity.values_mut() {
        seq.sort();
    }

    for pair in &model.precedence {
        let before = by_activity.get(pair.before.as_str());
        let after = by_activity.get(pair.after.as_str());
        let (Some(before_seq), Some(after_seq)) = (before, after) else {
            // After observed without before → missing evidence of the required
            // predecessor. After absent → constraint not exercised.
            if let Some(after_seq) = after {
                if !after_seq.is_empty() {
                    missing.push(format!(
                        "{}: after `{}` observed without before `{}`",
                        model.process_id, pair.after, pair.before
                    ));
                }
            }
            continue;
        };
        // repeat_parties 的活動（如雙簽）視為**同一環節**的多主體記錄：
        // 全體記錄共用第一次出現的前置配對，不逐簽重複要求前置。
        let keyed_len = if model.repeat_parties.iter().any(|x| x == &pair.after) {
            1
        } else {
            after_seq.len()
        };
        for (k, (after_ts, _)) in after_seq.iter().enumerate().take(keyed_len) {
            match before_seq.get(k) {
                Some((before_ts, _)) => {
                    if before_ts.as_str() > after_ts.as_str() {
                        deviations.push(format!(
                            "{}: precedence `{}` > `{}` violated ({} after {})",
                            model.process_id, pair.before, pair.after, before_ts, after_ts
                        ));
                    }
                }
                None => missing.push(format!(
                    "{}: `{}` occurrence {k} lacks preceding `{}`",
                    model.process_id, pair.after, pair.before
                )),
            }
        }
    }

    for pair in &model.forbidden_pairs {
        let Some(a_seq) = by_activity.get(pair[0].as_str()) else {
            continue;
        };
        let Some(b_seq) = by_activity.get(pair[1].as_str()) else {
            continue;
        };
        for (a_ts, _) in a_seq {
            for (b_ts, _) in b_seq {
                if a_ts < b_ts {
                    deviations.push(format!(
                        "{}: forbidden pair `{} → {}` observed ({} before {})",
                        model.process_id, pair[0], pair[1], a_ts, b_ts
                    ));
                }
            }
        }
    }

    // engine-spec v0.1.2 D-18（修訂）：非可重複流程的重複觀測缺陷——
    // (a) 同一活動由**同一主體**重複執行（無論活動為何）一律缺陷；
    // (b) 同一活動出現多筆**不同主體**觀測，僅當活動未列入
    //     `repeat_parties`（多簽白名單）時為缺陷（如 execute_change
    //     被兩人各執行一次 ＝ 程序缺陷；approve_change 雙簽 ＝ 合法）。
    if !model.repeatable {
        let mut per_actor_counts: std::collections::BTreeMap<(&str, &str), usize> =
            std::collections::BTreeMap::new();
        for (activity, actor) in events
            .iter()
            .map(|e| (e.activity.as_str(), e.actor.as_str()))
        {
            *per_actor_counts.entry((activity, actor)).or_default() += 1;
        }
        for ((activity, actor), count) in &per_actor_counts {
            if *count >= 2 {
                deviations.push(format!(
                    "{}: non-repeatable activity `{activity}` repeated by same actor `{actor}` ({count} times)",
                    model.process_id
                ));
            }
        }
        for (name, seq) in &by_activity {
            if seq.len() >= 2 && !model.repeat_parties.iter().any(|x| x == name) {
                deviations.push(format!(
                    "{}: non-repeatable activity `{}` observed {} times",
                    model.process_id,
                    name,
                    seq.len()
                ));
            }
        }
    }

    if !deviations.is_empty() {
        return ReplayFinding::Deviation(deviations);
    }
    let unconfirmed: Vec<String> = events
        .iter()
        .filter(|e| !model.activities.contains(&e.activity))
        .map(|e| {
            format!(
                "{}: activity `{}` outside model",
                model.process_id, e.activity
            )
        })
        .collect();
    if !missing.is_empty() {
        return ReplayFinding::Missing(missing);
    }
    if !unconfirmed.is_empty() {
        return ReplayFinding::Unconfirmed(unconfirmed);
    }
    ReplayFinding::Conform
}

/// Model-layer converse check: analyze one model against one case (helper for
/// engine + selftest).
pub fn replay_case(model: &ProcessModel, events: &[ReplayEvent]) -> ReplayFinding {
    replay(model, events)
}

/// 事件序列雜湊（snapshot 層的重放憑據）；活動時點與主體一併入列，
/// D-18 的活動×主體判定責成此憑據可重放。
pub fn events_sha256(events: &[ReplayEvent]) -> String {
    let mut canon = String::new();
    for e in events {
        canon.push_str(&format!("{}|{}|{}\n", e.activity, e.ts, e.actor));
    }
    sha256_hex(canon.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> ProcessModel {
        serde_json::from_str(
            r#"{
                "process_id": "P-TEST",
                "title": "test",
                "activities": ["approve_change", "execute_change", "risk_assessment_completed"],
                "precedence": [
                    {"before": "approve_change", "after": "execute_change"},
                    {"before": "risk_assessment_completed", "after": "approve_change"}
                ],
                "parallel_pairs": [],
                "forbidden_pairs": [["execute_change", "approve_change"]],
                "repeatable": false
            }"#,
        )
        .unwrap()
    }

    fn ev(activity: &str, ts: &str) -> ReplayEvent {
        ReplayEvent {
            activity: activity.to_string(),
            ts: ts.to_string(),
            actor: String::new(),
        }
    }

    #[test]
    fn conform_path() {
        // 危評(3/1) → 核准(3/4) → 執行(3/5)：合法。
        let events = vec![
            ev("risk_assessment_completed", "2026-03-01"),
            ev("approve_change", "2026-03-04"),
            ev("execute_change", "2026-03-05"),
        ];
        let finding = replay(&model(), &events);
        assert!(matches!(finding, ReplayFinding::Conform), "{finding:?}");
    }

    #[test]
    fn deviation_on_execute_before_approve() {
        let events = vec![
            ev("execute_change", "2026-03-01"),
            ev("approve_change", "2026-03-02"),
        ];
        let finding = replay(&model(), &events);
        match finding {
            ReplayFinding::Deviation(items) => {
                assert!(items.iter().any(|i| i.contains("forbidden pair")));
                assert!(items.iter().any(|i| i.contains("precedence")));
            }
            other => panic!("expected deviation, got {other:?}"),
        }
    }

    #[test]
    fn missing_on_approve_absent() {
        let events = vec![ev("execute_change", "2026-03-01")];
        let finding = replay(&model(), &events);
        assert!(matches!(finding, ReplayFinding::Missing(_)), "{finding:?}");
    }

    // engine-spec v0.1.2 D-18。
    #[test]
    fn repeated_activity_in_non_repeatable_model_is_deviation() {
        let events = vec![
            ev("execute_change", "2026-03-01"),
            ev("execute_change", "2026-03-02"),
        ];
        let finding = replay(&model(), &events);
        match finding {
            ReplayFinding::Deviation(items) => {
                assert!(
                    items
                        .iter()
                        .any(|i| i
                            .contains("non-repeatable activity `execute_change` observed 2 times")),
                    "repeat finding missing: {items:?}"
                );
            }
            other => panic!("expected deviation, got {other:?}"),
        }
    }

    #[test]
    fn repeatable_model_admits_iterations() {
        let capa: ProcessModel = serde_json::from_str(
            r#"{
                "process_id": "P-CAPA-T",
                "title": "loop",
                "activities": ["corrective_action_executed", "verification_failed"],
                "precedence": [
                    {"before": "corrective_action_executed", "after": "verification_failed"}
                ],
                "parallel_pairs": [],
                "forbidden_pairs": [],
                "repeatable": true
            }"#,
        )
        .unwrap();
        let events = vec![
            ev("corrective_action_executed", "2026-01-05"),
            ev("verification_failed", "2026-01-08"),
            ev("corrective_action_executed", "2026-01-10"),
            ev("verification_failed", "2026-01-12"),
        ];
        let finding = replay(&capa, &events);
        assert!(matches!(finding, ReplayFinding::Conform), "{finding:?}");
    }

    // D-18 修訂：(b) 多主體重複 —— 白名單內合法（雙簽）、白名單外缺陷。
    #[test]
    fn multi_party_repeats_governed_by_repeat_parties() {
        let dual_sign_events = vec![ev("approve_change", "2026-03-02"), {
            let mut e = ev("approve_change", "2026-03-03");
            e.actor = "dept_head".to_string();
            e
        }];
        // 模型本身無白名單 → 缺陷。
        let finding = replay(&model(), &dual_sign_events);
        assert!(
            matches!(finding, ReplayFinding::Deviation(_)),
            "{finding:?}"
        );

        // 白名單放行 approve_change 雙簽 → Conform。
        let mut m = model();
        m.repeat_parties = vec!["approve_change".to_string()];
        let events = vec![
            {
                let mut e = ev("risk_assessment_completed", "2026-03-01");
                e.actor = "QA_manager".to_string();
                e
            },
            ev("approve_change", "2026-03-02"),
            {
                let mut e = ev("approve_change", "2026-03-03");
                e.actor = "dept_head".to_string();
                e
            },
            {
                let mut e = ev("execute_change", "2026-03-04");
                e.actor = "prod_line".to_string();
                e
            },
        ];
        let finding = replay(&m, &events);
        assert!(matches!(finding, ReplayFinding::Conform), "{finding:?}");
    }

    // D-18 修訂：(a) 同主體重複執行 → 一律缺陷。
    #[test]
    fn same_actor_repeat_is_always_defect() {
        let events = vec![
            {
                let mut e = ev("execute_change", "2026-03-03");
                e.actor = "prod_line".to_string();
                e
            },
            {
                let mut e = ev("execute_change", "2026-03-04");
                e.actor = "prod_line".to_string();
                e
            },
        ];
        let finding = replay(&model(), &events);
        match finding {
            ReplayFinding::Deviation(items) => assert!(
                items
                    .iter()
                    .any(|i| i.contains("repeated by same actor `prod_line`")),
                "{items:?}"
            ),
            other => panic!("expected deviation, got {other:?}"),
        }
    }
}
