#![forbid(unsafe_code)]

//! R3：規則覆蓋率報告（CLI `--coverage-report`）與最小 markdown→HTML 渲染
//! （CLI `--html`）。庫界面：`ekp_platform` binary 消費之；本模組的單元
//! 測試位於此（binary 本體不再攜帶測試目標）。
//!
//! 反模擬：`coverage_markdown` 對兩個案例源（seed_cases.jsonl 9 案＋
//! casebook_v3.jsonl 46 案）逐一呼叫 [`crate::engine::Platform::decide`]——
//! 報告中的全部數字皆來自本次真機執行，非快取或常數。
//!
//! `markdown_to_html` 為不引入外部依賴的最小渲染：`#`/`##`/`###` 標題、
//! `|` 表格（`|---|` 分隔行跳過）、段落；全部文字 HTML 逃逸（& < >）；
//! 輸入先剝 BOM。遵循良構：`<!DOCTYPE html>` + `lang="zh-Hant"` + charset。

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::engine::Platform;

use ekp_eval::types;

pub fn default_cases_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("eval").join("cases").join("seed_cases.jsonl"))
        .unwrap_or_else(|| PathBuf::from("../eval/cases/seed_cases.jsonl"))
}
/// casebook 案例預設路徑（manifest 上層 `eval/cases/casebook_v3.jsonl`）。
pub fn default_casebook_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("eval").join("cases").join("casebook_v3.jsonl"))
        .unwrap_or_else(|| PathBuf::from("../eval/cases/casebook_v3.jsonl"))
}

/// 載入一個案例源；形狀警告逐行輸出 stderr（不靜默吞掉）。
pub fn load_cases_at(cases_path: &std::path::Path) -> Result<Vec<types::Case>, String> {
    let (cases, warnings) = types::load_cases(cases_path)?;
    for w in &warnings {
        eprintln!("case-shape-warning: {w}");
    }
    Ok(cases)
}

/// 覆蓋率統計的單規則累積器（按 rule_id 粒度；引擎 `rule_ids_used` 同粒度）。
#[derive(Default)]
struct RuleStat {
    cases: BTreeSet<String>,
    verdicts: BTreeMap<String, usize>,
}

