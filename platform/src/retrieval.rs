#![forbid(unsafe_code)]

//! R2-4：MVP 檢索——字符 bigram cosine 相似度。
//!
//! 用途：跨文件證據定位（例：例外規則綁定的附錄條款必須能在案件可用文件集中
//! 被定位，否則判 `insufficient_evidence`；EVD-001 語義）。語意向量與 reranker
//! 待 R2 後期/ R3 接入；本模組只依賴字面相似度，不調用任何模型。

use crate::docs::DocDatabase;
use crate::tenant::TenantContext;
use std::sync::OnceLock;

/// Global cache for parsed JSON documents to improve performance.
static JSON_CACHE: OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>,
> = OnceLock::new();

/// Get or initialize the global JSON cache.
fn get_json_cache()
-> &'static std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>> {
    JSON_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Parse JSON with caching to avoid repeated parsing overhead.
pub fn parse_json_cached(key: &str, json_str: &str) -> Result<serde_json::Value, String> {
    let cache = get_json_cache();
    let mut cache_guard = cache.lock().map_err(|e| format!("cache lock error: {e}"))?;

    if let Some(cached) = cache_guard.get(key) {
        return Ok(cached.clone());
    }

    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error for {key}: {e}"))?;
    cache_guard.insert(key.to_string(), parsed.clone());
    Ok(parsed)
}

/// Clear the JSON cache (useful for testing or memory management).
pub fn clear_json_cache() {
    if let Some(cache) = JSON_CACHE.get()
        && let Ok(mut cache_guard) = cache.lock()
    {
        cache_guard.clear();
    }
}

/// Permission context for retrieval filtering.
/// Determines which documents a user may access.
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub tenant_id: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

impl PermissionContext {
    pub fn new(tenant_id: &str, user_id: &str, roles: Vec<String>) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            roles,
        }
    }

    /// Check if the user has any of the required roles. Empty required = public.
    pub fn can_access(&self, doc_permissions: &[String]) -> bool {
        if doc_permissions.is_empty() {
            return true;
        }
        doc_permissions
            .iter()
            .any(|p| self.roles.iter().any(|r| r == p))
    }
}

impl From<&TenantContext> for PermissionContext {
    fn from(ctx: &TenantContext) -> Self {
        Self {
            tenant_id: ctx.tenant_id.clone(),
            user_id: ctx.user_id.clone(),
            roles: ctx.roles.clone(),
        }
    }
}

/// Inverted index for bigram-based lexical search.
#[derive(Debug, Clone)]
pub struct InvertedIndex {
    /// Maps bigram to list of document positions (PoolSection)
    pub bigram_index: std::collections::HashMap<String, Vec<PoolSection>>,
    /// Set of all unique bigrams in the corpus
    pub bigrams: std::collections::HashSet<String>,
}

impl InvertedIndex {
    /// Build an inverted index from a pool of text sections.
    pub fn build(pool: &[PoolSection]) -> Self {
        let mut bigram_index: std::collections::HashMap<String, Vec<PoolSection>> =
            std::collections::HashMap::new();
        let mut bigrams = std::collections::HashSet::new();

        for section in pool {
            let bigrams_in_text = extract_bigrams(&section.verbatim_text);
            for bigram in bigrams_in_text {
                bigrams.insert(bigram.clone());
                bigram_index
                    .entry(bigram)
                    .or_default()
                    .push(section.clone());
            }
        }

        InvertedIndex {
            bigram_index,
            bigrams,
        }
    }

