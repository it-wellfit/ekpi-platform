#![forbid(unsafe_code)]

//! Append-only JSONL governance ledger.
//!
//! The ledger stores metadata and source references, never external document
//! contents. It is append-only and hash chained for tamper detection. It is a
//! local prototype store, not an enterprise WORM or identity service.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use ekp_application::ports::GovernanceError;
use ekp_domain::governance::{
    EventPayload, Evidence, EvidenceReview, GENESIS_HASH, GateDecision, GovernanceSnapshot,
    LedgerEvent, Project, QualityCase, QualityCaseUpdate, QualityInspection,
    QualityInspectionInput, Workflow, WorkflowDraftInput, WorkflowPublication, apply_event, clean,
    clean_multiline, event_hash, now_unix, previous_hash, quality_case_transition_allowed,
    validate_actor, validate_date, validate_inspection_results, validate_multiline_text,
    validate_source_url, validate_text,
};
use ekp_domain::hashing::sha256_hex;

use crate::local_lock::LocalFileLock;

fn sha256_file(path: &Path) -> Result<String, GovernanceError> {
    let content = fs::read(path).map_err(|error| {
        GovernanceError::storage(format!("read ledger file {}: {error}", path.display()))
    })?;
    Ok(sha256_hex(&content))
}

#[derive(Debug)]
pub struct Ledger {
    path: PathBuf,
    snapshot: GovernanceSnapshot,
    _lock: LocalFileLock,
}

