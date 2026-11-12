#![forbid(unsafe_code)]

//! 流程符合性指標（conformance metrics）——自由工具鏈、零新增依賴、
//! 以**精確有理數**（u128 分子／分母，gcd 約分）表示，杜絕浮點誤差。
//!
//! - `fitness`（token replay 近似）：`1 − (missing + remaining) / total`
//!   - `missing`：日誌中無法被前置活動啟用的 token（前置活動從未出現時，
//!     其後繼活動的每一筆觀測各記一枚）。
//!   - `remaining`：模型宣告但日誌從未出現的活動（每個留下一枚未消費 token）。
//!   - `total`：模型活動的觀測數 + `remaining`；`total == 0`（空模型且空日誌）
//!     → `None`（不可測，**不回報 0 或 1 冒充**）。
//! - `precision`（逃脫邊）：`1 − escaping / observed_pairs`
//!   - 相鄰事件對（**兩端皆在模型活動內**）若不被 `precedence`／`parallel_pairs`
//!     （對稱）／`repeatable`／`repeat_parties`（不同主體）允許，即為逃脫邊。
//!   - 含未知活動的相鄰對**不計入**：未知活動由 replay 的 `Unconfirmed` 處理；
//!     此處不跨未知活動捏造相鄰（避免製造不存在的因果邊）。
//!   - 無可評對（`observed_pairs == 0`）→ `None`。
//!
//! 兩指標分母為 0 一律 `None`（不可測）。本模組為 library-only：可由 CLI／
//! 報告層引用，本身不對外服務（契約：CTR-DATA-009）。

use std::collections::BTreeMap;

use crate::process::ProcessModel;

/// 精確有理數指標（已 gcd 約分；分母恆 > 0，故 3/3 與 1/1 相等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metric {
    numerator: u128,
    denominator: u128,
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let rest = a % b;
        a = b;
        b = rest;
    }
    a
}

impl Metric {
    /// 建構並約分；分母為 0 代表不可測，回傳 `None`。
    pub fn new(numerator: u128, denominator: u128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let numerator = numerator.min(denominator);
        let divisor = gcd(numerator, denominator);
        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    pub fn numerator(&self) -> u128 {
        self.numerator
    }

    pub fn denominator(&self) -> u128 {
        self.denominator
    }

    /// 完全相符（分子＝分母；空分母不可表示，見 [`Metric::new`]）。
    pub fn is_perfect(&self) -> bool {
        self.numerator == self.denominator
    }

    /// 千分比（整數運算；粗粒度比較用）。
    pub fn permille(&self) -> u128 {
        self.numerator * 1000 / self.denominator
    }

    /// 百分比兩位小數（整數運算，四捨五入至分）。
    pub fn percent_2dp(&self) -> String {
        let hundredths = (self.numerator * 10_000 + self.denominator / 2) / self.denominator;
        format!("{}.{:02}%", hundredths / 100, hundredths % 100)
    }
}

/// 一次案例的符合性結果（原始計數全部保留，供審核與重現）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConformanceReport {
    /// `None` 僅在空模型且空日誌時（無 token 可評）。
    pub fitness: Option<Metric>,
    /// `None` 在無可評事件對時。
    pub precision: Option<Metric>,
    pub log_events: usize,
    pub model_activities: usize,
    pub missing_tokens: u128,
    pub remaining_tokens: u128,
    pub observed_pairs: u128,
    pub escaping_pairs: u128,
}

