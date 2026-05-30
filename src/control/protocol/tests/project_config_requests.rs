use super::*;

#[test]
fn request_ping_round_trip() {
    let req = ControlRequest::Ping;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::Ping));
}

#[test]
fn request_create_project_round_trip() {
    let req = ControlRequest::CreateProject {
        id: "test".to_string(),
        path: "/tmp/test".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::CreateProject { id, path } = back {
        assert_eq!(id, "test");
        assert_eq!(path, "/tmp/test");
    } else {
        panic!("expected CreateProject variant");
    }
}

#[test]
fn request_delete_project_round_trip() {
    let req = ControlRequest::DeleteProject {
        id: "x".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::DeleteProject { .. }));
}

#[test]
fn request_list_projects_round_trip() {
    let req = ControlRequest::ListProjects;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ListProjects));
}

#[test]
fn request_list_monitors_round_trip() {
    let req = ControlRequest::ListMonitors;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ListMonitors));
}

#[test]
fn request_get_stats_round_trip() {
    let req = ControlRequest::GetStats {
        id: "proj".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    if let ControlRequest::GetStats { id } = back {
        assert_eq!(id, "proj");
    } else {
        panic!("expected GetStats variant");
    }
}

#[test]
fn request_get_global_config_round_trip() {
    let req = ControlRequest::GetGlobalConfig;
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetGlobalConfig));
}

#[test]
fn request_get_project_config_round_trip() {
    let req = ControlRequest::GetProjectConfig {
        id: "p".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::GetProjectConfig { .. }));
}

#[test]
fn request_update_global_config_round_trip() {
    let patch = ConfigPatch {
        ignore_patterns: Some(vec!["*.log".to_string()]),
        process_whitelist: None,
        retention: None,
        add_ignore_patterns: vec![],
        remove_ignore_patterns: vec![],
        add_process_whitelist: vec![],
        remove_process_whitelist: vec![],
    };
    let req = ControlRequest::UpdateGlobalConfig(patch);
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json_str).unwrap();
    if let ControlRequest::UpdateGlobalConfig(p) = back {
        assert_eq!(p.ignore_patterns.as_ref().unwrap().len(), 1);
    } else {
        panic!("expected UpdateGlobalConfig variant");
    }
}

#[test]
fn request_update_project_config_round_trip() {
    let fields = UpdateProjectConfigFields {
        id: "proj1".to_string(),
        patch: ProjectConfigPatch::default(),
    };
    let req = ControlRequest::UpdateProjectConfig(fields);
    let json_str = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json_str).unwrap();
    if let ControlRequest::UpdateProjectConfig(f) = back {
        assert_eq!(f.id, "proj1");
    } else {
        panic!("expected UpdateProjectConfig variant");
    }
}

#[test]
fn request_reload_project_config_round_trip() {
    let req = ControlRequest::ReloadProjectConfig {
        id: "r".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::ReloadProjectConfig { .. }));
}
