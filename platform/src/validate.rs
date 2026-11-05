#![forbid(unsafe_code)]

//! R3：內建迷你 JSON Schema（draft-07 子集）校驗器——合規鏈路零外部新增依賴。
//!
//! 用點：active 規則在進引擎之前以 `--declared-rules-schema` 對
//! `design/rule-schema-v0.1.1.schema.json` 校驗；形狀違規在資料邊界被拒，
//! 而不是靜默流入判定管線。
//!
//! 支援子集（覆蓋該 schema 的全部用點）：
//! - `type`（字串或陣列形式；`integer` 只接受整數值）
//! - `enum` / `required` / `properties` / `additionalProperties`（bool 或 schema）
//! - `items`（單一 schema 形；tuple 形按索引校驗）
//! - `$ref`（僅內部 `#/...` 指標）
//! - `oneOf`（必須恰好一個分支命中，否則記錯誤）
//! - `minimum` / `minLength` / `minItems`
//! - `pattern`（保守正則子集：錨點 `^` `$`、字元類 `[...]`（含範圍與負向）、
//!   字面字元、`.`、量詞 `+` `*` `?`。**未知構造（如反斜線轉義、`|`、群組）
//!   一律 fail-closed 報錯，不靜默放行。**）
//!
//! 選擇自研而非引入 jsonschema crate：合規判定鏈路依賴最小化，每一依賴皆須
//! license 與供應鏈審計（見 research/license-notes.md 與 tech-selection）。
//! 反模擬：校驗錯誤帶 JSON 指標路徑；本模組的單元測試至少包含負向案例，
//! 確保「false PASS」有真機測試把關。

use serde_json::Value;

/// 對 `instance` 校驗；返回錯誤清單（空 = 通過）。schema 內 `$ref` 以 `root` 解析。
pub fn validate(instance: &Value, schema: &Value, root: &Value) -> Vec<String> {
    let mut errs = Vec::new();
    check(instance, schema, root, "$", &mut errs);
    errs
}

fn check(instance: &Value, schema: &Value, root: &Value, path: &str, errs: &mut Vec<String>) {
    match schema {
        Value::Bool(true) => {}
        Value::Bool(false) => errs.push(format!("{path}: schema rejects any value")),
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                let Some(target) = resolve_pointer(root, reference) else {
                    errs.push(format!("{path}: unresolvable internal $ref {reference}"));
                    return;
                };
                check(instance, target, root, path, errs);
                return;
            }

            if let Some(branches) = map.get("oneOf").and_then(Value::as_array) {
                let mut matched = 0usize;
                let mut first_errs: Option<Vec<String>> = None;
                for branch in branches {
                    let mut sub = Vec::new();
                    check(instance, branch, root, path, &mut sub);
                    if sub.is_empty() {
                        matched += 1;
                    } else if first_errs.is_none() {
                        first_errs = Some(sub);
                    }
                }
                if matched != 1 {
                    errs.push(format!(
                        "{path}: oneOf matched {matched} branches (need exactly 1); first rejection: {:?}",
                        first_errs.unwrap_or_default().first().cloned().unwrap_or_default()
                    ));
                }
            }

            if let Some(type_spec) = map.get("type")
                && !type_conforms(type_spec, instance)
            {
                errs.push(format!(
                    "{path}: type {}",
                    describe_expected_type(type_spec)
                ));
                return;
            }

            if let Some(variants) = map.get("enum").and_then(Value::as_array)
                && !variants.iter().any(|v| v == instance)
            {
                errs.push(format!("{path}: value not in enum"));
            }

            match instance {
                Value::Object(obj) => {
                    if let Some(required) = map.get("required").and_then(Value::as_array) {
                        for key in required {
                            match key.as_str() {
                                Some(k) => {
                                    if !obj.contains_key(k) {
                                        errs.push(format!("{path}.{k}: missing required property"));
                                    }
                                }
                                None => errs.push(format!("{path}: malformed required list")),
                            }
                        }
                    }
                    let props = map.get("properties").and_then(Value::as_object);
                    if let Some(props) = props {
                        for (k, sub) in props {
                            if let Some(v) = obj.get(k) {
                                let p = format!("{path}.{k}");
                                check(v, sub, root, &p, errs);
                            }
                        }
                    }
                    if let Some(additional) = map.get("additionalProperties") {
                        let extras: Vec<(&String, &Value)> = obj
                            .iter()
                            .filter(|(k, _)| {
                                map.get("properties")
                                    .and_then(Value::as_object)
                                    .is_none_or(|p| !p.contains_key(*k))
                            })
                            .collect();
                        match additional {
                            Value::Bool(false) => {
                                for (k, _) in extras {
                                    errs.push(format!(
                                        "{path}.{k}: additionalProperties=false forbids this key"
                                    ));
                                }
                            }
                            Value::Bool(true) => {}
                            sub_schema => {
                                for (k, v) in extras {
                                    let p = format!("{path}.{k}");
                                    check(v, sub_schema, root, &p, errs);
                                }
                            }
                        }
                    }
                }
                Value::Array(items) => {
                    if let Some(item_schema) = map.get("items") {
                        match item_schema {
                            Value::Array(tuple) => {
                                for (i, item) in items.iter().enumerate().take(tuple.len()) {
                                    let p = format!("{path}[{i}]");
                                    check(item, &tuple[i], root, &p, errs);
                                }
                            }
                            Value::Object(_) | Value::Bool(_) => {
                                for (i, item) in items.iter().enumerate() {
                                    let p = format!("{path}[{i}]");
                                    check(item, item_schema, root, &p, errs);
                                }
                            }
                            _ => errs.push(format!("{path}: unsupported items schema form")),
                        }
                    }
                    if let Some(min) = map.get("minItems").and_then(Value::as_u64)
                        && (items.len() as u64) < min
                    {
                        errs.push(format!("{path}: minItems {min} (got {})", items.len()));
                    }
                }
                Value::String(text) => {
                    if let Some(min) = map.get("minLength").and_then(Value::as_u64)
                        && (text.chars().count() as u64) < min
                    {
                        errs.push(format!(
                            "{path}: minLength {min} (got {})",
                            text.chars().count()
                        ));
                    }
                    if let Some(pattern) = map.get("pattern").and_then(Value::as_str) {
                        match pattern_matches(pattern, text) {
                            Ok(true) => {}
                            Ok(false) => errs.push(format!("{path}: pattern {pattern} (got {text:?})")),
                            Err(reason) => errs.push(format!(
                                "{path}: pattern {pattern} not in supported subset — fail-closed ({reason})"
                            )),
                        }
                    }
                }
                Value::Number(num) => {
                    if let Some(min) = map.get("minimum").and_then(Value::as_f64)
                        && num.as_f64().is_some_and(|v| v < min)
                    {
                        errs.push(format!("{path}: minimum {min} (got {num})"));
                    }
                }
                _ => {}
            }
        }
        _ => errs.push(format!("{path}: unsupported schema form")),
    }
}

