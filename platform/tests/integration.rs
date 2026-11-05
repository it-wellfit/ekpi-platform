#![forbid(unsafe_code)]

//! 整合測試：真實 `platform/data/` + 真實種子案例端到端。
//!
//! 斷言全部來自載入的資料與引擎執行，不注入 mock 期望值以外路徑；
//! 案例期望以 `seed_cases.jsonl` 的 `expected_verdict` / `citations` 為準。

use std::collections::BTreeSet;
use std::path::Path;

use ekp_eval::types;
use ekp_platform::engine::Platform;

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn seed_cases_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("eval").join("cases").join("seed_cases.jsonl"))
        .expect("manifest root has a parent")
}

fn platform_fixture() -> Platform {
    let mut platform = Platform::load(&data_dir()).expect("platform data loads");
    platform.retrieval_threshold = 0.70;
    platform
}

fn cases_fixture() -> Vec<types::Case> {
    let (cases, warnings) = types::load_cases(&seed_cases_path()).expect("seed cases load");
    // 形狀告警不阻斷（loader 語義），但不得有任何告警——有即資料退化。
    assert!(
        warnings.is_empty(),
        "seed cases carry shape warnings: {warnings:?}"
    );
    cases
}

#[test]
fn nine_seed_cases_match_expected_verdict_and_citations() {
    let platform = platform_fixture();
    let cases = cases_fixture();
    assert_eq!(cases.len(), 9, "seed set must be the 9 registered cases");

    for case in &cases {
        let outcome = platform.decide(case);
        let exp = case.expected().expect("expected_verdict known");
        assert_eq!(
            outcome.verdict,
            exp,
            "case {}: verdict expected {exp:?} got {:?}; notes {}",
            case.case_id,
            outcome.verdict,
            outcome.notes.join(" | "),
        );

        let got: BTreeSet<&str> = outcome.citations.iter().map(String::as_str).collect();
        let want: BTreeSet<&str> = case.citations.iter().map(String::as_str).collect();
        assert_eq!(
            got, want,
            "case {}: citation set mismatch (engine {:?} vs seed {:?})",
            case.case_id, outcome.citations, case.citations,
        );
    }
}

#[test]
fn draft_rule_never_enters_engine_space() {
    let platform = platform_fixture();
    assert_eq!(
        platform.rejected_rule_notes.len(),
        1,
        "exactly the ai_extracted draft rule is rejected: {:?}",
        platform.rejected_rule_notes,
    );
    assert!(platform.rejected_rule_notes[0].contains("RULE-CHG-001-DRAFT"));
    // active 規則層數：20 條登記中 19 條 active（R3 增補 12 條治理規則後）。
    assert_eq!(platform.rules.active_rules().count(), 19);
}

#[test]
fn replay_check_passes_for_all_cases() {
    let platform = platform_fixture();
    for case in cases_fixture() {
        platform
            .replay_check(&case)
            .unwrap_or_else(|e| panic!("case {} replay check failed: {e}", case.case_id));
    }
}

#[test]
fn forbidden_pair_is_deviation_then_violate() {
    // H-1 保護帶：注入「execute→approve」禁止對的觀測事件序列，
    // replay 必得 Deviation，引擎必出 violate（絕不放行）。
    use ekp_platform::process::{ReplayEvent, ReplayFinding, replay_case};

    let platform = platform_fixture();
    let cases = cases_fixture();
    let ord001 = cases
        .iter()
        .find(|c| c.case_id == "ORD-001")
        .expect("seed contains ORD-001");

    let model = platform
        .processes
        .match_for(
            &ord001
                .events
                .iter()
                .map(|e| ReplayEvent {
                    activity: e.activity.clone(),
                    ts: e.ts.clone(),
                    actor: e.actor.clone(),
                })
                .collect::<Vec<_>>(),
        )
        .expect("P-CHANGE model exists");
    match replay_case(
        model,
        &ord001
            .events
            .iter()
            .map(|e| ReplayEvent {
                activity: e.activity.clone(),
                ts: e.ts.clone(),
                actor: e.actor.clone(),
            })
            .collect::<Vec<_>>(),
    ) {
        ReplayFinding::Deviation(items) => {
            assert!(items.iter().any(|i| i.contains("forbidden pair")));
        }
        other => panic!("expected deviation, got {other:?}"),
    }
    let outcome = platform.decide(ord001);
    assert_eq!(outcome.verdict, ekp_eval::types::Ver::Violate);
}

