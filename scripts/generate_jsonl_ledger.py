"""Generate crates/adapters/src/jsonl_ledger.rs from the historical platform module.

The transformation is mechanical: the Ledger keeps its behaviour, but its
error type becomes the application layer's GovernanceError so that storage
failures can be distinguished from rule violations.
"""

import io
import os
import sys

ROOT = r"C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\ekpi-platform"
SRC = os.path.join(ROOT, "platform", "src", "governance.rs")
DST = os.path.join(ROOT, "crates", "adapters", "src", "jsonl_ledger.rs")

with io.open(SRC, "r", encoding="utf-8", newline="") as handle:
    lines = handle.read().split("\n")


def block(start, end):
    """Return source lines 1-indexed inclusive."""
    return lines[start - 1 : end]


def find_close(text, open_index):
    """Return the index of the parenthesis that closes the one at open_index."""
    depth = 0
    index = open_index
    in_string = False
    escaped = False
    while index < len(text):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
        else:
            if char == '"':
                in_string = True
            elif char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    return index
        index += 1
    raise ValueError("unbalanced parenthesis")


def wrap_err(text):
    """Turn `Err(format!(...))` and `Err("...".to_string())` into typed errors."""
    out = []
    index = 0
    while True:
        position = text.find("Err(", index)
        if position < 0:
            out.append(text[index:])
            break
        out.append(text[index:position])
        close = find_close(text, position + 3)
        inner = text[position + 4 : close]
        stripped = inner.strip()
        if stripped.startswith("format!(") or stripped.endswith(".to_string()"):
            out.append("Err(GovernanceError::validation(" + inner + "))")
        else:
            out.append(text[position : close + 1])
        index = close + 1
    return "".join(out)


def wrap_map_err(text):
    """Turn `map_err(|e| format!(...))` into a storage error."""
    out = []
    index = 0
    marker = "map_err(|e| format!("
    while True:
        position = text.find(marker, index)
        if position < 0:
            out.append(text[index:])
            break
        out.append(text[index:position])
        out.append("map_err(|e| GovernanceError::storage(format!(")
        format_open = position + len(marker) - 1
        close = find_close(text, format_open)
        out.append(text[format_open : close + 1])
        out.append(")")
        index = close + 1
    return "".join(out)


ledger_struct = "\n".join(block(349, 354))
ledger_impl = "\n".join(block(365, 901))
ledger_path = "\n".join(block(1267, 1279))
ledger_tests = "\n".join(block(1434, len(lines)))

body = ledger_impl
body = body.replace("Result<(), String>", "Result<(), GovernanceError>")
body = body.replace("Result<String, String>", "Result<String, GovernanceError>")
body = body.replace("Result<(String, u32), String>", "Result<(String, u32), GovernanceError>")
body = wrap_err(body)
body = wrap_map_err(body)

header = '''#![forbid(unsafe_code)]

//! Append-only JSONL governance ledger.
//!
//! The ledger stores metadata and source references, never external document
//! contents. It is append-only and hash chained for tamper detection. It is a
//! local prototype store, not an enterprise WORM or identity service.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ekp_application::ports::GovernanceError;
use ekp_domain::governance::{
    Evidence, EvidenceReview, EventPayload, GateDecision, GENESIS_HASH, GovernanceSnapshot,
    LedgerEvent, Project, QualityCase, QualityCaseUpdate, QualityCheckResult, QualityInspection,
    QualityInspectionInput, Workflow, WorkflowDraftInput, WorkflowPublication, apply_event,
    clean, clean_multiline, event_hash, now_unix, previous_hash, quality_case_transition_allowed,
    require_payload_actor, validate_actor, validate_date, validate_inspection_results,
    validate_multiline_text, validate_record_id, validate_source_url, validate_text,
};
use ekp_domain::hashing::sha256_hex;

use crate::local_lock::LocalFileLock;

'''

footer = '''
impl crate::ports::GovernanceRepository for Ledger {
    fn snapshot(&self) -> GovernanceSnapshot {
        Ledger::snapshot(self)
    }

    fn record_project(
        &mut self,
        name: &str,
        department: &str,
        owner: &str,
        risk_tier: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        Ledger::record_project(self, name, department, owner, risk_tier, actor)
    }

    fn link_evidence(
        &mut self,
        project_id: &str,
        title: &str,
        source_system: &str,
        source_url: &str,
        source_version: &str,
        evidence_type: &str,
        owner: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        Ledger::link_evidence(
            self,
            project_id,
            title,
            source_system,
            source_url,
            source_version,
            evidence_type,
            owner,
            actor,
        )
    }

    fn review_evidence(
        &mut self,
        evidence_id: &str,
        result: &str,
        reviewer: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::review_evidence(self, evidence_id, result, reviewer, note)
    }

    fn save_workflow(
        &mut self,
        draft: WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError> {
        Ledger::save_workflow(self, draft)
    }

    fn publish_workflow(
        &mut self,
        id: &str,
        version: u32,
        reviewer: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::publish_workflow(self, id, version, reviewer)
    }

    fn create_quality_case(
        &mut self,
        project_id: &str,
        kind: &str,
        title: &str,
        description: &str,
        owner: &str,
        due_date: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        Ledger::create_quality_case(
            self,
            project_id,
            kind,
            title,
            description,
            owner,
            due_date,
            actor,
        )
    }

    fn update_quality_case(
        &mut self,
        id: &str,
        status: &str,
        actor: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::update_quality_case(self, id, status, actor, note)
    }

    fn record_quality_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, GovernanceError> {
        Ledger::record_quality_inspection(self, inspection)
    }

    fn record_gate_decision(
        &mut self,
        project_id: &str,
        gate_id: &str,
        decision: &str,
        actor: &str,
        reason: &str,
        evidence_reference: &str,
        risk_acceptor: &str,
        remediation_due: &str,
        non_waivable: bool,
    ) -> Result<String, GovernanceError> {
        Ledger::record_gate_decision(
            self,
            project_id,
            gate_id,
            decision,
            actor,
            reason,
            evidence_reference,
            risk_acceptor,
            remediation_due,
            non_waivable,
        )
    }

    fn verify_integrity(&self) -> Result<(), GovernanceError> {
        Ledger::verify(self)
    }
}
'''

sha256_file = '''fn sha256_file(path: &Path) -> Result<String, GovernanceError> {
    let content = fs::read(path).map_err(|error| {
        GovernanceError::storage(format!("read ledger file {}: {error}", path.display()))
    })?;
    Ok(sha256_hex(&content))
}'''

parts = [
    header,
    sha256_file,
    "",
    ledger_struct,
    "",
    body,
    "",
    ledger_path,
    footer,
    "",
    ledger_tests,
]

with io.open(DST, "w", encoding="utf-8", newline="\n") as handle:
    handle.write("\n".join(parts))

print("written", DST)