/// R3 `--coverage-report`：seed 9 案 + casebook 46 案真機逐案 decide，
/// 每條 active 規則統計被觸及案件、verdict 分布、觀測引用（anchors ∩ 引用池）；
/// 未被觸及的規則列示。反模擬：全部數字來自本次真機執行，非快取/常數。
pub fn coverage_markdown(platform: &Platform) -> Result<String, String> {
    let sources: [(&str, PathBuf); 2] = [
        ("seed_cases.jsonl", default_cases_path()),
        ("casebook_v3.jsonl", default_casebook_path()),
    ];

    let mut source_counts: Vec<(String, usize)> = Vec::new();
    let mut total_cases = 0usize;
    // 觀測引用池：非拒答案件的全部引用（去重）。
    let mut citation_pool: BTreeSet<String> = BTreeSet::new();
    let mut stats: BTreeMap<String, RuleStat> = BTreeMap::new();

    for (name, path) in sources {
        let cases = load_cases_at(&path)?;
        for case in &cases {
            let outcome = platform.decide(case);
            for citation in &outcome.citations {
                citation_pool.insert(citation.clone());
            }
            for rule_id in &outcome.rule_ids_used {
                let stat = stats.entry(rule_id.clone()).or_default();
                stat.cases.insert(case.case_id.clone());
                *stat
                    .verdicts
                    .entry(outcome.verdict.as_str().to_string())
                    .or_insert(0) += 1;
            }
            total_cases += 1;
        }
        source_counts.push((name.to_string(), cases.len()));
    }

    // 每條 active 規則的條款錨點（同 id 多版本皆列出）。
    let mut anchors_by_id: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for record in platform.rules.active_rules() {
        let anchors = record
            .source
            .clause_refs
            .iter()
            .map(|c| {
                format!(
                    "{}@{} §{}",
                    record.source.document_id, record.source.document_version, c.section
                )
            })
            .collect::<Vec<_>>();
        anchors_by_id
            .entry(record.rule_id.clone())
            .or_default()
            .extend(anchors);
    }

    let mut md = String::new();
    md.push_str("# EKPI 規則覆蓋率報告（真機統計）\n\n");
    md.push_str(
        "此報告由 CLI `--coverage-report` 於本次真機執行生成：兩個案例源逐案呼叫判定引擎，數字不是常數。\n\n",
    );
    md.push_str("| 案例源 | 案數 |\n|---|---|\n");
    for (name, count) in &source_counts {
        md.push_str(&format!("| {name} | {count} |\n"));
    }
    md.push_str(&format!(
        "| 合計 | {total_cases} |\n\n## 每條 active 規則的覆蓋\n\n"
    ));
    md.push_str(
        "| rule_id | 條款錨點（doc@ver §sec） | 觸及案數 | verdict 分布 | 觀測引用（錨點∩引用池） |\n|---|---|---|---|---|\n",
    );

    let mut covered_ids = 0usize;
    // 主表：以 active 規則 id 為完整清單（0 案觸及也列出，讓未覆蓋一目了然）。
    for (rule_id, anchors) in &anchors_by_id {
        let anchors_text = if anchors.is_empty() {
            "-".to_string()
        } else {
            anchors.join("; ")
        };
        let stat = stats.get(rule_id);
        let cases_count = stat.map_or(0, |s| s.cases.len());
        if stat.is_some() {
            covered_ids += 1;
        }
        let verdicts = stat.map_or("-".to_string(), |s| {
            s.verdicts
                .iter()
                .map(|(v, n)| format!("{v}={n}"))
                .collect::<Vec<_>>()
                .join("; ")
        });
        let observed: Vec<&String> = if stat.is_some() {
            citation_pool
                .iter()
                .filter(|c| anchors.contains(*c))
                .collect()
        } else {
            Vec::new()
        };
        let observed_text = if observed.is_empty() {
            "-".to_string()
        } else {
            observed
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        };
        md.push_str(&format!(
            "| {rule_id} | {anchors_text} | {cases_count} | {verdicts} | {observed_text} |\n"
        ));
    }

    // 引擎觸及的非 active 引用 id：declared 引用被引擎登記，但庫內無
    // active 紀錄（draft 規則 A1/A8 不可消費；未登記 rule_id D-02 不猜測）。
    // 兩者治理語義一致——顯式 note + 保守拒答，不進入判定路徑。
    let nonactive_touched: Vec<&String> = stats
        .keys()
        .filter(|id| !anchors_by_id.contains_key(*id))
        .collect();
    md.push_str("\n## 引擎觸及的非 active 引用 id\n\n");
    if nonactive_touched.is_empty() {
        md.push_str("無——引擎觸及的每個 id 皆有 active 紀錄。\n");
    } else {
        md.push_str(
            "以下 declared 引用被引擎登記但庫內無 active 紀錄；依治理語義以顯式 note 處置並保守拒答（A1/A8：draft 規則不可消費；D-02：未登記規則不猜測）：\n\n",
        );
        for id in &nonactive_touched {
            let stat = &stats[*id];
            let verdicts = stat
                .verdicts
                .iter()
                .map(|(v, n)| format!("{v}={n}"))
                .collect::<Vec<_>>()
                .join("; ");
            md.push_str(&format!("- {id}：{} 案（{verdicts}）\n", stat.cases.len()));
        }
    }

    // 未覆蓋規則：active 但 0 案件觸及（審計用途，需人工研討補案）。
    let uncovered: Vec<&String> = anchors_by_id
        .keys()
        .filter(|id| !stats.contains_key(*id))
        .collect();
    md.push_str("\n## 未覆蓋規則\n\n");
    if uncovered.is_empty() {
        md.push_str("無——每一條 active 規則皆被至少一案件觸及。\n");
    } else {
        md.push_str("以下規則載入後從未被任何案例觸及（依 active 記錄顯列，供案例補強討論）：\n\n");
        for id in &uncovered {
            md.push_str(&format!("- {id}\n"));
        }
    }

    let active_ids = anchors_by_id.len();
    md.push_str("\n## 總結\n\n");
    let count_of = |name: &str| -> usize {
        source_counts
            .iter()
            .find(|(n, _)| n == name)
            .map_or(0, |(_, c)| *c)
    };
    md.push_str(&format!(
        "執行案件合計 {total_cases}（seed {} + casebook {}，全部真機 decide）；\
         active 規則 id {active_ids} 個，覆蓋 {covered_ids} 個（0 案觸及 {} 個）；\
         引擎觸及的非 active 引用 id {} 個；觀測引用池 {} 條。\n",
        count_of("seed_cases.jsonl"),
        count_of("casebook_v3.jsonl"),
        uncovered.len(),
        nonactive_touched.len(),
        citation_pool.len()
    ));
    Ok(md)
}

