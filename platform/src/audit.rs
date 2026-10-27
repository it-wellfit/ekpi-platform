#![forbid(unsafe_code)]

//! Audit event log (Master Prompt Phase 9c).
//!
//! Append-only event log with SHA-256 hash chain.
//! Event types: document_import, rule_change, case_decision, process_conformance, evidence_review.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const GENESIS_HASH: &str = "GENESIS";
const LOCK_RETRY_LIMIT: usize = 200;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(25);

/// Audit event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    DocumentImport,
    RuleChange,
    CaseDecision,
    ProcessConformance,
    EvidenceReview,
    KnowledgeSearch,
}

impl AuditEventType {
    pub fn name(&self) -> &'static str {
        match self {
            AuditEventType::DocumentImport => "document_import",
            AuditEventType::RuleChange => "rule_change",
            AuditEventType::CaseDecision => "case_decision",
            AuditEventType::ProcessConformance => "process_conformance",
            AuditEventType::EvidenceReview => "evidence_review",
            AuditEventType::KnowledgeSearch => "knowledge_search",
        }
    }
}

/// An audit event with hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub actor: String,
    pub tenant_id: String,
    pub payload: String,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: String,
}

impl AuditEvent {
    pub fn new(
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
        prev_hash: &str,
    ) -> Self {
        let hash = Self::compute_hash(id, event_type, actor, tenant_id, payload, prev_hash);
        AuditEvent {
            id: id.to_string(),
            event_type,
            actor: actor.to_string(),
            tenant_id: tenant_id.to_string(),
            payload: payload.to_string(),
            prev_hash: prev_hash.to_string(),
            hash,
            timestamp: String::new(),
        }
    }

    fn compute_hash(
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(event_type.name().as_bytes());
        hasher.update(actor.as_bytes());
        hasher.update(tenant_id.as_bytes());
        hasher.update(payload.as_bytes());
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Audit log with hash chain integrity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(
        &mut self,
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
    ) {
        let prev_hash = self
            .events
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let event = AuditEvent::new(id, event_type, actor, tenant_id, payload, &prev_hash);
        self.events.push(event);
    }

    pub fn verify_chain(&self) -> bool {
        if self.events.is_empty() {
            return true;
        }
        if self.events[0].prev_hash != GENESIS_HASH {
            return false;
        }
        for i in 1..self.events.len() {
            if self.events[i].prev_hash != self.events[i - 1].hash {
                return false;
            }
            let expected = AuditEvent::compute_hash(
                &self.events[i].id,
                self.events[i].event_type,
                &self.events[i].actor,
                &self.events[i].tenant_id,
                &self.events[i].payload,
                &self.events[i].prev_hash,
            );
            if expected != self.events[i].hash {
                return false;
            }
        }
        // Verify first entry
        let expected = AuditEvent::compute_hash(
            &self.events[0].id,
            self.events[0].event_type,
            &self.events[0].actor,
            &self.events[0].tenant_id,
            &self.events[0].payload,
            &self.events[0].prev_hash,
        );
        expected == self.events[0].hash
    }

    pub fn detect_tampering(&self) -> Vec<usize> {
        let mut tampered = Vec::new();
        for i in 0..self.events.len() {
            let prev_hash = if i == 0 {
                GENESIS_HASH.to_string()
            } else {
                self.events[i - 1].hash.clone()
            };
            let expected = AuditEvent::compute_hash(
                &self.events[i].id,
                self.events[i].event_type,
                &self.events[i].actor,
                &self.events[i].tenant_id,
                &self.events[i].payload,
                &prev_hash,
            );
            if expected != self.events[i].hash || self.events[i].prev_hash != prev_hash {
                tampered.push(i);
            }
        }
        tampered
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events_by_type(&self, et: AuditEventType) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.event_type == et).collect()
    }
}

fn lock_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(".lock");
    PathBuf::from(name)
}