fn describe_expected_type(type_spec: &Value) -> String {
    match type_spec {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("|"),
        _ => "unknown".to_string(),
    }
}

fn type_conforms(type_spec: &Value, instance: &Value) -> bool {
    let names: Vec<&str> = match type_spec {
        Value::String(s) => vec![s.as_str()],
        Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
        _ => return false,
    };
    names.iter().any(|name| match (*name, instance) {
        ("object", Value::Object(_))
        | ("array", Value::Array(_))
        | ("string", Value::String(_))
        | ("boolean", Value::Bool(_))
        | ("null", Value::Null)
        | ("number", Value::Number(_)) => true,
        ("integer", Value::Number(n)) => n.is_u64() || n.is_i64(),
        _ => false,
    })
}

fn resolve_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    let rest = pointer.strip_prefix('#').unwrap_or(pointer);
    if rest.is_empty() {
        return Some(root);
    }
    let rest = rest.strip_prefix('/')?;
    let mut cur = root;
    for seg in rest.split('/') {
        let decoded = seg.replace("~1", "/").replace("~0", "~");
        cur = match cur {
            Value::Object(map) => map.get(&decoded)?,
            Value::Array(arr) => arr.get(decoded.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(cur)
}

// ─── pattern 子集 ─────────────────────────────────────────────────────────

enum Piece {
    Start,
    End,
    Atom(Atom),
    Plus(Atom),
    Star(Atom),
    Quest(Atom),
}

enum Atom {
    Char(char),
    Any,
    Class {
        negated: bool,
        ranges: Vec<(char, char)>,
        singles: Vec<char>,
    },
}

fn parse_pattern(pattern: &str) -> Result<Vec<Piece>, String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut pieces = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '^' => pieces.push(Piece::Start),
            '$' => pieces.push(Piece::End),
            '.' => pieces.push(Piece::Atom(Atom::Any)),
            '[' => {
                let (atom, next) = parse_class(&chars, i)?;
                i = next;
                pieces.push(quantifier(&chars, &mut i, atom)?);
                i += 1; // consume ']'
                continue;
            }
            '*' | '+' | '?' => return Err(format!("quantifier `{c}` without preceding atom")),
            '{' | '(' | ')' | '|' => return Err(format!("unsupported construct `{c}`")),
            '\\' => return Err("escape sequences not supported".to_string()),
            other => pieces.push(quantifier(&chars, &mut i, Atom::Char(other))?),
        }
        i += 1;
    }
    Ok(pieces)
}

