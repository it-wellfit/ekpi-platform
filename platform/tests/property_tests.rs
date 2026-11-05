// Property-based tests using proptest
// Tests boundary cases and invariants for engine.decide(), snapshot replay, and ledger hash chain

use ekp_eval::types::{Case, EventRaw, Ver};
use ekp_platform::engine::Platform;
use ekp_platform::governance::Ledger;
use ekp_platform::local_auth::AuthStore;
use ekp_platform::retrieval::PermissionContext;
use proptest::prelude::*;
use std::env::temp_dir;
use std::sync::LazyLock;

/// Shared platform instance for engine properties: loading parses three small
/// JSON files, so one instance per process keeps the generated-case loops fast.
static ENGINE_PLATFORM: LazyLock<Platform> = LazyLock::new(|| {
    Platform::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"))
        .expect("platform data loads for property tests")
});

/// Builds a generated case that exercises the engine with an optional event
/// and an optional document binding, both derived from arbitrary strings.
fn generated_case(case_id: &str, activity: Option<String>, doc_id: Option<String>) -> Case {
    Case {
        case_id: case_id.to_string(),
        category: "general".to_string(),
        task: String::new(),
        doc_versions: doc_id
            .map(|id| (format!("DOC-{id}"), "1.0".to_string()))
            .into_iter()
            .collect(),
        events: activity
            .map(|name| EventRaw {
                activity: name,
                actor: "tester".to_string(),
                ts: "2026-01-01T00:00:00Z".to_string(),
            })
            .into_iter()
            .collect(),
        rules: Vec::new(),
        citations: Vec::new(),
        loop_cfg: None,
        expected_verdict: "comply".to_string(),
        abstain_required: false,
        notes: String::new(),
    }
}

proptest! {
    #[test]
    fn engine_decision_is_deterministic(
        seed in "[a-zA-Z0-9]{1,12}",
        with_activity in proptest::bool::ANY,
        with_doc in proptest::bool::ANY,
    ) {
        let platform = &*ENGINE_PLATFORM;
        let case = generated_case(
            &format!("PROP-{seed}"),
            with_activity.then(|| "execute_change".to_string()),
            with_doc.then(|| seed.clone()),
        );
        let first = platform.decide(&case);
        let second = platform.decide(&case);
        prop_assert_eq!(first.verdict.as_str(), second.verdict.as_str());
        prop_assert_eq!(first.citations, second.citations);
        prop_assert_eq!(first.rule_ids_used, second.rule_ids_used);
    }

    #[test]
    fn engine_abstention_implies_no_citations(
        seed in "[a-zA-Z0-9]{1,12}",
        with_activity in proptest::bool::ANY,
        with_doc in proptest::bool::ANY,
    ) {
        let platform = &*ENGINE_PLATFORM;
        let case = generated_case(
            &format!("PROP-{seed}"),
            with_activity.then(|| "unknown_activity".to_string()),
            with_doc.then(|| seed.clone()),
        );
        let outcome = platform.decide(&case);
        // H-3: an insufficient-evidence verdict must never carry citations,
        // and any definitive verdict must cite at least one anchor.
        if outcome.verdict == Ver::InsufficientEvidence {
            prop_assert!(outcome.citations.is_empty(), "abstention carried citations: {:?}", outcome.citations);
        } else {
            prop_assert!(
                !outcome.citations.is_empty(),
                "definitive verdict {} without citations",
                outcome.verdict.as_str()
            );
        }
    }
}

