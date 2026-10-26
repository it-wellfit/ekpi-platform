#![forbid(unsafe_code)]

//! R2-1：版本化文件庫 + 條款錨點 + 內容雜湊。
//!
//! 語義：
//! - 案件以 `doc_versions` 宣告其可用的 (doc_id → version) 綁定範圍；
//! - 文件版本具 `effective_from`/`effective_to` → 版本時間引擎判斷「案件時點在何時有效」；
//! - `sections` 是引擎唯一可引用的條款錨點（section/page/verbatim_text）；
//! - `injection_directive` 標記文件內嵌指令的存在（H-5：僅偵測、絕不當系統規則）。

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize)]
pub struct DocDatabase {
    pub documents: Vec<Doc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Doc {
    pub doc_id: String,
    #[serde(default)]
    pub title: String,
    pub versions: Vec<DocVersion>,
    /// Roles that may access this document. Empty means public (all roles).
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocVersion {
    pub version: String,
    pub effective_from: String,
    #[serde(default)]
    pub effective_to: Option<String>,
    #[serde(default)]
    pub injection_directive: Option<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct Section {
    pub section: String,
    #[serde(default)]
    pub page: Option<u32>,
    pub verbatim_text: String,
}

impl DocDatabase {
    pub fn load(path: &Path) -> Result<DocDatabase, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let db: DocDatabase =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        db.validate_shape()?;
        Ok(db)
    }

    /// Shape check: duplicate doc ids, duplicate (doc_id, version) bindings or
    /// empty section ids would make lookups and citations ambiguous; fail
    /// loudly at load time.
    pub fn validate_shape(&self) -> Result<(), String> {
        let mut doc_ids = std::collections::BTreeSet::new();
        let mut seen: std::collections::BTreeMap<(String, String), ()> =
            std::collections::BTreeMap::new();
        for doc in &self.documents {
            if doc.doc_id.is_empty() {
                return Err("docs.json: doc_id empty".to_string());
            }
            if !doc_ids.insert(doc.doc_id.clone()) {
                return Err(format!("docs.json: duplicate doc_id {}", doc.doc_id));
            }
            for ver in &doc.versions {
                let key = (doc.doc_id.clone(), ver.version.clone());
                if seen.insert(key, ()).is_some() {
                    return Err(format!(
                        "docs.json: duplicate binding {}@{}",
                        doc.doc_id, ver.version
                    ));
                }
                if ver.effective_from.is_empty() {
                    return Err(format!(
                        "docs.json: {}@{} effective_from empty",
                        doc.doc_id, ver.version
                    ));
                }
                let mut sec_seen = std::collections::BTreeSet::new();
                for s in &ver.sections {
                    if s.section.is_empty() || s.verbatim_text.is_empty() {
                        return Err(format!(
                            "docs.json: {}@{} section/verbatim empty",
                            doc.doc_id, ver.version
                        ));
                    }
                    if !sec_seen.insert(s.section.clone()) {
                        return Err(format!(
                            "docs.json: {}@{} duplicate section {}",
                            doc.doc_id, ver.version, s.section
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn doc(&self, doc_id: &str) -> Option<&Doc> {
        self.documents.iter().find(|d| d.doc_id == doc_id)
    }

    pub fn binding(&self, doc_id: &str, version: &str) -> Option<&DocVersion> {
        self.doc(doc_id)?
            .versions
            .iter()
            .find(|v| v.version == version)
    }

    /// Latest declared version for every document. This is the binding used
    /// by the CLI search and eval surfaces; the engine uses case-specific
    /// `doc_versions` instead.
    pub fn latest_versions(&self) -> BTreeMap<String, String> {
        self.documents
            .iter()
            .filter_map(|doc| {
                doc.versions
                    .last()
                    .map(|version| (doc.doc_id.clone(), version.version.clone()))
            })
            .collect()
    }

    /// Case-time validity (ISO date strings compare lexicographically; the
    /// data layer guarantees the YYYY-MM-DD shape).
    pub fn in_force(&self, doc_id: &str, version: &str, ts: &str) -> Result<bool, String> {
        let ver = self.binding(doc_id, version).ok_or_else(|| {
            format!("unknown binding {doc_id}@{version} (not present in docs.json)")
        })?;
        let in_force = ver.effective_from.as_str() <= ts
            && ver.effective_to.as_deref().is_none_or(|to| ts <= to);
        Ok(in_force)
    }

    /// SHA-256 over the canonical JSON rendering of one doc version's
    /// sections (the citation surface). Deterministic within a build.
    /// Save the document database to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize docs: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(())
    }

    pub fn content_sha256(&self, doc_id: &str, version: &str) -> Result<String, String> {
        let ver = self
            .binding(doc_id, version)
            .ok_or_else(|| format!("unknown binding {doc_id}@{version}"))?;
        let canonical = serde_json::to_string(&ver.sections)
            .map_err(|e| format!("canonicalize {doc_id}@{version}: {e}"))?;
        Ok(sha256_hex(canonical.as_bytes()))
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<DocDatabase, String> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("docs.json");
        DocDatabase::load(&path)
    }

    #[test]
    fn loads_and_resolves_bindings() {
        let db = fixture().unwrap();
        assert!(db.binding("SOP-CHANGE", "3.2").is_some());
        assert!(db.binding("SOP-CHANGE", "9.9").is_none());
    }

    #[test]
    fn effective_window_checks() {
        let db = fixture().unwrap();
        // SOP-CHANGE 2.1 was valid through 2026-02-28.
        assert_eq!(db.in_force("SOP-CHANGE", "2.1", "2026-02-01"), Ok(true));
        assert_eq!(db.in_force("SOP-CHANGE", "2.1", "2026-03-01"), Ok(false));
        // ORD-001 declares QM-01@2.1 but its case time is 2026-03-01 → stale.
        assert_eq!(db.in_force("QM-01", "2.1", "2026-03-01"), Ok(false));
        assert!(db.in_force("NOPE", "1.0", "2026-01-01").is_err());
    }

    #[test]
    fn content_hash_is_stable() {
        let db = fixture().unwrap();
        let a = db.content_sha256("SOP-CHANGE", "3.2").unwrap();
        let b = db.content_sha256("SOP-CHANGE", "3.2").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn latest_versions_use_the_last_declared_version() {
        let db = fixture().unwrap();
        let versions = db.latest_versions();
        assert_eq!(versions.get("SOP-CHANGE"), Some(&"3.2".to_string()));
        assert_eq!(versions.get("SOP-PURCH"), Some(&"4.5".to_string()));
        assert_eq!(versions.len(), db.documents.len());
    }

    #[test]
    fn duplicate_doc_id_is_rejected() {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "DUP",
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "one"}]
                    }]
                },
                {
                    "doc_id": "DUP",
                    "versions": [{
                        "version": "2.0",
                        "effective_from": "2026-02-01",
                        "sections": [{"section": "s1", "verbatim_text": "two"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let error = db.validate_shape().unwrap_err();
        assert!(error.contains("duplicate doc_id"));
    }
}
