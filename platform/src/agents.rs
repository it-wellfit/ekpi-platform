#![forbid(unsafe_code)]

//! Agent 協作與獨立驗證機制（Master Prompt Phase 6, §7）。
//!
//! 採用可組合的 Agent 工作流，而不是讓多個 Agent 無限制地互相對話。
//! 每個 Agent 有明確的輸入、輸出、權限、工具與停止條件。
//! 高風險結論必須交給獨立驗證器，而非由同一個生成步驟自行宣布成功。
//!
//! 迭代閉環：任務分析 → 檢索 → 建立主張 → 證據驗證 → 發現缺口 → 補充檢索 → 規則檢查 → 最終驗證。
//! 停止條件：所有主張有足夠證據 / 找不到新證據 / 達到上限 / 不可調和衝突 / 需人工決策 / 無法證明。

use serde::{Deserialize, Serialize};

/// Agent 角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Planner,
    DocumentAnalyst,
    Retriever,
    KnowledgeAnalyst,
    ProcessAnalyst,
    CausalAnalyst,
    EvidenceAuditor,
    RuleValidator,
    ReportGenerator,
    FinalValidator,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::Planner => "Planner",
            AgentRole::DocumentAnalyst => "DocumentAnalyst",
            AgentRole::Retriever => "Retriever",
            AgentRole::KnowledgeAnalyst => "KnowledgeAnalyst",
            AgentRole::ProcessAnalyst => "ProcessAnalyst",
            AgentRole::CausalAnalyst => "CausalAnalyst",
            AgentRole::EvidenceAuditor => "EvidenceAuditor",
            AgentRole::RuleValidator => "RuleValidator",
            AgentRole::ReportGenerator => "ReportGenerator",
            AgentRole::FinalValidator => "FinalValidator",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AgentRole::Planner => "理解任務、拆分子問題",
            AgentRole::DocumentAnalyst => "解析文件結構及版本",
            AgentRole::Retriever => "選擇並執行檢索策略",
            AgentRole::KnowledgeAnalyst => "分析跨文件關係",
            AgentRole::ProcessAnalyst => "重建流程及狀態",
            AgentRole::CausalAnalyst => "建立可檢驗的因果假設",
            AgentRole::EvidenceAuditor => "驗證主張與證據",
            AgentRole::RuleValidator => "執行明確的規則檢查",
            AgentRole::ReportGenerator => "產生可讀報告",
            AgentRole::FinalValidator => "檢查輸出格式、引用、矛盾及未解決問題",
        }
    }
}

/// Agent 輸入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub role: AgentRole,
    pub task: String,
    pub context: Vec<String>,
    pub evidence_pool: Vec<String>,
}

/// Agent 輸出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub role: AgentRole,
    pub findings: Vec<String>,
    pub new_evidence: Vec<String>,
    pub claims_modified: bool,
    pub rules_re_evaluated: bool,
    pub unresolved: Vec<String>,
    pub stopping_reason: Option<StoppingReason>,
}

/// 停止條件（A10：迭代次數不得作為正確性證據）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoppingReason {
    /// 所有必要主張都有足夠證據。
    AllClaimsSupported,
    /// 找不到新證據。
    NoNewEvidence,
    /// 達到時間/成本/迭代上限。
    LimitReached,
    /// 發現不可調和的證據衝突。
    IrreconcilableConflict,
    /// 需要人工決策。
    RequiresHumanDecision,
    /// 無法證明結論成立。
    CannotProve,
}

/// Agent 工作流步驟。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStep {
    TaskAnalysis,
    Retrieval,
    ClaimBuilding,
    EvidenceVerification,
    GapDiscovery,
    SupplementalRetrieval,
    RuleCheck,
    FinalValidation,
}

impl WorkflowStep {
    pub fn next(self) -> Option<WorkflowStep> {
        match self {
            WorkflowStep::TaskAnalysis => Some(WorkflowStep::Retrieval),
            WorkflowStep::Retrieval => Some(WorkflowStep::ClaimBuilding),
            WorkflowStep::ClaimBuilding => Some(WorkflowStep::EvidenceVerification),
            WorkflowStep::EvidenceVerification => Some(WorkflowStep::GapDiscovery),
            WorkflowStep::GapDiscovery => Some(WorkflowStep::SupplementalRetrieval),
            WorkflowStep::SupplementalRetrieval => Some(WorkflowStep::RuleCheck),
            WorkflowStep::RuleCheck => Some(WorkflowStep::FinalValidation),
            WorkflowStep::FinalValidation => None,
        }
    }
}