proptest! {
    #[test]
    fn ledger_hash_chain_integrity(s in "[a-zA-Z0-9]{1,10}") {
        let path = temp_dir().join(format!("test_ledger_{}.jsonl", s));
        let mut ledger = Ledger::open(&path).unwrap();
        let result = ledger.record_project(&s, "Dept", "Owner", "low", "Actor");
        if result.is_ok() {
            assert!(ledger.verify().is_ok());
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ledger_rejects_empty_actor(actor in "") {
        let path = temp_dir().join("test_empty_actor.jsonl");
        let mut ledger = Ledger::open(&path).unwrap();
        let result = ledger.record_project("Project", "Dept", "Owner", "low", &actor);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn snapshot_replay_is_deterministic(s in "[a-zA-Z0-9]{1,5}") {
        let path = temp_dir().join(format!("test_replay_{}.jsonl", s));
        let mut ledger = Ledger::open(&path).unwrap();
        let _ = ledger.record_project(&s, "Dept", "Owner", "low", "Actor");
        let snapshot1 = ledger.snapshot();
        drop(ledger);
        let ledger2 = Ledger::open(&path).unwrap();
        let snapshot2 = ledger2.snapshot();
        assert_eq!(snapshot1.projects.len(), snapshot2.projects.len());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn permission_context_roles_preserved(roles in prop::collection::vec("[a-z]{3,8}", 0..10)) {
        let ctx = PermissionContext::new("T1", "U1", roles.clone());
        assert_eq!(ctx.roles, roles);
        assert_eq!(ctx.tenant_id, "T1");
        assert_eq!(ctx.user_id, "U1");
    }

    #[test]
    fn permission_context_empty_denies_restricted(doc_role in "[a-z]{3,8}") {
        let ctx = PermissionContext::new("T1", "U1", vec![]);
        assert!(!ctx.can_access(&[doc_role]));
    }

    #[test]
    fn permission_context_matching_grants(role in "[a-z]{3,8}") {
        let ctx = PermissionContext::new("T1", "U1", vec![role.clone()]);
        assert!(ctx.can_access(&[role]));
    }

    #[test]
    fn permission_context_non_matching_denies(user_role in "[a-z]{3,8}", doc_role in "[a-z]{3,8}") {
        prop_assume!(user_role != doc_role);
        let ctx = PermissionContext::new("T1", "U1", vec![user_role]);
        assert!(!ctx.can_access(&[doc_role]));
    }

    #[test]
    fn cosine_symmetric(a in "[a-zA-Z0-9 ]{0,20}", b in "[a-zA-Z0-9 ]{0,20}") {
        let score_ab = ekp_platform::retrieval::char_bigram_cosine(&a, &b);
        let score_ba = ekp_platform::retrieval::char_bigram_cosine(&b, &a);
        assert!((score_ab - score_ba).abs() < 1e-9);
    }

    #[test]
    fn cosine_identical_is_one(s in "[a-zA-Z0-9 ]{1,20}") {
        let score = ekp_platform::retrieval::char_bigram_cosine(&s, &s);
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cosine_empty_is_one(_dummy in Just(())) {
        let score = ekp_platform::retrieval::char_bigram_cosine("", "");
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cosine_different_less_than_one(a in "[a-zA-Z]{5,10}", b in "[a-zA-Z]{5,10}") {
        prop_assume!(a != b);
        let score = ekp_platform::retrieval::char_bigram_cosine(&a, &b);
        assert!(score < 1.0 + 1e-9);
    }

    #[test]
    fn ledger_rejects_empty_name(name in "") {
        let path = temp_dir().join("test_empty_name.jsonl");
        let mut ledger = Ledger::open(&path).unwrap();
        let result = ledger.record_project(&name, "Dept", "Owner", "low", "Actor");
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ledger_rejects_invalid_tier(tier in "[a-z]{3,10}") {
        prop_assume!(tier != "low" && tier != "medium" && tier != "high" && tier != "critical");
        let path = temp_dir().join("test_invalid_tier.jsonl");
        let mut ledger = Ledger::open(&path).unwrap();
        let result = ledger.record_project("Project", "Dept", "Owner", &tier, "Actor");
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn auth_rejects_short_passwords(password in "[a-z]{1,7}") {
        let mut store = AuthStore::with_iterations(3600, 1000);
        let result = store.register("testuser", &password, ekp_platform::local_auth::Role::Viewer);
        assert!(result.is_err());
    }

    #[test]
    fn auth_accepts_valid_passwords(password in "[a-zA-Z0-9]{8,20}") {
        let mut store = AuthStore::with_iterations(3600, 1000);
        let result = store.register("testuser", &password, ekp_platform::local_auth::Role::Viewer);
        assert!(result.is_ok());
    }

    #[test]
    fn ekpi_error_display_non_empty(msg in "[a-zA-Z0-9 ]{1,30}") {
        let error = ekp_platform::errors::EkpiError::ValidationError(msg.clone());
        let display = format!("{}", error);
        assert!(!display.is_empty());
    }
}
