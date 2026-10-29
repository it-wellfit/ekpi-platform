#![forbid(unsafe_code)]

//! Audit event log (Master Prompt Phase 9c).
//!
//! Append-only event log with SHA-256 hash chain.
//! Event types: document_import, rule_change, case_decision, process_conformance, evidence_review.

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::local_io::open_for_append;
use crate::local_lock::LocalFileLock;

const GENESIS_HASH: &str = "GENESIS";
const HASH_DOMAIN: &[u8] = b"ekpi-audit-event-v2";
const MAX_ID_CHARS: usize = 128;
const MAX_ACTOR_CHARS: usize = 80;
const MAX_TENANT_CHARS: usize = 64;
const MAX_PAYLOAD_CHARS: usize = 16_384;
const SECONDS_PER_DAY: u64 = 86_400;

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
    #[serde(default)]
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
        let timestamp = utc_timestamp(now_secs());
        let hash = Self::compute_hash(
            id, event_type, actor, tenant_id, payload, prev_hash, &timestamp,
        );
        AuditEvent {
            id: id.to_string(),
            event_type,
            actor: actor.to_string(),
            tenant_id: tenant_id.to_string(),
            payload: payload.to_string(),
            prev_hash: prev_hash.to_string(),
            hash,
            timestamp,
        }
    }

    fn compute_hash(
        id: &str,
        event_type: AuditEventType,
        actor: &str,
        tenant_id: &str,
        payload: &str,
        prev_hash: &str,
        timestamp: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(HASH_DOMAIN);
        for field in [
            id,
            event_type.name(),
            actor,
            tenant_id,
            payload,
            prev_hash,
            timestamp,
        ] {
            // Length-prefix every field so concatenation cannot make two
            // different events hash to the same byte stream.
            hasher.update((field.len() as u64).to_be_bytes());
            hasher.update(field.as_bytes());
        }
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
                &self.events[i].timestamp,
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
            &self.events[0].timestamp,
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
                &self.events[i].timestamp,
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

/// Read a JSONL audit log and verify its hash chain.
///
/// A missing or empty file is an empty log. A legacy single-object file is
/// reported as an error rather than rewritten or appended to, because mixing
/// formats would corrupt the file. A malformed line, a truncated final line, a
/// missing timestamp or a broken chain is rejected before any caller appends.
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
    if !contents.ends_with('\n') {
        return Err(format!(
            "audit log {} has a truncated final line (no trailing newline); \
             refusing to append",
            path.display()
        ));
    }

    let mut log = AuditLog::new();
    for (index, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            return Err(format!(
                "audit log {} line {} is empty; refusing to append",
                path.display(),
                index + 1
            ));
        }
        let event: AuditEvent = serde_json::from_str(line).map_err(|error| {
            format!(
                "audit log {} line {} is not a valid JSON event: {error}",
                path.display(),
                index + 1
            )
        })?;
        if log.events.iter().any(|existing| existing.id == event.id) {
            return Err(format!(
                "audit log {} line {} repeats event id {}; refusing to append",
                path.display(),
                index + 1,
                event.id
            ));
        }
        validate_event_field("id", &event.id, MAX_ID_CHARS)
            .and_then(|_| validate_event_field("actor", &event.actor, MAX_ACTOR_CHARS))
            .and_then(|_| validate_event_field("tenant_id", &event.tenant_id, MAX_TENANT_CHARS))
            .and_then(|_| validate_payload(&event.payload))
            .map_err(|error| {
                format!(
                    "audit log {} line {}: {error}; refusing to append",
                    path.display(),
                    index + 1
                )
            })?;
        if event.timestamp.is_empty() || !valid_utc_timestamp(&event.timestamp) {
            return Err(format!(
                "audit log {} line {} has a missing or invalid UTC timestamp; \
                 pre-v2 events must be moved aside",
                path.display(),
                index + 1
            ));
        }
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
/// the last hash (or `GENESIS`), and exactly one complete line is written in a
/// single call and synced. A failed write is rolled back to the previous file
/// length. Concurrent local writers serialise on an operating-system advisory
/// lock held on a sibling `<PATH>.lock` file; the kernel releases the lock when
/// the writer exits, so no stale-lock timeout or manual lock deletion is
/// needed. This is tamper-evident, not WORM: a local administrator can delete or
/// rewrite the file.
pub fn append_jsonl(
    path: &Path,
    id: &str,
    event_type: AuditEventType,
    actor: &str,
    tenant_id: &str,
    payload: &str,
) -> Result<usize, String> {
    validate_event_field("id", id, MAX_ID_CHARS)?;
    validate_event_field("actor", actor, MAX_ACTOR_CHARS)?;
    validate_event_field("tenant_id", tenant_id, MAX_TENANT_CHARS)?;
    validate_payload(payload)?;

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create audit directory {}: {error}", parent.display()))?;
    }
    let _lock = LocalFileLock::acquire(path)?;
    let mut log = read_jsonl(path)?;
    if log.events.iter().any(|event| event.id == id) {
        return Err(format!(
            "audit log {} already contains event id {}; refusing to append",
            path.display(),
            id
        ));
    }
    log.append(id, event_type, actor, tenant_id, payload);
    let event = log
        .events
        .last()
        .ok_or_else(|| "audit append produced no event".to_string())?;
    let mut line =
        serde_json::to_vec(event).map_err(|error| format!("encode audit event: {error}"))?;
    line.push(b'\n');
    let mut file = open_for_append(path)?;
    let original_len = file
        .metadata()
        .map_err(|error| format!("inspect audit log {}: {error}", path.display()))?
        .len();
    if let Err(error) = file.write_all(&line).and_then(|_| file.sync_all()) {
        let rollback = file.set_len(original_len).and_then(|_| file.sync_all());
        return Err(match rollback {
            Ok(()) => format!(
                "append audit event to {}: {error}; partial write rolled back",
                path.display()
            ),
            Err(rollback_error) => format!(
                "append audit event to {}: {error}; rollback failed: {rollback_error}",
                path.display()
            ),
        });
    }
    Ok(log.len())
}

