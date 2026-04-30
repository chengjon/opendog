use serde_json::Value;
use std::fs;
use tempfile::TempDir;

use super::common::run_cli;

#[test]
fn test_cli_export_writes_json_and_csv_artifacts() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();
    let project_dir = dir.path().join("export-project");
    fs::create_dir_all(project_dir.join("src")).unwrap();
    fs::write(project_dir.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(project_dir.join("README.md"), "# export").unwrap();

    let create = run_cli(
        home,
        &[
            "create",
            "--id",
            "export-demo",
            "--path",
            project_dir.to_str().unwrap(),
        ],
    );
    assert!(create.status.success(), "{:?}", create);

    let snapshot = run_cli(home, &["snapshot", "--id", "export-demo"]);
    assert!(snapshot.status.success(), "{:?}", snapshot);

    let json_output = home.join("stats-export.json");
    let json_export = run_cli(
        home,
        &[
            "export",
            "--id",
            "export-demo",
            "--format",
            "json",
            "--view",
            "stats",
            "--output",
            json_output.to_str().unwrap(),
        ],
    );
    assert!(json_export.status.success(), "{:?}", json_export);

    let json_value: Value = serde_json::from_slice(&fs::read(&json_output).unwrap()).unwrap();
    assert_eq!(json_value["project_id"], "export-demo");
    assert_eq!(json_value["format"], "json");
    assert_eq!(json_value["view"], "stats");
    assert!(json_value["rows"].as_array().unwrap().len() >= 2);
    assert!(json_value["summary"].is_object());

    let csv_output = home.join("stats-export.csv");
    let csv_export = run_cli(
        home,
        &[
            "export",
            "--id",
            "export-demo",
            "--format",
            "csv",
            "--view",
            "stats",
            "--output",
            csv_output.to_str().unwrap(),
        ],
    );
    assert!(csv_export.status.success(), "{:?}", csv_export);

    let csv_text = fs::read_to_string(&csv_output).unwrap();
    let mut lines = csv_text.lines();
    assert_eq!(
        lines.next().unwrap_or_default(),
        "file_path,file_type,size,access_count,estimated_duration_ms,modification_count,last_access_time,first_seen_time"
    );
    assert!(lines.any(|line| line.contains("src/main.rs")));
}