    /// Search for sections that match a query text using bigram cosine similarity.
    pub fn search(&self, query: &str, threshold: f64) -> Vec<RetrievalHit> {
        let query_bigrams = extract_bigrams(query);
        if query_bigrams.is_empty() {
            return Vec::new();
        }

        let mut query_bigram_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for bigram in &query_bigrams {
            *query_bigram_counts.entry(bigram.clone()).or_insert(0) += 1;
        }

        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for bigram in &query_bigrams {
            if let Some(sections) = self.bigram_index.get(bigram) {
                for section in sections {
                    let key = (
                        section.doc_id.clone(),
                        section.version.clone(),
                        section.section.clone(),
                    );
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.insert(key);

                    // Use char_bigram_cosine for consistent scoring with original implementation
                    let hit_score = char_bigram_cosine(query, &section.verbatim_text);

                    if hit_score >= threshold {
                        results.push(RetrievalHit {
                            doc_id: section.doc_id.clone(),
                            version: section.version.clone(),
                            section: section.section.clone(),
                            score: hit_score,
                            char_range: (0, section.verbatim_text.len()),
                        });
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

/// Extract all bigrams (2-character sequences) from a text string.
/// Uses char indices for correct UTF-8 handling.
fn extract_bigrams(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut bigrams = std::collections::HashSet::new();

    for i in 0..chars.len().saturating_sub(1) {
        let bigram: String = chars[i..i + 2].iter().collect();
        if bigram.chars().count() == 2 {
            bigrams.insert(bigram);
        }
    }

    bigrams.into_iter().collect()
}

/// A citation linking a retrieval result to its exact source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    pub doc_id: String,
    pub version: String,
    pub section: String,
    pub score: f64,
    /// The verbatim text from the source document.
    pub verbatim_text: String,
    /// Character offset range in the verbatim text (start, end).
    pub char_range: (usize, usize),
}

impl Citation {
    /// Create a citation from a retrieval hit and its source text.
    pub fn from_hit(hit: &RetrievalHit, verbatim_text: &str) -> Self {
        Self {
            doc_id: hit.doc_id.clone(),
            version: hit.version.clone(),
            section: hit.section.clone(),
            score: hit.score,
            verbatim_text: verbatim_text.to_string(),
            char_range: hit.char_range,
        }
    }

    /// Verify that the citation's verbatim text matches the source document.
    pub fn verify(&self, db: &DocDatabase) -> bool {
        if let Some(ver) = db.binding(&self.doc_id, &self.version)
            && let Some(sec) = ver.sections.iter().find(|s| s.section == self.section)
        {
            return sec.verbatim_text == self.verbatim_text;
        }
        false
    }

    /// Format as a human-readable citation string.
    pub fn format(&self) -> String {
        format!(
            "[{}] {}@{} §{} (score: {:.2})",
            self.doc_id, self.doc_id, self.version, self.section, self.score
        )
    }
}

/// Result of a knowledge search: either hits with citations, or a refusal.
#[derive(Debug, Clone)]
pub enum SearchResult {
    /// Successful retrieval with citations.
    Hits(Vec<Citation>),
    /// No evidence found; system refuses to answer.
    InsufficientEvidence { reason: String },
}

impl SearchResult {
    /// Returns true if the result contains at least one citation.
    pub fn has_evidence(&self) -> bool {
        match self {
            SearchResult::Hits(hits) => !hits.is_empty(),
            SearchResult::InsufficientEvidence { .. } => false,
        }
    }

    /// Number of citations in the result.
    pub fn citation_count(&self) -> usize {
        match self {
            SearchResult::Hits(hits) => hits.len(),
            SearchResult::InsufficientEvidence { .. } => 0,
        }
    }
}

/// A retrieval hit with citation metadata.
#[derive(Debug, Clone)]
pub struct RetrievalHit {
    pub doc_id: String,
    pub version: String,
    pub section: String,
    pub score: f64,
    /// Character offset range in the verbatim text (start, end).
    pub char_range: (usize, usize),
}

/// 檢索池中一個候選片段。
#[derive(Debug, Clone)]
pub struct PoolSection {
    pub doc_id: String,
    pub version: String,
    pub section: String,
    pub verbatim_text: String,
}

/// 對案件可用文件集建立的檢索器（pool 由引擎依 case doc_versions 組裝）。
pub struct Retriever {
    pool: Vec<PoolSection>,
    threshold: f64,
}

impl Retriever {
    /// Build a retriever from the full document set (no permission filtering).
    /// Backward-compatible: equivalent to `build_with_permissions(db, doc_versions, threshold, None)`.
    pub fn build(
        db: &DocDatabase,
        doc_versions: &std::collections::BTreeMap<String, String>,
        threshold: f64,
    ) -> Retriever {
        Self::build_with_permissions(db, doc_versions, threshold, None)
    }

    /// Build a retriever, optionally filtering documents by user permissions.
    /// When `perm` is `Some`, only documents whose `permissions` list is empty
    /// (public) or contains at least one of the user's roles are included.
    /// When `perm` is `None`, all documents are included (backward-compatible).
    pub fn build_with_permissions(
        db: &DocDatabase,
        doc_versions: &std::collections::BTreeMap<String, String>,
        threshold: f64,
        perm: Option<&PermissionContext>,
    ) -> Retriever {
        let mut pool = Vec::new();
        for (doc_id, version) in doc_versions {
            // Permission check: skip documents the user cannot access.
            if let Some(ctx) = perm
                && let Some(doc) = db.doc(doc_id)
                && !ctx.can_access(&doc.permissions)
            {
                continue;
            }
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

    /// Build an inverted index for efficient search operations.
    pub fn build_inverted_index(&self) -> InvertedIndex {
        InvertedIndex::build(&self.pool)
    }

    /// Search using the inverted index for better performance.
    pub fn search_with_index(&self, query: &str, threshold: f64) -> Vec<RetrievalHit> {
        let inverted_index = self.build_inverted_index();
        inverted_index.search(query, threshold)
    }

    /// 依字面相似度定位 `anchor` 最相似片段；低於閾值回 None。
    /// 引擎以替換判定：命中 doc/version 與規則綁定不一致視為
    /// 證據歧義 → 保守拒答。
    pub fn locate(&self, anchor: &str) -> Option<RetrievalHit> {
        let mut best: Option<RetrievalHit> = None;
        for cand in &self.pool {
            let score = char_bigram_cosine(anchor, &cand.verbatim_text);
            if score >= self.threshold {
                let char_range = (0, cand.verbatim_text.chars().count());
                let hit = RetrievalHit {
                    doc_id: cand.doc_id.clone(),
                    version: cand.version.clone(),
                    section: cand.section.clone(),
                    score,
                    char_range,
                };
                best = match best {
                    Some(b) if b.score >= score => Some(b),
                    _ => Some(hit),
                };
            }
        }
        best
    }

    /// Return the number of sections in the retrieval pool.
    pub fn pool_size(&self) -> usize {
        self.pool.len()
    }

    /// Return the doc_ids present in the pool (for permission verification).
    pub fn pool_doc_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.pool.iter().map(|p| p.doc_id.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Search and return a SearchResult with verified citations.
    /// Returns InsufficientEvidence if no section meets the threshold.
    pub fn search_with_citations(
        &self,
        query: &str,
        top_k: usize,
        db: &DocDatabase,
    ) -> SearchResult {
        let hits = self.search(query, top_k);
        if hits.is_empty() {
            return SearchResult::InsufficientEvidence {
                reason: format!(
                    "no section scored above threshold {:.2} for query",
                    self.threshold
                ),
            };
        }
        let citations: Vec<Citation> = hits
            .iter()
            .filter_map(|hit| {
                if let Some(ver) = db.binding(&hit.doc_id, &hit.version)
                    && let Some(sec) = ver.sections.iter().find(|s| s.section == hit.section)
                {
                    let c = Citation::from_hit(hit, &sec.verbatim_text);
                    if c.verify(db) {
                        return Some(c);
                    }
                }
                None
            })
            .collect();
        if citations.is_empty() {
            SearchResult::InsufficientEvidence {
                reason: "retrieved sections failed citation verification".to_string(),
            }
        } else {
            SearchResult::Hits(citations)
        }
    }

    /// Search the pool for sections matching the query, returning ranked hits.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<RetrievalHit> {
        // Use inverted index for better performance
        if !self.pool.is_empty() {
            let mut hits = self.search_with_index(query, self.threshold);
            hits.truncate(top_k);
            return hits;
        }
        // Fallback to linear search for empty pool
        let mut hits: Vec<RetrievalHit> = self
            .pool
            .iter()
            .filter_map(|cand| {
                let score = char_bigram_cosine(query, &cand.verbatim_text);
                if score >= self.threshold {
                    Some(RetrievalHit {
                        doc_id: cand.doc_id.clone(),
                        version: cand.version.clone(),
                        section: cand.section.clone(),
                        score,
                        char_range: (0, cand.verbatim_text.chars().count()),
                    })
                } else {
                    None
                }
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
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

    #[test]
    fn permission_context_public_doc_accessible() {
        let ctx = PermissionContext::new("T1", "U1", vec!["viewer".to_string()]);
        assert!(ctx.can_access(&[]));
    }

    #[test]
    fn permission_context_restricted_doc_accessible_with_role() {
        let ctx = PermissionContext::new("T1", "U1", vec!["admin".to_string()]);
        assert!(ctx.can_access(&["admin".to_string()]));
        assert!(ctx.can_access(&["admin".to_string(), "editor".to_string()]));
    }

    #[test]
    fn permission_context_restricted_doc_denied_without_role() {
        let ctx = PermissionContext::new("T1", "U1", vec!["viewer".to_string()]);
        assert!(!ctx.can_access(&["admin".to_string()]));
        assert!(!ctx.can_access(&["editor".to_string()]));
    }

    #[test]
    fn permission_context_tenant_context_conversion() {
        use crate::tenant::TenantContext;
        let tc = TenantContext::new("T1", "U1").with_roles(vec!["editor".to_string()]);
        let pc = PermissionContext::from(&tc);
        assert_eq!(pc.tenant_id, "T1");
        assert_eq!(pc.user_id, "U1");
        assert!(pc.can_access(&["editor".to_string()]));
        assert!(!pc.can_access(&["admin".to_string()]));
    }

    #[test]
    fn build_with_permissions_filters_restricted_docs() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "PUBLIC-DOC",
                    "title": "Public Document",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "public content here"}]
                    }]
                },
                {
                    "doc_id": "SECRET-DOC",
                    "title": "Secret Document",
                    "permissions": ["admin"],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "secret content here"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("PUBLIC-DOC".to_string(), "1.0".to_string());
        doc_versions.insert("SECRET-DOC".to_string(), "1.0".to_string());

        let viewer_ctx = PermissionContext::new("T1", "U1", vec!["viewer".to_string()]);
        let r_viewer =
            Retriever::build_with_permissions(&db, &doc_versions, 0.1, Some(&viewer_ctx));
        assert_eq!(r_viewer.pool_doc_ids(), vec!["PUBLIC-DOC"]);
        assert_eq!(r_viewer.pool_size(), 1);

        let admin_ctx = PermissionContext::new("T1", "U2", vec!["admin".to_string()]);
        let r_admin = Retriever::build_with_permissions(&db, &doc_versions, 0.1, Some(&admin_ctx));
        assert_eq!(r_admin.pool_doc_ids(), vec!["PUBLIC-DOC", "SECRET-DOC"]);
        assert_eq!(r_admin.pool_size(), 2);

        let r_none = Retriever::build_with_permissions(&db, &doc_versions, 0.1, None);
        assert_eq!(r_none.pool_doc_ids(), vec!["PUBLIC-DOC", "SECRET-DOC"]);
    }

    #[test]
    fn permission_isolation_no_leakage() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "ADMIN-ONLY",
                    "title": "Admin Only",
                    "permissions": ["admin"],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "confidential admin data"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("ADMIN-ONLY".to_string(), "1.0".to_string());

        let viewer_ctx = PermissionContext::new("T1", "U1", vec!["viewer".to_string()]);
        let r = Retriever::build_with_permissions(&db, &doc_versions, 0.0, Some(&viewer_ctx));
        assert!(r.is_empty(), "viewer must not see admin-only doc");
        assert!(
            r.locate("confidential").is_none(),
            "no retrieval hit for unauthorized user"
        );
    }

    #[test]
    fn search_returns_ranked_results() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "DOC-A",
                    "title": "Doc A",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [
                            {"section": "s1", "verbatim_text": "quality management system requires document control"},
                            {"section": "s2", "verbatim_text": "procurement process for purchasing goods"}
                        ]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("DOC-A".to_string(), "1.0".to_string());

        let r = Retriever::build(&db, &doc_versions, 0.1);
        let hits = r.search("quality management", 5);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].doc_id, "DOC-A");
        assert_eq!(hits[0].section, "s1");
        for w in hits.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
    }

    #[test]
    fn search_respects_permissions() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "PUBLIC",
                    "title": "Public",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "public information about quality"}]
                    }]
                },
                {
                    "doc_id": "SECRET",
                    "title": "Secret",
                    "permissions": ["admin"],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "secret admin information"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("PUBLIC".to_string(), "1.0".to_string());
        doc_versions.insert("SECRET".to_string(), "1.0".to_string());

        let viewer_ctx = PermissionContext::new("T1", "U1", vec!["viewer".to_string()]);
        let r = Retriever::build_with_permissions(&db, &doc_versions, 0.1, Some(&viewer_ctx));
        let hits = r.search("information", 10);
        assert!(hits.iter().all(|h| h.doc_id == "PUBLIC"));
        assert!(!hits.iter().any(|h| h.doc_id == "SECRET"));
    }

    #[test]
    fn citation_verify_matches_source() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "DOC-A",
                    "title": "Doc A",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [
                            {"section": "s1", "verbatim_text": "quality management system requires document control"}
                        ]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("DOC-A".to_string(), "1.0".to_string());

        let r = Retriever::build(&db, &doc_versions, 0.1);
        let result = r.search_with_citations("quality management", 5, &db);
        assert!(result.has_evidence());
        assert!(result.citation_count() > 0);
        if let SearchResult::Hits(citations) = result {
            assert!(citations[0].verify(&db));
            assert_eq!(citations[0].doc_id, "DOC-A");
            assert_eq!(citations[0].version, "1.0");
            assert_eq!(citations[0].section, "s1");
        }
    }

    #[test]
    fn citation_verify_fails_on_tampered_text() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "DOC-A",
                    "title": "Doc A",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [
                            {"section": "s1", "verbatim_text": "original text here"}
                        ]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let hit = RetrievalHit {
            doc_id: "DOC-A".to_string(),
            version: "1.0".to_string(),
            section: "s1".to_string(),
            score: 0.9,
            char_range: (0, 10),
        };
        let citation = Citation::from_hit(&hit, "tampered text");
        assert!(!citation.verify(&db));
    }

    #[test]
    fn search_with_citations_returns_insufficient_evidence() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "DOC-A",
                    "title": "Doc A",
                    "permissions": [],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "completely unrelated content about cooking"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut doc_versions = std::collections::BTreeMap::new();
        doc_versions.insert("DOC-A".to_string(), "1.0".to_string());

        let r = Retriever::build(&db, &doc_versions, 0.99);
        let result = r.search_with_citations("quantum physics", 5, &db);
        assert!(!result.has_evidence());
        assert_eq!(result.citation_count(), 0);
        if let SearchResult::InsufficientEvidence { reason } = result {
            assert!(!reason.is_empty());
        } else {
            panic!("expected InsufficientEvidence");
        }
    }
}
