use crate::config::RetentionPolicy;
use crate::core::retention::StorageMetrics;

pub(in crate::mcp::storage_maintenance) fn storage_reclaim_ratio(metrics: &StorageMetrics) -> f64 {
    if metrics.approx_db_size_bytes <= 0 {
        0.0
    } else {
        metrics.approx_reclaimable_bytes as f64 / metrics.approx_db_size_bytes as f64
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::mcp::storage_maintenance) struct StorageMaintenanceAssessment {
    pub(in crate::mcp::storage_maintenance) reclaim_ratio: f64,
    pub(in crate::mcp::storage_maintenance) cleanup_review_candidate: bool,
    pub(in crate::mcp::storage_maintenance) evidence_pressure_candidate: bool,
    pub(in crate::mcp::storage_maintenance) maintenance_candidate: bool,
    pub(in crate::mcp::storage_maintenance) vacuum_candidate: bool,
    pub(in crate::mcp::storage_maintenance) suggested_mode: &'static str,
    pub(in crate::mcp::storage_maintenance) pressure_level: &'static str,
    pub(in crate::mcp::storage_maintenance) summary: &'static str,
}

impl StorageMaintenanceAssessment {
    pub(in crate::mcp::storage_maintenance) fn from_inputs(
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
