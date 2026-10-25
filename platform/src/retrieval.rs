#![forbid(unsafe_code)]

//! R2-4：MVP 檢索——字符 bigram cosine 相似度。
//!
//! 用途：跨文件證據定位（例：例外規則綁定的附錄條款必須能在案件可用文件集中
//! 被定位，否則判 `insufficient_evidence`；EVD-001 語義）。語意向量與 reranker
//! 待 R2 後期/ R3 接入；本模組只依賴字面相似度，不調用任何模型。

use crate::docs::DocDatabase;

/// 檢索池中一個候選片段。
#[derive(Debug, Clone)]
pub struct PoolSection {
    pub doc_id: String,
    pub version: String,
    pub section: String,
    pub verbatim_text: String,
}

#[derive(Debug, Clone)]
pub struct RetrievalHit {
    pub doc_id: String,
    pub version: String,
    pub section: String,
    pub score: f64,
}

/// 對案件可用文件集建立的檢索器（pool 由引擎依 case doc_versions 組裝）。
pub struct Retriever {
    pool: Vec<PoolSection>,
    threshold: f64,
}

impl Retriever {
    pub fn build(
        db: &DocDatabase,
        doc_versions: &std::collections::BTreeMap<String, String>,
        threshold: f64,
    ) -> Retriever {
        let mut pool = Vec::new();
        for (doc_id, version) in doc_versions {
            if let Some(ver) = db.binding(doc_id, version) {
                for s in &ver.sections {
                    pool.push(PoolSection {
                        doc_id: doc_id.clone(),
                        version: version.clone(),
                        section: s.section.clone(),
                        verbatim_text: s.verbatim_text.clone(),
                    });
                }
            }
        }
        Retriever { pool, threshold }
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// 依字面相似度定位 `anchor` 最相似片段；低於閾值回 None。
    /// 引擎以替換判定：命中 doc/version 與規則綁定不一致視為
    /// 證據歧義 → 保守拒答。
    pub fn locate(&self, anchor: &str) -> Option<RetrievalHit> {
        let mut best: Option<RetrievalHit> = None;
        for cand in &self.pool {
            let score = char_bigram_cosine(anchor, &cand.verbatim_text);
            if score >= self.threshold {
                let hit = RetrievalHit {
                    doc_id: cand.doc_id.clone(),
                    version: cand.version.clone(),
                    section: cand.section.clone(),
                    score,
                };
                best = match best {
                    Some(b) if b.score >= score => Some(b),
                    _ => Some(hit),
                };
            }
        }
        best
    }
}

/// 字符 bigram cosine。中文按 Unicode scalar 拆 bigram；ASCII 亦然；
/// 空串/單字符回退為精確相等比較（全等=1.0，否則 0.0）。
pub fn char_bigram_cosine(a: &str, b: &str) -> f64 {
    let av = bigrams(a);
    let bv = bigrams(b);
    match (av.len(), bv.len()) {
        (0, 0) => {
            if a == b {
                1.0
            } else {
                0.0
            }
        }
        _ => cosine(&av, &bv),
    }
}

fn bigrams(text: &str) -> std::collections::BTreeMap<String, usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    if chars.len() == 1 {
        map.insert(chars[0].to_string(), 1);
        return map;
    }
    for w in chars.windows(2) {
        *map.entry(w.iter().collect::<String>()).or_default() += 1;
    }
    map
}

fn cosine(
    a: &std::collections::BTreeMap<String, usize>,
    b: &std::collections::BTreeMap<String, usize>,
) -> f64 {
    let dot: usize = a.iter().filter_map(|(k, v)| b.get(k).map(|w| v * w)).sum();
    let na = magnitude(a);
    let nb = magnitude(b);
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot as f64 / (na * nb)
}

fn magnitude(m: &std::collections::BTreeMap<String, usize>) -> f64 {
    m.values().map(|v| (v * v) as f64).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_scores_one() {
        let s = "經部門主管核准且屬無 QMS 影響的變更，得免除品質經理複核。";
        assert!((char_bigram_cosine(s, s) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unrelated_text_scores_low() {
        let a = "經部門主管核准且屬無 QMS 影響的變更，得免除品質經理複核。";
        let b = "採購金額達新臺幣壹佰萬元以上者，須依簽核流程取得主管核准後始得下單執行。";
        assert!(char_bigram_cosine(a, b) < 0.7);
    }

    #[test]
    fn partial_overlap_is_finite_signal() {
        let a = "驗證失敗累計達三次（含）以上者，必須停止回退循環並升級人工裁定。";
        let b = "驗證失敗累計達三次時必須停止回退。";
        let s = char_bigram_cosine(a, b);
        assert!(s > 0.2 && s < 0.95, "score={s}");
    }
}
