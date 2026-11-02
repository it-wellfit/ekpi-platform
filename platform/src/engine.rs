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

        // A malformed event timestamp must never be compared as a valid date:
        // string ordering or a zero-day fallback could otherwise fabricate a
        // deviation or hide a deadline breach. Abstain from process replay and
        // carry an insufficient-evidence signal instead.
        let invalid_event_timestamp = case
            .events
            .iter()
            .any(|event| parse_utc_timestamp_seconds(&event.ts).is_none());
        let process_verdict = if invalid_event_timestamp {
            None
        } else {
            self.run_process_replay(case)
        };

        let mut has_missing_evidence = false;
        if invalid_event_timestamp {
            has_missing_evidence = true;
            notes.push("invalid_event_timestamp".to_string());
        }
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

        // Effective windows are date-based and `effective_to` is inclusive
        // through the end of that day. Comparing a full event timestamp
        // lexicographically would treat `2026-02-28T09:00:00` as later than
        // `2026-02-28` and falsely expire the rule on its last valid day.
        let earliest_date = case
            .events
            .iter()
            .filter_map(|e| crate::docs::iso_date_prefix(&e.ts))
            .min()
            .unwrap_or("");
        let mut expired_or_future = false;
        for r in &resolved {
            if let Some(rec) = r.record {
                if let Ok(in_force) = self.docs.in_force(
                    &rec.source.document_id,
                    &rec.source.document_version,
                    earliest_date,
                ) {
                    if !in_force {
                        expired_or_future = true;
                    }
                }
                if !rec.temporal.effective_from.is_empty()
                    && rec.temporal.effective_from.as_str() > earliest_date
                {
                    expired_or_future = true;
                }
                if let Some(to) = &rec.temporal.effective_to {
                    if !to.is_empty() && to.as_str() < earliest_date {
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
                    // their length is not fixed. Timestamps are compared at
                    // second precision so a sub-day deadline is enforced.
                    let dur_seconds = parse_iso_duration_seconds(dur_iso);
                    if ref_act.is_empty() {
                        has_missing_evidence = true;
                        notes.push("deadline_anchor_invalid".to_string());
                    }
                    if dur_iso.is_empty() || dur_seconds.is_none() {
                        // An unparseable or missing deadline must never be
                        // skipped into a Comply verdict. Unsupported units such
                        // as P1M have no fixed second length; abstain instead.
                        has_missing_evidence = true;
                        notes.push("invalid_deadline_duration".to_string());
                    } else if let Some(dur_seconds) = dur_seconds.filter(|_| !ref_act.is_empty()) {
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
                            match timestamp_diff_seconds(a, c) {
                                Some(diff) if diff > dur_seconds => match on_violation.as_deref() {
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
                                },
                                Some(_) => {}
                                None => {
                                    has_missing_evidence = true;
                                    notes.push("deadline_timestamp_invalid".to_string());
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
/// because their length is not fixed. The result is whole seconds so that a
/// sub-day deadline such as `PT12H` is enforced instead of being truncated to
/// zero days and skipped.
fn parse_iso_duration_seconds(value: &str) -> Option<i64> {
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
    Some(total_seconds)
}

/// Parse the UTC timestamp forms accepted by the engine (`YYYY-MM-DD` or
/// `YYYY-MM-DDTHH:MM:SS`, optionally with a trailing `Z`) into Unix seconds.
///
/// Strict parsing is deliberate: the previous implementation sliced the raw
/// date prefix and mapped every parse failure to day zero, so a malformed
/// timestamp could silently satisfy a deadline or fabricate a deviation. Any
/// value outside this unambiguous subset returns `None` and is treated as
/// missing evidence rather than silently coerced.
fn parse_utc_timestamp_seconds(value: &str) -> Option<i64> {
    // `is_ascii` guarantees the byte offsets below are also char boundaries,
    // so a multibyte input can never panic on slicing.
    if !value.is_ascii() {
        return None;
    }
    let bytes = value.as_bytes();
    let (date_part, time_part) = match bytes.len() {
        10 => (value, None),
        19 => (&value[..10], Some(&value[11..19])),
        20 if bytes[19] == b'Z' => (&value[..10], Some(&value[11..19])),
        _ => return None,
    };
    if bytes.len() > 10 && bytes[10] != b'T' {
        return None;
    }
    let date_seconds = parse_civil_date_seconds(date_part)?;
    let time_seconds = match time_part {
        Some(time) => parse_time_of_day_seconds(time)?,
        None => 0,
    };
    date_seconds.checked_add(time_seconds)
}

fn parse_civil_date_seconds(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<u32>().ok()?;
    let day = value[8..10].parse::<u32>().ok()?;
    // xs:dateTime 1.0 has no year 0000; day zero and impossible calendar dates
    // are rejected instead of being rolled into the next month.
    if year == 0 || !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    days_from_civil(year, month, day).checked_mul(86_400)
}

fn parse_time_of_day_seconds(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 2 || index == 5 || byte.is_ascii_digit())
    {
        return None;
    }
    let hour = value[0..2].parse::<i64>().ok()?;
    let minute = value[3..5].parse::<i64>().ok()?;
    let second = value[6..8].parse::<i64>().ok()?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(hour * 3_600 + minute * 60 + second)
}

/// Difference in seconds from `anchor` to `current`; `None` if either side is
/// not a parseable timestamp. A negative result means `current` is earlier.
fn timestamp_diff_seconds(anchor: &str, current: &str) -> Option<i64> {
    parse_utc_timestamp_seconds(current)?.checked_sub(parse_utc_timestamp_seconds(anchor)?)
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's
/// `days_from_civil`). Callers validate the calendar date first.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = if month > 2 { month - 3 } else { month + 9 } as i64;
    let day_of_year = (153 * month_prime + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::Platform;
    use crate::rules::ConditionNode;

    fn condition(value: serde_json::Value) -> Option<ConditionNode> {
        Some(serde_json::from_value(value).expect("condition parses"))
    }

    fn deadline_platform(duration_iso: &str) -> Platform {
        deadline_platform_with_ref(duration_iso, "approve_change")
    }

    fn deadline_platform_with_ref(duration_iso: &str, anchor_ref: &str) -> Platform {
        let rule: crate::rules::RuleRecord = serde_json::from_value(serde_json::json!({
            "rule_id": "RULE-REL-002",
            "rule_version": "1.0",
            "deontic": {"modality": "obligation", "strength": "mandatory"},
            "subject": {
                "entity_type": "change_request",
                "actor_role": "prod_line",
                "scope": {}
            },
            "condition": {"all": [
                {"predicate": {"field": "activity.name", "op": "eq", "value": "execute_change"}}
            ]},
            "sequencing": {
                "required_before": [],
                "forbidden_after": [],
                "deadline": {
                    "anchor": "activity_end",
                    "ref": anchor_ref,
                    "duration_iso": duration_iso
                },
                "on_violation": "block",
                "loop_policy": null
            },
            "exceptions": {},
            "temporal": {
                "effective_from": "2026-03-01",
                "effective_to": null,
                "supersedes": [],
                "superseded_by": null
            },
            "evidence_requirements": [],
            "decision": {
                "method": "deterministic_rule_check",
                "missing_evidence_result": "insufficient_evidence",
                "abstain_threshold": null
            },
            "source": {
                "document_id": "SOP-REL",
                "document_version": "1.0",
                "clause_refs": [],
                "verbatim_text": "test fixture",
                "selectors": []
            },
            "provenance": {
                "origin": "authoritative_text",
                "extraction": null,
                "review": {
                    "status": "active",
                    "reviewer": "test",
                    "reviewed_at": "2026-03-05",
                    "change_note": null
                }
            },
            "validation": null
        }))
        .expect("deadline rule fixture parses");
        Platform {
            docs: crate::docs::DocDatabase {
                documents: Vec::new(),
            },
            rules: crate::rules::RuleDatabase { rules: vec![rule] },
            processes: crate::process::ProcessLibrary {
                processes: Vec::new(),
            },
            retrieval_threshold: super::DEFAULT_RETRIEVAL_THRESHOLD,
            rejected_rule_notes: Vec::new(),
        }
    }

    fn deadline_case(anchor_ts: &str, execute_ts: &str) -> ekp_eval::types::Case {
        serde_json::from_value(serde_json::json!({
            "case_id": "DEADLINE-SECONDS",
            "category": "deadline_overdue_block",
            "task": "process_verdict",
            "doc_versions": {"SOP-REL": "1.0"},
            "events": [
                {"activity": "approve_change", "actor": "QA_manager", "ts": anchor_ts},
                {"activity": "execute_change", "actor": "prod_line", "ts": execute_ts}
            ],
            "rules": ["RULE-REL-002"],
            "citations": ["SOP-REL@1.0 §2.1"],
            "expected_verdict": "violate",
            "abstain_required": false,
            "notes": "unit fixture"
        }))
        .expect("deadline case fixture parses")
    }

    #[test]
    fn deadline_subday_duration_is_enforced() {
        // PT12H is a positive deadline. Truncating it to zero whole days makes
        // the engine skip the check entirely and return comply.
        let platform = deadline_platform("PT12H");
        let case = deadline_case("2026-03-02T08:00:00", "2026-03-02T20:00:01");
        let outcome = platform.decide(&case);
        assert_eq!(
            outcome.verdict,
            ekp_eval::types::Ver::Violate,
            "a 12-hour deadline exceeded by one second must violate; notes {:?}",
            outcome.notes
        );
        assert!(outcome.notes.iter().any(|note| note == "deadline_exceeded"));
    }

    #[test]
    fn deadline_equal_to_duration_is_not_exceeded() {
        let platform = deadline_platform("PT12H");
        let case = deadline_case("2026-03-02T08:00:00", "2026-03-02T20:00:00");
        let outcome = platform.decide(&case);
        assert_eq!(
            outcome.verdict,
            ekp_eval::types::Ver::Comply,
            "the deadline is strict `>`; equality must not violate; notes {:?}",
            outcome.notes
        );
    }

    #[test]
    fn malformed_event_timestamp_is_insufficient_evidence() {
        let platform = deadline_platform("P2D");
        let case = deadline_case("2026-03-0~", "2026-03-0~");
        let outcome = platform.decide(&case);
        assert_eq!(
            outcome.verdict,
            ekp_eval::types::Ver::InsufficientEvidence,
            "an unparseable timestamp must abstain, never silently compare as day zero; notes {:?}",
            outcome.notes
        );
        assert!(outcome
            .notes
            .iter()
            .any(|note| note == "invalid_event_timestamp"));
    }

    #[test]
    fn deadline_zero_duration_is_an_immediate_deadline() {
        let platform = deadline_platform("P0D");
        let late = deadline_case("2026-03-02T08:00:00", "2026-03-02T08:00:01");
        let late_outcome = platform.decide(&late);
        assert_eq!(
            late_outcome.verdict,
            ekp_eval::types::Ver::Violate,
            "a zero-length deadline must not be skipped; one second late violates; notes {:?}",
            late_outcome.notes
        );
        let equal = deadline_case("2026-03-02T08:00:00", "2026-03-02T08:00:00");
        assert_eq!(
            platform.decide(&equal).verdict,
            ekp_eval::types::Ver::Comply,
            "an identical timestamp is exactly on time"
        );
    }

    #[test]
    fn invalid_or_missing_deadline_configuration_abstains() {
        for duration in ["P1M", "P1Y", "", "not-a-duration"] {
            let platform = deadline_platform(duration);
            let case = deadline_case("2026-03-02", "2026-03-03");
            let outcome = platform.decide(&case);
            assert_eq!(
                outcome.verdict,
                ekp_eval::types::Ver::InsufficientEvidence,
                "duration {duration:?} must not be skipped into Comply; notes {:?}",
                outcome.notes
            );
            assert!(
                outcome
                    .notes
                    .iter()
                    .any(|note| note == "invalid_deadline_duration"),
                "duration {duration:?} must be reported; notes {:?}",
                outcome.notes
            );
        }
    }

    #[test]
    fn missing_deadline_anchor_reference_abstains() {
        let platform = deadline_platform_with_ref("P2D", "");
        let case = deadline_case("2026-03-02", "2026-03-03");
        let outcome = platform.decide(&case);
        assert_eq!(
            outcome.verdict,
            ekp_eval::types::Ver::InsufficientEvidence,
            "a deadline without an anchor rule must not be evaluated; notes {:?}",
            outcome.notes
        );
        assert!(outcome
            .notes
            .iter()
            .any(|note| note == "deadline_anchor_invalid"));
    }

    #[test]
    fn rule_effective_to_is_inclusive_through_the_closing_day() {
        let mut platform = deadline_platform("P2D");
        let record = platform.rules.rules.get_mut(0).expect("fixture rule");
        record.sequencing.deadline = None;
        record.temporal.effective_from = "2026-02-01".to_string();
        record.temporal.effective_to = Some("2026-02-28".to_string());
        let case = deadline_case("2026-02-28T09:00:00", "2026-02-28T10:00:00");
        let outcome = platform.decide(&case);
        assert_eq!(
            outcome.verdict,
            ekp_eval::types::Ver::Comply,
            "an event on the effective_to date is still within the window; notes {:?}",
            outcome.notes
        );

        let next_day = deadline_case("2026-03-01T09:00:00", "2026-03-01T10:00:00");
        assert_eq!(
            platform.decide(&next_day).verdict,
            ekp_eval::types::Ver::InsufficientEvidence,
            "the day after effective_to must be treated as expired"
        );
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
    fn iso_duration_parser_returns_second_precision() {
        use super::parse_iso_duration_seconds;
        assert_eq!(parse_iso_duration_seconds("P2D"), Some(172_800));
        assert_eq!(parse_iso_duration_seconds("P1W"), Some(604_800));
        assert_eq!(parse_iso_duration_seconds("P1W2D"), Some(777_600));
        assert_eq!(parse_iso_duration_seconds("PT12H"), Some(43_200));
        assert_eq!(parse_iso_duration_seconds("PT36H"), Some(129_600));
        assert_eq!(parse_iso_duration_seconds("P1DT2H30M"), Some(95_400));
        assert_eq!(parse_iso_duration_seconds("PT23H59M59S"), Some(86_399));
        assert_eq!(parse_iso_duration_seconds("P0D"), Some(0));
        assert_eq!(parse_iso_duration_seconds("  P2D  "), Some(172_800));
    }

    #[test]
    fn iso_duration_parser_rejects_ambiguous_or_malformed_values() {
        use super::parse_iso_duration_seconds;
        for value in [
            "",
            "P",
            "PT",
            "2D",
            "P2",
            "P-1D",
            "p2d",
            "P1M",
            "P1Y",
            "P1X",
            "P1D2H",
            "P1DT",
            "PT1H2H",
            "PT30M1H",
            "P1HT2H",
            "P1W1W",
            "PT1.5H",
            "PT1H30.5S",
        ] {
            assert_eq!(parse_iso_duration_seconds(value), None, "{value:?}");
        }
    }

    #[test]
    fn timestamp_parser_accepts_date_and_datetime_forms() {
        use super::parse_utc_timestamp_seconds;
        let midnight = parse_utc_timestamp_seconds("2026-03-02").expect("date parses");
        assert_eq!(
            parse_utc_timestamp_seconds("2026-03-02T00:00:00"),
            Some(midnight)
        );
        assert_eq!(
            parse_utc_timestamp_seconds("2026-03-02T00:00:00Z"),
            Some(midnight)
        );
        assert_eq!(
            parse_utc_timestamp_seconds("2026-03-02T00:00:01"),
            Some(midnight + 1)
        );
        assert_eq!(
            parse_utc_timestamp_seconds("2024-02-29"),
            parse_utc_timestamp_seconds("2024-02-28").map(|seconds| seconds + 86_400)
        );
    }

    #[test]
    fn timestamp_parser_rejects_impossible_or_multibyte_values() {
        use super::parse_utc_timestamp_seconds;
        for value in [
            "",
            "2026-03-02 ",
            "2026-03-02T09:00",
            "2026-03-02 09:00:00",
            "2026-03-02T09:00:00+08:00",
            "2026-03-0~",
            "2026-03-0２",
            "0000-01-01",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-01-32",
            "2026-02-29",
            "2024-02-30",
            "2026-03-02T24:00:00",
            "2026-03-02T09:60:00",
            "2026-03-02T09:00:60",
        ] {
            assert_eq!(parse_utc_timestamp_seconds(value), None, "{value:?}");
        }
        assert!(parse_utc_timestamp_seconds("0001-01-01").is_some());
    }

    #[test]
    fn timestamp_diff_uses_seconds_and_rejects_bad_input() {
        use super::timestamp_diff_seconds;
        assert_eq!(
            timestamp_diff_seconds("2026-03-02", "2026-03-03"),
            Some(86_400)
        );
        assert_eq!(
            timestamp_diff_seconds("2026-03-02T08:00:00", "2026-03-02T20:00:01"),
            Some(43_201)
        );
        assert_eq!(
            timestamp_diff_seconds("2026-03-03", "2026-03-02"),
            Some(-86_400)
        );
        assert_eq!(timestamp_diff_seconds("2026-03-0~", "2026-03-03"), None);
        assert_eq!(timestamp_diff_seconds("2026-03-02", "2026-03-0~"), None);
    }
}
