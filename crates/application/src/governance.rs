#![forbid(unsafe_code)]

//! Governance use cases.
//!
//! Each use case names an intent, validates the caller's authority and then
//! asks the repository to persist the change. The use cases add no storage
//! knowledge of their own.

use ekp_domain::governance::{GovernanceSnapshot, QualityInspectionInput, WorkflowDraftInput};

use crate::ports::{GovernanceError, GovernanceRepository};

/// Register a project.
#[derive(Debug, Clone)]
pub struct RegisterProject {
    pub name: String,
    pub department: String,
    pub owner: String,
    pub risk_tier: String,
    pub actor: String,
}

/// Link a source reference to a project.
#[derive(Debug, Clone)]
pub struct LinkEvidence {
    pub project_id: String,
    pub title: String,
    pub source_system: String,
    pub source_url: String,
    pub source_version: String,
    pub evidence_type: String,
    pub owner: String,
    pub actor: String,
}

/// Record a human review of a source reference.
#[derive(Debug, Clone)]
pub struct ReviewEvidence {
    pub evidence_id: String,
    pub result: String,
    pub reviewer: String,
    pub note: String,
}

/// Publish a workflow draft.
#[derive(Debug, Clone)]
pub struct PublishWorkflow {
    pub id: String,
    pub version: u32,
    pub reviewer: String,
}

/// Open a quality or risk case.
#[derive(Debug, Clone)]
pub struct OpenQualityCase {
    pub project_id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub owner: String,
    pub due_date: String,
    pub actor: String,
}

/// Update the status of a quality case.
#[derive(Debug, Clone)]
pub struct UpdateQualityCase {
    pub id: String,
    pub status: String,
    pub actor: String,
    pub note: String,
}

/// Record a human gate decision.
#[derive(Debug, Clone)]
pub struct RecordGateDecision {
    pub project_id: String,
    pub gate_id: String,
    pub decision: String,
    pub actor: String,
    pub reason: String,
    pub evidence_reference: String,
    pub risk_acceptor: String,
    pub remediation_due: String,
    pub non_waivable: bool,
}

/// Governance use cases bound to one repository.
pub struct GovernanceService<'a> {
    repository: &'a mut dyn GovernanceRepository,
}

impl<'a> GovernanceService<'a> {
    pub fn new(repository: &'a mut dyn GovernanceRepository) -> Self {
        Self { repository }
    }

    /// Return the current governance state.
    pub fn snapshot(&self) -> GovernanceSnapshot {
        self.repository.snapshot()
    }

    /// Register a project.
    pub async fn register_project(
        &mut self,
        command: RegisterProject,
    ) -> Result<String, GovernanceError> {
        self.repository
            .record_project(
                &command.name,
                &command.department,
                &command.owner,
                &command.risk_tier,
                &command.actor,
            )
            .await
    }

    /// Link a source reference to a project.
    pub async fn link_evidence(
        &mut self,
        command: LinkEvidence,
    ) -> Result<String, GovernanceError> {
        self.repository
            .link_evidence(
                &command.project_id,
                &command.title,
                &command.source_system,
                &command.source_url,
                &command.source_version,
                &command.evidence_type,
                &command.owner,
                &command.actor,
            )
            .await
    }

    /// Record a human review of a source reference.
    pub async fn review_evidence(
        &mut self,
        command: ReviewEvidence,
    ) -> Result<(), GovernanceError> {
        self.repository
            .review_evidence(
                &command.evidence_id,
                &command.result,
                &command.reviewer,
                &command.note,
            )
            .await
    }

    /// Save a workflow draft.
    pub async fn save_workflow(
        &mut self,
        draft: WorkflowDraftInput,
    ) -> Result<(String, u32), GovernanceError> {
        self.repository.save_workflow(draft).await
    }

    /// Publish a workflow draft.
    pub async fn publish_workflow(
        &mut self,
        command: PublishWorkflow,
    ) -> Result<(), GovernanceError> {
        self.repository
            .publish_workflow(&command.id, command.version, &command.reviewer)
            .await
    }

    /// Open a quality or risk case.
    pub async fn open_quality_case(
        &mut self,
        command: OpenQualityCase,
    ) -> Result<String, GovernanceError> {
        self.repository
            .create_quality_case(
                &command.project_id,
                &command.kind,
                &command.title,
                &command.description,
                &command.owner,
                &command.due_date,
                &command.actor,
            )
            .await
    }

    /// Update the status of a quality case.
    pub async fn update_quality_case(
        &mut self,
        command: UpdateQualityCase,
    ) -> Result<(), GovernanceError> {
        self.repository
            .update_quality_case(&command.id, &command.status, &command.actor, &command.note)
            .await
    }

    /// Record a complete quality inspection.
    pub async fn record_inspection(
        &mut self,
        inspection: QualityInspectionInput,
    ) -> Result<String, GovernanceError> {
        self.repository.record_quality_inspection(inspection).await
    }

    /// Record a human gate decision.
    pub async fn record_gate_decision(
        &mut self,
        command: RecordGateDecision,
    ) -> Result<String, GovernanceError> {
        self.repository
            .record_gate_decision(
                &command.project_id,
                &command.gate_id,
                &command.decision,
                &command.actor,
                &command.reason,
                &command.evidence_reference,
                &command.risk_acceptor,
                &command.remediation_due,
                command.non_waivable,
            )
            .await
    }

    /// Verify that the stored audit chain is intact.
    pub fn verify_integrity(&self) -> Result<(), GovernanceError> {
        self.repository.verify_integrity()
    }
}
