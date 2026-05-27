use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReviewFocusCandidateFamily {
    HotFile,
    UnusedCandidate,
}

impl ReviewFocusCandidateFamily {
    fn as_str(self) -> &'static str {
        match self {
            Self::HotFile => "hot_file",
            Self::UnusedCandidate => "unused_candidate",
        }
    }

    fn basis(self) -> &'static [&'static str] {
        match self {
            Self::HotFile => &["highest_access_activity", "activity_present"],
            Self::UnusedCandidate => &["zero_recorded_access", "snapshot_present"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecommendationReviewFocus {
    candidate_family: ReviewFocusCandidateFamily,
    candidate_risk_hints: Vec<&'static str>,
}

impl RecommendationReviewFocus {
    pub(super) fn from_action(selected_action: &str, repo_risk: &Value) -> Option<Self> {
        match selected_action {
            "inspect_hot_files" => Some(Self {
                candidate_family: ReviewFocusCandidateFamily::HotFile,
                candidate_risk_hints: Self::hot_file_risk_hints(repo_risk),
            }),
            "review_unused_files" => Some(Self {
                candidate_family: ReviewFocusCandidateFamily::UnusedCandidate,
                candidate_risk_hints: Vec::new(),
            }),
            _ => None,
        }
    }

    fn hot_file_risk_hints(repo_risk: &Value) -> Vec<&'static str> {
        if repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
            || repo_risk["large_diff"].as_bool().unwrap_or(false)
        {
            vec!["repo_risk_elevated"]
        } else {
            Vec::new()
        }
    }

    #[cfg(test)]
    pub(super) fn candidate_family(&self) -> ReviewFocusCandidateFamily {
        self.candidate_family
    }

    #[cfg(test)]
    pub(super) fn risk_hints(&self) -> &[&'static str] {
        &self.candidate_risk_hints
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "candidate_family": self.candidate_family.as_str(),
            "candidate_basis": self.candidate_family.basis(),
            "candidate_risk_hints": self.candidate_risk_hints,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{RecommendationReviewFocus, ReviewFocusCandidateFamily};

    #[test]
    fn review_focus_model_marks_hot_files_with_elevated_repo_risk() {
        let repo_risk = json!({
            "risk_level": "high",
            "large_diff": false
        });

        let focus =
            RecommendationReviewFocus::from_action("inspect_hot_files", &repo_risk).unwrap();

        assert_eq!(
            focus.candidate_family(),
            ReviewFocusCandidateFamily::HotFile
        );
        assert_eq!(focus.risk_hints(), &["repo_risk_elevated"]);
    }

    #[test]
    fn review_focus_model_marks_hot_files_with_large_diff() {
        let repo_risk = json!({
            "risk_level": "low",
            "large_diff": true
        });

        let focus =
            RecommendationReviewFocus::from_action("inspect_hot_files", &repo_risk).unwrap();

        assert_eq!(
            focus.candidate_family(),
            ReviewFocusCandidateFamily::HotFile
        );
        assert_eq!(focus.risk_hints(), &["repo_risk_elevated"]);
    }

    #[test]
    fn review_focus_model_renders_unused_candidate_json_contract() {
        let repo_risk = json!({});

        let focus =
            RecommendationReviewFocus::from_action("review_unused_files", &repo_risk).unwrap();
        let json = focus.to_json();

        assert_eq!(json["candidate_family"], "unused_candidate");
        assert_eq!(
            json["candidate_basis"],
            json!(["zero_recorded_access", "snapshot_present"])
        );
        assert!(json["candidate_risk_hints"].as_array().unwrap().is_empty());
    }
}
