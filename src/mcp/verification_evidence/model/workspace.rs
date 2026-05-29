use serde_json::{json, Value};

use super::{project_gate_level, GateDistribution};

#[derive(Debug, Clone, PartialEq)]
struct VerificationEvidenceProjectSummary {
    project_id: Option<String>,
    verification_status: Option<String>,
    verification_freshness: Value,
    freshness_status: Option<String>,
    failing_run_count: usize,
    safe_for_cleanup: bool,
    safe_for_refactor: bool,
    cleanup_gate_level: String,
    refactor_gate_level: String,
    cleanup_reason: String,
    refactor_reason: String,
}

impl VerificationEvidenceProjectSummary {
    fn from_project_overview(project: &Value) -> Self {
        let verification_evidence = &project["verification_evidence"];
        let verification_freshness = project["observation"]["freshness"]["verification"].clone();
        Self {
            project_id: string_field(project, "project_id"),
            verification_status: string_field(verification_evidence, "status"),
            freshness_status: string_field(&verification_freshness, "status"),
            failing_run_count: verification_evidence["failing_runs"]
                .as_array()
                .map(|runs| runs.len())
                .unwrap_or(0),
            safe_for_cleanup: project["safe_for_cleanup"].as_bool().unwrap_or(false),
            safe_for_refactor: project["safe_for_refactor"].as_bool().unwrap_or(false),
            cleanup_gate_level: project_gate_level(project, "cleanup"),
            refactor_gate_level: project_gate_level(project, "refactor"),
            cleanup_reason: project["safe_for_cleanup_reason"]
                .as_str()
                .unwrap_or("Cleanup readiness is blocked.")
                .to_string(),
            refactor_reason: project["safe_for_refactor_reason"]
                .as_str()
                .unwrap_or("Refactor readiness is blocked.")
                .to_string(),
            verification_freshness,
        }
    }

    fn has_recorded_verification(&self) -> bool {
        self.verification_status.as_deref() == Some("available")
    }

    fn is_missing_verification(&self) -> bool {
        self.verification_status.as_deref() == Some("not_recorded")
    }

    fn has_failing_verification(&self) -> bool {
        self.failing_run_count > 0
    }

    fn has_stale_verification(&self) -> bool {
        matches!(self.freshness_status.as_deref(), Some("stale" | "unknown"))
    }

    fn is_blocking(&self) -> bool {
        !self.safe_for_cleanup || !self.safe_for_refactor
    }

    fn cleanup_blocked_by_verification(&self) -> bool {
        self.has_failing_verification()
            || self.is_missing_verification()
            || self.has_stale_verification()
    }

    fn primary_reason(&self) -> &str {
        if self.cleanup_blocked_by_verification() {
            &self.cleanup_reason
        } else {
            &self.refactor_reason
        }
    }

