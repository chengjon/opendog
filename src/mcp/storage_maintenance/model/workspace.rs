use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub(in crate::mcp::storage_maintenance) struct StorageMaintenanceProjectSummary {
    pub(in crate::mcp::storage_maintenance) project_id: Option<String>,
    pub(in crate::mcp::storage_maintenance) status: Option<String>,
    pub(in crate::mcp::storage_maintenance) maintenance_candidate: bool,
    pub(in crate::mcp::storage_maintenance) vacuum_candidate: bool,
    pub(in crate::mcp::storage_maintenance) cleanup_review_candidate: bool,
    pub(in crate::mcp::storage_maintenance) approx_db_size_bytes: i64,
    pub(in crate::mcp::storage_maintenance) approx_reclaimable_bytes: i64,
    pub(in crate::mcp::storage_maintenance) reclaim_ratio: f64,
    pub(in crate::mcp::storage_maintenance) suggested_mode: Option<String>,
    pub(in crate::mcp::storage_maintenance) summary: Option<String>,
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
pub(in crate::mcp::storage_maintenance) struct StorageMaintenanceWorkspaceSummary {
    pub(in crate::mcp::storage_maintenance) projects_with_candidates: usize,
    pub(in crate::mcp::storage_maintenance) projects_with_vacuum_candidates: usize,
    pub(in crate::mcp::storage_maintenance) total_approx_db_size_bytes: i64,
    pub(in crate::mcp::storage_maintenance) total_approx_reclaimable_bytes: i64,
    pub(in crate::mcp::storage_maintenance) priority_projects:
        Vec<StorageMaintenanceProjectSummary>,
}

impl StorageMaintenanceWorkspaceSummary {
    pub(in crate::mcp::storage_maintenance) fn from_project_overviews(
        project_overviews: &[Value],
    ) -> Self {
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

    pub(in crate::mcp::storage_maintenance) fn priority_projects_json(&self) -> Vec<Value> {
        self.priority_projects
            .iter()
            .map(StorageMaintenanceProjectSummary::priority_project_json)
            .collect()
    }
}

fn string_field(source: &Value, field: &str) -> Option<String> {
    source[field].as_str().map(str::to_string)
}
