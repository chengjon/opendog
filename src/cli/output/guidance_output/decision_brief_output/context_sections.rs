use serde_json::Value;

pub(super) fn print_context_sections(decision: &Value, layers: &Value) {
    let observation = &layers["workspace_observation"];
    println!();
    println!(
        "Workspace observation: status={} analysis_state={}",
        observation["status"].as_str().unwrap_or("-"),
        observation["analysis_state"].as_str().unwrap_or("-"),
    );
    println!(
        "  snapshot_missing={} snapshot_stale={} activity_missing={} activity_stale={} verification_missing={} verification_stale={}",
        observation["projects_missing_snapshot"].as_u64().unwrap_or(0),
        observation["projects_with_stale_snapshot"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_missing_activity"].as_u64().unwrap_or(0),
        observation["projects_with_stale_activity"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_missing_verification"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_with_stale_verification"]
            .as_u64()
            .unwrap_or(0),
    );
    println!(
        "  monitoring={} hardcoded_projects={} total_hardcoded={}",
        layers["multi_project_portfolio"]["monitoring_count"]
            .as_u64()
            .unwrap_or(0),
        observation["projects_with_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
        observation["total_hardcoded_candidates"]
            .as_u64()
            .unwrap_or(0),
    );

    let strategy = &layers["execution_strategy"];
    println!(
        "Execution strategy: mode={} evidence_priority={}",
        strategy["global_strategy_mode"].as_str().unwrap_or("-"),
        strategy["evidence_priority"].as_str().unwrap_or("-"),
    );

    let matched_overview = layers["multi_project_portfolio"]["project_overviews"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["project_id"].as_str() == decision["target_project_id"].as_str())
        });
    let repo_risk = matched_overview
        .map(|project| &project["repo_status_risk"])
        .unwrap_or(&layers["repo_status_risk"]);
    println!(
        "Repo risk: status={} level={} dirty={}",
        repo_risk["status"].as_str().unwrap_or("-"),
        repo_risk["risk_level"].as_str().unwrap_or("-"),
        repo_risk["is_dirty"].as_bool().unwrap_or(false),
    );
    println!(
        "Repo findings: total={} high={} medium={} low={}",
        decision["risk_profile"]["repo_risk_finding_counts"]["total"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["high"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["medium"]
            .as_u64()
            .unwrap_or(0),
        decision["risk_profile"]["repo_risk_finding_counts"]["low"]
            .as_u64()
            .unwrap_or(0),
    );
    let top_changed_directories = repo_risk["top_changed_directories"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !top_changed_directories.is_empty() {
        let hotspots = top_changed_directories
            .iter()
            .take(3)
            .map(|item| {
                format!(
                    "{}({})",
                    item["directory"].as_str().unwrap_or("-"),
                    item["changed_files"].as_u64().unwrap_or(0)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("Repo hotspots: {}", hotspots);
    }
    let operation_states = repo_risk["operation_states"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !operation_states.is_empty() {
        println!(
            "Repo operations: {}",
            operation_states
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let verification = &layers["verification_evidence"];
    println!(
        "Verification evidence: recorded={} missing={} failing={} stale={} confidence={}",
        verification["projects_with_recorded_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_missing_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_with_failing_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_with_stale_verification"]
            .as_u64()
            .unwrap_or(0),
        verification["confidence"].as_str().unwrap_or("-"),
    );
    if let Some(project) = matched_overview {
        println!(
            "  Target verification: project={} status={} freshness={} cleanup_safe={} refactor_safe={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["verification_evidence"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["verification"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["safe_for_cleanup"].as_bool().unwrap_or(false),
            project["safe_for_refactor"].as_bool().unwrap_or(false),
        );
        println!(
            "  Target observation: project={} coverage={} snapshot={} activity={} verification={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["observation"]["coverage_state"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["snapshot"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["activity"]["status"]
                .as_str()
                .unwrap_or("-"),
            project["observation"]["freshness"]["verification"]["status"]
                .as_str()
                .unwrap_or("-"),
        );
    }

    let toolchain = &layers["project_toolchain"];
    if let Some(project) = matched_overview {
        println!(
            "Toolchain: project={} type={} confidence={}",
            project["project_id"].as_str().unwrap_or("-"),
            project["project_toolchain"]["project_type"]
                .as_str()
                .unwrap_or("-"),
            project["project_toolchain"]["confidence"]
                .as_str()
                .unwrap_or("-"),
        );
    } else {
        println!(
            "Toolchain: status={} known_types={} unknown_projects={}",
            toolchain["status"].as_str().unwrap_or("-"),
            toolchain["known_project_types"].as_u64().unwrap_or(0),
            toolchain["projects_without_detected_toolchain"]
                .as_u64()
                .unwrap_or(0),
        );
    }

    let signals = &decision["signals"];
    println!(
        "Signals: repo_risk={} dirty={} hardcoded={} mock={}",
        signals["repo_risk_level"].as_str().unwrap_or("-"),
        signals["repo_is_dirty"].as_bool().unwrap_or(false),
        signals["hardcoded_candidate_count"].as_u64().unwrap_or(0),
        signals["mock_candidate_count"].as_u64().unwrap_or(0),
    );
}