fn quantifier(chars: &[char], i: &mut usize, atom: Atom) -> Result<Piece, String> {
    match chars.get(*i + 1) {
        Some('+') => {
            *i += 1;
            Ok(Piece::Plus(atom))
        }
        Some('*') => {
            *i += 1;
            Ok(Piece::Star(atom))
        }
        Some('?') => {
            *i += 1;
            Ok(Piece::Quest(atom))
        }
        _ => Ok(Piece::Atom(atom)),
    }
}

fn parse_class(chars: &[char], start: usize) -> Result<(Atom, usize), String> {
    debug_assert_eq!(chars[start], '[');
    let mut i = start + 1;
    let mut negated = false;
    if chars.get(i) == Some(&'^') {
        negated = true;
        i += 1;
    }
    let mut ranges: Vec<(char, char)> = Vec::new();
    let mut singles: Vec<char> = Vec::new();
    let mut first = true;
    loop {
        let Some(&c) = chars.get(i) else {
            return Err("unterminated character class".to_string());
        };
        if c == ']' && !first {
            return Ok((
                Atom::Class {
                    negated,
                    ranges,
                    singles,
                },
                i,
            ));
        }
        first = false;
        if c == '\\' {
            return Err("escape sequences not supported".to_string());
        }
        // 範圍 a-b：僅當 - 出現於兩個非邊界字元之間。
        if chars.get(i + 1) == Some(&'-') && chars.get(i + 2).is_some_and(|&n| n != ']') {
            let hi = chars[i + 2];
            if hi < c {
                return Err(format!("invalid range `{c}-{hi}`"));
            }
            ranges.push((c, hi));
            i += 3;
            continue;
        }
        singles.push(c);
        i += 1;
    }
}

fn atom_matches(atom: &Atom, text: &[char], pos: usize) -> Option<usize> {
    let ch = *text.get(pos)?;
    let hit = match atom {
        Atom::Any => true,
        Atom::Char(c) => ch == *c,
        Atom::Class {
            negated,
            ranges,
            singles,
        } => {
            let inside =
                singles.contains(&ch) || ranges.iter().any(|(lo, hi)| *lo <= ch && ch <= *hi);
            inside != *negated
        }
    };
    hit.then(|| pos + 1)
}

fn match_pieces(pieces: &[Piece], text: &[char], pos: usize) -> bool {
    let Some((piece, rest)) = pieces.split_first() else {
        return true;
    };
    match piece {
        Piece::Start => {
            if pos == 0 {
                match_pieces(rest, text, pos)
            } else {
                false
            }
        }
        Piece::End => {
            if pos == text.len() {
                match_pieces(rest, text, pos)
            } else {
                false
            }
        }
        Piece::Atom(atom) => atom_matches(atom, text, pos)
            .map(|next| match_pieces(rest, text, next))
            .unwrap_or(false),
        Piece::Quest(atom) => {
            match_pieces(rest, text, pos)
                || atom_matches(atom, text, pos)
                    .map(|next| match_pieces(rest, text, next))
                    .unwrap_or(false)
        }
        Piece::Star(atom) => star_body(atom, rest, text, pos),
        Piece::Plus(atom) => match atom_matches(atom, text, pos) {
            None => false,
            Some(next) => star_body(atom, rest, text, next),
        },
    }
}
/// 已消費一個之後的重複主體；貪婪優先、回溯確定性。
fn star_body(atom: &Atom, rest: &[Piece], text: &[char], pos: usize) -> bool {
    if match_pieces(rest, text, pos) {
        return true;
    }
    match atom_matches(atom, text, pos) {
        Some(next) => star_body(atom, rest, text, next),
        None => false,
    }
}

