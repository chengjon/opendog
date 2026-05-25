use serde_json::{json, Value};

pub(crate) fn review_focus_projection_for_top_project(top_recommendation: Option<&Value>) -> Value {
    let Some(recommendation) = top_recommendation else {
        return json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "review_focus": Value::Null
        });
    };

    json!({
        "status": "available",
        "source": "top_priority_project",
        "source_project_id": recommendation["project_id"].clone(),
        "review_focus": recommendation["review_focus"].clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn none_returns_no_priority_project() {
        let result = review_focus_projection_for_top_project(None);
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
        let result = review_focus_projection_for_top_project(Some(&rec));
        assert_eq!(result["status"], "available");
        assert_eq!(result["source"], "top_priority_project");
        assert_eq!(result["source_project_id"], "my-project");
        assert_eq!(result["review_focus"], "check unused files");
    }

    #[test]
    fn with_recommendation_missing_project_id() {
        let rec = json!({"review_focus": "something"});
        let result = review_focus_projection_for_top_project(Some(&rec));
        assert_eq!(result["status"], "available");
        assert!(result["source_project_id"].is_null());
        assert_eq!(result["review_focus"], "something");
    }

    #[test]
    fn with_recommendation_missing_review_focus() {
        let rec = json!({"project_id": "proj1"});
        let result = review_focus_projection_for_top_project(Some(&rec));
        assert_eq!(result["status"], "available");
        assert!(result["review_focus"].is_null());
    }

    #[test]
    fn with_recommendation_empty_json() {
        let rec = json!({});
        let result = review_focus_projection_for_top_project(Some(&rec));
        assert_eq!(result["status"], "available");
        assert!(result["source_project_id"].is_null());
        assert!(result["review_focus"].is_null());
    }
}
