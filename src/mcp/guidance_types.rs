use serde::Serialize;
use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Serialize)]
pub(crate) struct Recommendation {
    pub(crate) project_id: String,
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_flow: Vec<String>,
    pub(crate) reason: String,
    pub(crate) confidence: String,
    pub(crate) strategy_mode: String,
    pub(crate) strategy_profile: Value,
    pub(crate) verification_gate_levels: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cleanup_blockers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) refactor_blockers: Option<Value>,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) suggested_commands: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ProjectOverview {
    pub(crate) project_id: String,
    pub(crate) status: String,
    pub(crate) snapshot_available: bool,
    pub(crate) activity_available: bool,
    pub(crate) unused_files: i64,
    pub(crate) observation: Value,
    pub(crate) repo_status_risk: Value,
    pub(crate) verification_evidence: Value,
    pub(crate) mock_data_summary: Value,
    pub(crate) storage_maintenance: Value,
    pub(crate) project_toolchain: Value,
    pub(crate) verification_safe_for_cleanup: Value,
    pub(crate) verification_safe_for_refactor: Value,
    pub(crate) verification_gate_levels: Value,
    pub(crate) safe_for_cleanup: Value,
    pub(crate) safe_for_cleanup_reason: Value,
    pub(crate) cleanup_blockers: Value,
    pub(crate) safe_for_refactor: Value,
    pub(crate) safe_for_refactor_reason: Value,
    pub(crate) refactor_blockers: Value,
    pub(crate) recommended_next_action: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) recommended_reason: Value,
    pub(crate) strategy_confidence: Value,
}

#[derive(Serialize)]
pub(crate) struct AttentionSummary {
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<String>,
    pub(crate) evidence_quality: String,
    pub(crate) priority_basis: AttentionPriorityBasis,
}

#[derive(Serialize)]
pub(crate) struct AttentionPriorityBasis {
    pub(crate) recommended_next_action: String,
    pub(crate) recommended_action_base: i64,
    pub(crate) repo_risk_level: String,
    pub(crate) repo_in_operation: bool,
    pub(crate) repo_is_dirty: bool,
    pub(crate) verification_status: String,
    pub(crate) has_failing_verification: bool,
    pub(crate) coverage_state: String,
    pub(crate) snapshot_freshness: String,
    pub(crate) activity_freshness: String,
    pub(crate) verification_freshness: String,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) safe_for_cleanup: bool,
    pub(crate) safe_for_refactor: bool,
}

#[derive(Serialize)]
pub(crate) struct WorkspacePortfolioLayer {
    pub(crate) status: String,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) monitored_projects: Vec<Value>,
    pub(crate) priority_candidates: Vec<Value>,
    pub(crate) project_overviews: Vec<Value>,
    pub(crate) priority_model: String,
    pub(crate) dirty_projects: usize,
    pub(crate) high_risk_projects: usize,
    pub(crate) projects_with_failing_verification: usize,
    pub(crate) projects_safe_for_cleanup: usize,
    pub(crate) projects_safe_for_refactor: usize,
    pub(crate) projects_with_hardcoded_candidates: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) total_mock_candidates: u64,
    pub(crate) total_hardcoded_candidates: u64,
    pub(crate) projects_in_operation: Vec<Value>,
    pub(crate) attention_queue: Vec<Value>,
    pub(crate) attention_batches: Value,
}

#[derive(Serialize)]
pub(crate) struct DecisionBrief {
    pub(crate) summary: String,
    pub(crate) recommended_next_action: String,
    pub(crate) reason: Value,
    pub(crate) repo_truth_gaps: Value,
    pub(crate) mandatory_shell_checks: Value,
    pub(crate) external_truth_boundary: Value,
    pub(crate) review_focus: Value,
    pub(crate) execution_sequence: Value,
    pub(crate) data_risk_focus: Value,
    pub(crate) target_project_id: Option<String>,
    pub(crate) strategy_mode: Value,
    pub(crate) preferred_primary_tool: Value,
    pub(crate) preferred_secondary_tool: Value,
    pub(crate) recommended_flow: Value,
    pub(crate) safe_for_cleanup: Option<bool>,
    pub(crate) safe_for_refactor: Option<bool>,
    pub(crate) verification_status: String,
    pub(crate) requires_verification: bool,
    pub(crate) action_profile: Value,
    pub(crate) risk_profile: Value,
    pub(crate) signals: DecisionSignals,
}

