#![forbid(unsafe_code)]

//! 排名層（free-SOTA，library-only）：BM25／BM25+ 詞彙評分 ＋ RRF 排名融合。
//!
//! 文獻依據（採納決定與完整矩陣見 `docs/research/oekap-sota-frontier_v1.0.md`）：
//! - BM25：Robertson & Zaragoza 2009（"The Probabilistic Relevance Framework"）。
//! - BM25+ 的 delta 修正：Lv & Zhai 2011（長文件偏差修正）。
//! - RRF（k=60 穩健預設）：Cormack et al. 2009；2025 年多份獨立基準
//!   （TREC 2025 RAG track、BEIR SciFact 復現）一致顯示混合融合優於單一
//!   檢索器，且**加掛 cross-encoder reranker 多數不增反損**（延遲成本高）
//!   ——故本層依證據不實作 reranker。
//!
//! 設計約束：零新增依賴、決定性、整數計數＋f64 僅作比較排序（非金額）。
//! 本模組尚未接入服務路徑；接入需先完成門檻重校準（28 案固定集）。

use std::collections::BTreeMap;

/// 混合語彙切詞（決定性）：ASCII 英數連字 → 小寫詞；
/// CJK／非 ASCII 段 → 相鄰字元 bigram（單字長時退為 unigram）。
/// 回傳保留重複（term frequency 由此計算）。
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut ascii = String::new();
    let mut cjk: Vec<char> = Vec::new();

    fn flush_ascii(ascii: &mut String, tokens: &mut Vec<String>) {
        if !ascii.is_empty() {
            tokens.push(ascii.to_lowercase());
            ascii.clear();
        }
    }
    fn flush_cjk(cjk: &mut Vec<char>, tokens: &mut Vec<String>) {
        match cjk.len() {
            0 => {}
            1 => tokens.push(cjk[0].to_string()),
            _ => {
                for pair in cjk.windows(2) {
                    tokens.push(pair.iter().collect());
                }
            }
        }
        cjk.clear();
    }

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            flush_cjk(&mut cjk, &mut tokens);
            ascii.push(ch);
        } else if ch.is_whitespace() || ch.is_ascii_punctuation() {
            flush_ascii(&mut ascii, &mut tokens);
            flush_cjk(&mut cjk, &mut tokens);
        } else {
            flush_ascii(&mut ascii, &mut tokens);
            cjk.push(ch);
        }
    }
    flush_ascii(&mut ascii, &mut tokens);
    flush_cjk(&mut cjk, &mut tokens);
    tokens
}

/// BM25 索引（文件＝已切詞語彙的多重集；統計為整數計數）。
#[derive(Debug)]
pub struct Bm25Index {
    docs: Vec<Vec<String>>,
    doc_freq: BTreeMap<String, usize>,
    avg_len: f64,
}

impl Bm25Index {
    pub fn build(docs: &[String]) -> Self {
        let docs: Vec<Vec<String>> = docs.iter().map(|d| tokenize(d)).collect();
        let mut doc_freq: BTreeMap<String, usize> = BTreeMap::new();
        for tokens in &docs {
            let mut seen: Vec<&String> = tokens.iter().collect();
            seen.sort();
            seen.dedup();
            for token in seen {
                *doc_freq.entry(token.clone()).or_insert(0) += 1;
            }
        }
        let total: usize = docs.iter().map(|d| d.len()).sum();
        let avg_len = if docs.is_empty() || total == 0 {
            1.0 // 空語料防止 0/0 NaN；任何實際分數仍為 0
        } else {
            total as f64 / docs.len() as f64
        };
        Self {
            docs,
            doc_freq,
            avg_len,
        }
    }

    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    pub fn doc_len(&self, doc_idx: usize) -> usize {
        self.docs[doc_idx].len()
    }

    /// 標準 idf（Lucene/BM25+ 形式；恆正）。
    fn idf(&self, term: &str) -> f64 {
        let n = self.docs.len() as f64;
        let df = *self.doc_freq.get(term).unwrap_or(&0) as f64;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// BM25 分數（預設 k1=1.2、b=0.75、delta=0＝經典 BM25；delta>0 即 BM25+）。
    pub fn score(&self, query: &str, doc_idx: usize) -> f64 {
        self.score_with(query, doc_idx, 1.2, 0.75, 0.0)
    }

    pub fn score_with(&self, query: &str, doc_idx: usize, k1: f64, b: f64, delta: f64) -> f64 {
        let doc = &self.docs[doc_idx];
        let len = doc.len() as f64;
        let mut tf: BTreeMap<&str, usize> = BTreeMap::new();
        for token in doc {
            *tf.entry(token.as_str()).or_insert(0) += 1;
        }
        let mut score = 0.0;
        let terms = tokenize(query);
        let mut counted: Vec<&str> = Vec::new();
        for term in &terms {
            let term = term.as_str();
            if counted.contains(&term) {
                continue; // 查詢詞重複只計一次（Robertson & Zaragoza 慣例）
            }
            counted.push(term);
            let f = *tf.get(term).unwrap_or(&0) as f64;
            if f == 0.0 && delta == 0.0 {
                continue;
            }
            let denom = f + k1 * (1.0 - b + b * len / self.avg_len);
            let sat = if denom == 0.0 {
                0.0
            } else {
                f * (k1 + 1.0) / denom
            };
            score += self.idf(term) * (sat + delta);
        }
        score
    }

    /// 排名（分數降冪；平手以文件索引升冪，決定性）。
    pub fn rank(&self, query: &str) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = (0..self.docs.len())
            .map(|idx| (idx, self.score(query, idx)))
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        scored
    }
}