/// Read a JSONL audit log and verify its hash chain.
///
/// A missing or empty file is an empty log. A legacy single-object file is
/// reported as an error rather than rewritten or appended to, because mixing
/// formats would corrupt the file. A malformed line or a broken chain is
/// reject before any caller can append.
pub fn read_jsonl(path: &Path) -> Result<AuditLog, String> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(AuditLog::new()),
        Err(error) => {
            return Err(format!("read audit log {}: {error}", path.display()));
        }
    };
    if contents.trim().is_empty() {
        return Ok(AuditLog::new());
    }
    if contents.trim_start().starts_with('{') {
        if let Ok(legacy) = serde_json::from_str::<AuditLog>(&contents) {
            let state = if legacy.verify_chain() {
                "valid"
            } else {
                "invalid"
            };
            return Err(format!(
                "audit log {} uses the legacy single-object format ({state}); \
                 move it aside to start a JSONL chain",
                path.display()
            ));
        }
    }

    let mut log = AuditLog::new();
    for (index, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: AuditEvent = serde_json::from_str(line).map_err(|error| {
            format!(
                "audit log {} line {} is not a valid JSON event: {error}",
                path.display(),
                index + 1
            )
        })?;
        log.events.push(event);
    }
    if !log.verify_chain() {
        return Err(format!(
            "audit log {} failed hash-chain verification; refusing to append",
            path.display()
        ));
    }
    Ok(log)
}

