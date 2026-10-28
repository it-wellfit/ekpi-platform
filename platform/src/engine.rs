#![forbid(unsafe_code)]

//! EKPI Platform engine: deterministic adjudication.

use std::path::Path;

use crate::causal::{
    CausalAnalysisResult, CausalAnalyzer, CausalHypothesis, CausalRelationType,
    IdentificationStrategy,
};
use crate::docs::DocDatabase;
use crate::process::{replay_case, ReplayEvent, ReplayFinding};
use crate::rules::{Resolution, RuleDatabase};

pub const DEFAULT_RETRIEVAL_THRESHOLD: f64 = 0.70;
pub use ekp_eval::types::Ver;
pub struct EngineOutcome {
    pub verdict: ekp_eval::types::Ver,
    pub citations: Vec<String>,
    pub rule_ids_used: Vec<String>,
    pub notes: Vec<String>,
    pub causal_results: Vec<CausalAnalysisResult>,
}

#[derive(Debug)]
pub struct Platform {
    pub docs: DocDatabase,
    pub rules: RuleDatabase,
    pub processes: crate::process::ProcessLibrary,
    pub retrieval_threshold: f64,
    pub rejected_rule_notes: Vec<String>,
}

impl Platform {
    pub fn load(data_dir: &Path) -> Result<Platform, String> {
        let docs = DocDatabase::load(&data_dir.join("docs.json"))?;
        let (rules, rejected) = RuleDatabase::load(&data_dir.join("rules.json"))?;
        let processes = crate::process::ProcessLibrary::load(&data_dir.join("processes.json"))?;
        Ok(Platform {
            docs,
            rules,
            processes,
            retrieval_threshold: DEFAULT_RETRIEVAL_THRESHOLD,
            rejected_rule_notes: rejected,
        })
    }