/// Agent 工作流狀態。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub current_step: WorkflowStep,
    pub iteration: usize,
    pub max_iterations: usize,
    pub accumulated_evidence: Vec<String>,
    pub claims: Vec<String>,
    pub unresolved: Vec<String>,
    pub stopping_reason: Option<StoppingReason>,
}

impl WorkflowState {
    pub fn new(max_iterations: usize) -> Self {
        WorkflowState {
            current_step: WorkflowStep::TaskAnalysis,
            iteration: 0,
            max_iterations,
            accumulated_evidence: Vec::new(),
            claims: Vec::new(),
            unresolved: Vec::new(),
            stopping_reason: None,
        }
    }

    /// 檢查是否應停止（A10：每次迭代必須產出新證據/修改主張/重新評估規則）。
    pub fn should_stop(
        &self,
        produced_new_evidence: bool,
        modified_claims: bool,
        re_evaluated_rules: bool,
    ) -> bool {
        if produced_new_evidence || modified_claims || re_evaluated_rules {
            return false;
        }
        // 三者皆無 → 停止
        true
    }

    /// 推進到下一步。
    pub fn advance(&mut self) -> bool {
        if self.iteration >= self.max_iterations {
            self.stopping_reason = Some(StoppingReason::LimitReached);
            return false;
        }
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            self.iteration += 1;
            true
        } else {
            self.stopping_reason = Some(StoppingReason::AllClaimsSupported);
            false
        }
    }
}

/// Agent 協作框架。
pub struct AgentFramework;

impl AgentFramework {
    /// 取得所有 Agent 角色。
    pub fn all_roles() -> Vec<AgentRole> {
        vec![
            AgentRole::Planner,
            AgentRole::DocumentAnalyst,
            AgentRole::Retriever,
            AgentRole::KnowledgeAnalyst,
            AgentRole::ProcessAnalyst,
            AgentRole::CausalAnalyst,
            AgentRole::EvidenceAuditor,
            AgentRole::RuleValidator,
            AgentRole::ReportGenerator,
            AgentRole::FinalValidator,
        ]
    }

    /// 高風險結論必須由獨立驗證器確認（非同一生成步驟）。
    pub fn requires_independent_validation(role: AgentRole) -> bool {
        matches!(
            role,
            AgentRole::EvidenceAuditor | AgentRole::RuleValidator | AgentRole::FinalValidator
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ten_roles_defined() {
        assert_eq!(AgentFramework::all_roles().len(), 10);
    }

    #[test]
    fn role_names_are_unique() {
        let names: Vec<&str> = AgentFramework::all_roles()
            .iter()
            .map(|r| r.name())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn workflow_steps_advance_in_order() {
        let mut state = WorkflowState::new(10);
        assert_eq!(state.current_step, WorkflowStep::TaskAnalysis);
        assert!(state.advance());
        assert_eq!(state.current_step, WorkflowStep::Retrieval);
        assert!(state.advance());
        assert_eq!(state.current_step, WorkflowStep::ClaimBuilding);
    }

    #[test]
    fn workflow_stops_at_final_validation() {
        let mut state = WorkflowState::new(10);
        for _ in 0..7 {
            state.advance();
        }
        assert_eq!(state.current_step, WorkflowStep::FinalValidation);
        assert!(!state.advance());
    }

    #[test]
    fn iteration_limit_enforced() {
        let mut state = WorkflowState::new(1);
        state.advance();
        assert!(!state.advance());
        assert_eq!(state.stopping_reason, Some(StoppingReason::LimitReached));
    }

    #[test]
    fn stopping_requires_no_new_evidence() {
        let state = WorkflowState::new(10);
        // 有新證據 → 不停止
        assert!(!state.should_stop(true, false, false));
        // 有修改主張 → 不停止
        assert!(!state.should_stop(false, true, false));
        // 有重新評估規則 → 不停止
        assert!(!state.should_stop(false, false, true));
        // 三者皆無 → 停止
        assert!(state.should_stop(false, false, false));
    }

    #[test]
    fn independent_validation_required_for_auditors() {
        assert!(AgentFramework::requires_independent_validation(
            AgentRole::EvidenceAuditor
        ));
        assert!(AgentFramework::requires_independent_validation(
            AgentRole::RuleValidator
        ));
        assert!(AgentFramework::requires_independent_validation(
            AgentRole::FinalValidator
        ));
        assert!(!AgentFramework::requires_independent_validation(
            AgentRole::Planner
        ));
        assert!(!AgentFramework::requires_independent_validation(
            AgentRole::Retriever
        ));
    }

    #[test]
    fn all_roles_have_descriptions() {
        for role in AgentFramework::all_roles() {
            assert!(!role.description().is_empty());
        }
    }
}