#[test]
fn snapshot_manifest_stable_across_rebuilds() {
    use ekp_platform::process::events_sha256;
    use ekp_platform::snapshot::DecisionSnapshot;

    let platform = platform_fixture();
    for case in cases_fixture() {
        let outcome = platform.decide(&case);
        let events: Vec<ekp_platform::process::ReplayEvent> = case
            .events
            .iter()
            .map(|e| ekp_platform::process::ReplayEvent {
                activity: e.activity.clone(),
                ts: e.ts.clone(),
                actor: e.actor.clone(),
            })
            .collect();
        let s1 = DecisionSnapshot::build(
            &case.case_id,
            &outcome,
            platform.doc_binding_shas(&case),
            platform.rule_binding_shas(&case),
            events_sha256(&events),
        );
        let s2 = DecisionSnapshot::build(
            &case.case_id,
            &outcome,
            platform.doc_binding_shas(&case),
            platform.rule_binding_shas(&case),
            events_sha256(&events),
        );
        assert_eq!(s1.manifest_sha, s2.manifest_sha, "case {}", case.case_id);
        assert!(s1.replay_consistent(), "case {}", case.case_id);
        // manifest 至少 16 hex 字元（SHA-256 hex 為 64）。
        assert_eq!(s1.manifest_sha.len(), 64);
    }
}

#[test]
fn retrieval_threshold_is_a_parameter() {
    // 閾值可調公理：調高至少不讓「非法重試觸發上限」案件翻轉判定。
    let platform = platform_fixture();
    assert!((platform.retrieval_threshold - 0.70).abs() < 1e-9);
    let lowered = 0.5_f64;
    assert!(lowered < 0.70);
    // 閾值調低後同一案例的判定不變（穩定性優先於命中範圍）。
    let cases = cases_fixture();
    let evd = cases
        .iter()
        .find(|c| c.case_id == "EVD-001")
        .expect("seed contains EVD-001");
    let mut tuned = Platform::load(&data_dir()).expect("loads");
    tuned.retrieval_threshold = lowered;
    let outcome_a = platform.decide(evd);
    let outcome_b = tuned.decide(evd);
    assert_eq!(outcome_a.verdict, outcome_b.verdict);
    // 仍為 insufficient：EX-7 檔案根本不在案件 doc 集，閾值調低不能「撈出」
    // 不存在的綁定（保守拒答不因參數放寬而翻轉）。
    assert_eq!(
        outcome_b.verdict,
        ekp_eval::types::Ver::InsufficientEvidence
    );
}

// ─── R3：casebook_v3 考卷（BC 系列 47 題）───────────────────────────────

fn casebook_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("eval").join("cases").join("casebook_v3.jsonl"))
        .expect("manifest root has a parent")
}

fn casebook_fixture() -> Vec<types::Case> {
    let (cases, warnings) = types::load_cases(&casebook_path()).expect("casebook loads");
    assert!(
        warnings.is_empty(),
        "casebook must be shape-clean (loader warnings are data regressions): {warnings:?}"
    );
    cases
}

#[test]
fn casebook_v3_full_run_verdict_citations_abstain() {
    let platform = platform_fixture();
    let cases = casebook_fixture();
    assert!(cases.len() >= 40, "casebook expects ~50 scored cases");

    for case in &cases {
        let outcome = platform.decide(case);
        let exp = case.expected().expect("casebook expected_verdict known");

        assert_eq!(
            outcome.verdict,
            exp,
            "case {}: verdict expected {exp:?} got {:?}; notes {}",
            case.case_id,
            outcome.verdict,
            outcome.notes.join(" | "),
        );

        let got: BTreeSet<&str> = outcome.citations.iter().map(String::as_str).collect();
        let want: BTreeSet<&str> = case.citations.iter().map(String::as_str).collect();
        assert_eq!(
            got, want,
            "case {}: citation set mismatch (engine {:?} vs casebook {:?})",
            case.case_id, outcome.citations, case.citations,
        );

        // 拒答語義：insufficient_evidence 一律不攜引用；考卷的 abstain 標記
        // 必須與引擎可觀測行為互相印證。
        if exp == ekp_eval::types::Ver::InsufficientEvidence {
            assert!(
                case.abstain_required && outcome.citations.is_empty(),
                "case {}: insufficient verdict must abstain without citations",
                case.case_id
            );
        } else {
            assert!(
                !case.abstain_required,
                "case {}: abstain flag on a non-abstaining verdict",
                case.case_id
            );
        }
    }
}

#[test]
fn casebook_v3_replay_consistency() {
    // H-6：每張考卷題目兩次裁决的快照須逐位一致。
    let platform = platform_fixture();
    for case in casebook_fixture() {
        platform
            .replay_check(&case)
            .unwrap_or_else(|e| panic!("case {} replay check failed: {e}", case.case_id));
    }
}