/// HTML 逃逸（`--html` 的最小集：& < >）。
fn esc_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// R3 `--html`：把 report.md（自 `report::write` 產出，帶 BOM）渲染為
/// 最小 HTML。詳見模組頭。
pub fn markdown_to_html(md: &str) -> String {
    let md = md.trim_start_matches('\u{FEFF}');
    let mut out = String::with_capacity(md.len() * 2 + 256);
    out.push_str("<!DOCTYPE html>\n<html lang=\"zh-Hant\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n<title>EKPI eval report</title>\n");
    out.push_str(
        "<style>body{font-family:sans-serif;margin:2rem}table{border-collapse:collapse}th,td{border:1px solid #888;padding:4px 8px}code{font-family:monospace}</style>\n",
    );
    out.push_str("</head>\n<body>\n");

    let mut in_table = false;
    for line in md.lines() {
        let trimmed = line.trim_end_matches('\r').trim();
        if trimmed.is_empty() {
            if in_table {
                out.push_str("</table>\n");
                in_table = false;
            }
            continue;
        }
        // 標題行：`#`..=`###`（4 級以上按段落處理；來源 md 僅生成 1 級）。
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=3).contains(&hashes) {
            if in_table {
                out.push_str("</table>\n");
                in_table = false;
            }
            out.push_str(&format!(
                "<h{hashes}>{}</h{hashes}>\n",
                esc_html(trimmed[hashes..].trim_start())
            ));
            continue;
        }
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // 表格列：split 後剝去首尾空段；`|---|` 分隔行跳過。
            let mut cells: Vec<&str> = trimmed.split('|').collect();
            cells.pop();
            cells.remove(0);
            let separator_row = !cells.is_empty()
                && cells.iter().all(|c| {
                    let t = c.trim();
                    !t.is_empty() && t.chars().all(|ch| ch == '-' || ch == ':')
                });
            if separator_row {
                continue;
            }
            if !in_table {
                out.push_str("<table>\n");
                in_table = true;
            }
            out.push_str("<tr>");
            for cell in cells {
                out.push_str(&format!("<td>{}</td>", esc_html(cell.trim())));
            }
            out.push_str("</tr>\n");
            continue;
        }
        if in_table {
            out.push_str("</table>\n");
            in_table = false;
        }
        out.push_str(&format!("<p>{}</p>\n", esc_html(trimmed)));
    }
    if in_table {
        out.push_str("</table>\n");
    }
    out.push_str("</body>\n</html>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_markdown_covers_every_active_rule_id() {
        let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        let platform = Platform::load(&data_dir).expect("platform data loads for coverage test");
        let md = coverage_markdown(&platform).expect("coverage markdown produced");

        assert!(md.contains("覆蓋率"), "header must say 覆蓋率");
        assert!(md.contains("seed_cases.jsonl"));
        assert!(md.contains("casebook_v3.jsonl"));
        assert!(md.contains("seed_cases.jsonl | 9"), "seed count row");
        assert!(md.contains("casebook_v3.jsonl | 46"), "casebook count row");

        // 每條 active 規則 id 都必須出現在主表（0 案也列行）。
        let active_ids = platform.rules.active_rules().map(|r| r.rule_id.clone());
        for id in active_ids.collect::<BTreeSet<_>>() {
            assert!(
                md.contains(&format!("| {id} |")),
                "active rule {id} must appear in coverage table"
            );
        }
        // 非 active 引用（draft / 未登記）另列小節。
        assert!(md.contains("引擎觸及的非 active 引用"), "{md}");
        assert!(
            md.contains("- RULE-CHG-001-DRAFT"),
            "draft id listed in non-active section"
        );
        assert!(
            md.contains("- RULE-NOSUCH-999"),
            "unregistered id listed in non-active section"
        );
    }

    #[test]
    fn markdown_to_html_renders_table_and_escapes() {
        let md = "# 標題\n\n| a | b & c |\n|---|---|\n| x <y | z |\n\n正文 <>\n";
        let html = markdown_to_html(md);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("lang=\"zh-Hant\""));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<h1>標題</h1>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("<td>a</td><td>b &amp; c</td>"));
        assert!(html.contains("<td>x &lt;y</td><td>z</td>"), "{html}");
        // `|---|` 分隔行不得轉為輸出列。
        assert!(!html.contains("<td>---"), "{html}");
        assert!(html.contains("<p>正文 &lt;&gt;</p>"));
    }

    #[test]
    fn markdown_to_html_strips_bom_and_levels() {
        let md = "\u{FEFF}# 一\n## 二\n### 三\n#### 四\n";
        let html = markdown_to_html(md);
        assert!(!html.starts_with('\u{FEFF}'));
        assert!(html.contains("<h1>一</h1>"));
        assert!(html.contains("<h2>二</h2>"));
        assert!(html.contains("<h3>三</h3>"));
        // 4 級以上按段落處理（來源 md 僅生成 1~3 級；不做冒進渲染）。
        assert!(html.contains("<p>#### 四</p>"), "{html}");
    }

    #[test]
    fn default_casebook_path_exists_and_carries_46_cases() {
        let path = default_casebook_path();
        assert!(path.exists(), "{} must exist", path.display());
        let cases = load_cases_at(&path).expect("casebook loads");
        assert_eq!(cases.len(), 46, "casebook v3 carries 46 cases");
        assert_eq!(cases[0].case_id, "BC-101");
        assert_eq!(cases[45].case_id, "BC-196");
    }
}