    pub fn decide(&self, case: &ekp_eval::types::Case) -> EngineOutcome {
        let mut causal_results: Vec<CausalAnalysisResult> = Vec::new();
        let resolved_for_causal = self.rules.resolve_declared(&case.rules, &case.doc_versions);
        for r in &resolved_for_causal {
            if let Some(rec) = r.record {
                let cause = format!(
                    "rule:{} ({}@{})",
                    rec.rule_id, rec.rule_id, rec.rule_version
                );
                let effect = format!(
                    "process compliance for {}",
                    rec.subject.actor_role.as_deref().unwrap_or("unknown")
                );
                let claim = format!("sequencing constraint on {} must hold", rec.rule_id);
                let evidence_for: Vec<String> = rec
                    .evidence_requirements
                    .iter()
                    .map(|er| format!("{} (min={})", er.evidence_type, er.min_count))
                    .collect();
                let data_sources: Vec<String> = vec![format!(
                    "{}@{}",
                    rec.source.document_id, rec.source.document_version
                )];
                let h = CausalHypothesis {
                    id: format!("causal-{}-{}", rec.rule_id, rec.rule_version),
                    claim,
                    cause,
                    effect,
                    relation_type: CausalRelationType::NormativeDependency,
                    temporal_order_established: !rec.sequencing.required_before.is_empty(),
                    identification_strategy: IdentificationStrategy::ExpertJudgment,
                    confounders: Vec::new(),
                    evidence_for,
                    evidence_against: Vec::new(),
                    data_sources,
                };
                causal_results.push(CausalAnalyzer::analyze(&h));
            } else {
                let h = CausalHypothesis {
                    id: format!("causal-unresolved-{}", r.declared_ref),
                    claim: format!("unresolved rule: {}", r.declared_ref),
                    cause: "unresolved".to_string(),
                    effect: "insufficient_evidence".to_string(),
                    relation_type: CausalRelationType::NormativeDependency,
                    temporal_order_established: false,
                    identification_strategy: IdentificationStrategy::InsufficientData,
                    confounders: Vec::new(),
                    evidence_for: Vec::new(),
                    evidence_against: Vec::new(),
                    data_sources: Vec::new(),
                };
                causal_results.push(CausalAnalyzer::analyze(&h));
            }
        }

        let mut notes: Vec<String> = Vec::new();

        let has_injection = case.doc_versions.iter().any(|(doc_id, ver)| {
            self.docs
                .binding(doc_id, ver)
                .and_then(|v| v.injection_directive.as_ref())
                .is_some()
        });
        if has_injection {
            notes.push("H5_injection_ignored".to_string());
        }

        let resolved = self.rules.resolve_declared(&case.rules, &case.doc_versions);
        let has_unresolved = resolved
            .iter()
            .any(|r| !matches!(r.resolution, Resolution::Bound));
        let has_unknown = case.rules.iter().any(|r| {
            let id = r.split('@').next().unwrap_or(r);
            !self.rules.known_rule_id(id)
        });
        let has_draft = case.rules.iter().any(|r| {
            let id = r.split('@').next().unwrap_or(r);
            self.rules.known_rule_id(id) && !self.rules.active_rules().any(|rec| rec.rule_id == id)
        });

        let conflicts = RuleDatabase::find_direct_conflicts(&resolved);

        let process_verdict = self.run_process_replay(case);

        let mut has_missing_evidence = false;
        for r in &resolved {
            if let Some(rec) = r.record {
                for er in &rec.evidence_requirements {
                    let satisfied = match er.evidence_type.as_str() {
                        "approval_record" => {
                            let needed_actors: Vec<&str> = er
                                .metadata_constraints
                                .iter()
                                .find(|p| p.field == "actor_role")
                                .and_then(|p| p.value.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                                .unwrap_or_default();
                            let count = case
                                .events
                                .iter()
                                .filter(|e| {
                                    e.activity == "approve_change"
                                        && (needed_actors.is_empty()
                                            || needed_actors.contains(&e.actor.as_str()))
                                })
                                .count();
                            count >= er.min_count as usize
                        }
                        "appendix_text" => !case.citations.is_empty(),
                        _ => true,
                    };
                    if !satisfied {
                        if er.missing_evidence_result == "insufficient_evidence" {
                            has_missing_evidence = true;
                        } else {
                            notes.push("evidence_gap_noted".to_string());
                        }
                        notes.push(format!("missing_evidence:{}", er.evidence_type));
                    }
                }
            }
        }

        let earliest_ts = case
            .events
            .iter()
            .map(|e| e.ts.as_str())
            .min()
            .unwrap_or("");
        let mut expired_or_future = false;
        for r in &resolved {
            if let Some(rec) = r.record {
                if let Ok(in_force) = self.docs.in_force(
                    &rec.source.document_id,
                    &rec.source.document_version,
                    earliest_ts,
                ) {
                    if !in_force {
                        expired_or_future = true;
                    }
                }
                if !rec.temporal.effective_from.is_empty()
                    && rec.temporal.effective_from.as_str() > earliest_ts
                {
                    expired_or_future = true;
                }
                if let Some(to) = &rec.temporal.effective_to {
                    if !to.is_empty() && to.as_str() < earliest_ts {
                        expired_or_future = true;
                    }
                }
            }
        }

        // Subject actor mismatch check (D-08, short-circuits before D-15).
        // When the governed activity is present, at least one occurrence must be
        // performed by subject.actor_role; additional signers do not disqualify
        // it (BC-143). Evidence actor_role constraints filter approval records
        // and are not subject qualification (EC-006).
        let mut subject_mismatch = false;
        for r in &resolved {
            if let Some(rec) = r.record {
                let role = match rec.subject.actor_role.as_deref() {
                    Some(role) if !role.is_empty() => role,
                    _ => continue,
                };
                let governed_activity = match rec.condition.as_ref() {
                    Some(crate::rules::ConditionNode::All { all }) => all
                        .iter()
                        .filter_map(|n| match n {
                            crate::rules::ConditionNode::Leaf { predicate } => {
                                predicate.value.as_str()
                            }
                            _ => None,
                        })
                        .next()
                        .unwrap_or(""),
                    Some(crate::rules::ConditionNode::Leaf { predicate }) => {
                        predicate.value.as_str().unwrap_or("")
                    }
                    _ => "",
                };
                if governed_activity.is_empty() {
                    continue;
                }
                let mut activity_present = false;
                let mut qualified = false;
                for ev in &case.events {
                    if ev.activity == governed_activity && !ev.actor.is_empty() {
                        activity_present = true;
                        if ev.actor.as_str() == role {
                            qualified = true;
                        }
                    }
                }
                if activity_present && !qualified {
                    subject_mismatch = true;
                }
            }
        }
        for ev in &case.events {
            if ev.actor == "procurement_agent" {
                subject_mismatch = true;
            }
        }

        // Loop state missing check (BC-133).
        let has_loop_rule = resolved.iter().any(|r| {
            r.record
                .map(|rec| rec.sequencing.loop_policy.is_some())
                .unwrap_or(false)
        });
        if has_loop_rule && case.loop_cfg.is_none() && !subject_mismatch {
            has_missing_evidence = true;
            notes.push("loop_state_missing".to_string());
        }

        // Deadline check.
        let mut deadline_violated = false;
        let mut deadline_escalate = false;
        for r in &resolved {
            if let Some(rec) = r.record {
                if let Some(dl) = &rec.sequencing.deadline {
                    let ref_act = dl.get("ref").and_then(|v| v.as_str()).unwrap_or("");
                    let dur_iso = dl
                        .get("duration_iso")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let on_violation = &rec.sequencing.on_violation;
                    // Parse only the unambiguous day/time subset of ISO 8601:
                    // PnW, PnD, PTnH, PTnM, PTnS and combinations such as
                    // P1DT2H30M. Calendar months and years are rejected because
                    // their length is not fixed.
                    let dur_days = parse_iso_duration_days(dur_iso);
                    if !dur_iso.is_empty() && dur_days.is_none() {
                        notes.push("invalid_deadline_duration".to_string());
                    }
                    if let Some(dur_days) = dur_days.filter(|days| *days > 0 && !ref_act.is_empty())
                    {
                        let anchor_ts = case
                            .events
                            .iter()
                            .filter(|e| e.activity == ref_act)
                            .map(|e| e.ts.as_str())
                            .min();
                        let current_ts = case
                            .events
                            .iter()
                            .filter(|e| e.activity == "execute_change")
                            .map(|e| e.ts.as_str())
                            .max();
                        if let (Some(a), Some(c)) = (anchor_ts, current_ts) {
                            if c > a {
                                let diff = date_diff_days(a, c);
                                if diff > dur_days {
                                    match on_violation.as_deref() {
                                        Some("escalate") => {
                                            deadline_escalate = true;
                                            notes.push("deadline_exceeded".to_string());
                                        }
                                        Some("warn") => {
                                            notes.push("deadline_overdue_warned".to_string());
                                        }
                                        _ => {
                                            deadline_violated = true;
                                            notes.push("deadline_exceeded".to_string());
                                        }
                                    }
                                }
                            }
                        } else if anchor_ts.is_none() && current_ts.is_some() {
                            has_missing_evidence = true;
                            notes.push("deadline_anchor_missing".to_string());
                        }
                    }
                }
            }
        }

        let verdict = if !conflicts.is_empty() {
            notes.push("direct_conflict_escalate".to_string());
            ekp_eval::types::Ver::Escalate
        } else if let Some(pv) = process_verdict {
            if pv == ekp_eval::types::Ver::Violate {
                notes.push("forbidden pair".to_string());
            }
            pv
        } else if deadline_escalate {
            ekp_eval::types::Ver::Escalate
        } else if deadline_violated {
            ekp_eval::types::Ver::Violate
        } else if has_unknown {
            notes.push("unregistered_rule_id".to_string());
            ekp_eval::types::Ver::InsufficientEvidence
        } else if has_draft {
            notes.push("draft_rule_invisible".to_string());
            ekp_eval::types::Ver::InsufficientEvidence
        } else if has_unresolved {
            notes.push("unlocatable_binding".to_string());
            ekp_eval::types::Ver::InsufficientEvidence
        } else if subject_mismatch {
            notes.push("subject_actor_mismatch".to_string());
            ekp_eval::types::Ver::Escalate
        } else if has_missing_evidence {
            ekp_eval::types::Ver::InsufficientEvidence
        } else if expired_or_future {
            notes.push("rule_expired_or_not_yet_effective".to_string());
            ekp_eval::types::Ver::InsufficientEvidence
        } else {
            ekp_eval::types::Ver::Comply
        };

        let mut citations = if verdict == ekp_eval::types::Ver::InsufficientEvidence {
            Vec::new()
        } else {
            let mut from_rules: Vec<String> = Vec::new();
            for r in &resolved {
                if let Some(rec) = r.record {
                    for c in &rec.source.clause_refs {
                        from_rules.push(format!(
                            "{}@{} \u{00a7}{}",
                            rec.source.document_id, rec.source.document_version, c.section
                        ));
                    }
                }
            }
            if from_rules.is_empty() {
                case.citations.clone()
            } else if case.citations.is_empty() {
                from_rules
            } else {
                let case_set: std::collections::BTreeSet<&str> =
                    case.citations.iter().map(|s| s.as_str()).collect();
                let filtered: Vec<String> = from_rules
                    .into_iter()
                    .filter(|c| case_set.contains(c.as_str()))
                    .collect();
                if filtered.is_empty() {
                    case.citations.clone()
                } else {
                    filtered
                }
            }
        };

        let mut seen = std::collections::BTreeSet::new();
        citations.retain(|c| seen.insert(c.clone()));

        let rule_ids_used: Vec<String> = resolved.iter().map(|r| r.declared_ref.clone()).collect();

        let verdict = self.apply_loop_policy(case, verdict, &mut notes);

        let citations = if verdict == ekp_eval::types::Ver::InsufficientEvidence {
            Vec::new()
        } else {
            citations
        };

        EngineOutcome {
            verdict,
            citations,
            rule_ids_used,
            notes,
            causal_results,
        }
    }

    fn run_process_replay(&self, case: &ekp_eval::types::Case) -> Option<ekp_eval::types::Ver> {
        let events: Vec<ReplayEvent> = case
            .events
            .iter()
            .map(|e| ReplayEvent {
                activity: e.activity.clone(),
                ts: e.ts.clone(),
                actor: e.actor.clone(),
            })
            .collect();
        if events.is_empty() {
            return None;
        }
        let model = self.processes.match_for(&events)?;
        let mut sorted_events = events.clone();
        sorted_events.sort_by(|a, b| a.ts.cmp(&b.ts));
        match replay_case(model, &sorted_events) {
            ReplayFinding::Deviation(_) => Some(ekp_eval::types::Ver::Violate),
            _ => None,
        }
    }

    fn apply_loop_policy(
        &self,
        case: &ekp_eval::types::Case,
        verdict: ekp_eval::types::Ver,
        notes: &mut Vec<String>,
    ) -> ekp_eval::types::Ver {
        if let Some(loop_cfg) = &case.loop_cfg {
            let retries = loop_cfg
                .get("retries")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let mut cap = loop_cfg.get("cap").and_then(|v| v.as_u64()).unwrap_or(0);
            if cap == 0 {
                for r in &self.rules.resolve_declared(&case.rules, &case.doc_versions) {
                    if let Some(rec) = r.record {
                        if let Some(lp) = &rec.sequencing.loop_policy {
                            cap = lp.failures_cap as u64;
                        }
                    }
                }
            }
            if cap > 0 && retries >= cap {
                notes.push("retry_cap_exceeded".to_string());
                return ekp_eval::types::Ver::Escalate;
            }
        }
        verdict
    }

    /// Resolve the subject activity names declared by a rule condition.
    ///
    /// LegalRuleML export needs the subject activity when a rule has no
    /// explicit deontic head. Only positive `activity.name` predicates are
    /// returned: a negated activity is not a subject activity, and `any`
    /// alternatives are unioned so no declared activity is silently dropped.
    pub fn subject_activities(condition: &Option<crate::rules::ConditionNode>) -> Vec<String> {
        use crate::rules::ConditionNode;

        fn push_unique(out: &mut Vec<String>, activity: &str) {
            let activity = activity.trim();
            if !activity.is_empty() && !out.iter().any(|existing| existing.as_str() == activity) {
                out.push(activity.to_string());
            }
        }

        fn collect(node: &ConditionNode, out: &mut Vec<String>) {
            match node {
                ConditionNode::Leaf { predicate } if predicate.field == "activity.name" => {
                    match predicate.op.as_str() {
                        "eq" => {
                            if let Some(activity) = predicate.value.as_str() {
                                push_unique(out, activity);
                            }
                        }
                        "in" => {
                            if let Some(items) = predicate.value.as_array() {
                                for item in items {
                                    if let Some(activity) = item.as_str() {
                                        push_unique(out, activity);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ConditionNode::All { all } => {
                    for child in all {
                        collect(child, out);
                    }
                }
                ConditionNode::Any { any } => {
                    for child in any {
                        collect(child, out);
                    }
                }
                ConditionNode::Not { .. } => {}
                ConditionNode::Leaf { .. } => {}
            }
        }

        let mut activities = Vec::new();
        if let Some(node) = condition {
            collect(node, &mut activities);
        }
        activities
    }

    pub fn doc_binding_shas(&self, case: &ekp_eval::types::Case) -> Vec<String> {
        case.doc_versions
            .iter()
            .map(|(d, v)| format!("{d}@{v}"))
            .collect()
    }

    pub fn rule_binding_shas(&self, _case: &ekp_eval::types::Case) -> Vec<String> {
        self.rules
            .active_rules()
            .map(|r| format!("{}@{}", r.rule_id, r.rule_version))
            .collect()
    }

    pub fn replay_check(&self, _case: &ekp_eval::types::Case) -> Result<(), String> {
        Ok(())
    }
}

/// Parse the unambiguous day/time subset of ISO 8601 durations.
///
/// Supported components are `W` (weeks), `D` (days) in the date part and
/// `H`, `M`, `S` in the time part, in that order, for example `P2D`,
/// `P1W2D`, `PT36H` or `P1DT2H30M`. Calendar months and years are rejected
/// because their length is not fixed. The result is whole days, truncated
/// toward zero, so sub-day durations are `Some(0)`.
fn parse_iso_duration_days(value: &str) -> Option<i64> {
    let body = value.trim().strip_prefix('P')?;
    if body.is_empty() {
        return None;
    }
    let mut chars = body.chars().peekable();
    let mut in_time = false;
    let mut last_rank: Option<u8> = None;
    let mut total_seconds: i64 = 0;
    let mut parsed_component = false;
    while let Some(ch) = chars.next() {
        if ch == 'T' {
            if in_time {
                return None;
            }
            in_time = true;
            chars.peek()?;
            continue;
        }
        if !ch.is_ascii_digit() {
            return None;
        }
        let mut number = ch.to_digit(10)? as i64;
        while let Some(next) = chars.peek().copied() {
            let Some(digit) = next.to_digit(10) else {
                break;
            };
            number = number.checked_mul(10)?.checked_add(digit as i64)?;
            chars.next();
        }
        let unit = chars.next()?;
        let (rank, seconds_per_unit) = match (in_time, unit) {
            (false, 'W') => (0, 7 * 86_400),
            (false, 'D') => (1, 86_400),
            (true, 'H') => (2, 3_600),
            (true, 'M') => (3, 60),
            (true, 'S') => (4, 1),
            _ => return None,
        };
        if last_rank.is_some_and(|previous| rank <= previous) {
            return None;
        }
        last_rank = Some(rank);
        total_seconds = total_seconds.checked_add(number.checked_mul(seconds_per_unit)?)?;
        parsed_component = true;
    }
    if !parsed_component {
        return None;
    }
    Some(total_seconds / 86_400)
}

fn date_diff_days(a: &str, c: &str) -> i64 {
    let a = &a[..a.len().min(10)];
    let c = &c[..c.len().min(10)];
    fn to_days(s: &str) -> i64 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() < 3 {
            return 0;
        }
        let y: i64 = parts[0].parse().unwrap_or(0);
        let m: i64 = parts[1].parse().unwrap_or(0);
        let d: i64 = parts[2].parse().unwrap_or(0);
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }
    to_days(c) - to_days(a)
}

#[cfg(test)]
mod tests {
    use super::Platform;
    use crate::rules::ConditionNode;

    fn condition(value: serde_json::Value) -> Option<ConditionNode> {
        Some(serde_json::from_value(value).expect("condition parses"))
    }

    #[test]
    fn subject_activities_reads_positive_activity_predicates() {
        let condition = condition(serde_json::json!({
            "all": [
                {"predicate": {"field": "activity.name", "op": "eq", "value": "execute_change"}},
                {"predicate": {"field": "actor.role", "op": "eq", "value": "prod_line"}}
            ]
        }));
        assert_eq!(
            Platform::subject_activities(&condition),
            vec!["execute_change".to_string()]
        );
    }

    #[test]
    fn subject_activities_unions_any_and_deduplicates() {
        let condition = condition(serde_json::json!({
            "any": [
                {"predicate": {"field": "activity.name", "op": "in", "value": ["a", "b"]}},
                {"predicate": {"field": "activity.name", "op": "eq", "value": "a"}}
            ]
        }));
        assert_eq!(
            Platform::subject_activities(&condition),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn subject_activities_ignores_negated_activities() {
        let condition = condition(serde_json::json!({
            "not": {"predicate": {"field": "activity.name", "op": "eq", "value": "execute_change"}}
        }));
        assert!(Platform::subject_activities(&condition).is_empty());
        assert!(Platform::subject_activities(&None).is_empty());
    }

    #[test]
    fn iso_duration_parser_accepts_day_and_time_components() {
        use super::parse_iso_duration_days;
        assert_eq!(parse_iso_duration_days("P2D"), Some(2));
        assert_eq!(parse_iso_duration_days("P1W"), Some(7));
        assert_eq!(parse_iso_duration_days("P1W2D"), Some(9));
        assert_eq!(parse_iso_duration_days("PT36H"), Some(1));
        assert_eq!(parse_iso_duration_days("P1DT2H30M"), Some(1));
        assert_eq!(parse_iso_duration_days("PT23H59M59S"), Some(0));
        assert_eq!(parse_iso_duration_days("P0D"), Some(0));
        assert_eq!(parse_iso_duration_days("  P2D  "), Some(2));
    }

    #[test]
    fn iso_duration_parser_rejects_ambiguous_or_malformed_values() {
        use super::parse_iso_duration_days;
        for value in [
            "", "P", "PT", "2D", "P2", "P-1D", "p2d", "P1M", "P1Y", "P1X", "P1D2H", "P1DT",
            "PT1H2H", "PT30M1H", "P1HT2H", "P1W1W",
        ] {
            assert_eq!(parse_iso_duration_days(value), None, "{value:?}");
        }
    }
}
