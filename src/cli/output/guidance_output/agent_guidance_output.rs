use serde_json::Value;

use super::print_recommended_flow;

pub(super) fn print_agent_guidance(guidance: &Value) {
    println!(
        "Agent guidance — projects={} monitoring={}",
        guidance["project_count"].as_u64().unwrap_or(0),
        guidance["monitoring_count"].as_u64().unwrap_or(0),
    );

    if let Some(notes) = guidance["notes"].as_array() {
        for note in notes {
            if let Some(text) = note.as_str() {
                println!("  Note: {}", text);
            }
        }
    }

    print_recommended_flow(&guidance["recommended_flow"]);

    let strategy = &guidance["layers"]["execution_strategy"];
    println!();
    println!(
        "Strategy: mode={} primary={} secondary={}",
        strategy["global_strategy_mode"].as_str().unwrap_or("-"),
        strategy["preferred_primary_tool"].as_str().unwrap_or("-"),
        strategy["preferred_secondary_tool"].as_str().unwrap_or("-"),
    );

    let observation = &guidance["layers"]["workspace_observation"];
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

    let verification = &guidance["layers"]["verification_evidence"];
    println!();
    println!(
        "Verification evidence: recorded={} missing={} failing={} stale={} cleanup_ready={} refactor_ready={} confidence={}",
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
        verification["projects_safe_for_cleanup"]
            .as_u64()
            .unwrap_or(0),
        verification["projects_safe_for_refactor"]
            .as_u64()
            .unwrap_or(0),
        verification["confidence"].as_str().unwrap_or("-"),
    );
    let blocking_projects = verification["blocking_projects"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !blocking_projects.is_empty() {
        println!("Blocking projects:");
        for project in blocking_projects.iter().take(3) {
            println!(
                "  {} verification={} freshness={} reason={}",
                project["project_id"].as_str().unwrap_or("-"),
                project["verification_status"].as_str().unwrap_or("-"),
                project["verification_freshness"]["status"]
                    .as_str()
                    .unwrap_or("-"),
                project["primary_reason"].as_str().unwrap_or("-"),
            );
        }
    }

    let storage = &guidance["layers"]["storage_maintenance"];
    if storage["projects_with_candidates"].as_u64().unwrap_or(0) > 0 {
        println!();
        println!(
            "Storage maintenance: candidates={} vacuum_candidates={}",
            storage["projects_with_candidates"].as_u64().unwrap_or(0),
            storage["projects_with_vacuum_candidates"]
                .as_u64()
                .unwrap_or(0),
        );
        if let Some(priority_projects) = storage["priority_projects"].as_array() {
            for project in priority_projects.iter().take(3) {
                println!(
                    "  {} mode={} reclaimable_bytes={}",
                    project["project_id"].as_str().unwrap_or("-"),
                    project["suggested_mode"].as_str().unwrap_or("-"),
                    project["approx_reclaimable_bytes"].as_i64().unwrap_or(0),
                );
            }
        }
    }

    let portfolio = &guidance["layers"]["multi_project_portfolio"];
    println!();
    println!("Priority projects:");
    let priority_candidates = portfolio["priority_candidates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if priority_candidates.is_empty() {
        println!("  None");
    } else {
        let project_overviews = portfolio["project_overviews"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for project in priority_candidates.iter().take(5) {
            println!(
                "  {} action={} confidence={} attention={}({})",
                project["project_id"].as_str().unwrap_or("-"),
                project["recommended_next_action"].as_str().unwrap_or("-"),
                project["confidence"].as_str().unwrap_or("-"),
                project["attention_score"].as_i64().unwrap_or(0),
                project["attention_band"].as_str().unwrap_or("-"),
            );
            if let Some(reason) = project["reason"].as_str() {
                println!("    reason: {}", reason);
            }
            if let Some(overview) = project_overviews
                .iter()
                .find(|overview| overview["project_id"].as_str() == project["project_id"].as_str())
            {
                println!(
                    "    observation: coverage={} snapshot={} activity={} verification={}",
                    overview["observation"]["coverage_state"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["observation"]["freshness"]["snapshot"]["status"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["observation"]["freshness"]["activity"]["status"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["observation"]["freshness"]["verification"]["status"]
                        .as_str()
                        .unwrap_or("-"),
                );
                println!(
                    "    verification: verification_status={} cleanup_safe={} refactor_safe={}",
                    overview["verification_evidence"]["status"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["safe_for_cleanup"].as_bool().unwrap_or(false),
                    overview["safe_for_refactor"].as_bool().unwrap_or(false),
                );
                println!(
                    "    repo: repo_status={} repo_level={} top_finding={}",
                    overview["repo_status_risk"]["status"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["repo_status_risk"]["risk_level"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["repo_status_risk"]["highest_priority_finding"]["kind"]
                        .as_str()
                        .unwrap_or("-"),
                );
                println!(
                    "    toolchain: toolchain_type={} toolchain_confidence={}",
                    overview["project_toolchain"]["project_type"]
                        .as_str()
                        .unwrap_or("-"),
                    overview["project_toolchain"]["confidence"]
                        .as_str()
                        .unwrap_or("-"),
                );
            }
            let attention_reasons = project["attention_reasons"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if let Some(primary_attention_reason) = attention_reasons.first() {
                if let Some(text) = primary_attention_reason.as_str() {
                    println!("    attention: {}", text);
                }
            }
            let flow = project["recommended_flow"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for (index, step) in flow.iter().take(3).enumerate() {
                if let Some(text) = step.as_str() {
                    println!("    {}. {}", index + 1, text);
                }
            }
        }
    }
}