#[derive(Serialize)]
pub(crate) struct DecisionSignals {
    pub(crate) repo_risk_level: String,
    pub(crate) repo_is_dirty: bool,
    pub(crate) hardcoded_candidate_count: u64,
    pub(crate) mock_candidate_count: u64,
    pub(crate) mixed_review_file_count: u64,
    pub(crate) storage_maintenance_candidate: bool,
    pub(crate) storage_vacuum_candidate: bool,
    pub(crate) storage_reclaimable_bytes: i64,
    pub(crate) storage_db_size_bytes: i64,
    pub(crate) attention_score: i64,
    pub(crate) attention_band: String,
    pub(crate) attention_reasons: Vec<Value>,
    pub(crate) monitoring_count: u64,
}

#[derive(Serialize)]
pub(crate) struct RepoTruthSummary {
    pub(crate) projects_with_repo_truth_gaps: u64,
    pub(crate) repo_truth_gap_distribution: RepoTruthGapDistribution,
    pub(crate) mandatory_shell_check_examples: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct RepoTruthGapDistribution {
    counts: BTreeMap<String, u64>,
}

impl RepoTruthGapDistribution {
    pub(crate) fn increment_gap(&mut self, gap_key: &str) {
        *self.counts.entry(gap_key.to_string()).or_insert(0) += 1;
    }

    #[cfg(test)]
    pub(crate) fn count(&self, gap_key: &str) -> u64 {
        self.counts.get(gap_key).copied().unwrap_or(0)
    }

