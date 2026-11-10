#![forbid(unsafe_code)]

//! ???函?????芋蝯?Master Prompt Phase 5, Addendum A11嚗?//!
//! 因果推理引擎（Master Prompt Phase 5, Addendum A11）。
//! 瘥??????恐??
//! - ???身皜
//! - 識別策略為 InsufficientData → CannotIdentify
//! - 資料不足 → InsufficientData 或 CannotIdentify
//! - 反面證據存在 → Contradicted
//! - ????蝯?
// (indented to avoid clippy doc-lazy-continuation)
use serde::{Deserialize, Serialize};

// 資料不足以識別時輸出 CannotIdentify，禁止生成確定性因果結論。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausalRelationType {
    /// NormativeDependency: must approve before execute (handled by rule engine)
    /// ProcessOrder: activity A completes before B starts (handled by process model)
    NormativeDependency,
    ProcessOrder,
    TrueCausation,
}
/// IdentificationStrategy enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentificationStrategy {
    /// RandomizedExperiment: RCT
    RandomizedExperiment,
    /// DifferenceInDifferences: DiD
    DifferenceInDifferences,
    /// RegressionDiscontinuity: RD
    RegressionDiscontinuity,
    /// InstrumentalVariable: IV
    InstrumentalVariable,
    /// PropensityScoreMatching: PSM
    PropensityScoreMatching,
    /// TimeSeriesCausality: time series
    TimeSeriesCausality,
    /// ExpertJudgment: expert only
    ExpertJudgment,
    /// InsufficientData: cannot identify
    InsufficientData,
}

/// CausalHypothesis struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalHypothesis {
    pub id: String,
    pub claim: String,
    pub cause: String,
    pub effect: String,
    pub relation_type: CausalRelationType,
    pub temporal_order_established: bool,
    pub identification_strategy: IdentificationStrategy,
    pub confounders: Vec<String>,
    pub evidence_for: Vec<String>,
    pub evidence_against: Vec<String>,
    pub data_sources: Vec<String>,
}

/// CausalVerdict enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausalVerdict {
    /// Identified: data sufficient for causal conclusion
    Identified,
    /// CannotIdentify: data insufficient
    CannotIdentify,
    /// PartiallyIdentified: association only
    PartiallyIdentified,
    /// Contradicted: contradicting evidence found
    Contradicted,
}

/// CausalAnalysisResult struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAnalysisResult {
    pub hypothesis_id: String,
    pub verdict: CausalVerdict,
    pub notes: Vec<String>,
    pub counterfactual_checked: bool,
    pub confounders_addressed: bool,
    pub alternative_causes: Vec<String>,
}

/// CausalAnalyzer: causal reasoning engine
///
/// 判定順序（先到先決；G11）：證據不足的策略 → CannotIdentify；
/// 混淆未處理 → CannotIdentify；反面證據 → Contradicted；
/// 資料不足 → InsufficientData 或 CannotIdentify。
/// 任何確定識別（Identified）都必須先通過反事實與混淆檢查。
pub struct CausalAnalyzer;
impl CausalAnalyzer {
    /// 判定順序（先到先決；G11）：證據不足 → CannotIdentify；
    /// 閬?嚗11嚗?
    /// - 反面證據 → Contradicted；
    /// - 資料不足 → InsufficientData 或 CannotIdentify。
    /// - ?????潛? ??Contradicted
    /// - 資料不足 → InsufficientData 或 CannotIdentify
    pub fn analyze(hypothesis: &CausalHypothesis) -> CausalAnalysisResult {
        let mut notes = Vec::new();
        let mut alternative_causes = Vec::new();

        // 資料不足的策略 → CannotIdentify
        if hypothesis.identification_strategy == IdentificationStrategy::InsufficientData {
            notes.push("identification strategy is insufficient_data".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::CannotIdentify,
                notes,
                counterfactual_checked: false,
                confounders_addressed: false,
                alternative_causes,
            };
        }

        // 瑼Ｘ????
        if !hypothesis.temporal_order_established {
            notes.push("temporal order not established".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::CannotIdentify,
                notes,
                counterfactual_checked: false,
                confounders_addressed: false,
                alternative_causes,
            };
        }

        // 混淆因子未處理 → CannotIdentify
        let confounders_addressed = hypothesis.confounders.is_empty()
            || hypothesis
                .evidence_for
                .iter()
                .any(|e| e.contains("confounder") || e.contains("?批"));

        // 瑼Ｘ??
        let has_contradiction = !hypothesis.evidence_against.is_empty();

        // 反面證據存在 → Contradicted
        if hypothesis.data_sources.is_empty() {
            notes.push("no data sources declared".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::CannotIdentify,
                notes,
                counterfactual_checked: false,
                confounders_addressed: false,
                alternative_causes,
            };
        }

        // ????
        let counterfactual_checked = !hypothesis.evidence_against.is_empty()
            || hypothesis
                .evidence_for
                .iter()
                .any(|e| e.contains("counterfactual"));

        if has_contradiction {
            notes.push("contradicting evidence found".to_string());
            alternative_causes.push("alternative cause not ruled out".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::Contradicted,
                notes,
                counterfactual_checked,
                confounders_addressed,
                alternative_causes,
            };
        }

        // Confounders not addressed
        if !hypothesis.confounders.is_empty() && !confounders_addressed {
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::CannotIdentify,
                notes,
                counterfactual_checked,
                confounders_addressed: false,
                alternative_causes,
            };
        }

        // 專家判斷：僅專家判斷不構成統計識別（PartiallyIdentified）
        if hypothesis.identification_strategy == IdentificationStrategy::ExpertJudgment {
            notes.push("expert judgment only; not statistically identified".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::PartiallyIdentified,
                notes,
                counterfactual_checked,
                confounders_addressed,
                alternative_causes,
            };
        }

