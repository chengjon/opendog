use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GateDistribution {
    pub(super) allow: usize,
    pub(super) caution: usize,
    pub(super) blocked: usize,
}

impl GateDistribution {
    fn from_levels(levels: impl Iterator<Item = String>) -> Self {
        let mut distribution = Self {
            allow: 0,
            caution: 0,
            blocked: 0,
        };
        for level in levels {
            match level.as_str() {
                "allow" => distribution.allow += 1,
                "caution" => distribution.caution += 1,
                "blocked" => distribution.blocked += 1,
                _ => {}
            }
        }
        distribution
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "allow": self.allow,
            "caution": self.caution,
            "blocked": self.blocked,
        })
    }
}

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
pub(super) struct VerificationEvidenceWorkspaceSummary {
    pub(super) project_count: usize,
    pub(super) monitoring_count: usize,
    pub(super) projects_with_recorded_verification: usize,
    pub(super) projects_missing_verification: usize,
    pub(super) projects_with_failing_verification: usize,
    pub(super) projects_with_stale_verification: usize,
    pub(super) projects_safe_for_cleanup: usize,
    pub(super) projects_safe_for_refactor: usize,
    pub(super) cleanup_gate_distribution: GateDistribution,
    pub(super) refactor_gate_distribution: GateDistribution,
    projects: Vec<VerificationEvidenceProjectSummary>,
}

impl VerificationEvidenceWorkspaceSummary {
    pub(super) fn from_project_overviews(
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

    pub(super) fn blocking_projects_json(&self) -> Vec<Value> {
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

    pub(super) fn verified_conclusions_json(&self) -> Vec<Value> {
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

    pub(super) fn unverified_conclusions_json(&self) -> Vec<Value> {
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

    pub(super) fn direct_observations(&self) -> Vec<String> {
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

    pub(super) fn confidence(&self) -> &'static str {
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

pub(super) fn project_gate_level(project: &Value, target: &str) -> String {
    project["verification_evidence"]["gate_assessment"][target]["level"]
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            let key = match target {
                "refactor" => "safe_for_refactor",
                _ => "safe_for_cleanup",
            };
            if project[key].as_bool().unwrap_or(false) {
                "allow".to_string()
            } else {
                "blocked".to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::VerificationEvidenceWorkspaceSummary;
    use serde_json::json;

    #[test]
    fn workspace_summary_counts_gate_levels_and_safety_flags() {
        let projects = vec![
            json!({
                "project_id": "ready",
                "safe_for_cleanup": true,
                "safe_for_refactor": true,
                "safe_for_cleanup_reason": "cleanup ok",
                "safe_for_refactor_reason": "refactor ok",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "allow"},
                        "refactor": {"level": "allow"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "fresh"}}}
            }),
            json!({
                "project_id": "stale",
                "safe_for_cleanup": true,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "cleanup caution",
                "safe_for_refactor_reason": "Refactor readiness is blocked.",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "caution"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "stale"}}}
            }),
            json!({
                "project_id": "missing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "Cleanup readiness is blocked.",
                "safe_for_refactor_reason": "Refactor readiness is blocked.",
                "verification_evidence": {
                    "status": "not_recorded",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "unknown"}}}
            }),
        ];

        let summary = VerificationEvidenceWorkspaceSummary::from_project_overviews(&projects, 3, 1);

        assert_eq!(summary.project_count, 3);
        assert_eq!(summary.monitoring_count, 1);
        assert_eq!(summary.projects_with_recorded_verification, 2);
        assert_eq!(summary.projects_missing_verification, 1);
        assert_eq!(summary.projects_with_failing_verification, 0);
        assert_eq!(summary.projects_with_stale_verification, 2);
        assert_eq!(summary.projects_safe_for_cleanup, 2);
        assert_eq!(summary.projects_safe_for_refactor, 1);
        assert_eq!(summary.cleanup_gate_distribution.allow, 1);
        assert_eq!(summary.cleanup_gate_distribution.caution, 1);
        assert_eq!(summary.cleanup_gate_distribution.blocked, 1);
        assert_eq!(summary.refactor_gate_distribution.allow, 1);
        assert_eq!(summary.refactor_gate_distribution.caution, 0);
        assert_eq!(summary.refactor_gate_distribution.blocked, 2);
        assert_eq!(summary.confidence(), "medium");
    }

    #[test]
    fn blocking_projects_sort_by_failing_missing_stale_then_project_id() {
        let projects = vec![
            json!({
                "project_id": "stale-a",
                "safe_for_cleanup": false,
                "safe_for_refactor": true,
                "safe_for_cleanup_reason": "stale cleanup",
                "safe_for_refactor_reason": "refactor ok",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "allow"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "stale"}}}
            }),
            json!({
                "project_id": "failing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "failing cleanup",
                "safe_for_refactor_reason": "failing refactor",
                "verification_evidence": {
                    "status": "available",
                    "failing_runs": [{"kind": "test"}],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "fresh"}}}
            }),
            json!({
                "project_id": "missing",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "safe_for_cleanup_reason": "missing cleanup",
                "safe_for_refactor_reason": "missing refactor",
                "verification_evidence": {
                    "status": "not_recorded",
                    "failing_runs": [],
                    "gate_assessment": {
                        "cleanup": {"level": "blocked"},
                        "refactor": {"level": "blocked"}
                    }
                },
                "observation": {"freshness": {"verification": {"status": "unknown"}}}
            }),
        ];

        let summary = VerificationEvidenceWorkspaceSummary::from_project_overviews(&projects, 3, 0);
        let blocking = summary.blocking_projects_json();

        assert_eq!(blocking[0]["project_id"], "failing");
        assert_eq!(blocking[1]["project_id"], "missing");
        assert_eq!(blocking[2]["project_id"], "stale-a");
        assert_eq!(blocking[0]["failing_run_count"], 1);
        assert_eq!(blocking[0]["primary_reason"], "failing cleanup");
        assert_eq!(blocking[1]["verification_status"], "not_recorded");
        assert_eq!(
            blocking[2]["verification_freshness"],
            json!({"status": "stale"})
        );
    }
}