    fn blocking_project_json(&self) -> Value {
        json!({
            "project_id": self.project_id.as_deref(),
            "verification_status": self.verification_status.as_deref(),
            "verification_freshness": self.verification_freshness,
            "failing_run_count": self.failing_run_count,
            "safe_for_cleanup": self.safe_for_cleanup,
            "safe_for_refactor": self.safe_for_refactor,
            "cleanup_gate_level": self.cleanup_gate_level,
            "refactor_gate_level": self.refactor_gate_level,
            "cleanup_reason": self.cleanup_reason,
            "refactor_reason": self.refactor_reason,
            "primary_reason": self.primary_reason(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::mcp::verification_evidence) struct VerificationEvidenceWorkspaceSummary {
    pub(in crate::mcp::verification_evidence) project_count: usize,
    pub(in crate::mcp::verification_evidence) monitoring_count: usize,
    pub(in crate::mcp::verification_evidence) projects_with_recorded_verification: usize,
    pub(in crate::mcp::verification_evidence) projects_missing_verification: usize,
    pub(in crate::mcp::verification_evidence) projects_with_failing_verification: usize,
    pub(in crate::mcp::verification_evidence) projects_with_stale_verification: usize,
    pub(in crate::mcp::verification_evidence) projects_safe_for_cleanup: usize,
    pub(in crate::mcp::verification_evidence) projects_safe_for_refactor: usize,
    pub(in crate::mcp::verification_evidence) cleanup_gate_distribution: GateDistribution,
    pub(in crate::mcp::verification_evidence) refactor_gate_distribution: GateDistribution,
    projects: Vec<VerificationEvidenceProjectSummary>,
}

impl VerificationEvidenceWorkspaceSummary {
    pub(in crate::mcp::verification_evidence) fn from_project_overviews(
        project_overviews: &[Value],
        project_count: usize,
        monitoring_count: usize,
    ) -> Self {
        let projects = project_overviews
            .iter()
            .map(VerificationEvidenceProjectSummary::from_project_overview)
            .collect::<Vec<_>>();
        let projects_with_recorded_verification = projects
            .iter()
            .filter(|project| project.has_recorded_verification())
            .count();
        let projects_missing_verification = projects
            .iter()
            .filter(|project| project.is_missing_verification())
            .count();
        let projects_with_failing_verification = projects
            .iter()
            .filter(|project| project.has_failing_verification())
            .count();
        let projects_with_stale_verification = projects
            .iter()
            .filter(|project| project.has_stale_verification())
            .count();
        let projects_safe_for_cleanup = projects
            .iter()
            .filter(|project| project.safe_for_cleanup)
            .count();
        let projects_safe_for_refactor = projects
            .iter()
            .filter(|project| project.safe_for_refactor)
            .count();
        let cleanup_gate_distribution = GateDistribution::from_levels(
            projects
                .iter()
                .map(|project| project.cleanup_gate_level.clone()),
        );
        let refactor_gate_distribution = GateDistribution::from_levels(
            projects
                .iter()
                .map(|project| project.refactor_gate_level.clone()),
        );

        Self {
            project_count,
            monitoring_count,
            projects_with_recorded_verification,
            projects_missing_verification,
            projects_with_failing_verification,
            projects_with_stale_verification,
            projects_safe_for_cleanup,
            projects_safe_for_refactor,
            cleanup_gate_distribution,
            refactor_gate_distribution,
            projects,
        }
    }

    pub(in crate::mcp::verification_evidence) fn blocking_projects_json(&self) -> Vec<Value> {
        let mut blocking_projects = self
            .projects
            .iter()
            .filter(|project| project.is_blocking())
            .cloned()
            .collect::<Vec<_>>();
        blocking_projects.sort_by(|a, b| {
            b.failing_run_count
                .cmp(&a.failing_run_count)
                .then_with(|| {
                    b.is_missing_verification()
                        .cmp(&a.is_missing_verification())
                })
                .then_with(|| b.has_stale_verification().cmp(&a.has_stale_verification()))
                .then_with(|| {
                    a.project_id
                        .as_deref()
                        .unwrap_or("")
                        .cmp(b.project_id.as_deref().unwrap_or(""))
                })
        });
        blocking_projects
            .iter()
            .map(VerificationEvidenceProjectSummary::blocking_project_json)
            .collect()
    }

    pub(in crate::mcp::verification_evidence) fn verified_conclusions_json(&self) -> Vec<Value> {
        let mut conclusions = Vec::new();
        if self.projects_safe_for_cleanup > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have verification evidence that supports cleanup review.",
                    self.projects_safe_for_cleanup
                ),
                "basis": [
                    "verification_evidence.safe_for_cleanup == true",
                    "latest recorded verification for those projects is not blocked"
                ]
            }));
        }
        if self.projects_safe_for_refactor > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have verification evidence that supports scoped refactor work.",
                    self.projects_safe_for_refactor
                ),
                "basis": [
                    "verification_evidence.safe_for_refactor == true",
                    "required test/build evidence is recorded for those projects"
                ]
            }));
        }
        conclusions
    }

    pub(in crate::mcp::verification_evidence) fn unverified_conclusions_json(&self) -> Vec<Value> {
        let mut conclusions = Vec::new();
        if self.projects_missing_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) are still missing verification evidence.",
                    self.projects_missing_verification
                ),
                "basis": [
                    "verification_evidence.status == not_recorded"
                ]
            }));
        }
        if self.projects_with_stale_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) only have stale verification evidence.",
                    self.projects_with_stale_verification
                ),
                "basis": [
                    "observation.freshness.verification.status in [stale, unknown]"
                ]
            }));
        }
        if self.projects_with_failing_verification > 0 {
            conclusions.push(json!({
                "summary": format!(
                    "{} project(s) currently have failing or uncertain verification runs.",
                    self.projects_with_failing_verification
                ),
                "basis": [
                    "verification_evidence.failing_runs is non-empty"
                ]
            }));
        }
        conclusions
    }

    pub(in crate::mcp::verification_evidence) fn direct_observations(&self) -> Vec<String> {
        vec![
            format!("Registered projects: {}.", self.project_count),
            format!(
                "Projects currently marked as monitoring: {}.",
                self.monitoring_count
            ),
            format!(
                "Projects with recorded verification evidence: {}.",
                self.projects_with_recorded_verification
            ),
            format!(
                "Projects missing verification evidence: {}.",
                self.projects_missing_verification
            ),
            format!(
                "Projects with failing or uncertain verification runs: {}.",
                self.projects_with_failing_verification
            ),
            format!(
                "Projects with stale verification evidence: {}.",
                self.projects_with_stale_verification
            ),
        ]
    }

    pub(in crate::mcp::verification_evidence) fn confidence(&self) -> &'static str {
        if self.projects.is_empty() {
            "low"
        } else if self.projects_missing_verification == 0
            && self.projects_with_stale_verification == 0
        {
            "high"
        } else if self.projects_with_recorded_verification > 0 {
            "medium"
        } else {
            "low"
        }
    }
}

fn string_field(source: &Value, field: &str) -> Option<String> {
    source[field].as_str().map(str::to_string)
}
