use serde_json::Value;

use super::super::guidance_types::ReviewFocusProjection;

pub(crate) fn review_focus_projection_for_top_project(
    top_recommendation: Option<&Value>,
) -> ReviewFocusProjection {
    let Some(recommendation) = top_recommendation else {
        return ReviewFocusProjection::no_priority_project();
    };

    ReviewFocusProjection::available(
        recommendation["project_id"].clone(),
        recommendation["review_focus"].clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn none_returns_no_priority_project() {
        let result = serde_json::to_value(review_focus_projection_for_top_project(None)).unwrap();
        assert_eq!(result["status"], "no_priority_project");
        assert!(result["source"].is_null());
        assert!(result["source_project_id"].is_null());
        assert!(result["review_focus"].is_null());
    }

    #[test]
    fn with_recommendation_returns_available() {
        let rec = json!({
            "project_id": "my-project",
            "review_focus": "check unused files"
        });
        let result =
            serde_json::to_value(review_focus_projection_for_top_project(Some(&rec))).unwrap();
        assert_eq!(result["status"], "available");
        assert_eq!(result["source"], "top_priority_project");
        assert_eq!(result["source_project_id"], "my-project");
        assert_eq!(result["review_focus"], "check unused files");
    }

    #[test]
    fn with_recommendation_missing_project_id() {
        let rec = json!({"review_focus": "something"});
        let result =
            serde_json::to_value(review_focus_projection_for_top_project(Some(&rec))).unwrap();
        assert_eq!(result["status"], "available");
        assert!(result["source_project_id"].is_null());
        assert_eq!(result["review_focus"], "something");
    }

    #[test]
    fn with_recommendation_missing_review_focus() {
        let rec = json!({"project_id": "proj1"});
        let result =
            serde_json::to_value(review_focus_projection_for_top_project(Some(&rec))).unwrap();
        assert_eq!(result["status"], "available");
        assert!(result["review_focus"].is_null());
    }

    #[test]
    fn with_recommendation_empty_json() {
        let rec = json!({});
        let result =
            serde_json::to_value(review_focus_projection_for_top_project(Some(&rec))).unwrap();
        assert_eq!(result["status"], "available");
        assert!(result["source_project_id"].is_null());
        assert!(result["review_focus"].is_null());
    }
}
