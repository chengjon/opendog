use serde::Serialize;
use serde_json::Value;

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