/// RRF 融合（Cormack et al. 2009）：`score(d) = Σ 1/(k + rank_i(d))`，
/// rank 由 1 起算；預設 k=60。融合後依分數降冪、平手以索引升冪。
pub fn rrf_fuse(rankings: &[Vec<usize>], k: f64) -> Vec<(usize, f64)> {
    let mut scores: BTreeMap<usize, f64> = BTreeMap::new();
    for ranking in rankings {
        for (position, doc) in ranking.iter().enumerate() {
            *scores.entry(*doc).or_insert(0.0) += 1.0 / (k + (position + 1) as f64);
        }
    }
    let mut fused: Vec<(usize, f64)> = scores.into_iter().collect();
    fused.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_mixed_cjk_and_ascii_deterministically() {
        let tokens = tokenize("採購單 Approval-PROCESS");
        // 「採購單」→ bigram 採購/購單；連字號視為邊界；ASCII 轉小寫。
        assert_eq!(
            tokens,
            vec!["採購", "購單", "approval", "process"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn tokenize_single_cjk_char_falls_back_to_unigram() {
        assert_eq!(tokenize("單"), vec!["單".to_string()]);
    }

    #[test]
    fn rare_term_ranks_its_document_first() {
        let docs = vec![
            "apple banana".to_string(),
            "apple apple apple cherry".to_string(),
            "durian".to_string(),
        ]
        .into_iter()
        .map(|d| tokenize(&d).join(" "))
        .collect::<Vec<_>>();
        let index = Bm25Index::build(&docs);
        let ranked = index.rank("cherry");
        assert_eq!(ranked[0].0, 1, "唯一含 cherry 的文件須排第一");
        assert!(ranked[0].1 > 0.0);
        assert_eq!(index.score("cherry", 0), 0.0, "不含查詢詞的文件分數為 0");
    }

    #[test]
    fn bm25_plus_delta_never_lowers_matching_scores() {
        let docs = vec!["合規 審查 approval".to_string(), "其他 內容".to_string()];
        let index = Bm25Index::build(&docs);
        let plain = index.score_with("approval", 0, 1.2, 0.75, 0.0);
        let plus = index.score_with("approval", 0, 1.2, 0.75, 1.0);
        assert!(plus > plain, "delta 對命中詞加分（BM25+）");
        assert_eq!(
            index.score_with("approval", 1, 1.2, 0.75, 0.0),
            0.0,
            "未命中的文件 delta 不憑空加分（idf 存在但 tf=0）"
        );
    }

    #[test]
    fn query_terms_count_once_even_if_repeated() {
        let docs = vec!["alpha beta".to_string()];
        let index = Bm25Index::build(&docs);
        assert_eq!(
            index.score("alpha alpha alpha", 0),
            index.score("alpha", 0),
            "重複查詢詞不重複計分"
        );
    }

    #[test]
    fn rrf_known_values_and_tie_break_by_index() {
        // 名單 A：[2, 0]；名單 B：[0]。k=60。
        let fused = rrf_fuse(&[vec![2, 0], vec![0]], 60.0);
        let score = |doc: usize| {
            fused
                .iter()
                .find(|(d, _)| *d == doc)
                .map(|(_, s)| *s)
                .unwrap()
        };
        assert!((score(0) - (1.0 / 62.0 + 1.0 / 61.0)).abs() < 1e-12);
        assert!((score(2) - 1.0 / 61.0).abs() < 1e-12);
        assert_eq!(fused[0].0, 0, "雙榜命中者勝單榜");
        // 完全平手 → 索引升冪。
        let tie = rrf_fuse(&[vec![0, 1], vec![1, 0]], 60.0);
        assert_eq!(tie[0].0, 0);
        assert_eq!(tie[1].0, 1);
    }

    #[test]
    fn ranking_is_deterministic() {
        let docs = vec!["alpha beta gamma".to_string(), "beta".to_string()];
        let first = Bm25Index::build(&docs).rank("beta");
        let second = Bm25Index::build(&docs).rank("beta");
        assert_eq!(first, second);
    }
}
