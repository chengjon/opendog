use super::*;

fn test_pm() -> ProjectManager {
    let dir = tempfile::tempdir().unwrap();
    let pm = ProjectManager::with_data_dir(dir.path().join("data").as_path()).unwrap();
    Box::leak(Box::new(dir));
    pm
}

#[test]
fn create_rejects_empty_project_id() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    let err = pm.create("", root.path()).unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
}

#[test]
fn create_rejects_project_id_with_spaces() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    let err = pm.create("has spaces", root.path()).unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
}

#[test]
fn create_rejects_project_id_with_dots() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    let err = pm.create("../../etc", root.path()).unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidProjectId(_)));
}

#[test]
fn create_rejects_relative_root_path() {
    let pm = test_pm();
    let err = pm
        .create("valid-id", Path::new("relative/path"))
        .unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidPath(_)));
}

#[test]
fn create_rejects_duplicate_project_id() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    pm.create("dup", root.path()).unwrap();
    let err = pm.create("dup", root.path()).unwrap_err();
    assert!(matches!(err, OpenDogError::ProjectExists(_)));
}

#[test]
fn create_succeeds_with_valid_inputs() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    let info = pm.create("my-project", root.path()).unwrap();
    assert_eq!(info.id, "my-project");
    assert_eq!(info.root_path, root.path());
    assert!(info.db_path.to_string_lossy().contains("my-project.db"));
}

#[test]
fn get_returns_created_project() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    pm.create("find-me", root.path()).unwrap();
    let info = pm.get("find-me").unwrap().unwrap();
    assert_eq!(info.id, "find-me");
}

#[test]
fn get_returns_none_for_unknown() {
    let pm = test_pm();
    assert!(pm.get("ghost").unwrap().is_none());
}

#[test]
fn list_returns_all_projects() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    pm.create("alpha", root.path()).unwrap();
    pm.create("beta", root.path()).unwrap();
    let list = pm.list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn delete_soft_deletes_project() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    pm.create("bye", root.path()).unwrap();
    assert!(pm.delete("bye").unwrap());
    let info = pm.get("bye").unwrap().unwrap();
    assert_eq!(info.status, "deleted");
}

#[test]
fn delete_returns_false_for_unknown() {
    let pm = test_pm();
    assert!(!pm.delete("ghost").unwrap());
}

#[test]
fn update_global_config_rejects_empty_patch() {
    let pm = test_pm();
    let err = pm
        .update_global_config(ConfigPatch {
            ignore_patterns: None,
            process_whitelist: None,
            retention: None,
            add_ignore_patterns: vec![],
            remove_ignore_patterns: vec![],
            add_process_whitelist: vec![],
            remove_process_whitelist: vec![],
        })
        .unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidInput(_)));
}

#[test]
fn update_project_config_rejects_empty_patch() {
    let pm = test_pm();
    let root = tempfile::tempdir().unwrap();
    pm.create("cfg-test", root.path()).unwrap();
    let err = pm
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                ignore_patterns: None,
                process_whitelist: None,
                retention: None,
                add_ignore_patterns: vec![],
                remove_ignore_patterns: vec![],
                add_process_whitelist: vec![],
                remove_process_whitelist: vec![],
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                inherit_retention: false,
            },
        )
        .unwrap_err();
    assert!(matches!(err, OpenDogError::InvalidInput(_)));
}

#[test]
fn update_project_config_rejects_unknown_project() {
    let pm = test_pm();
    let err = pm
        .update_project_config(
            "ghost",
            ProjectConfigPatch {
                ignore_patterns: Some(vec!["*.log".to_string()]),
                process_whitelist: None,
                retention: None,
                add_ignore_patterns: vec![],
                remove_ignore_patterns: vec![],
                add_process_whitelist: vec![],
                remove_process_whitelist: vec![],
                inherit_ignore_patterns: false,
                inherit_process_whitelist: false,
                inherit_retention: false,
            },
        )
        .unwrap_err();
    assert!(matches!(err, OpenDogError::ProjectNotFound(_)));
}
