use crate::config::RetentionPolicy;
use crate::core::retention::StorageMetrics;
use serde_json::{json, Value};

pub(super) fn storage_reclaim_ratio(metrics: &StorageMetrics) -> f64 {
    if metrics.approx_db_size_bytes <= 0 {
        0.0
    } else {
        metrics.approx_reclaimable_bytes as f64 / metrics.approx_db_size_bytes as f64
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StorageMaintenanceAssessment {
    pub(super) reclaim_ratio: f64,
    pub(super) cleanup_review_candidate: bool,
    pub(super) evidence_pressure_candidate: bool,
    pub(super) maintenance_candidate: bool,
    pub(super) vacuum_candidate: bool,
    pub(super) suggested_mode: &'static str,
    pub(super) pressure_level: &'static str,
    pub(super) summary: &'static str,
}

impl StorageMaintenanceAssessment {
    pub(super) fn from_inputs(
        metrics: &StorageMetrics,
        has_cleanup_recommendations: bool,
        policy: &RetentionPolicy,
    ) -> Self {
        let reclaim_ratio = storage_reclaim_ratio(metrics);
        let cleanup_review_candidate =
            metrics.approx_db_size_bytes >= policy.cleanup_review_db_bytes_threshold;
        let evidence_pressure_candidate = has_cleanup_recommendations;
        let vacuum_candidate = metrics.approx_reclaimable_bytes
            >= policy.vacuum_reclaimable_bytes_threshold
            && reclaim_ratio >= policy.vacuum_reclaim_ratio_threshold_percent as f64 / 100.0;
        let maintenance_candidate =
            cleanup_review_candidate || evidence_pressure_candidate || vacuum_candidate;
        let suggested_mode = if vacuum_candidate {
            "review_cleanup_then_vacuum"
        } else if cleanup_review_candidate || evidence_pressure_candidate {
            "review_cleanup"
        } else {
            "none"
        };
        let pressure_level = if vacuum_candidate || evidence_pressure_candidate {
            "high"
        } else if cleanup_review_candidate {
            "medium"
        } else {
            "low"
        };
        let summary = if vacuum_candidate {
            "Project database has reclaimable space; review retained OPENDOG evidence and consider vacuum after cleanup."
        } else if evidence_pressure_candidate {
            "Project retained evidence counts exceed storage pressure thresholds; review scope-specific cleanup-data dry-runs."
        } else if cleanup_review_candidate {
            "Project database is large enough that retained OPENDOG evidence should be reviewed with cleanup-data dry-run."
        } else {
            "Project database size does not currently suggest dedicated OPENDOG retention maintenance."
        };

        Self {
            reclaim_ratio,
            cleanup_review_candidate,
            evidence_pressure_candidate,
            maintenance_candidate,
            vacuum_candidate,
            suggested_mode,
            pressure_level,
            summary,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StorageMaintenanceProjectSummary {
    pub(super) project_id: Option<String>,
    pub(super) status: Option<String>,
    pub(super) maintenance_candidate: bool,
    pub(super) vacuum_candidate: bool,
    pub(super) cleanup_review_candidate: bool,
    pub(super) approx_db_size_bytes: i64,
    pub(super) approx_reclaimable_bytes: i64,
    pub(super) reclaim_ratio: f64,
    pub(super) suggested_mode: Option<String>,
    pub(super) summary: Option<String>,
}

impl StorageMaintenanceProjectSummary {
    fn from_project_overview(project: &Value) -> Self {
        let storage = &project["storage_maintenance"];
        Self {
            project_id: string_field(project, "project_id"),
            status: string_field(project, "status"),
            maintenance_candidate: storage["maintenance_candidate"].as_bool().unwrap_or(false),
            vacuum_candidate: storage["vacuum_candidate"].as_bool().unwrap_or(false),
            cleanup_review_candidate: storage["cleanup_review_candidate"]
                .as_bool()
                .unwrap_or(false),
            approx_db_size_bytes: storage["approx_db_size_bytes"].as_i64().unwrap_or(0),
            approx_reclaimable_bytes: storage["approx_reclaimable_bytes"].as_i64().unwrap_or(0),
            reclaim_ratio: storage["reclaim_ratio"].as_f64().unwrap_or(0.0),
            suggested_mode: string_field(storage, "suggested_mode"),
            summary: string_field(storage, "summary"),
        }
    }

    fn priority_project_json(&self) -> Value {
        json!({
            "project_id": self.project_id.as_deref(),
            "status": self.status.as_deref(),
            "vacuum_candidate": self.vacuum_candidate,
            "cleanup_review_candidate": self.cleanup_review_candidate,
            "approx_db_size_bytes": self.approx_db_size_bytes,
            "approx_reclaimable_bytes": self.approx_reclaimable_bytes,
            "reclaim_ratio": self.reclaim_ratio,
            "suggested_mode": self.suggested_mode.as_deref(),
            "summary": self.summary.as_deref(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StorageMaintenanceWorkspaceSummary {
    pub(super) projects_with_candidates: usize,
    pub(super) projects_with_vacuum_candidates: usize,
    pub(super) total_approx_db_size_bytes: i64,
    pub(super) total_approx_reclaimable_bytes: i64,
    pub(super) priority_projects: Vec<StorageMaintenanceProjectSummary>,
}

impl StorageMaintenanceWorkspaceSummary {
    pub(super) fn from_project_overviews(project_overviews: &[Value]) -> Self {
        let projects = project_overviews
            .iter()
            .map(StorageMaintenanceProjectSummary::from_project_overview)
            .collect::<Vec<_>>();
        let projects_with_candidates = projects
            .iter()
            .filter(|project| project.maintenance_candidate)
            .count();
        let projects_with_vacuum_candidates = projects
            .iter()
            .filter(|project| project.vacuum_candidate)
            .count();
        let total_approx_db_size_bytes = projects
            .iter()
            .map(|project| project.approx_db_size_bytes)
            .sum::<i64>();
        let total_approx_reclaimable_bytes = projects
            .iter()
            .map(|project| project.approx_reclaimable_bytes)
            .sum::<i64>();

        let mut priority_projects = projects
            .into_iter()
            .filter(|project| project.maintenance_candidate)
            .collect::<Vec<_>>();
        priority_projects.sort_by(|a, b| {
            b.vacuum_candidate
                .cmp(&a.vacuum_candidate)
                .then_with(|| b.approx_reclaimable_bytes.cmp(&a.approx_reclaimable_bytes))
                .then_with(|| b.approx_db_size_bytes.cmp(&a.approx_db_size_bytes))
        });
        priority_projects.truncate(5);

        Self {
            projects_with_candidates,
            projects_with_vacuum_candidates,
            total_approx_db_size_bytes,
            total_approx_reclaimable_bytes,
            priority_projects,
        }
    }

    pub(super) fn priority_projects_json(&self) -> Vec<Value> {
        self.priority_projects
            .iter()
            .map(StorageMaintenanceProjectSummary::priority_project_json)
            .collect()
    }
}

fn string_field(source: &Value, field: &str) -> Option<String> {
    source[field].as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use crate::config::RetentionPolicy;
    use crate::core::retention::StorageMetrics;
    use serde_json::json;

    use super::{StorageMaintenanceAssessment, StorageMaintenanceWorkspaceSummary};

    fn policy() -> RetentionPolicy {
        RetentionPolicy {
            cleanup_review_db_bytes_threshold: 100,
            vacuum_reclaimable_bytes_threshold: 20,
            vacuum_reclaim_ratio_threshold_percent: 25,
            ..Default::default()
        }
    }

    #[test]
    fn assessment_marks_vacuum_candidate_as_high_priority() {
        let assessment = StorageMaintenanceAssessment::from_inputs(
            &StorageMetrics {
                approx_db_size_bytes: 100,
                approx_reclaimable_bytes: 30,
                ..Default::default()
            },
            false,
            &policy(),
        );

        assert!(assessment.vacuum_candidate);
        assert!(assessment.maintenance_candidate);
        assert_eq!(assessment.suggested_mode, "review_cleanup_then_vacuum");
        assert_eq!(assessment.pressure_level, "high");
    }

    #[test]
    fn assessment_uses_evidence_pressure_without_large_database() {
        let assessment = StorageMaintenanceAssessment::from_inputs(
            &StorageMetrics {
                approx_db_size_bytes: 10,
                approx_reclaimable_bytes: 0,
                ..Default::default()
            },
            true,
            &policy(),
        );

        assert!(!assessment.cleanup_review_candidate);
        assert!(assessment.evidence_pressure_candidate);
        assert!(assessment.maintenance_candidate);
        assert_eq!(assessment.suggested_mode, "review_cleanup");
        assert_eq!(assessment.pressure_level, "high");
    }

    #[test]
    fn assessment_keeps_small_clean_database_low_priority() {
        let assessment = StorageMaintenanceAssessment::from_inputs(
            &StorageMetrics {
                approx_db_size_bytes: 10,
                approx_reclaimable_bytes: 0,
                ..Default::default()
            },
            false,
            &policy(),
        );

        assert!(!assessment.maintenance_candidate);
        assert_eq!(assessment.suggested_mode, "none");
        assert_eq!(assessment.pressure_level, "low");
    }

    #[test]
    fn workspace_summary_aggregates_and_sorts_priority_projects() {
        let project_overviews = vec![
            json!({
                "project_id": "large-cleanup",
                "status": "registered",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 900,
                    "approx_reclaimable_bytes": 100,
                    "reclaim_ratio": 0.11,
                    "suggested_mode": "review_cleanup",
                    "summary": "large database"
                }
            }),
            json!({
                "project_id": "vacuum-first",
                "status": "registered",
                "storage_maintenance": {
                    "maintenance_candidate": true,
                    "vacuum_candidate": true,
                    "cleanup_review_candidate": true,
                    "approx_db_size_bytes": 500,
                    "approx_reclaimable_bytes": 200,
                    "reclaim_ratio": 0.4,
                    "suggested_mode": "review_cleanup_then_vacuum",
                    "summary": "vacuum candidate"
                }
            }),
            json!({
                "project_id": "healthy",
                "status": "registered",
                "storage_maintenance": {
                    "maintenance_candidate": false,
                    "vacuum_candidate": false,
                    "cleanup_review_candidate": false,
                    "approx_db_size_bytes": 50,
                    "approx_reclaimable_bytes": 0,
                    "reclaim_ratio": 0.0,
                    "suggested_mode": "none",
                    "summary": "healthy"
                }
            }),
        ];

        let summary =
            StorageMaintenanceWorkspaceSummary::from_project_overviews(&project_overviews);

        assert_eq!(summary.projects_with_candidates, 2);
        assert_eq!(summary.projects_with_vacuum_candidates, 1);
        assert_eq!(summary.total_approx_db_size_bytes, 1450);
        assert_eq!(summary.total_approx_reclaimable_bytes, 300);
        assert_eq!(
            summary.priority_projects[0].project_id.as_deref(),
            Some("vacuum-first")
        );
        assert_eq!(
            summary.priority_projects[1].project_id.as_deref(),
            Some("large-cleanup")
        );
    }

    #[test]
    fn priority_project_json_preserves_contract_shape() {
        let project_overviews = vec![json!({
            "project_id": "demo",
            "status": "registered",
            "storage_maintenance": {
                "maintenance_candidate": true,
                "vacuum_candidate": true,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 500,
                "approx_reclaimable_bytes": 200,
                "reclaim_ratio": 0.4,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "vacuum candidate"
            }
        })];
        let summary =
            StorageMaintenanceWorkspaceSummary::from_project_overviews(&project_overviews);

        assert_eq!(
            summary.priority_projects_json(),
            vec![json!({
                "project_id": "demo",
                "status": "registered",
                "vacuum_candidate": true,
                "cleanup_review_candidate": true,
                "approx_db_size_bytes": 500,
                "approx_reclaimable_bytes": 200,
                "reclaim_ratio": 0.4,
                "suggested_mode": "review_cleanup_then_vacuum",
                "summary": "vacuum candidate"
            })]
        );
    }
}