/// Append one event to a JSONL audit log under a local writer lock.
///
/// The existing chain is read and verified first, the new event is linked to
/// the last hash (or `GENESIS`), and exactly one line is appended and synced.
/// Concurrent local writers serialise on a sibling `<PATH>.lock` directory.
/// This is tamper-evident, not WORM: a local administrator can delete or
/// rewrite the file.
pub fn append_jsonl(
    path: &Path,
    id: &str,
    event_type: AuditEventType,
    actor: &str,
    tenant_id: &str,
    payload: &str,
) -> Result<usize, String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create audit directory {}: {error}", parent.display()))?;
    }
    let _lock = AuditFileLock::acquire(path)?;
    let mut log = read_jsonl(path)?;
    log.append(id, event_type, actor, tenant_id, payload);
    let event = log
        .events
        .last()
        .ok_or_else(|| "audit append produced no event".to_string())?;
    let encoded =
        serde_json::to_string(event).map_err(|error| format!("encode audit event: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("open audit log {} for append: {error}", path.display()))?;
    file.write_all(encoded.as_bytes())
        .map_err(|error| format!("append audit event to {}: {error}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|error| format!("append audit line ending to {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync audit log {}: {error}", path.display()))?;
    Ok(log.len())
}

struct AuditFileLock {
    path: PathBuf,
}

impl AuditFileLock {
    fn acquire(target: &Path) -> Result<Self, String> {
        let path = lock_path(target);
        for _ in 0..LOCK_RETRY_LIMIT {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    std::thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) => {
                    return Err(format!(
                        "cannot create audit lock {}: {error}",
                        path.display()
                    ));
                }
            }
        }
        Err(format!(
            "audit log {} is locked by another writer; if no writer is active, \
             remove the stale lock {}",
            target.display(),
            path.display()
        ))
    }
}

impl Drop for AuditFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_verify() {
        let mut log = AuditLog::new();
        log.append(
            "A1",
            AuditEventType::DocumentImport,
            "user1",
            "T1",
            "doc imported",
        );
        log.append(
            "A2",
            AuditEventType::CaseDecision,
            "user1",
            "T1",
            "case decided",
        );
        assert!(log.verify_chain());
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn empty_log_verifies() {
        let log = AuditLog::new();
        assert!(log.verify_chain());
        assert!(log.is_empty());
    }

    #[test]
    fn tampering_detected() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "data");
        log.append("A2", AuditEventType::CaseDecision, "user1", "T1", "data2");
        log.events[1].payload = "tampered".to_string();
        assert!(!log.verify_chain());
        let tampered = log.detect_tampering();
        assert!(tampered.contains(&1));
    }

    #[test]
    fn events_by_type_filter() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "doc");
        log.append("A2", AuditEventType::CaseDecision, "user1", "T1", "case");
        log.append("A3", AuditEventType::DocumentImport, "user2", "T1", "doc2");
        assert_eq!(log.events_by_type(AuditEventType::DocumentImport).len(), 2);
        assert_eq!(log.events_by_type(AuditEventType::CaseDecision).len(), 1);
    }

    #[test]
    fn audit_event_type_names_unique() {
        let types = [
            AuditEventType::DocumentImport,
            AuditEventType::RuleChange,
            AuditEventType::CaseDecision,
            AuditEventType::ProcessConformance,
            AuditEventType::EvidenceReview,
            AuditEventType::KnowledgeSearch,
        ];
        let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn knowledge_search_event_type_name() {
        assert_eq!(AuditEventType::KnowledgeSearch.name(), "knowledge_search");
    }

    #[test]
    fn genesis_prev_hash() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "data");
        assert_eq!(log.events[0].prev_hash, "GENESIS");
    }

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    struct TempAuditFile(PathBuf);

    impl TempAuditFile {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "ekpi-audit-test-{}-{}.jsonl",
                std::process::id(),
                NEXT_FILE.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_file(&path);
            let _ = fs::remove_dir(lock_path(&path));
            Self(path)
        }
    }

    impl Drop for TempAuditFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
            let _ = fs::remove_dir(lock_path(&self.0));
        }
    }

    #[test]
    fn jsonl_append_and_read_preserve_the_chain() {
        let file = TempAuditFile::new();
        assert!(read_jsonl(&file.0)
            .expect("missing log is empty")
            .is_empty());

        append_jsonl(
            &file.0,
            "A1",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            "first",
        )
        .expect("first append");
        append_jsonl(
            &file.0,
            "A2",
            AuditEventType::KnowledgeSearch,
            "search-cli",
            "default",
            "second",
        )
        .expect("second append");

        let log = read_jsonl(&file.0).expect("valid chain reads");
        assert_eq!(log.len(), 2);
        assert!(log.verify_chain());
        assert_eq!(log.events[0].prev_hash, GENESIS_HASH);
        assert_eq!(log.events[1].prev_hash, log.events[0].hash);
        assert_eq!(
            fs::read_to_string(&file.0)
                .expect("read audit log")
                .lines()
                .count(),
            2
        );
    }

    #[test]
    fn jsonl_rejects_a_tampered_event_without_appending() {
        let file = TempAuditFile::new();
        append_jsonl(
            &file.0,
            "A1",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            "first",
        )
        .expect("append");
        let contents = fs::read_to_string(&file.0).expect("read log");
        let tampered = contents.replace("\"payload\":\"first\"", "\"payload\":\"changed\"");
        assert_ne!(contents, tampered, "test must actually modify the payload");
        fs::write(&file.0, &tampered).expect("write tampered log");

        let error = read_jsonl(&file.0).expect_err("tampered chain is rejected");
        assert!(error.contains("failed hash-chain verification"), "{error}");

        let before = fs::read_to_string(&file.0).expect("read tampered log");
        let append_error = append_jsonl(
            &file.0,
            "A2",
            AuditEventType::KnowledgeSearch,
            "search-cli",
            "default",
            "second",
        )
        .expect_err("append must fail closed");
        assert!(
            append_error.contains("failed hash-chain verification"),
            "{append_error}"
        );
        assert_eq!(
            fs::read_to_string(&file.0).expect("read log after refusal"),
            before,
            "a refused append must not modify the log"
        );
    }

    #[test]
    fn legacy_single_object_log_is_refused() {
        let file = TempAuditFile::new();
        let mut legacy = AuditLog::new();
        legacy.append("A1", AuditEventType::DocumentImport, "alice", "T1", "data");
        fs::write(
            &file.0,
            serde_json::to_string_pretty(&legacy).expect("encode legacy log"),
        )
        .expect("write legacy log");

        let error = read_jsonl(&file.0).expect_err("legacy format is not appended");
        assert!(error.contains("legacy single-object format"), "{error}");
    }

    #[test]
    fn concurrent_appends_share_one_verified_chain() {
        let file = Arc::new(TempAuditFile::new());
        let mut handles = Vec::new();
        for index in 0..8 {
            let file = Arc::clone(&file);
            handles.push(std::thread::spawn(move || {
                append_jsonl(
                    &file.0,
                    &format!("A{index}"),
                    AuditEventType::KnowledgeSearch,
                    "search-cli",
                    "default",
                    &format!("event {index}"),
                )
                .expect("concurrent append");
            }));
        }
        for handle in handles {
            handle.join().expect("writer thread joins");
        }

        let log = read_jsonl(&file.0).expect("concurrent chain reads");
        assert_eq!(log.len(), 8);
        assert!(log.verify_chain());
        let mut hashes: Vec<String> = log.events.iter().map(|event| event.hash.clone()).collect();
        hashes.sort();
        hashes.dedup();
        assert_eq!(
            hashes.len(),
            8,
            "each appended event must have a unique hash"
        );
    }
}
