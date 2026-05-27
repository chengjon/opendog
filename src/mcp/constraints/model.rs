use serde_json::{json, Value};

use super::string_array_field;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadinessTarget {
    Cleanup,
    Refactor,
}

impl ReadinessTarget {
    pub(super) fn from_name(target: &str) -> Self {
        match target {
            "refactor" => Self::Refactor,
            _ => Self::Cleanup,
        }
    }

    fn blocker_key(self) -> &'static str {
        match self {
            Self::Cleanup => "cleanup_blockers",
            Self::Refactor => "refactor_blockers",
        }
    }

    fn gate_key(self) -> &'static str {
        match self {
            Self::Cleanup => "cleanup",
            Self::Refactor => "refactor",
        }
    }

    pub(super) fn reason_summary(self, safe: bool, reasons: &[String]) -> String {
        if safe {
            match self {
                Self::Refactor => {
                    "Current evidence supports scoped refactor work: verification gates passed and no repository-level blocker is active."
                        .to_string()
                }
                Self::Cleanup => {
                    "Current evidence supports cleanup review: required verification gates passed and no repository-level blocker is active."
                        .to_string()
                }
            }
        } else if let Some(reason) = reasons.first() {
            reason.clone()
        } else {
            match self {
                Self::Refactor => {
                    "Refactor readiness is blocked by missing evidence or repository risk."
                        .to_string()
                }
                Self::Cleanup => {
                    "Cleanup readiness is blocked by missing evidence or repository risk."
                        .to_string()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RepositoryReadinessSignals {
    operation_states: Vec<String>,
    conflicted_count: u64,
    lockfile_anomaly_count: usize,
    large_diff: bool,
    changed_file_count: u64,
}

impl RepositoryReadinessSignals {
    fn from_value(repo_risk: &Value) -> Self {
        Self {
            operation_states: string_array_field(repo_risk, "operation_states"),
            conflicted_count: repo_risk["conflicted_count"].as_u64().unwrap_or(0),
            lockfile_anomaly_count: repo_risk["lockfile_anomalies"]
                .as_array()
                .map(|items| items.len())
                .unwrap_or(0),
            large_diff: repo_risk["large_diff"].as_bool().unwrap_or(false),
            changed_file_count: repo_risk["changed_file_count"].as_u64().unwrap_or(0),
        }
    }

    fn append_reasons(&self, target: ReadinessTarget, reasons: &mut Vec<String>) {
        if !self.operation_states.is_empty() {
            reasons.push(format!(
                "Repository is mid-operation: {}.",
                self.operation_states.join(", ")
            ));
        }

        if self.conflicted_count > 0 {
            reasons.push(format!(
                "Repository has {} conflicted paths in the working tree.",
                self.conflicted_count
            ));
        }

        if self.lockfile_anomaly_count > 0 {
            reasons.push(format!(
                "Dependency manifest/lockfile mismatches are present ({} signals).",
                self.lockfile_anomaly_count
            ));
        }

        if target == ReadinessTarget::Refactor && self.large_diff {
            reasons.push(format!(
                "Working tree already has a large diff ({} changed files), so broad refactors should wait.",
                self.changed_file_count
            ));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectReadinessAssessment {
    verification_safe_for_cleanup: bool,
    verification_safe_for_refactor: bool,
    cleanup_gate_level: String,
    refactor_gate_level: String,
    cleanup_blockers: Vec<String>,
    refactor_blockers: Vec<String>,
}

impl ProjectReadinessAssessment {
    pub(super) fn from_layers(repo_risk: &Value, verification_layer: &Value) -> Self {
        let repo_signals = RepositoryReadinessSignals::from_value(repo_risk);
        let verification_safe_for_cleanup = verification_layer["safe_for_cleanup"]
            .as_bool()
            .unwrap_or(false);
        let verification_safe_for_refactor = verification_layer["safe_for_refactor"]
            .as_bool()
            .unwrap_or(false);

        Self {
            verification_safe_for_cleanup,
            verification_safe_for_refactor,
            cleanup_gate_level: Self::gate_level(
                verification_layer,
                ReadinessTarget::Cleanup,
                verification_safe_for_cleanup,
            ),
            refactor_gate_level: Self::gate_level(
                verification_layer,
                ReadinessTarget::Refactor,
                verification_safe_for_refactor,
            ),
            cleanup_blockers: Self::reasons_for_target(
                verification_layer,
                &repo_signals,
                ReadinessTarget::Cleanup,
            ),
            refactor_blockers: Self::reasons_for_target(
                verification_layer,
                &repo_signals,
                ReadinessTarget::Refactor,
            ),
        }
    }

    fn gate_level(
        verification_layer: &Value,
        target: ReadinessTarget,
        verification_safe: bool,
    ) -> String {
        verification_layer["gate_assessment"][target.gate_key()]["level"]
            .as_str()
            .unwrap_or(if verification_safe {
                "allow"
            } else {
                "blocked"
            })
            .to_string()
    }

    fn reasons_for_target(
        verification_layer: &Value,
        repo_signals: &RepositoryReadinessSignals,
        target: ReadinessTarget,
    ) -> Vec<String> {
        let mut reasons = string_array_field(verification_layer, target.blocker_key());
        repo_signals.append_reasons(target, &mut reasons);
        reasons
    }

    #[cfg(test)]
    pub(super) fn reasons_for(&self, target: ReadinessTarget) -> &[String] {
        match target {
            ReadinessTarget::Cleanup => &self.cleanup_blockers,
            ReadinessTarget::Refactor => &self.refactor_blockers,
        }
    }

    pub(super) fn safe_for_cleanup(&self) -> bool {
        self.verification_safe_for_cleanup && self.cleanup_blockers.is_empty()
    }

    pub(super) fn safe_for_refactor(&self) -> bool {
        self.verification_safe_for_refactor && self.refactor_blockers.is_empty()
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "verification_safe_for_cleanup": self.verification_safe_for_cleanup,
            "verification_safe_for_refactor": self.verification_safe_for_refactor,
            "cleanup_gate_level": self.cleanup_gate_level,
            "refactor_gate_level": self.refactor_gate_level,
            "safe_for_cleanup": self.safe_for_cleanup(),
            "safe_for_cleanup_reason": ReadinessTarget::Cleanup.reason_summary(
                self.safe_for_cleanup(),
                &self.cleanup_blockers,
            ),
            "cleanup_blockers": self.cleanup_blockers,
            "safe_for_refactor": self.safe_for_refactor(),
            "safe_for_refactor_reason": ReadinessTarget::Refactor.reason_summary(
                self.safe_for_refactor(),
                &self.refactor_blockers,
            ),
            "refactor_blockers": self.refactor_blockers,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ProjectReadinessAssessment, ReadinessTarget};

    #[test]
    fn readiness_model_combines_verification_and_repo_blockers() {
        let repo_risk = json!({
            "operation_states": ["merge"],
            "conflicted_count": 2,
            "lockfile_anomalies": [{}],
            "large_diff": false
        });
        let verification = json!({
            "safe_for_cleanup": true,
            "safe_for_refactor": true,
            "cleanup_blockers": ["cleanup gate blocker"],
            "refactor_blockers": []
        });

        let assessment = ProjectReadinessAssessment::from_layers(&repo_risk, &verification);

        assert!(!assessment.safe_for_cleanup());
        assert_eq!(assessment.reasons_for(ReadinessTarget::Cleanup).len(), 4);
        assert_eq!(
            assessment.reasons_for(ReadinessTarget::Cleanup)[0],
            "cleanup gate blocker"
        );
    }

    #[test]
    fn readiness_model_blocks_refactor_for_large_diff_only() {
        let repo_risk = json!({
            "large_diff": true,
            "changed_file_count": 12
        });
        let verification = json!({
            "safe_for_cleanup": true,
            "safe_for_refactor": true
        });

        let assessment = ProjectReadinessAssessment::from_layers(&repo_risk, &verification);

        assert!(assessment.safe_for_cleanup());
        assert!(!assessment.safe_for_refactor());
        assert!(assessment.reasons_for(ReadinessTarget::Refactor)[0].contains("12 changed files"));
    }

    #[test]
    fn readiness_model_renders_stable_json_contract() {
        let repo_risk = json!({});
        let verification = json!({
            "safe_for_cleanup": true,
            "safe_for_refactor": false,
            "gate_assessment": {
                "cleanup": { "level": "allow" },
                "refactor": { "level": "blocked" }
            },
            "refactor_blockers": ["missing lint"]
        });

        let assessment = ProjectReadinessAssessment::from_layers(&repo_risk, &verification);
        let json = assessment.to_json();

        assert_eq!(json["verification_safe_for_cleanup"], true);
        assert_eq!(json["verification_safe_for_refactor"], false);
        assert_eq!(json["cleanup_gate_level"], "allow");
        assert_eq!(json["refactor_gate_level"], "blocked");
        assert_eq!(json["safe_for_cleanup"], true);
        assert_eq!(json["safe_for_refactor"], false);
        assert_eq!(json["refactor_blockers"][0], "missing lint");
    }
}
