use opendog::contracts::{
    CLI_DATA_RISK_V1, CLI_DECISION_BRIEF_V1, CLI_RECORD_VERIFICATION_V1, CLI_RUN_VERIFICATION_V1,
    CLI_VERIFICATION_STATUS_V1, CLI_WORKSPACE_DATA_RISK_V1, MCP_GUIDANCE_V1,
};
use std::fs;

use tempfile::TempDir;

use super::{run_cli, run_cli_json};

#[test]
fn test_cli_json_outputs_for_guidance_and_data_risk() {
    let dir = TempDir::new().unwrap();
    let home = dir.path();

    let risky_project_dir = dir.path().join("risky-project");
    fs::create_dir_all(risky_project_dir.join("src")).unwrap();
    fs::create_dir_all(risky_project_dir.join("tests/fixtures")).unwrap();
    fs::write(
        risky_project_dir.join("src/customer_seed.json"),
        r#"{"customer":"Demo User","email":"demo@example.com","invoice_id":"INV-001"}"#,
    )
    .unwrap();
    fs::write(
        risky_project_dir.join("tests/fixtures/mock_response.json"),
        r#"{"mock":true,"sample":"fixture"}"#,
    )
    .unwrap();

    let clean_project_dir = dir.path().join("clean-project");
    fs::create_dir_all(clean_project_dir.join("src")).unwrap();
    fs::write(clean_project_dir.join("src/main.rs"), "fn main() {}").unwrap();

    let create_risky = run_cli(
        home,
        &[
            "create",
            "--id",
            "risky",
            "--path",
            risky_project_dir.to_str().unwrap(),
        ],
    );
    assert!(create_risky.status.success(), "{:?}", create_risky);

    let create_clean = run_cli(
        home,
        &[
            "create",
            "--id",
            "clean",
            "--path",
            clean_project_dir.to_str().unwrap(),
        ],
    );
    assert!(create_clean.status.success(), "{:?}", create_clean);

    let snapshot_risky = run_cli(home, &["snapshot", "--id", "risky"]);
    assert!(snapshot_risky.status.success(), "{:?}", snapshot_risky);

    let snapshot_clean = run_cli(home, &["snapshot", "--id", "clean"]);
    assert!(snapshot_clean.status.success(), "{:?}", snapshot_clean);

    let agent_guidance = run_cli_json(home, &["agent-guidance", "--project", "risky", "--json"]);
    assert_eq!(
        agent_guidance["guidance"]["project_recommendations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        agent_guidance["guidance"]["schema_version"].as_str(),
        Some(MCP_GUIDANCE_V1)
    );
    assert!(agent_guidance["guidance"]["recommended_flow"].is_array());
    assert!(agent_guidance["guidance"]["layers"]["storage_maintenance"].is_object());
    assert!(
        agent_guidance["guidance"]["layers"]["workspace_observation"]
            ["projects_with_storage_maintenance_candidates"]
            .is_u64()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["workspace_observation"]["projects_missing_snapshot"]
            .is_u64()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["workspace_observation"]
            ["projects_missing_verification"]
            .is_u64()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"][0]
            ["observation"]["freshness"]["snapshot"]["status"]
            .is_string()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"][0]
            ["observation"]["coverage_state"]
            .is_string()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"][0]
            ["repo_status_risk"]["risk_findings"]
            .is_array()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"][0]
            ["repo_status_risk"]["finding_counts"]
            .is_object()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"][0]
            ["attention_score"]
            .is_i64()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"][0]
            ["attention_reasons"]
            .is_array()
    );
    assert!(
        agent_guidance["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"][0]
            ["priority_basis"]["recommended_next_action"]
            .is_string()
    );

    let data_risk = run_cli_json(
        home,
        &[
            "data-risk",
            "--id",
            "risky",
            "--candidate-type",
            "all",
            "--min-review-priority",
            "low",
            "--json",
        ],
    );
    assert_eq!(data_risk["schema_version"].as_str(), Some(CLI_DATA_RISK_V1));
    assert_eq!(data_risk["project_id"], "risky");
    assert!(
        data_risk["hardcoded_candidate_count"].as_u64().unwrap_or(0) >= 1,
        "{data_risk:#?}"
    );
    assert!(
        data_risk["mock_candidate_count"].as_u64().unwrap_or(0) >= 1,
        "{data_risk:#?}"
    );
    assert!(data_risk["guidance"]["recommended_flow"].is_array());

    let workspace_data_risk = run_cli_json(
        home,
        &[
            "workspace-data-risk",
            "--candidate-type",
            "all",
            "--min-review-priority",
            "low",
            "--project-limit",
            "5",
            "--json",
        ],
    );
    assert_eq!(
        workspace_data_risk["schema_version"].as_str(),
        Some(CLI_WORKSPACE_DATA_RISK_V1)
    );
    assert_eq!(workspace_data_risk["total_registered_projects"], 2);
    assert_eq!(workspace_data_risk["matched_project_count"], 1);
    assert_eq!(
        workspace_data_risk["projects"][0]["project_id"].as_str(),
        Some("risky")
    );
    assert!(workspace_data_risk["guidance"]["recommended_flow"].is_array());
    assert!(
        workspace_data_risk["guidance"]["layers"]["multi_project_portfolio"]["priority_projects"]
            [0]["priority_reason"]
            .is_string()
    );

    let decision_brief = run_cli_json(
        home,
        &[
            "decision-brief",
            "--project",
            "risky",
            "--top",
            "3",
            "--json",
        ],
    );
    assert_eq!(
        decision_brief["schema_version"].as_str(),
        Some(CLI_DECISION_BRIEF_V1)
    );
    assert_eq!(decision_brief["scope"], "project");
    assert_eq!(decision_brief["selected_project_id"], "risky");
    assert!(decision_brief["decision"]["recommended_next_action"].is_string());
    assert!(decision_brief["decision"]["action_profile"].is_object());
    assert!(decision_brief["decision"]["risk_profile"].is_object());
    assert!(decision_brief["entrypoints"]["next_mcp_tools"].is_array());
    assert!(decision_brief["entrypoints"]["selection_reasons"].is_array());
    assert!(decision_brief["entrypoints"]["execution_templates"].is_array());
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["parameter_schema"].is_object()
    );
    assert!(decision_brief["entrypoints"]["execution_templates"][0]["should_run_if"].is_array());
    assert!(decision_brief["entrypoints"]["execution_templates"][0]["skip_if"].is_array());
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["expected_output_fields"]
            .is_array()
    );
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["follow_up_on_success"].is_array()
    );
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["follow_up_on_failure"].is_array()
    );
    assert!(decision_brief["entrypoints"]["execution_templates"][0]["plan_stage"].is_string());
    assert!(decision_brief["entrypoints"]["execution_templates"][0]["terminality"].is_string());
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["can_run_in_parallel"].is_boolean()
    );
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["requires_human_confirmation"]
            .is_boolean()
    );
    assert!(
        decision_brief["entrypoints"]["execution_templates"][0]["evidence_written_to_opendog"]
            .is_boolean()
    );
    assert!(decision_brief["entrypoints"]["execution_templates"][0]["retry_policy"].is_object());
    assert!(decision_brief["decision"]["signals"].is_object());
    assert!(decision_brief["decision"]["signals"]["storage_maintenance_candidate"].is_boolean());
    assert!(decision_brief["decision"]["signals"]["storage_reclaimable_bytes"].is_i64());
    assert!(decision_brief["decision"]["signals"]["attention_score"].is_i64());
    assert!(decision_brief["decision"]["signals"]["attention_band"].is_string());
    assert!(decision_brief["decision"]["signals"]["attention_reasons"].is_array());
    assert!(decision_brief["decision"]["risk_profile"]["repo_risk_findings"].is_array());
    assert!(decision_brief["decision"]["risk_profile"]["repo_risk_finding_counts"].is_object());
    assert!(decision_brief["decision"]["risk_profile"]["primary_repo_risk_finding"].is_null());
    assert!(decision_brief["layers"]["workspace_observation"].is_object());
    assert!(
        decision_brief["layers"]["workspace_observation"]["projects_with_stale_snapshot"].is_u64()
    );
    assert!(
        decision_brief["layers"]["workspace_observation"]["projects_with_stale_verification"]
            .is_u64()
    );
    assert!(decision_brief["layers"]["storage_maintenance"].is_object());
    assert!(decision_brief["layers"]["constraints_boundaries"].is_object());

    let record_verification = run_cli_json(
        home,
        &[
            "record-verification",
            "--id",
            "risky",
            "--kind",
            "test",
            "--status",
            "passed",
            "--command",
            "cargo test",
            "--exit-code",
            "0",
            "--summary",
            "all good",
            "--json",
        ],
    );
    assert_eq!(
        record_verification["schema_version"].as_str(),
        Some(CLI_RECORD_VERIFICATION_V1)
    );
    assert_eq!(
        record_verification["recorded"]["kind"].as_str(),
        Some("test")
    );
    assert_eq!(
        record_verification["recorded"]["status"].as_str(),
        Some("passed")
    );

    let verification_status = run_cli_json(home, &["verification", "--id", "risky", "--json"]);
    assert_eq!(
        verification_status["schema_version"].as_str(),
        Some(CLI_VERIFICATION_STATUS_V1)
    );
    assert_eq!(verification_status["project_id"], "risky");
    assert_eq!(
        verification_status["verification"]["latest_runs"][0]["kind"].as_str(),
        Some("test")
    );

    let run_verification = run_cli_json(
        home,
        &[
            "run-verification",
            "--id",
            "clean",
            "--kind",
            "test",
            "--command",
            "printf ok-from-verification",
            "--json",
        ],
    );
    assert_eq!(
        run_verification["schema_version"].as_str(),
        Some(CLI_RUN_VERIFICATION_V1)
    );
    assert_eq!(
        run_verification["executed"]["run"]["status"].as_str(),
        Some("passed")
    );
    assert!(run_verification["executed"]["stdout_tail"]
        .as_str()
        .unwrap_or("")
        .contains("ok-from-verification"));
}