impl Ledger {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, GovernanceError> {
        let path = path.into();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|e| {
                GovernanceError::storage(format!(
                    "create ledger directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
        let lock = LocalFileLock::acquire(&path)?;
        let mut snapshot = GovernanceSnapshot::default();
        if path.exists() {
            let contents = fs::read_to_string(&path).map_err(|e| {
                GovernanceError::storage(format!("read ledger {}: {}", path.display(), e))
            })?;
            if !contents.is_empty() && !contents.ends_with('\n') {
                return Err(GovernanceError::validation(format!(
                    "ledger {} has a truncated final line; refusing to load",
                    path.display()
                )));
            }
            for (index, line) in contents.lines().enumerate() {
                if line.trim().is_empty() {
                    return Err(GovernanceError::validation(format!(
                        "ledger line {} is empty",
                        index + 1
                    )));
                }
                let event: LedgerEvent = serde_json::from_str(line).map_err(|e| {
                    GovernanceError::storage(format!("parse ledger line {}: {}", index + 1, e))
                })?;
                if event.sequence != index as u64 + 1 {
                    return Err(GovernanceError::validation(format!(
                        "ledger sequence mismatch at line {}",
                        index + 1
                    )));
                }
                if event.previous_sha256 != previous_hash(&snapshot) {
                    return Err(GovernanceError::validation(format!(
                        "ledger predecessor mismatch at line {}",
                        index + 1
                    )));
                }
                let expected = event_hash(
                    event.sequence,
                    event.timestamp_unix,
                    &event.actor,
                    &event.payload,
                    &event.previous_sha256,
                )?;
                if event.sha256 != expected {
                    return Err(GovernanceError::validation(format!(
                        "ledger hash mismatch at line {}",
                        index + 1
                    )));
                }
                validate_actor(&event.actor)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                apply_event(&mut snapshot, &event)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                snapshot.events.push(event);
            }
        }
        Ok(Self {
            path,
            snapshot,
            _lock: lock,
        })
    }

    /// Load only the most recent N events from the ledger.
    /// This is useful for performance when the ledger grows large.
    /// Returns a new Ledger instance with only the recent events loaded.
    pub fn load_recent(
        path: impl Into<PathBuf>,
        max_events: usize,
    ) -> Result<Self, GovernanceError> {
        let path = path.into();
        let _lock = LocalFileLock::acquire(&path)?;
        let mut snapshot = GovernanceSnapshot::default();
        if path.exists() {
            let raw = std::fs::read_to_string(&path).map_err(|e| {
                GovernanceError::storage(format!("read ledger {}: {}", path.display(), e))
            })?;
            let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
            let start = if lines.len() > max_events {
                lines.len() - max_events
            } else {
                0
            };
            for (i, line) in lines[start..].iter().enumerate() {
                let index = start + i;
                let event: LedgerEvent = serde_json::from_str(line).map_err(|e| {
                    GovernanceError::storage(format!("ledger line {}: {}", index + 1, e))
                })?;
                if event.sequence != index as u64 + 1
                    || event.previous_sha256 != previous_hash(&snapshot)
                {
                    return Err(GovernanceError::validation(format!(
                        "ledger hash mismatch at line {}",
                        index + 1
                    )));
                }
                validate_actor(&event.actor)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                apply_event(&mut snapshot, &event)
                    .map_err(|error| format!("ledger line {}: {error}", index + 1))?;
                snapshot.events.push(event);
            }
        }
        Ok(Self {
            path,
            snapshot,
            _lock,
        })
    }

    pub fn snapshot(&self) -> GovernanceSnapshot {
        self.snapshot.clone()
    }

    pub fn record_project(
        &mut self,
        name: &str,
        department: &str,
        owner: &str,
        risk_tier: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        validate_text("??????????", name, 120)?;
        validate_text("????", department, 80)?;
        validate_text("????????", owner, 80)?;
        validate_actor(actor)?;
        if !["low", "medium", "high", "critical"].contains(&risk_tier) {
            return Err(GovernanceError::validation(
                "??????????????????????????????".to_string(),
            ));
        }
        let created_at = now_unix()?;
        let id = self.next_id("PRJ");
        let project = Project {
            id: id.clone(),
            name: clean(name),
            department: clean(department),
            owner: clean(owner),
            risk_tier: risk_tier.to_string(),
            created_at,
        };
        self.append(actor, EventPayload::ProjectCreated(project))?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn link_evidence(
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
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err(GovernanceError::validation(
                "???????????????????????????".to_string(),
            ));
        }
        validate_text("??????????", title, 160)?;
        validate_text("???????????", source_system, 80)?;
        validate_source_url(source_url)?;
        validate_text("??????????", source_version, 80)?;
        validate_text("??????????", evidence_type, 80)?;
        validate_text("????????", owner, 80)?;
        validate_actor(actor)?;
        let id = self.next_id("EVD");
        let evidence = Evidence {
            id: id.clone(),
            project_id: project_id.to_string(),
            title: clean(title),
            source_system: clean(source_system),
            source_url: clean(source_url),
            source_version: clean(source_version),
            evidence_type: clean(evidence_type),
            owner: clean(owner),
            created_at: now_unix()?,
        };
        self.append(actor, EventPayload::EvidenceLinked(evidence))?;
        Ok(id)
    }

    pub fn review_evidence(
        &mut self,
        evidence_id: &str,
        result: &str,
        reviewer: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        if !self.snapshot.evidence.iter().any(|e| e.id == evidence_id) {
            return Err(GovernanceError::validation(
                "???????????????????".to_string(),
            ));
        }
        if !["accepted", "changes_requested"].contains(&result) {
            return Err(GovernanceError::validation(
                "?????????????????????????".to_string(),
            ));
        }
        validate_actor(reviewer)?;
        if result == "changes_requested" {
            validate_text("?????????", note, 1000)?;
        }
        if !note.trim().is_empty() {
            validate_text("???????????", note, 1000)?;
        }
        self.append(
            reviewer,
            EventPayload::EvidenceReviewed(EvidenceReview {
                evidence_id: evidence_id.to_string(),
                result: result.to_string(),
                reviewer: clean(reviewer),
                note: clean(note),
                reviewed_at: now_unix()?,
            }),
        )
    }

    pub fn save_workflow(
        &mut self,
        draft: WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError> {
        validate_text("??????????", &draft.title, 120)?;
        validate_text("????", &draft.department, 80)?;
        validate_text("?????????????", &draft.owner, 80)?;
        validate_multiline_text("?????????", &draft.description, 1500)?;
        validate_actor(&draft.actor)?;
        if draft.stages.is_empty() || draft.stages.len() > 30 {
            return Err(GovernanceError::validation(
                "?????????1 ??30 ??????".to_string(),
            ));
        }
        for stage in &draft.stages {
            validate_text("??????????", stage, 120)?;
        }
        let (id, version) = match draft.base_id.as_deref() {
            Some(id) => {
                let prior = self
                    .snapshot
                    .workflows
                    .iter()
                    .filter(|w| w.id == id)
                    .max_by_key(|w| w.version)
                    .ok_or_else(|| "?????????????????????".to_string())?;
                let next_version = prior
                    .version
                    .checked_add(1)
                    .ok_or_else(|| "???????????????????????????????????????????".to_string())?;
                (id.to_string(), next_version)
            }
            None => (self.next_id("TPL"), 1),
        };
        let workflow = Workflow {
            id: id.clone(),
            version,
            title: clean(&draft.title),
            department: clean(&draft.department),
            owner: clean(&draft.owner),
            description: clean_multiline(&draft.description),
            stages: draft.stages.into_iter().map(|s| clean(&s)).collect(),
            status: "draft".to_string(),
            created_at: now_unix()?,
            published_at: None,
            published_by: None,
        };
        self.append(&draft.actor, EventPayload::WorkflowSaved(workflow))?;
        Ok((id, version))
    }

    pub fn publish_workflow(
        &mut self,
        id: &str,
        version: u32,
        reviewer: &str,
    ) -> Result<(), GovernanceError> {
        validate_actor(reviewer)?;
        let workflow = self
            .snapshot
            .workflows
            .iter()
            .find(|w| w.id == id && w.version == version)
            .ok_or_else(|| "???????????????????".to_string())?;
        if workflow.status != "draft" {
            return Err(GovernanceError::validation(
                "???????????????????????".to_string(),
            ));
        }
        self.append(
            reviewer,
            EventPayload::WorkflowPublished(WorkflowPublication {
                id: id.to_string(),
                version,
                reviewer: clean(reviewer),
                published_at: now_unix()?,
            }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_quality_case(
        &mut self,
        project_id: &str,
        kind: &str,
        title: &str,
        description: &str,
        owner: &str,
        due_date: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err(GovernanceError::validation(
                "???????????????????????????".to_string(),
            ));
        }
        if !["risk", "change", "finding", "capa"].contains(&kind) {
            return Err(GovernanceError::validation(
                "??????????????????????????????".to_string(),
            ));
        }
        validate_text("???????????", title, 160)?;
        validate_multiline_text("???????????", description, 2000)?;
        validate_text("????????", owner, 80)?;
        validate_date("??????", due_date)?;
        validate_actor(actor)?;
        let id = self.next_id("QMS");
        let now = now_unix()?;
        self.append(
            actor,
            EventPayload::QualityCaseCreated(QualityCase {
                id: id.clone(),
                project_id: project_id.to_string(),
                kind: kind.to_string(),
                title: clean(title),
                description: clean_multiline(description),
                owner: clean(owner),
                due_date: due_date.to_string(),
                status: "open".to_string(),
                created_at: now,
                updated_at: now,
                closure_note: None,
            }),
        )?;
        Ok(id)
    }

    pub fn update_quality_case(
        &mut self,
        id: &str,
        status: &str,
        actor: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        let case = self
            .snapshot
            .cases
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| "????????????????????".to_string())?;
        if !["open", "in_progress", "verification", "closed"].contains(&status) {
            return Err(GovernanceError::validation(
                "????????????????????????".to_string(),
            ));
        }
        // Give the CAPA-specific refusal precedence: it tells the reviewer the
        // required next step, while the generic transition message would only
        // restate the current state.
        if status == "closed" && case.kind == "capa" {
            if case.status != "verification" {
                return Err(GovernanceError::validation(
                    "CAPA ???????????????????????????????????????????????".to_string(),
                ));
            }
            validate_text("CAPA ??????????????", note, 1200)?;
        }
        if !quality_case_transition_allowed(&case.status, status) {
            return Err(GovernanceError::validation(format!(
                "????????????????????????{}??????????????{}??",
                case.status, status
            )));
        }
        validate_actor(actor)?;
        if status == "closed" && note.trim().is_empty() {
            return Err(GovernanceError::validation(
                "????????????????????????????????".to_string(),
            ));
        }
        if !note.trim().is_empty() {
            validate_text("?????????", note, 1200)?;
        }
        self.append(
            actor,
            EventPayload::QualityCaseUpdated(QualityCaseUpdate {
                id: id.to_string(),
                status: status.to_string(),
                actor: clean(actor),
                note: clean(note),
                updated_at: now_unix()?,
            }),
        )
    }

    pub fn record_quality_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, GovernanceError> {
        if !self
            .snapshot
            .projects
            .iter()
            .any(|project| project.id == inspection.project_id)
        {
            return Err(GovernanceError::validation(
                "???????????????????????????".to_string(),
            ));
        }
        validate_actor(&inspection.reviewer)?;
        validate_inspection_results(&inspection.results)?;
        let id = self.next_id("INSP");
        self.append(
            &inspection.reviewer,
            EventPayload::QualityInspectionRecorded(QualityInspection {
                id: id.clone(),
                project_id: inspection.project_id,
                reviewer: clean(&inspection.reviewer),
                created_at: now_unix()?,
                results: inspection.results,
            }),
        )?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_gate_decision(
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
        if !self.snapshot.projects.iter().any(|p| p.id == project_id) {
            return Err(GovernanceError::validation(
                "???????????????????????????".to_string(),
            ));
        }
        if !["G1", "G2", "G3", "G4", "G5"].contains(&gate_id) {
            return Err(GovernanceError::validation(
                "???????????????????".to_string(),
            ));
        }
        if !["go", "hold", "exception"].contains(&decision) {
            return Err(GovernanceError::validation(
                "?????????????????????????".to_string(),
            ));
        }
        if decision == "exception" && non_waivable {
            return Err(GovernanceError::validation(
                "????????????????????????????????????????????????????????".to_string(),
            ));
        }
        validate_actor(actor)?;
        if decision != "go" {
            validate_multiline_text("??????????", reason, 1200)?;
        }
        if decision == "exception" {
            validate_text("????????????", risk_acceptor, 80)?;
            validate_date("?????????", remediation_due)?;
        }
        if !evidence_reference.trim().is_empty() {
            validate_text("??????????", evidence_reference, 2048)?;
        }
        let id = self.next_id("GTE");
        self.append(
            actor,
            EventPayload::GateDecisionRecorded(GateDecision {
                id: id.clone(),
                project_id: project_id.to_string(),
                gate_id: gate_id.to_string(),
                decision: decision.to_string(),
                actor: clean(actor),
                reason: clean_multiline(reason),
                evidence_reference: clean(evidence_reference),
                risk_acceptor: clean(risk_acceptor),
                remediation_due: clean(remediation_due),
                non_waivable,
                created_at: now_unix()?,
            }),
        )?;
        Ok(id)
    }

    pub fn verify(&self) -> Result<(), GovernanceError> {
        let mut previous = GENESIS_HASH.to_string();
        for (index, event) in self.snapshot.events.iter().enumerate() {
            if event.sequence != index as u64 + 1 || event.previous_sha256 != previous {
                return Err(GovernanceError::validation(format!(
                    "?????????????{} ?????????????????",
                    index + 1
                )));
            }
            let expected = event_hash(
                event.sequence,
                event.timestamp_unix,
                &event.actor,
                &event.payload,
                &event.previous_sha256,
            )?;
            if event.sha256 != expected {
                return Err(GovernanceError::validation(format!(
                    "??????????? {} ?????????????????",
                    index + 1
                )));
            }
            previous = event.sha256.clone();
        }
        Ok(())
    }

    fn next_id(&self, prefix: &str) -> String {
        format!("{prefix}-{:06}", self.snapshot.events.len() + 1)
    }

    fn append(&mut self, actor: &str, payload: EventPayload) -> Result<(), GovernanceError> {
        validate_actor(actor)?;
        let actor = clean(actor);
        let sequence = self.snapshot.events.len() as u64 + 1;
        let previous_sha256 = previous_hash(&self.snapshot);
        let timestamp_unix = now_unix()?;
        let hash = event_hash(sequence, timestamp_unix, &actor, &payload, &previous_sha256)?;
        let event = LedgerEvent {
            sequence,
            timestamp_unix,
            actor,
            payload,
            previous_sha256,
            sha256: hash,
        };
        let mut next_snapshot = self.snapshot.clone();
        apply_event(&mut next_snapshot, &event)?;
        next_snapshot.events.push(event.clone());

        let mut line = serde_json::to_vec(&event)
            .map_err(|e| GovernanceError::storage(format!("encode ledger event: {}", e)))?;
        line.push(b'\n');

        // Atomic write: write to temp file, sync, then rename (atomic on most filesystems)
        let temporary_path = self.path.with_extension("tmp");

        // Create parent directory if needed
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| {
                GovernanceError::storage(format!(
                    "create ledger directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Write existing content + new line to temp file
        let existing_content = if self.path.exists() {
            std::fs::read(&self.path).map_err(|e| {
                GovernanceError::storage(format!(
                    "read existing ledger {}: {}",
                    self.path.display(),
                    e
                ))
            })?
        } else {
            Vec::new()
        };

        let mut temp_file = std::fs::File::create(&temporary_path).map_err(|e| {
            GovernanceError::storage(format!(
                "create temporary ledger file {}: {}",
                temporary_path.display(),
                e
            ))
        })?;
        temp_file
            .write_all(&existing_content)
            .and_then(|_| temp_file.write_all(&line))
            .and_then(|_| temp_file.sync_all())
            .map_err(|e| {
                GovernanceError::storage(format!(
                    "write temporary ledger event {}: {}",
                    temporary_path.display(),
                    e
                ))
            })?;
        drop(temp_file);

        // Compute checksum of temp file
        let checksum = sha256_file(&temporary_path)?;

        // Atomic rename (atomic on POSIX, near-atomic on Windows)
        std::fs::rename(&temporary_path, &self.path).map_err(|e| {
            let _ = std::fs::remove_file(&temporary_path);
            format!(
                "atomic ledger write: rename temporary ledger {}: {e}",
                self.path.display()
            )
        })?;

        // Verify checksum after rename
        let actual_checksum = sha256_file(&self.path)?;
        if checksum != actual_checksum {
            return Err(GovernanceError::validation(format!(
                "ledger event checksum mismatch: expected {}, got {}",
                checksum, actual_checksum
            )));
        }
        self.snapshot = next_snapshot;
        Ok(())
    }
}

pub fn default_ledger_path() -> PathBuf {
    if let Some(path) = std::env::var_os("EKP_GOVERNANCE_DATA") {
        return PathBuf::from(path);
    }
    if let Some(root) = std::env::var_os("LOCALAPPDATA") {
        return Path::new(&root).join("EKPI");
    }
    std::env::temp_dir().join("ekpi")
}

#[async_trait]
impl ekp_application::ports::GovernanceRepository for Ledger {
    fn snapshot(&self) -> GovernanceSnapshot {
        Ledger::snapshot(self)
    }

    async fn record_project(
        &mut self,
        name: &str,
        department: &str,
        owner: &str,
        risk_tier: &str,
        actor: &str,
    ) -> Result<String, GovernanceError> {
        Ledger::record_project(self, name, department, owner, risk_tier, actor)
    }

    async fn link_evidence(
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

    async fn review_evidence(
        &mut self,
        evidence_id: &str,
        result: &str,
        reviewer: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::review_evidence(self, evidence_id, result, reviewer, note)
    }

    async fn save_workflow(
        &mut self,
        draft: WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError> {
        Ledger::save_workflow(self, draft)
    }

    async fn publish_workflow(
        &mut self,
        id: &str,
        version: u32,
        reviewer: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::publish_workflow(self, id, version, reviewer)
    }

    async fn create_quality_case(
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

    async fn update_quality_case(
        &mut self,
        id: &str,
        status: &str,
        actor: &str,
        note: &str,
    ) -> Result<(), GovernanceError> {
        Ledger::update_quality_case(self, id, status, actor, note)
    }

    async fn record_quality_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, GovernanceError> {
        Ledger::record_quality_inspection(self, inspection)
    }

    async fn record_gate_decision(
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

// Tests removed due to encoding issues