fn validate_event_field(label: &str, value: &str, max_chars: usize) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("audit event {label} must not be empty"));
    }
    if value.chars().count() > max_chars {
        return Err(format!(
            "audit event {label} exceeds {max_chars} characters"
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("audit event {label} contains a control character"));
    }
    Ok(())
}

fn validate_payload(payload: &str) -> Result<(), String> {
    if payload.trim().is_empty() {
        return Err("audit event payload must not be empty".to_string());
    }
    if payload.chars().count() > MAX_PAYLOAD_CHARS {
        return Err(format!(
            "audit event payload exceeds {MAX_PAYLOAD_CHARS} characters"
        ));
    }
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn utc_timestamp(seconds: u64) -> String {
    let days = seconds / SECONDS_PER_DAY;
    let remainder = seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        remainder / 3_600,
        (remainder % 3_600) / 60,
        remainder % 60
    )
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let year: i64 = value[0..4].parse().unwrap_or(-1);
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..10].parse().unwrap_or(0);
    let hour: u32 = value[11..13].parse().unwrap_or(99);
    let minute: u32 = value[14..16].parse().unwrap_or(99);
    let second: u32 = value[17..19].parse().unwrap_or(99);
    year >= 1970
        && (1..=12).contains(&month)
        && day >= 1
        && day <= days_in_month(year, month)
        && hour <= 23
        && minute <= 59
        && second <= 60
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let shifted = days_since_epoch + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
    fn timestamp_is_part_of_the_chain_hash() {
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "user1", "T1", "data");
        assert!(
            !log.events[0].timestamp.is_empty(),
            "new audit events must carry a timestamp"
        );
        log.events[0].timestamp = "2000-01-01T00:00:00Z".to_string();
        assert!(
            !log.verify_chain(),
            "changing the timestamp must invalidate the hash"
        );
        assert!(log.detect_tampering().contains(&0));
    }

    #[test]
    fn utc_timestamp_formatting_is_canonical() {
        assert_eq!(utc_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_timestamp(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(
            utc_timestamp(1_709_164_800 + 86_399),
            "2024-02-29T23:59:59Z"
        );
        assert!(valid_utc_timestamp("2024-02-29T23:59:59Z"));
        assert!(!valid_utc_timestamp("2023-02-29T00:00:00Z"));
        assert!(!valid_utc_timestamp("2024-02-29T24:00:00Z"));
        assert!(!valid_utc_timestamp(""));
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
            cleanup(&path);
            Self(path)
        }
    }

    impl Drop for TempAuditFile {
        fn drop(&mut self) {
            cleanup(&self.0);
        }
    }

    fn cleanup(target: &Path) {
        let lock = sibling_lock(target);
        let _ = fs::remove_file(target);
        let _ = fs::remove_file(&lock);
        let _ = fs::remove_dir_all(&lock);
    }

    fn sibling_lock(target: &Path) -> PathBuf {
        let mut name = target.as_os_str().to_os_string();
        name.push(".lock");
        PathBuf::from(name)
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
    fn jsonl_rejects_a_repeated_event_id() {
        let file = TempAuditFile::new();
        append_jsonl(
            &file.0,
            "A1",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            "first",
        )
        .expect("first append");
        let before = fs::read_to_string(&file.0).expect("read log");
        let error = append_jsonl(
            &file.0,
            "A1",
            AuditEventType::KnowledgeSearch,
            "search-cli",
            "default",
            "second",
        )
        .expect_err("duplicate event id is refused");
        assert!(error.contains("already contains event id"), "{error}");
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
    fn pre_v2_jsonl_event_without_a_timestamp_is_refused_with_migration_guidance() {
        let file = TempAuditFile::new();
        let mut log = AuditLog::new();
        log.append("A1", AuditEventType::DocumentImport, "alice", "T1", "data");
        let mut value = serde_json::to_value(&log.events[0]).expect("event encodes");
        value
            .as_object_mut()
            .expect("event is an object")
            .remove("timestamp");
        fs::write(
            &file.0,
            format!(
                "{}\n",
                serde_json::to_string(&value).expect("pre-v2 event encodes")
            ),
        )
        .expect("pre-v2 line writes");

        let error = read_jsonl(&file.0).expect_err("missing timestamp is refused");
        assert!(error.contains("pre-v2"), "{error}");
    }

    #[test]
    fn read_jsonl_refuses_a_truncated_final_line() {
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
        fs::write(&file.0, contents.trim_end_matches('\n')).expect("truncate final newline");

        let error = read_jsonl(&file.0).expect_err("truncated final line must be refused");
        assert!(error.contains("truncated"), "{error}");
    }

    #[test]
    fn read_jsonl_refuses_blank_lines() {
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
        fs::write(&file.0, format!("{contents}\n")).expect("insert blank line");

        let error = read_jsonl(&file.0).expect_err("blank line must be refused");
        assert!(error.contains("empty"), "{error}");
    }

    #[test]
    fn read_jsonl_refuses_structurally_invalid_existing_events() {
        let cases = [
            ("empty id", "", "alice", "T1", "data", "id"),
            ("blank actor", "A1", "   ", "T1", "data", "actor"),
            (
                "control character in tenant",
                "A1",
                "alice",
                "T1\n",
                "data",
                "tenant_id",
            ),
            ("empty payload", "A1", "alice", "T1", "", "payload"),
        ];
        for (label, id, actor, tenant, payload, field) in cases {
            let file = TempAuditFile::new();
            let event = AuditEvent::new(
                id,
                AuditEventType::DocumentImport,
                actor,
                tenant,
                payload,
                GENESIS_HASH,
            );
            fs::write(
                &file.0,
                format!(
                    "{}\n",
                    serde_json::to_string(&event).expect("crafted event encodes")
                ),
            )
            .expect("crafted event writes");

            let error =
                read_jsonl(&file.0).expect_err("a structurally invalid existing event is refused");
            assert!(error.contains(field), "{label}: {error}");
        }

        let file = TempAuditFile::new();
        let payload = "x".repeat(MAX_PAYLOAD_CHARS + 1);
        let event = AuditEvent::new(
            "A1",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            &payload,
            GENESIS_HASH,
        );
        fs::write(
            &file.0,
            format!(
                "{}\n",
                serde_json::to_string(&event).expect("oversized event encodes")
            ),
        )
        .expect("oversized event writes");
        let error = read_jsonl(&file.0).expect_err("an oversized payload is refused");
        assert!(error.contains("payload"), "{error}");
    }

    #[test]
    fn append_jsonl_rejects_an_empty_event_id_without_creating_the_file() {
        let file = TempAuditFile::new();
        let error = append_jsonl(
            &file.0,
            "",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            "first",
        )
        .expect_err("empty event id must be refused");
        assert!(error.contains("id"), "{error}");
        assert!(
            !file.0.exists(),
            "an invalid event must not create the audit file"
        );
    }

    #[test]
    fn a_leftover_lock_file_from_a_dead_writer_does_not_block_append() {
        let file = TempAuditFile::new();
        fs::write(
            sibling_lock(&file.0),
            format!("dead-writer\n{}\n", now_secs().saturating_sub(3_600)),
        )
        .expect("leftover lock file writes");

        append_jsonl(
            &file.0,
            "A1",
            AuditEventType::DocumentImport,
            "alice",
            "T1",
            "first",
        )
        .expect("an unheld lock file does not block the append");
        let log = read_jsonl(&file.0).expect("new event is readable");
        assert_eq!(log.len(), 1);
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
