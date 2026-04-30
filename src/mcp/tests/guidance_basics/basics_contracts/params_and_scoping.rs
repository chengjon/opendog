use super::*;

#[test]
fn agent_guidance_params_deserialize_project_scope_and_top() {
    let params: AgentGuidanceParams = serde_json::from_value(json!({
        "project_id": "demo",
        "top": 2
    }))
    .unwrap();

    assert_eq!(params.project_id.as_deref(), Some("demo"));
    assert_eq!(params.top, Some(2));
}

#[test]
fn scoped_projects_or_error_filters_requested_project() {
    let projects = vec![
        ProjectInfo {
            id: "alpha".to_string(),
            root_path: std::path::PathBuf::from("/tmp/alpha"),
            db_path: std::path::PathBuf::from("/tmp/alpha.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-04-26T00:00:00Z".to_string(),
            status: "active".to_string(),
        },
        ProjectInfo {
            id: "beta".to_string(),
            root_path: std::path::PathBuf::from("/tmp/beta"),
            db_path: std::path::PathBuf::from("/tmp/beta.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-04-26T00:00:00Z".to_string(),
            status: "active".to_string(),
        },
    ];

    let filtered = scoped_projects_or_error(projects.clone(), Some("beta")).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "beta");

    let err = scoped_projects_or_error(projects, Some("missing")).unwrap_err();
    assert_eq!(err.to_string(), "Project 'missing' not found");
}
