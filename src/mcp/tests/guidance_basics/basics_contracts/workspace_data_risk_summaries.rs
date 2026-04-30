use super::*;

#[test]
fn collect_workspace_data_risk_summaries_sorts_by_hardcoded_priority() {
    let dir = TempDir::new().unwrap();
    let alpha_root = dir.path().join("alpha");
    let beta_root = dir.path().join("beta");
    std::fs::create_dir_all(alpha_root.join("tests/fixtures")).unwrap();
    std::fs::create_dir_all(beta_root.join("src")).unwrap();
    std::fs::write(
        alpha_root.join("tests/fixtures/demo.json"),
        r#"{"mock": true, "customer": "Demo"}"#,
    )
    .unwrap();
    std::fs::write(
        beta_root.join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();

    let projects = vec![
        ProjectInfo {
            id: "alpha".to_string(),
            root_path: alpha_root.clone(),
            db_path: alpha_root.join("alpha.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-04-26T00:00:00Z".to_string(),
            status: "active".to_string(),
        },
        ProjectInfo {
            id: "beta".to_string(),
            root_path: beta_root.clone(),
            db_path: beta_root.join("beta.db"),
            config: ProjectConfigOverrides::default(),
            created_at: "2026-04-26T00:00:00Z".to_string(),
            status: "monitoring".to_string(),
        },
    ];

    let summaries = collect_workspace_data_risk_summaries(&projects, "all", "low", |project| {
        if project.id == "alpha" {
            vec![StatsEntry {
                file_path: "tests/fixtures/demo.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            }]
        } else {
            vec![StatsEntry {
                file_path: "src/customer_seed.rs".to_string(),
                size: 10,
                file_type: "rs".to_string(),
                access_count: 2,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            }]
        }
    });

    assert_eq!(summaries[0]["project_id"], "beta");
    assert_eq!(summaries[1]["project_id"], "alpha");
    assert_eq!(summaries[0]["hardcoded_candidate_count"], json!(1));
    assert_eq!(summaries[1]["hardcoded_candidate_count"], json!(0));
}