/// 正則子集匹配（draft-07 語義：無 ^ 錨點時為「搜尋」語義）。
/// 解析失敗 → Err（呼叫端 fail-closed）。
fn pattern_matches(pattern: &str, text: &str) -> Result<bool, String> {
    let pieces = parse_pattern(pattern)?;
    let chars: Vec<char> = text.chars().collect();
    let anchored_start = matches!(pieces.first(), Some(Piece::Start));
    let starts: Vec<usize> = if anchored_start {
        vec![0]
    } else {
        (0..=chars.len()).collect()
    };
    Ok(starts.iter().any(|&s| match_pieces(&pieces, &chars, s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../design/rule-schema-v0.1.1.schema.json"
    );
    const RULES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/rules.json");

    fn schema() -> Value {
        let text = std::fs::read_to_string(SCHEMA_PATH).expect("schema file loads");
        serde_json::from_str(&text).expect("schema parses")
    }

    #[test]
    fn schema_v011_accepts_every_recorded_rule() {
        let root = schema();
        let rules_text = std::fs::read_to_string(RULES_PATH).expect("rules.json loads");
        let rules: Value = serde_json::from_str(&rules_text).expect("rules.json parses");
        for (i, record) in rules
            .get("rules")
            .and_then(Value::as_array)
            .expect("rules array")
            .iter()
            .enumerate()
        {
            let errs = validate(record, &root, &root);
            assert!(
                errs.is_empty(),
                "record {i} ({}) violates schema: {errs:?}",
                record.get("rule_id").and_then(Value::as_str).unwrap_or("?")
            );
        }
    }

    #[test]
    fn rejects_unknown_top_level_property() {
        let root = schema();
        let base = serde_json::json!({
            "rule_id": "RULE-X",
            "rule_version": "1.0",
            "deontic": {"modality": "obligation", "strength": "mandatory"},
            "subject": {"entity_type": "t", "actor_role": null, "scope": {}},
            "condition": {"all": []},
            "evidence_requirements": [],
            "decision": {"method": "deterministic_rule_check", "missing_evidence_result": "insufficient_evidence"},
            "source": {"document_id": "D", "document_version": "1", "clause_refs": [], "verbatim_text": "t"},
            "provenance": {"origin": "authoritative_text", "review": {"status": "active"}},
            "temporal": {"effective_from": "2026-01-01"}
        });
        let mut injected = base.clone();
        injected.as_object_mut().expect("object").insert(
            "engine_backdoor".to_string(),
            serde_json::json!("allow_all"),
        );
        let errs = validate(&injected, &root, &root);
        assert!(
            errs.iter().any(|e| e.contains("engine_backdoor")),
            "{errs:?}"
        );

        // 移除 backdoor 後的乾淨記錄應通過（正向对照）。
        let errs = validate(&base, &root, &root);
        assert!(errs.is_empty(), "clean record must validate: {errs:?}");
    }

    #[test]
    fn rejects_enum_violation_and_bad_integer() {
        let sub = serde_json::json!({"modality": "whatever", "strength": "mandatory"});
        let deontic = schema()
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|p| p.get("deontic"))
            .cloned()
            .expect("deontic schema");
        let errs = validate(&sub, &deontic, &serde_json::json!({}));
        assert!(errs.iter().any(|e| e.contains("enum")), "{errs:?}");

        let ser = schema()
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|p| p.get("sequencing"))
            .and_then(Value::as_object)
            .and_then(|s| s.get("properties"))
            .and_then(Value::as_object)
            .and_then(|s| s.get("loop_policy"))
            .and_then(Value::as_object)
            .and_then(|s| s.get("properties"))
            .and_then(Value::as_object)
            .and_then(|s| s.get("failures_cap"))
            .cloned()
            .expect("failures_cap schema");
        let errs = validate(&serde_json::json!(0), &ser, &serde_json::json!({}));
        assert!(errs.iter().any(|e| e.contains("minimum")), "{errs:?}");
    }

    #[test]
    fn pattern_subset_semantics() {
        // schema 實際使用的形式。
        assert!(pattern_matches("^[A-Z][A-Z0-9-]+$", "RULE-CHG-001").unwrap_or(false));
        assert!(!pattern_matches("^[A-Z][A-Z0-9-]+$", "rule-x").unwrap_or(true));
        assert!(!pattern_matches("^[A-Z][A-Z0-9-]+$", "X-含中文").unwrap_or(true));
        assert!(pattern_matches("^[A-Z][A-Z0-9-]+$", "A-9").unwrap_or(false));
    }

    #[test]
    fn unsupported_pattern_fails_closed() {
        let errs = validate(
            &serde_json::json!("12345"),
            &serde_json::json!({"type": "string", "pattern": "\\d+"}),
            &serde_json::json!({}),
        );
        assert!(errs.iter().any(|e| e.contains("fail-closed")), "{errs:?}");
    }

    #[test]
    fn one_of_requires_exactly_one() {
        let root_schema = schema();
        let sub = root_schema
            .get("definitions")
            .and_then(Value::as_object)
            .and_then(|d| d.get("conditionNode"))
            .cloned()
            .expect("conditionNode schema");
        // 既無 predicate 過應命中 all/any/not 任一 → 0 個分支。
        let errs = validate(&serde_json::json!({"other": 1}), &sub, &root_schema);
        assert!(errs.iter().any(|e| e.contains("oneOf")), "{errs:?}");
        // 合法 leaf。
        let errs = validate(
            &serde_json::json!({"predicate": {"field": "actor_role", "op": "eq", "value": "QA"}}),
            &sub,
            &root_schema,
        );
        assert!(errs.is_empty(), "{errs:?}");
    }
}
