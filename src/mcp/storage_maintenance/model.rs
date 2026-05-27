use crate::config::RetentionPolicy;
use crate::core::retention::StorageMetrics;

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

#[cfg(test)]
mod tests {
    use crate::config::RetentionPolicy;
    use crate::core::retention::StorageMetrics;

    use super::StorageMaintenanceAssessment;

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
}
