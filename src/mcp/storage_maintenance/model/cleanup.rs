use crate::config::RetentionPolicy;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::mcp::storage_maintenance) enum StorageCleanupScope {
    Activity,
    Snapshots,
    Verification,
    All,
}

impl StorageCleanupScope {
    fn from_str(scope: &str) -> Option<Self> {
        match scope {
            "activity" => Some(Self::Activity),
            "snapshots" => Some(Self::Snapshots),
            "verification" => Some(Self::Verification),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub(in crate::mcp::storage_maintenance) fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Snapshots => "snapshots",
            Self::Verification => "verification",
            Self::All => "all",
        }
    }

    pub(in crate::mcp::storage_maintenance) fn default_older_than_days(
        self,
        policy: &RetentionPolicy,
    ) -> i64 {
        match self {
            Self::Verification => policy.verification_retention_days,
            Self::Activity | Self::All => policy.activity_retention_days,
            Self::Snapshots => policy.activity_retention_days,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::mcp::storage_maintenance) struct StorageCleanupRecommendation {
    pub(in crate::mcp::storage_maintenance) scope: StorageCleanupScope,
    pub(in crate::mcp::storage_maintenance) older_than_days: Option<i64>,
    pub(in crate::mcp::storage_maintenance) keep_snapshot_runs: Option<i64>,
}

impl StorageCleanupRecommendation {
    fn from_value(recommendation: &Value) -> Option<Self> {
        let scope = StorageCleanupScope::from_str(recommendation["scope"].as_str()?)?;
        if scope == StorageCleanupScope::All {
            return None;
        }

        Some(Self {
            scope,
            older_than_days: recommendation["older_than_days"].as_i64(),
            keep_snapshot_runs: recommendation["keep_snapshot_runs"].as_i64(),
        })
    }

    pub(in crate::mcp::storage_maintenance) fn older_than_days_or_default(
        &self,
        policy: &RetentionPolicy,
    ) -> i64 {
        self.older_than_days
            .unwrap_or_else(|| self.scope.default_older_than_days(policy))
    }

    pub(in crate::mcp::storage_maintenance) fn keep_snapshot_runs_or_default(
        &self,
        policy: &RetentionPolicy,
    ) -> i64 {
        self.keep_snapshot_runs.unwrap_or(policy.keep_snapshot_runs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::mcp::storage_maintenance) struct StorageCleanupPlanStep {
    pub(in crate::mcp::storage_maintenance) scope: StorageCleanupScope,
    pub(in crate::mcp::storage_maintenance) older_than_days: Option<i64>,
    pub(in crate::mcp::storage_maintenance) keep_snapshot_runs: Option<i64>,
}

impl StorageCleanupPlanStep {
    fn from_value(plan_step: &Value) -> Option<Self> {
        if plan_step["phase"].as_str() != Some("execute_cleanup") {
            return None;
        }

        Some(Self {
            scope: StorageCleanupScope::from_str(plan_step["scope"].as_str()?)?,
            older_than_days: plan_step["older_than_days"]
                .as_i64()
                .or_else(|| plan_step["retention_parameters"]["older_than_days"].as_i64()),
            keep_snapshot_runs: plan_step["keep_snapshot_runs"]
                .as_i64()
                .or_else(|| plan_step["retention_parameters"]["keep_snapshot_runs"].as_i64()),
        })
    }

    pub(in crate::mcp::storage_maintenance) fn older_than_days_or_default(
        &self,
        policy: &RetentionPolicy,
    ) -> i64 {
        self.older_than_days
            .unwrap_or_else(|| self.scope.default_older_than_days(policy))
    }

    pub(in crate::mcp::storage_maintenance) fn keep_snapshot_runs_or_default(
        &self,
        policy: &RetentionPolicy,
    ) -> i64 {
        self.keep_snapshot_runs.unwrap_or(policy.keep_snapshot_runs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::mcp::storage_maintenance) struct StorageMaintenanceTemplateContext {
    project_id_value: String,
    project_placeholder_required: bool,
    maintenance_candidate: bool,
    pub(in crate::mcp::storage_maintenance) vacuum_candidate: bool,
    pub(in crate::mcp::storage_maintenance) approx_reclaimable_bytes: i64,
    pub(in crate::mcp::storage_maintenance) cleanup_recommendations:
        Vec<StorageCleanupRecommendation>,
    pub(in crate::mcp::storage_maintenance) cleanup_plan_steps: Vec<StorageCleanupPlanStep>,
}

impl StorageMaintenanceTemplateContext {
    pub(in crate::mcp::storage_maintenance) fn from_inputs(
        project_id: Option<&str>,
        storage_maintenance: &Value,
    ) -> Self {
        let project_id_value = project_id.unwrap_or("<project>").to_string();
        let cleanup_recommendations = storage_maintenance["cleanup_recommendations"]
            .as_array()
            .map(|recommendations| {
                recommendations
                    .iter()
                    .filter_map(StorageCleanupRecommendation::from_value)
                    .collect()
            })
            .unwrap_or_default();
        let cleanup_plan_steps = storage_maintenance["cleanup_plan"]["steps"]
            .as_array()
            .map(|steps| {
                steps
                    .iter()
                    .filter_map(StorageCleanupPlanStep::from_value)
                    .collect()
            })
            .unwrap_or_default();

        Self {
            project_id_value,
            project_placeholder_required: project_id.is_none(),
            maintenance_candidate: storage_maintenance["maintenance_candidate"]
                .as_bool()
                .unwrap_or(false),
            vacuum_candidate: storage_maintenance["vacuum_candidate"]
                .as_bool()
                .unwrap_or(false),
            approx_reclaimable_bytes: storage_maintenance["approx_reclaimable_bytes"]
                .as_i64()
                .unwrap_or(0),
            cleanup_recommendations,
            cleanup_plan_steps,
        }
    }

    pub(in crate::mcp::storage_maintenance) fn should_emit_templates(&self) -> bool {
        self.maintenance_candidate
    }

    pub(in crate::mcp::storage_maintenance) fn project_id_value(&self) -> &str {
        &self.project_id_value
    }

    pub(in crate::mcp::storage_maintenance) fn project_placeholder_required(&self) -> bool {
        self.project_placeholder_required
    }

    pub(in crate::mcp::storage_maintenance) fn project_placeholder_hint_json(&self) -> Value {
        if self.project_placeholder_required() {
            json!([{
                "field": "id",
                "placeholder": "<project>",
                "description": "replace with a registered OPENDOG project id"
            }])
        } else {
            json!([])
        }
    }
}
