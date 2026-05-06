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
