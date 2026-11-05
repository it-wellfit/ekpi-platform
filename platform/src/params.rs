#![forbid(unsafe_code)]

//! R3：參數治理檔（CLI `--params`）。
//!
//! 兩類參數、一條治理紀律：
//! 1. **declared_invariants**——引擎語義不變參數的「宣告檔」：引擎碼中本無
//!    對應開關（決策路徑全確定性，見 engine-spec §1 各 D 節）；此檔以
//!    fail-closed 方式把不變參數鎖死為唯一可選值，未被宣告的鍵、缺鍵、
//!    值型非 bool、鎖定值不符，一律載入失敗（拒執行）。對應註解標出
//!    語源節（D-xx/H-x/A-x）。
//! 2. **adjustable**——既定架構可調參數（非硬編碼語義）：
//!    `retrieval_threshold`（檢索閾值；DEFAULT_RETRIEVAL_THRESHOLD 同值），
//!    `--threshold` CLI 覆寫之。
//!
//! 反模擬：治理檔被篡改（如 `forbidden_pair_release_allowed: true`）必須
//! 不可裝載——單元測試含負向案例把守。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const PARAMS_VERSION: &str = "1.0";

/// 鎖定表：鍵 → 引擎語義唯一合法值。鍵集與值集皆 fail-closed。
fn locked_invariants() -> BTreeMap<&'static str, bool> {
    BTreeMap::from([
        // A1/A8、D-02：非 active 規則不可被引擎消費。
        ("draft_rules_consumable", false),
        // D-02：多筆 active 紀錄同時命中一件 binding → Ambiguous → 拒答。
        ("ambiguous_binding_refuses", true),
        // D-02 顯式聲明主義：未登錄 rule_id 不猜測語義 → 拒答。
        ("unregistered_rule_guessing_allowed", false),
        // D-04：deadline 嚴格大於，等於不觸發。
        ("equal_timestamp_counts_as_overdue", false),
        // D-21/H-3：insufficient_evidence 拒答一律清空引用。
        ("abstain_clears_citations", true),
        // H-1：forbidden pair 觀測到即 violate，release 為 0。
        ("forbidden_pair_release_allowed", false),
        // H-5：文件內嵌指令只登記不執行。
        ("injection_directive_operates_as_system_rule", false),
        // D-26：判定路徑零機率成分。
        ("decision_path_contains_probability_component", false),
    ])
}

/// 可調參數之外層結構（未列鍵拒收）。
#[derive(Deserialize, Debug)]
struct Adjustable {
    retrieval_threshold: f64,
}

/// 治理檔外層結構。`spec_ref` / `notes` 為治理檔的說明文字：白名單允許存在、
/// 快速路徑不讀（serde 靜默略過它們）；治理語義全部由鍵與鎖定值承載。
#[derive(Deserialize, Debug)]
struct ParamsFile {
    params_version: String,
    declared_invariants: BTreeMap<String, bool>,
    adjustable: Adjustable,
}

/// 載入結果：決策相關的可調參數（不變參數僅做載入時驗證，語義不由此流動）。
#[derive(Debug, Clone)]
pub struct Params {
    pub retrieval_threshold: f64,
}

pub fn default_params_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("engine-params.json")
}

pub fn load(path: &Path) -> Result<Params, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text)
}