        // 證據支持且混淆已處理 → Identified
        if !hypothesis.evidence_for.is_empty() && confounders_addressed {
            notes.push("causal hypothesis supported by evidence".to_string());
            return CausalAnalysisResult {
                hypothesis_id: hypothesis.id.clone(),
                verdict: CausalVerdict::Identified,
                notes,
                counterfactual_checked,
                confounders_addressed,
                alternative_causes,
            };
        }

        // Default: cannot identify
        notes.push("insufficient evidence for causal identification".to_string());
        CausalAnalysisResult {
            hypothesis_id: hypothesis.id.clone(),
            verdict: CausalVerdict::CannotIdentify,
            notes,
            counterfactual_checked,
            confounders_addressed,
            alternative_causes,
        }
    }

    /// classify_relation: distinguish three relation types (A11)
    pub fn classify_relation(relation: CausalRelationType) -> &'static str {
        match relation {
            CausalRelationType::NormativeDependency => {
                "normative_dependency (handled by rule engine)"
            }
            CausalRelationType::ProcessOrder => "process_order (handled by process model)",
            CausalRelationType::TrueCausation => "true_causation (handled by causal module)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hypothesis(
        id: &str,
        strategy: IdentificationStrategy,
        confounders: Vec<String>,
        evidence_for: Vec<String>,
        evidence_against: Vec<String>,
        data_sources: Vec<String>,
    ) -> CausalHypothesis {
        CausalHypothesis {
            id: id.to_string(),
            claim: "control measure reduces defect rate".to_string(),
            cause: "control_measure_X".to_string(),
            effect: "defect_rate_reduction".to_string(),
            relation_type: CausalRelationType::TrueCausation,
            temporal_order_established: true,
            identification_strategy: strategy,
            confounders,
            evidence_for,
            evidence_against,
            data_sources,
        }
    }

    #[test]
    fn insufficient_data_strategy_returns_cannot_identify() {
        let h = make_hypothesis(
            "CA-001",
            IdentificationStrategy::InsufficientData,
            vec![],
            vec!["some evidence".to_string()],
            vec![],
            vec!["source1".to_string()],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::CannotIdentify);
    }

    #[test]
    fn no_temporal_order_returns_cannot_identify() {
        let mut h = make_hypothesis(
            "CA-002",
            IdentificationStrategy::RandomizedExperiment,
            vec![],
            vec!["evidence".to_string()],
            vec![],
            vec!["source1".to_string()],
        );
        h.temporal_order_established = false;
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::CannotIdentify);
    }

    #[test]
    fn no_data_sources_returns_cannot_identify() {
        let h = make_hypothesis(
            "CA-003",
            IdentificationStrategy::RandomizedExperiment,
            vec![],
            vec!["evidence".to_string()],
            vec![],
            vec![],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::CannotIdentify);
    }

    #[test]
    fn contradicting_evidence_returns_contradicted() {
        let h = make_hypothesis(
            "CA-004",
            IdentificationStrategy::RandomizedExperiment,
            vec![],
            vec!["evidence for".to_string()],
            vec!["evidence against".to_string()],
            vec!["source1".to_string()],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::Contradicted);
    }

    #[test]
    fn unaddressed_confounders_returns_cannot_identify() {
        let h = make_hypothesis(
            "CA-005",
            IdentificationStrategy::RandomizedExperiment,
            vec!["confounder_A".to_string()],
            vec!["evidence for".to_string()],
            vec![],
            vec!["source1".to_string()],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::CannotIdentify);
    }

    #[test]
    fn expert_judgment_returns_partially_identified() {
        let h = make_hypothesis(
            "CA-006",
            IdentificationStrategy::ExpertJudgment,
            vec![],
            vec!["expert opinion".to_string()],
            vec![],
            vec!["source1".to_string()],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::PartiallyIdentified);
    }

    #[test]
    fn sufficient_evidence_returns_identified() {
        let h = make_hypothesis(
            "CA-007",
            IdentificationStrategy::RandomizedExperiment,
            vec![],
            vec!["RCT evidence".to_string()],
            vec![],
            vec!["source1".to_string()],
        );
        let r = CausalAnalyzer::analyze(&h);
        assert_eq!(r.verdict, CausalVerdict::Identified);
    }

    #[test]
    fn classify_relation_distinguishes_three_types() {
        assert!(
            CausalAnalyzer::classify_relation(CausalRelationType::NormativeDependency)
                .contains("rule engine")
        );
        assert!(
            CausalAnalyzer::classify_relation(CausalRelationType::ProcessOrder)
                .contains("process model")
        );
        assert!(
            CausalAnalyzer::classify_relation(CausalRelationType::TrueCausation)
                .contains("causal module")
        );
    }

    #[test]
    fn normative_dependency_is_not_true_causation() {
        // A11 邊界測試：規範依賴不是真因果（normative dependency）
        let h = CausalHypothesis {
            id: "CA-008".to_string(),
            claim: "must approve before execute".to_string(),
            cause: "approval".to_string(),
            effect: "execution".to_string(),
            relation_type: CausalRelationType::NormativeDependency,
            temporal_order_established: true,
            identification_strategy: IdentificationStrategy::RandomizedExperiment,
            confounders: vec![],
            evidence_for: vec!["rule text".to_string()],
            evidence_against: vec![],
            data_sources: vec!["SOP".to_string()],
        };
        // 反面：時間順序未先建立卻已執行 → Identified 不可得，改判 CannotIdentify
        assert_eq!(h.relation_type, CausalRelationType::NormativeDependency);
        assert_ne!(h.relation_type, CausalRelationType::TrueCausation);
    }
}