    pub(crate) fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RepoRiskCouplingStatus {
    NoRepoRiskSignal,
    Coupled,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct RepoRiskCoupling {
    status: RepoRiskCouplingStatus,
    source: Option<String>,
    source_project_id: Option<String>,
    recommended_next_action: Value,
    strategy_mode: Value,
    preferred_primary_tool: Value,
    primary_repo_risk_finding: Value,
    summary: Option<String>,
}

impl RepoRiskCoupling {
    pub(crate) fn no_signal(
        recommended_next_action: Value,
        strategy_mode: Value,
        preferred_primary_tool: Value,
    ) -> Self {
        Self {
            status: RepoRiskCouplingStatus::NoRepoRiskSignal,
            source: None,
            source_project_id: None,
            recommended_next_action,
            strategy_mode,
            preferred_primary_tool,
            primary_repo_risk_finding: Value::Null,
            summary: None,
        }
    }

    pub(crate) fn coupled(
        source_project_id: &str,
        recommended_next_action: Value,
        strategy_mode: Value,
        preferred_primary_tool: Value,
        primary_repo_risk_finding: Value,
        summary: String,
    ) -> Self {
        Self {
            status: RepoRiskCouplingStatus::Coupled,
            source: Some("primary_repo_risk_finding".to_string()),
            source_project_id: Some(source_project_id.to_string()),
            recommended_next_action,
            strategy_mode,
            preferred_primary_tool,
            primary_repo_risk_finding,
            summary: Some(summary),
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Serialize)]
pub(crate) struct StabilizationSummary {
    pub(crate) projects_requiring_repo_stabilization: u64,
    pub(crate) repo_stabilization_priority_projects: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct VerificationSummary {
    pub(crate) projects_requiring_verification_run: u64,
    pub(crate) projects_requiring_failing_verification_repair: u64,
}

#[derive(Serialize)]
pub(crate) struct ObservationSummary {
    pub(crate) projects_requiring_monitor_start: u64,
    pub(crate) projects_requiring_snapshot_refresh: u64,
    pub(crate) projects_requiring_activity_generation: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct DataRiskFocusDistribution {
    pub(crate) hardcoded: u64,
    pub(crate) mixed: u64,
    pub(crate) mock: u64,
    pub(crate) none: u64,
}

impl DataRiskFocusDistribution {
    pub(crate) fn increment_focus(&mut self, focus: &str) {
        match focus {
            "hardcoded" => self.hardcoded += 1,
            "mixed" => self.mixed += 1,
            "mock" => self.mock += 1,
            _ => self.none += 1,
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Serialize)]
pub(crate) struct DataRiskFocusSummary {
    pub(crate) data_risk_focus_distribution: DataRiskFocusDistribution,
    pub(crate) projects_requiring_hardcoded_review: u64,
    pub(crate) projects_requiring_mock_review: u64,
    pub(crate) projects_requiring_mixed_file_review: u64,
}

#[derive(Serialize)]
pub(crate) struct WorkspaceObservationLayer {
    pub(crate) status: String,
    pub(crate) project_count: usize,
    pub(crate) monitoring_count: usize,
    pub(crate) analysis_state: String,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) total_storage_reclaimable_bytes: Value,
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: Value,
    pub(crate) projects_requiring_mock_review: Value,
    pub(crate) projects_requiring_mixed_file_review: Value,
    pub(crate) notes: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ExecutionStrategyLayer {
    pub(crate) status: String,
    pub(crate) recommended_flow: Value,
    pub(crate) project_recommendations: Vec<Value>,
    pub(crate) global_strategy_mode: Value,
    pub(crate) preferred_primary_tool: Value,
    pub(crate) preferred_secondary_tool: Value,
    pub(crate) evidence_priority: Value,
    pub(crate) risk_strategy_coupling: RepoRiskCoupling,
    pub(crate) external_truth_boundary: Value,
    pub(crate) review_focus_projection: Value,
    pub(crate) when_to_use_opendog: Vec<&'static str>,
    pub(crate) when_to_use_shell: Vec<&'static str>,
    pub(crate) guardrails: Vec<&'static str>,
    pub(crate) projects_not_ready_for_cleanup: usize,
    pub(crate) projects_not_ready_for_refactor: usize,
    pub(crate) projects_with_hardcoded_data_candidates: usize,
    pub(crate) projects_missing_snapshot: usize,
    pub(crate) projects_with_stale_snapshot: usize,
    pub(crate) projects_missing_activity: usize,
    pub(crate) projects_with_stale_activity: usize,
    pub(crate) projects_missing_verification: usize,
    pub(crate) projects_with_stale_verification: usize,
    pub(crate) projects_with_storage_maintenance_candidates: u64,
    pub(crate) projects_with_vacuum_candidates: u64,
    pub(crate) review_opendog_retention_before_large_cleanup: bool,
    pub(crate) recommend_manual_review_for_hardcoded_data: bool,
    pub(crate) data_risk_focus_distribution: Value,
    pub(crate) projects_requiring_hardcoded_review: Value,
    pub(crate) projects_requiring_mock_review: Value,
    pub(crate) projects_requiring_mixed_file_review: Value,
    pub(crate) projects_requiring_monitor_start: Value,
    pub(crate) projects_requiring_snapshot_refresh: Value,
    pub(crate) projects_requiring_activity_generation: Value,
    pub(crate) projects_with_repo_truth_gaps: Value,
    pub(crate) repo_truth_gap_distribution: Value,
    pub(crate) mandatory_shell_check_examples: Value,
    pub(crate) projects_requiring_verification_run: Value,
    pub(crate) projects_requiring_failing_verification_repair: Value,
    pub(crate) projects_requiring_repo_stabilization: Value,
    pub(crate) repo_stabilization_priority_projects: Value,
}

#[derive(Serialize)]
pub(crate) struct ConstraintsBoundariesLayer {
    pub(crate) status: String,
    pub(crate) direct_observations: Vec<String>,
    pub(crate) inferences: Vec<String>,
    pub(crate) blind_spots: Vec<String>,
    pub(crate) guardrails: Vec<String>,
    pub(crate) destructive_operations_requiring_confirmation: Vec<String>,
    pub(crate) human_review_required_for: Vec<String>,
    pub(crate) cleanup_blockers: Vec<String>,
    pub(crate) refactor_blockers: Vec<String>,
    pub(crate) requires_shell_verification: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_cleanup: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_not_ready_for_refactor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_hardcoded_data_candidates: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_snapshot: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_activity: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_missing_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_stale_verification: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) projects_with_storage_maintenance_candidates: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recommendation_serializes_all_fields() {
        let rec = Recommendation {
            project_id: "proj".into(),
            recommended_next_action: "take_snapshot".into(),
            recommended_flow: vec!["step1".into()],
            reason: "reason".into(),
            confidence: "high".into(),
            strategy_mode: "evidence_first".into(),
            strategy_profile: json!({"mode": "evidence_first"}),
            verification_gate_levels: json!({}),
            cleanup_blockers: Some(json!(["blocker1"])),
            refactor_blockers: Some(json!(["blocker2"])),
            repo_truth_gaps: json!({}),
            mandatory_shell_checks: json!({}),
            suggested_commands: vec!["cargo test".into()],
        };
        let v = serde_json::to_value(&rec).unwrap();
        assert_eq!(v["project_id"], "proj");
        assert!(v.get("cleanup_blockers").is_some());
        assert!(v.get("refactor_blockers").is_some());
    }

    #[test]
    fn recommendation_skips_none_optionals() {
        let rec = Recommendation {
            project_id: "p".into(),
            recommended_next_action: "a".into(),
            recommended_flow: vec![],
            reason: "r".into(),
            confidence: "c".into(),
            strategy_mode: "s".into(),
            strategy_profile: json!(null),
            verification_gate_levels: json!(null),
            cleanup_blockers: None,
            refactor_blockers: None,
            repo_truth_gaps: json!(null),
            mandatory_shell_checks: json!(null),
            suggested_commands: vec![],
        };
        let v = serde_json::to_value(&rec).unwrap();
        assert!(v.get("cleanup_blockers").is_none());
        assert!(v.get("refactor_blockers").is_none());
    }

    #[test]
    fn project_overview_serializes() {
        let po = ProjectOverview {
            project_id: "x".into(),
            status: "active".into(),
            snapshot_available: true,
            activity_available: false,
            unused_files: 5,
            observation: json!({}),
            repo_status_risk: json!({}),
            verification_evidence: json!({}),
            mock_data_summary: json!({}),
            storage_maintenance: json!({}),
            project_toolchain: json!({}),
            verification_safe_for_cleanup: json!(true),
            verification_safe_for_refactor: json!(true),
            verification_gate_levels: json!({}),
            safe_for_cleanup: json!(true),
            safe_for_cleanup_reason: json!("ok"),
            cleanup_blockers: json!([]),
            safe_for_refactor: json!(true),
            safe_for_refactor_reason: json!("ok"),
            refactor_blockers: json!([]),
            recommended_next_action: json!("none"),
            recommended_flow: json!([]),
            recommended_reason: json!(""),
            strategy_confidence: json!("high"),
        };
        let v = serde_json::to_value(&po).unwrap();
        assert_eq!(v["project_id"], "x");
        assert!(v["snapshot_available"].as_bool().unwrap());
    }

    #[test]
    fn attention_summary_serializes() {
        let a = AttentionSummary {
            attention_score: 42,
            attention_band: "high".into(),
            attention_reasons: vec!["reason1".into()],
            evidence_quality: "good".into(),
            priority_basis: AttentionPriorityBasis {
                recommended_next_action: "stabilize".into(),
                recommended_action_base: 10,
                repo_risk_level: "medium".into(),
                repo_in_operation: false,
                repo_is_dirty: true,
                verification_status: "passed".into(),
                has_failing_verification: false,
                coverage_state: "partial".into(),
                snapshot_freshness: "fresh".into(),
                activity_freshness: "fresh".into(),
                verification_freshness: "stale".into(),
                hardcoded_candidate_count: 3,
                mock_candidate_count: 1,
                safe_for_cleanup: false,
                safe_for_refactor: false,
            },
        };
        let v = serde_json::to_value(&a).unwrap();
        assert_eq!(v["attention_score"], 42);
        assert!(v["priority_basis"]["repo_is_dirty"].as_bool().unwrap());
    }

    #[test]
    fn workspace_portfolio_layer_serializes() {
        let w = WorkspacePortfolioLayer {
            status: "available".into(),
            project_count: 2,
            monitoring_count: 1,
            monitored_projects: vec![json!("p1")],
            priority_candidates: vec![],
            project_overviews: vec![],
            priority_model: "attention".into(),
            dirty_projects: 0,
            high_risk_projects: 1,
            projects_with_failing_verification: 0,
            projects_safe_for_cleanup: 1,
            projects_safe_for_refactor: 1,
            projects_with_hardcoded_candidates: 0,
            projects_with_hardcoded_data_candidates: 0,
            total_mock_candidates: 0,
            total_hardcoded_candidates: 0,
            projects_in_operation: vec![],
            attention_queue: vec![],
            attention_batches: json!({}),
        };
        let v = serde_json::to_value(&w).unwrap();
        assert_eq!(v["project_count"], 2);
        assert_eq!(v["priority_model"], "attention");
    }

    #[test]
    fn decision_brief_serializes_with_options() {
        let d = DecisionBrief {
            summary: "test".into(),
            recommended_next_action: "act".into(),
            reason: json!("reason"),
            repo_truth_gaps: json!([]),
            mandatory_shell_checks: json!([]),
            external_truth_boundary: json!(null),
            review_focus: json!(null),
            execution_sequence: json!({}),
            data_risk_focus: json!("none"),
            target_project_id: Some("proj".into()),
            strategy_mode: json!("evidence"),
            preferred_primary_tool: json!("opendog"),
            preferred_secondary_tool: json!("shell"),
            recommended_flow: json!([]),
            safe_for_cleanup: Some(true),
            safe_for_refactor: Some(false),
            verification_status: "passed".into(),
            requires_verification: false,
            action_profile: json!({}),
            risk_profile: json!({}),
            signals: DecisionSignals {
                repo_risk_level: "low".into(),
                repo_is_dirty: false,
                hardcoded_candidate_count: 0,
                mock_candidate_count: 0,
                mixed_review_file_count: 0,
                storage_maintenance_candidate: false,
                storage_vacuum_candidate: false,
                storage_reclaimable_bytes: 0,
                storage_db_size_bytes: 1024,
                attention_score: 10,
                attention_band: "low".into(),
                attention_reasons: vec![],
                monitoring_count: 1,
            },
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["target_project_id"], "proj");
        assert!(v["safe_for_cleanup"].as_bool().unwrap());
        assert!(!v["safe_for_refactor"].as_bool().unwrap());
    }

    #[test]
    fn decision_brief_skips_none_options() {
        let d = DecisionBrief {
            summary: "s".into(),
            recommended_next_action: "a".into(),
            reason: json!(null),
            repo_truth_gaps: json!(null),
            mandatory_shell_checks: json!(null),
            external_truth_boundary: json!(null),
            review_focus: json!(null),
            execution_sequence: json!(null),
            data_risk_focus: json!(null),
            target_project_id: None,
            strategy_mode: json!(null),
            preferred_primary_tool: json!(null),
            preferred_secondary_tool: json!(null),
            recommended_flow: json!(null),
            safe_for_cleanup: None,
            safe_for_refactor: None,
            verification_status: "unknown".into(),
            requires_verification: false,
            action_profile: json!(null),
            risk_profile: json!(null),
            signals: DecisionSignals {
                repo_risk_level: "low".into(),
                repo_is_dirty: false,
                hardcoded_candidate_count: 0,
                mock_candidate_count: 0,
                mixed_review_file_count: 0,
                storage_maintenance_candidate: false,
                storage_vacuum_candidate: false,
                storage_reclaimable_bytes: 0,
                storage_db_size_bytes: 0,
                attention_score: 0,
                attention_band: "low".into(),
                attention_reasons: vec![],
                monitoring_count: 0,
            },
        };
        let v = serde_json::to_value(&d).unwrap();
        assert!(v["target_project_id"].is_null());
        assert!(v["safe_for_cleanup"].is_null());
        assert!(v["safe_for_refactor"].is_null());
    }

    #[test]
    fn decision_signals_serializes() {
        let s = DecisionSignals {
            repo_risk_level: "high".into(),
            repo_is_dirty: true,
            hardcoded_candidate_count: 5,
            mock_candidate_count: 2,
            mixed_review_file_count: 1,
            storage_maintenance_candidate: true,
            storage_vacuum_candidate: false,
            storage_reclaimable_bytes: 4096,
            storage_db_size_bytes: 8192,
            attention_score: 80,
            attention_band: "critical".into(),
            attention_reasons: vec![json!("r1")],
            monitoring_count: 3,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["hardcoded_candidate_count"], 5);
        assert!(v["repo_is_dirty"].as_bool().unwrap());
    }

    #[test]
    fn repo_truth_summary_serializes() {
        let mut distribution = RepoTruthGapDistribution::default();
        distribution.increment_gap("missing_test");
        distribution.increment_gap("missing_test");

        let s = RepoTruthSummary {
            projects_with_repo_truth_gaps: 2,
            repo_truth_gap_distribution: distribution,
            mandatory_shell_check_examples: vec!["cargo test".into()],
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["projects_with_repo_truth_gaps"], 2);
        assert_eq!(v["repo_truth_gap_distribution"]["missing_test"], 2);
    }

    #[test]
    fn repo_truth_gap_distribution_counts_dynamic_keys() {
        let mut distribution = RepoTruthGapDistribution::default();

        distribution.increment_gap("missing_test");
        distribution.increment_gap("missing_test");
        distribution.increment_gap("missing_lint");

        assert_eq!(distribution.count("missing_test"), 2);
        assert_eq!(distribution.count("missing_lint"), 1);
        assert_eq!(distribution.count("missing_build"), 0);

        let v = serde_json::to_value(&distribution).unwrap();
        assert_eq!(v["missing_test"], 2);
        assert_eq!(v["missing_lint"], 1);
    }

    #[test]
    fn repo_risk_coupling_no_signal_serializes_null_boundaries() {
        let coupling = RepoRiskCoupling::no_signal(
            json!("start_monitor"),
            json!("defensive"),
            json!("opendog"),
        );

        let v = serde_json::to_value(&coupling).unwrap();
        assert_eq!(v["status"], "no_repo_risk_signal");
        assert!(v["source"].is_null());
        assert!(v["source_project_id"].is_null());
        assert_eq!(v["recommended_next_action"], "start_monitor");
        assert_eq!(v["strategy_mode"], "defensive");
        assert_eq!(v["preferred_primary_tool"], "opendog");
        assert!(v["primary_repo_risk_finding"].is_null());
        assert!(v["summary"].is_null());
    }

    #[test]
    fn repo_risk_coupling_coupled_serializes_context() {
        let coupling = RepoRiskCoupling::coupled(
            "proj_a",
            json!("stabilize_repository_state"),
            json!("stabilize_first"),
            json!("shell_verification"),
            json!({"summary": "merge in progress"}),
            "Top repository risk keeps the workspace in stabilize_first mode.".to_string(),
        );

        let v = serde_json::to_value(&coupling).unwrap();
        assert_eq!(v["status"], "coupled");
        assert_eq!(v["source"], "primary_repo_risk_finding");
        assert_eq!(v["source_project_id"], "proj_a");
        assert_eq!(
            v["primary_repo_risk_finding"]["summary"],
            "merge in progress"
        );
        assert_eq!(
            v["summary"],
            "Top repository risk keeps the workspace in stabilize_first mode."
        );
    }

    #[test]
    fn stabilization_summary_serializes() {
        let s = StabilizationSummary {
            projects_requiring_repo_stabilization: 1,
            repo_stabilization_priority_projects: vec!["proj1".into()],
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["projects_requiring_repo_stabilization"], 1);
    }

    #[test]
    fn verification_summary_serializes() {
        let s = VerificationSummary {
            projects_requiring_verification_run: 3,
            projects_requiring_failing_verification_repair: 1,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["projects_requiring_verification_run"], 3);
    }

    #[test]
    fn observation_summary_serializes() {
        let s = ObservationSummary {
            projects_requiring_monitor_start: 1,
            projects_requiring_snapshot_refresh: 2,
            projects_requiring_activity_generation: 0,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["projects_requiring_snapshot_refresh"], 2);
    }

    #[test]
    fn data_risk_focus_summary_serializes() {
        let mut distribution = DataRiskFocusDistribution::default();
        distribution.increment_focus("hardcoded");
        distribution.increment_focus("none");
        distribution.increment_focus("none");

        let s = DataRiskFocusSummary {
            data_risk_focus_distribution: distribution,
            projects_requiring_hardcoded_review: 1,
            projects_requiring_mock_review: 0,
            projects_requiring_mixed_file_review: 0,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["projects_requiring_hardcoded_review"], 1);
        assert_eq!(v["data_risk_focus_distribution"]["none"], 2);
    }

    #[test]
    fn data_risk_focus_distribution_counts_known_and_unknown_focuses() {
        let mut distribution = DataRiskFocusDistribution::default();

        distribution.increment_focus("hardcoded");
        distribution.increment_focus("mock");
        distribution.increment_focus("mixed");
        distribution.increment_focus("unexpected");

        let v = serde_json::to_value(&distribution).unwrap();
        assert_eq!(v["hardcoded"], 1);
        assert_eq!(v["mock"], 1);
        assert_eq!(v["mixed"], 1);
        assert_eq!(v["none"], 1);
    }

    #[test]
    fn workspace_observation_layer_serializes() {
        let w = WorkspaceObservationLayer {
            status: "available".into(),
            project_count: 3,
            monitoring_count: 2,
            analysis_state: "ready".into(),
            projects_missing_snapshot: 0,
            projects_with_stale_snapshot: 1,
            projects_missing_activity: 0,
            projects_with_stale_activity: 0,
            projects_missing_verification: 1,
            projects_with_stale_verification: 0,
            projects_with_storage_maintenance_candidates: 0,
            projects_with_vacuum_candidates: 0,
            total_storage_reclaimable_bytes: json!(0),
            data_risk_focus_distribution: json!({}),
            projects_requiring_hardcoded_review: json!(0),
            projects_requiring_mock_review: json!(0),
            projects_requiring_mixed_file_review: json!(0),
            notes: vec!["note1".into()],
        };
        let v = serde_json::to_value(&w).unwrap();
        assert_eq!(v["project_count"], 3);
        assert_eq!(v["analysis_state"], "ready");
    }

    #[test]
    fn execution_strategy_layer_serializes() {
        let e = ExecutionStrategyLayer {
            status: "available".into(),
            recommended_flow: json!([]),
            project_recommendations: vec![],
            global_strategy_mode: json!("evidence_first"),
            preferred_primary_tool: json!("opendog"),
            preferred_secondary_tool: json!("shell"),
            evidence_priority: json!("high"),
            risk_strategy_coupling: RepoRiskCoupling::no_signal(
                Value::Null,
                json!("evidence_first"),
                json!("opendog"),
            ),
            external_truth_boundary: json!({}),
            review_focus_projection: json!({}),
            when_to_use_opendog: vec![],
            when_to_use_shell: vec![],
            guardrails: vec![],
            projects_not_ready_for_cleanup: 0,
            projects_not_ready_for_refactor: 0,
            projects_with_hardcoded_data_candidates: 0,
            projects_missing_snapshot: 0,
            projects_with_stale_snapshot: 0,
            projects_missing_activity: 0,
            projects_with_stale_activity: 0,
            projects_missing_verification: 0,
            projects_with_stale_verification: 0,
            projects_with_storage_maintenance_candidates: 0,
            projects_with_vacuum_candidates: 0,
            review_opendog_retention_before_large_cleanup: false,
            recommend_manual_review_for_hardcoded_data: false,
            data_risk_focus_distribution: json!({}),
            projects_requiring_hardcoded_review: json!(0),
            projects_requiring_mock_review: json!(0),
            projects_requiring_mixed_file_review: json!(0),
            projects_requiring_monitor_start: json!(0),
            projects_requiring_snapshot_refresh: json!(0),
            projects_requiring_activity_generation: json!(0),
            projects_with_repo_truth_gaps: json!(0),
            repo_truth_gap_distribution: json!({}),
            mandatory_shell_check_examples: json!([]),
            projects_requiring_verification_run: json!(0),
            projects_requiring_failing_verification_repair: json!(0),
            projects_requiring_repo_stabilization: json!(0),
            repo_stabilization_priority_projects: json!([]),
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["status"], "available");
    }

    #[test]
    fn constraints_boundaries_layer_skips_none_optionals() {
        let c = ConstraintsBoundariesLayer {
            status: "available".into(),
            direct_observations: vec![],
            inferences: vec![],
            blind_spots: vec![],
            guardrails: vec![],
            destructive_operations_requiring_confirmation: vec![],
            human_review_required_for: vec![],
            cleanup_blockers: vec![],
            refactor_blockers: vec![],
            requires_shell_verification: vec![],
            projects_not_ready_for_cleanup: None,
            projects_not_ready_for_refactor: None,
            projects_with_hardcoded_data_candidates: None,
            projects_missing_snapshot: None,
            projects_with_stale_snapshot: None,
            projects_missing_activity: None,
            projects_with_stale_activity: None,
            projects_missing_verification: None,
            projects_with_stale_verification: None,
            projects_with_storage_maintenance_candidates: None,
        };
        let v = serde_json::to_value(&c).unwrap();
        assert!(v.get("projects_not_ready_for_cleanup").is_none());
        assert!(v
            .get("projects_with_storage_maintenance_candidates")
            .is_none());
    }

    #[test]
    fn constraints_boundaries_layer_includes_some_optionals() {
        let c = ConstraintsBoundariesLayer {
            status: "available".into(),
            direct_observations: vec![],
            inferences: vec![],
            blind_spots: vec![],
            guardrails: vec![],
            destructive_operations_requiring_confirmation: vec![],
            human_review_required_for: vec![],
            cleanup_blockers: vec![],
            refactor_blockers: vec![],
            requires_shell_verification: vec![],
            projects_not_ready_for_cleanup: Some(2),
            projects_not_ready_for_refactor: Some(1),
            projects_with_hardcoded_data_candidates: Some(3),
            projects_missing_snapshot: Some(0),
            projects_with_stale_snapshot: Some(1),
            projects_missing_activity: Some(0),
            projects_with_stale_activity: Some(0),
            projects_missing_verification: Some(1),
            projects_with_stale_verification: Some(0),
            projects_with_storage_maintenance_candidates: Some(5),
        };
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["projects_not_ready_for_cleanup"], 2);
        assert_eq!(v["projects_with_storage_maintenance_candidates"], 5);
    }
}