/// 解析治理檔（測試直接呼叫；`load` 的無檔案依賴核心）。
fn parse(text: &str) -> Result<Params, String> {
    // 先以 Value 檢視頂層鍵集，再定點反序列化——兩步都是 fail-closed。
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("params parse error: {e}"))?;
    let Some(obj) = value.as_object() else {
        return Err("params must be a JSON object".to_string());
    };
    let allowed = [
        "params_version",
        "spec_ref",
        "notes",
        "declared_invariants",
        "adjustable",
    ];
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!(
                "params: unknown top-level key `{key}` (fail-closed)"
            ));
        }
    }

    let file: ParamsFile =
        serde_json::from_value(value).map_err(|e| format!("params parse error: {e}"))?;
    if file.params_version != PARAMS_VERSION {
        return Err(format!(
            "params: params_version {} not supported (locked to {PARAMS_VERSION})",
            file.params_version
        ));
    }

    let locked = locked_invariants();
    // 未知鍵／值型不符已被上一層與類型擋下；此處比對缺鍵與值不符。
    for (key, expected) in &locked {
        match file.declared_invariants.get(*key) {
            None => {
                return Err(format!(
                    "params: declared_invariants.{key} missing (every invariant must be declared explicitly)"
                ));
            }
            Some(actual) if actual == expected => {}
            Some(actual) => {
                return Err(format!(
                    "params: declared_invariants.{key}={actual} contradicts locked engine semantics {expected} — refuse to run ({key} has no engine switch; tampering refused)"
                ));
            }
        }
    }
    for key in file.declared_invariants.keys() {
        if !locked.contains_key(key.as_str()) {
            return Err(format!(
                "params: declared_invariants.{key} is not a declared invariant (fail-closed)"
            ));
        }
    }

    let threshold = file.adjustable.retrieval_threshold;
    if !(0.0..=1.0).contains(&threshold) {
        return Err(format!(
            "params: adjustable.retrieval_threshold {threshold} outside [0,1]"
        ));
    }
    Ok(Params {
        retrieval_threshold: threshold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // SINGLE SOURCE OF TRUTH：治理檔的閾值與引擎常數同值（避免複製常數）。
    use crate::engine::DEFAULT_RETRIEVAL_THRESHOLD;

    fn default_text() -> String {
        std::fs::read_to_string(default_params_path()).expect("engine-params.json exists")
    }

    #[test]
    fn default_params_load_with_locked_threshold() {
        let params = parse(&default_text()).expect("shipped params parse");
        assert!(
            (params.retrieval_threshold - DEFAULT_RETRIEVAL_THRESHOLD).abs() < f64::EPSILON,
            "shipped threshold must mirror DEFAULT_RETRIEVAL_THRESHOLD: {}",
            params.retrieval_threshold
        );
    }

    #[test]
    fn tampered_invariant_is_refused() {
        let tampered = default_text().replace(
            "\"forbidden_pair_release_allowed\": false",
            "\"forbidden_pair_release_allowed\": true",
        );
        let err = parse(&tampered).expect_err("tampering must fail-closed");
        assert!(err.contains("forbidden_pair_release_allowed"), "{err}");
    }

    #[test]
    fn unknown_and_missing_invariants_fail_closed() {
        let with_unknown = default_text().replace(
            "\"draft_rules_consumable\": false",
            "\"draft_rules_consumable\": false,\n    \"engine_llm_fallback\": true",
        );
        let err = parse(&with_unknown).expect_err("unknown invariant refused");
        assert!(err.contains("not a declared invariant"), "{err}");

        let missing = default_text().replace("\"abstain_clears_citations\": true,\n", "");
        let err = parse(&missing).expect_err("missing invariant refused");
        assert!(err.contains("missing"), "{err}");
    }

    #[test]
    fn threshold_domain_and_version_locked() {
        let out_of_range = default_text().replace("0.70", "1.5");
        assert!(
            parse(&out_of_range)
                .err()
                .unwrap_or_default()
                .contains("outside [0,1]")
        );
        let bad_version =
            default_text().replace("\"params_version\": \"1.0\"", "\"params_version\": \"2.0\"");
        assert!(
            parse(&bad_version)
                .err()
                .unwrap_or_default()
                .contains("not supported")
        );
        // 頂層未知鍵。
        let extra = default_text().replace(
            "\"declared_invariants\"",
            "\"engine_overrides\": {},\n  \"declared_invariants\"",
        );
        assert!(
            parse(&extra)
                .err()
                .unwrap_or_default()
                .contains("unknown top-level key")
        );
    }
}