/// 對單一流程模型與事件序列計算 fitness／precision（純函式、決定性）。
pub fn conformance(
    model: &ProcessModel,
    events: &[crate::process::ReplayEvent],
) -> ConformanceReport {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for event in events {
        if model.activities.iter().any(|a| a == &event.activity) {
            *counts.entry(event.activity.as_str()).or_insert(0) += 1;
        }
    }

    // missing：前置活動從未出現 → 其後繼的每筆觀測缺一枚前置 token。
    let mut missing_tokens: u128 = 0;
    for pair in &model.precedence {
        if !counts.contains_key(pair.before.as_str()) {
            missing_tokens += counts.get(pair.after.as_str()).copied().unwrap_or(0) as u128;
        }
    }

    // remaining：模型活動在日誌中完全缺席。
    let mut remaining_tokens: u128 = 0;
    for activity in &model.activities {
        if !counts.contains_key(activity.as_str()) {
            remaining_tokens += 1;
        }
    }

    let observed: u128 = counts.values().map(|c| *c as u128).sum();
    let total = observed + remaining_tokens;
    // 未消費（remaining）與缺前置（missing）的 token 皆扣分：1 − (missing + remaining)/total。
    let fitness = Metric::new(
        total.saturating_sub(missing_tokens + remaining_tokens),
        total,
    );

    // precision：相鄰事件對是否落在模型允許的邊集合（對稱平行、多主體自環）。
    let mut observed_pairs: u128 = 0;
    let mut escaping_pairs: u128 = 0;
    for pair in events.windows(2) {
        let from = pair[0].activity.as_str();
        let to = pair[1].activity.as_str();
        if !model.activities.iter().any(|a| a == from) || !model.activities.iter().any(|a| a == to)
        {
            continue;
        }
        observed_pairs += 1;
        let precedence_ok = model
            .precedence
            .iter()
            .any(|p| p.before == from && p.after == to);
        let parallel_ok = model
            .parallel_pairs
            .iter()
            .any(|p| (p[0] == from && p[1] == to) || (p[1] == from && p[0] == to));
        let self_loop_ok = from == to
            && (model.repeatable
                || (model.repeat_parties.iter().any(|a| a == from)
                    && pair[0].actor != pair[1].actor));
        if !(precedence_ok || parallel_ok || self_loop_ok) {
            escaping_pairs += 1;
        }
    }
    let precision = Metric::new(observed_pairs - escaping_pairs, observed_pairs);

    ConformanceReport {
        fitness,
        precision,
        log_events: events.len(),
        model_activities: model.activities.len(),
        missing_tokens,
        remaining_tokens,
        observed_pairs,
        escaping_pairs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ReplayEvent;

    fn model() -> ProcessModel {
        serde_json::from_str(
            r#"{
                "process_id": "P-CONF",
                "title": "conformance",
                "activities": ["a", "b", "c"],
                "precedence": [
                    {"before": "a", "after": "b"},
                    {"before": "b", "after": "c"}
                ],
                "parallel_pairs": [],
                "forbidden_pairs": [],
                "repeatable": false
            }"#,
        )
        .unwrap()
    }

    fn ev(activity: &str) -> ReplayEvent {
        ReplayEvent {
            activity: activity.to_string(),
            ts: "2026-11-12".to_string(),
            actor: "tester".to_string(),
        }
    }

    fn ev_by(activity: &str, actor: &str) -> ReplayEvent {
        ReplayEvent {
            activity: activity.to_string(),
            ts: "2026-11-12".to_string(),
            actor: actor.to_string(),
        }
    }

    #[test]
    fn perfect_trace_is_one() {
        let report = conformance(&model(), &[ev("a"), ev("b"), ev("c")]);
        assert_eq!(report.fitness, Metric::new(1, 1));
        assert_eq!(report.precision, Metric::new(1, 1));
        assert_eq!(report.missing_tokens, 0);
        assert_eq!(report.remaining_tokens, 0);
        assert_eq!(report.escaping_pairs, 0);
        assert!(report.fitness.unwrap().is_perfect());
        assert_eq!(report.fitness.unwrap().percent_2dp(), "100.00%");
    }

    #[test]
    fn missing_predecessor_is_exact_rational() {
        // 只有 b、c：a 缺席 → b 缺前置 token（1 missing）、a 為未觀測活動（1 remaining）；
        // total = 2 觀測 + 1 未觀測 = 3；可正確重放者僅 c → fitness = 1/3。
        let report = conformance(&model(), &[ev("b"), ev("c")]);
        assert_eq!(report.missing_tokens, 1);
        assert_eq!(report.remaining_tokens, 1);
        assert_eq!(report.fitness, Metric::new(1, 3));
        assert_eq!(report.fitness.unwrap().numerator(), 1);
        assert_eq!(report.fitness.unwrap().denominator(), 3);
        assert_eq!(report.fitness.unwrap().percent_2dp(), "33.33%");
    }

    #[test]
    fn escaping_edge_lowers_precision() {
        // c 出現在 b 之前：邊 (c,b) 不被允許 → 逃脫邊。
        let report = conformance(&model(), &[ev("a"), ev("c"), ev("b"), ev("c")]);
        assert_eq!(report.observed_pairs, 3);
        assert_eq!(report.escaping_pairs, 2);
        assert_eq!(report.precision, Metric::new(1, 3));
    }

    #[test]
    fn empty_log_is_not_reported_as_perfect() {
        let report = conformance(&model(), &[]);
        assert_eq!(report.fitness, Metric::new(0, 3));
        assert_eq!(report.precision, None);
        assert_eq!(report.log_events, 0);
        assert!(!report.fitness.unwrap().is_perfect());
    }

    #[test]
    fn empty_model_and_empty_log_are_not_measurable() {
        let mut m = model();
        m.activities.clear();
        m.precedence.clear();
        let report = conformance(&m, &[]);
        assert_eq!(report.fitness, None, "無 token 可評 → 不回報 0 或 1");
        assert_eq!(report.precision, None);
    }

    #[test]
    fn single_event_has_no_pairs() {
        let report = conformance(&model(), &[ev("a")]);
        assert_eq!(report.precision, None);
        assert_eq!(report.observed_pairs, 0);
    }

    #[test]
    fn unknown_activities_are_excluded_from_fitness_and_pairs() {
        let report = conformance(&model(), &[ev("a"), ev("zzz"), ev("b"), ev("c")]);
        assert_eq!(report.log_events, 4);
        // windows(2) 的三對中，含 zzz 的兩對不可評（不跨未知活動捏造相鄰）。
        assert_eq!(report.observed_pairs, 1);
        assert_eq!(report.escaping_pairs, 0);
        assert_eq!(report.precision, Metric::new(1, 1));
        assert_eq!(report.fitness, Metric::new(1, 1));
    }

    #[test]
    fn repeatable_allows_self_loop() {
        let mut m = model();
        m.activities = vec!["a".to_string()];
        m.precedence.clear();
        m.repeatable = true;
        let report = conformance(&m, &[ev("a"), ev("a")]);
        assert_eq!(report.observed_pairs, 1);
        assert_eq!(report.escaping_pairs, 0);
        assert_eq!(report.precision, Metric::new(1, 1));
    }

    #[test]
    fn repeat_parties_allows_self_loop_only_across_actors() {
        let mut m = model();
        m.activities = vec!["approve".to_string()];
        m.precedence.clear();
        m.repeat_parties = vec!["approve".to_string()];
        let distinct = conformance(&m, &[ev_by("approve", "alice"), ev_by("approve", "bob")]);
        assert_eq!(distinct.escaping_pairs, 0, "不同主體雙簽合法");
        let same = conformance(&m, &[ev_by("approve", "alice"), ev_by("approve", "alice")]);
        assert_eq!(same.escaping_pairs, 1, "同一主體重複即逃脫邊");
    }

    #[test]
    fn parallel_pairs_are_symmetric() {
        let mut m = model();
        m.activities = vec!["x".to_string(), "y".to_string()];
        m.precedence.clear();
        m.parallel_pairs = vec![["x".to_string(), "y".to_string()]];
        let forward = conformance(&m, &[ev("x"), ev("y")]);
        let backward = conformance(&m, &[ev("y"), ev("x")]);
        assert_eq!(forward.escaping_pairs, 0);
        assert_eq!(backward.escaping_pairs, 0, "平行對兩個方向皆合法");
        assert_eq!(forward.precision, Metric::new(1, 1));
        assert_eq!(backward.precision, Metric::new(1, 1));
    }

    #[test]
    fn metric_reduces_fractions_so_equality_is_structural() {
        assert_eq!(Metric::new(3, 3), Metric::new(1, 1));
        assert_eq!(Metric::new(2, 4), Metric::new(1, 2));
        assert_eq!(Metric::new(0, 5), Metric::new(0, 1));
        assert_eq!(Metric::new(1, 0), None);
    }

    #[test]
    fn deterministic_and_integer_only() {
        let events = vec![ev("b"), ev("c")];
        let first = conformance(&model(), &events);
        let second = conformance(&model(), &events);
        assert_eq!(first, second);
        assert_eq!(first.fitness, Metric::new(1, 3));
        assert_eq!(first.fitness.unwrap().permille(), 333);
    }
}
