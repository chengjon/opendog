mod guidance;
mod report;
mod rules;
mod workspace;

#[cfg(test)]
pub(crate) use self::guidance::data_risk_guidance;
pub(crate) use self::guidance::project_data_risk_payload;
pub(crate) use self::rules::{
    normalize_candidate_type, normalize_min_review_priority, path_kind_score, review_priority_score,
};
pub(crate) use self::workspace::workspace_data_risk_overview_payload;

#[derive(Debug, Clone)]
pub(crate) struct DataCandidate {
    pub(crate) file_path: String,
    pub(crate) confidence: &'static str,
    pub(crate) review_priority: &'static str,
    pub(crate) path_classification: &'static str,
    pub(crate) rule_hits: Vec<String>,
    pub(crate) matched_keywords: Vec<String>,
    pub(crate) reasons: Vec<String>,
    pub(crate) evidence: Vec<String>,
    pub(crate) access_count: i64,
    pub(crate) file_type: String,
}

pub(super) struct DataRiskRuleMeta {
    pub(super) rule: &'static str,
    pub(super) group: &'static str,
    pub(super) severity: &'static str,
    pub(super) description: &'static str,
}

pub(super) const DATA_RISK_RULES: &[DataRiskRuleMeta] = &[
    DataRiskRuleMeta {
        rule: "path.mock_token",
        group: "path",
        severity: "low",
        description: "Path contains explicit mock, fixture, demo, or seed markers.",
    },
    DataRiskRuleMeta {
        rule: "content.mock_token",
        group: "content",
        severity: "medium",
        description: "Content contains explicit mock, fixture, fake, or sample-data markers.",
    },
    DataRiskRuleMeta {
        rule: "path.test_only",
        group: "classification",
        severity: "low",
        description: "File sits under a test-only or example-style path.",
    },
    DataRiskRuleMeta {
        rule: "path.generated_artifact",
        group: "classification",
        severity: "low",
        description: "File sits inside a generated-artifact directory and should be down-ranked.",
    },
    DataRiskRuleMeta {
        rule: "content.business_literal_combo",
        group: "content",
        severity: "high",
        description: "Content combines business-like keywords with literal-value markers.",
    },
    DataRiskRuleMeta {
        rule: "path.runtime_shared",
        group: "classification",
        severity: "high",
        description:
            "Candidate appears in a runtime/shared source path rather than a test-only area.",
    },
];

#[derive(Debug, Clone, Default)]
pub(crate) struct MockDataReport {
    pub(crate) mock_candidates: Vec<DataCandidate>,
    pub(crate) hardcoded_candidates: Vec<DataCandidate>,
    pub(crate) mixed_review_files: Vec<String>,
}
