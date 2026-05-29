use serde_json::Value;

mod context_sections;
mod execution_templates;

use self::context_sections::print_context_sections;
use self::execution_templates::print_execution_templates;
use super::print_recommended_flow;

pub(super) fn print_decision_brief(payload: &Value) {
    println!(
        "Decision brief — scope={} top={}",
        payload["scope"].as_str().unwrap_or("-"),
        payload["top"].as_u64().unwrap_or(0),
    );

    let decision = &payload["decision"];
    println!(
        "  Next action: {} | primary={} secondary={}",
        decision["recommended_next_action"].as_str().unwrap_or("-"),
        decision["preferred_primary_tool"].as_str().unwrap_or("-"),
        decision["preferred_secondary_tool"].as_str().unwrap_or("-"),
    );
    if let Some(project_id) = decision["target_project_id"].as_str() {
        println!("  Target project: {}", project_id);
    }
    if let Some(summary) = decision["summary"].as_str() {
        println!("  Summary: {}", summary);
    }
    println!(
        "  Action profile: class={} phase={} verification_required={}",
        decision["action_profile"]["action_class"]
            .as_str()
            .unwrap_or("-"),
        decision["action_profile"]["phase"].as_str().unwrap_or("-"),
        decision["action_profile"]["verification_required"]
            .as_bool()
            .unwrap_or(false),
    );
    println!(
        "  Risk profile: tier={} repo_risk={} manual_review={}",
        decision["risk_profile"]["risk_tier"]
            .as_str()
            .unwrap_or("-"),
        decision["risk_profile"]["repo_risk_level"]
            .as_str()
            .unwrap_or("-"),
        decision["risk_profile"]["manual_review_required"]
            .as_bool()
            .unwrap_or(false),
    );
    if let Some(primary_repo_risk) =
        decision["risk_profile"]["primary_repo_risk_finding"].as_object()
    {
        println!(
            "  Repo risk focus: {} [{} / {}]",
            primary_repo_risk
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            primary_repo_risk
                .get("severity")
                .and_then(Value::as_str)
                .unwrap_or("-"),
            primary_repo_risk
                .get("priority")
                .and_then(Value::as_str)
                .unwrap_or("-"),
        );
        if let Some(summary) = primary_repo_risk.get("summary").and_then(Value::as_str) {
            println!("  Repo risk reason: {}", summary);
        }
    }
    println!(
        "  Attention: score={} band={}",
        decision["signals"]["attention_score"].as_i64().unwrap_or(0),
        decision["signals"]["attention_band"]
            .as_str()
            .unwrap_or("-"),
    );
    let attention_reasons = decision["signals"]["attention_reasons"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if let Some(primary_attention_reason) = attention_reasons.first() {
        if let Some(text) = primary_attention_reason.as_str() {
            println!("  Attention reason: {}", text);
        }
    }
    if decision["signals"]["storage_maintenance_candidate"]
        .as_bool()
        .unwrap_or(false)
    {
        println!(
            "  Storage maintenance: candidate=true vacuum_candidate={} reclaimable_bytes={}",
            decision["signals"]["storage_vacuum_candidate"]
                .as_bool()
                .unwrap_or(false),
            decision["signals"]["storage_reclaimable_bytes"]
                .as_i64()
                .unwrap_or(0),
        );
    }
    print_recommended_flow(&decision["recommended_flow"]);

    println!();
    println!("Suggested MCP tools:");
    let next_tools = payload["entrypoints"]["next_mcp_tools"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if next_tools.is_empty() {
        println!("  None");
    } else {
        for tool in next_tools {
            if let Some(name) = tool.as_str() {
                println!("  {}", name);
            }
        }
    }

    println!();
    println!("Suggested CLI commands:");
    let next_commands = payload["entrypoints"]["next_cli_commands"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if next_commands.is_empty() {
        println!("  None");
    } else {
        for command in next_commands {
            if let Some(text) = command.as_str() {
                println!("  {}", text);
            }
        }
    }

    let selection_reasons = payload["entrypoints"]["selection_reasons"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !selection_reasons.is_empty() {
        println!();
        println!("Why these entrypoints:");
        for item in selection_reasons.iter().take(3) {
            println!(
                "  {} [{}] {}",
                item["target"].as_str().unwrap_or("-"),
                item["kind"].as_str().unwrap_or("-"),
                item["why"].as_str().unwrap_or("-"),
            );
        }
    }

    let execution_templates = payload["entrypoints"]["execution_templates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    print_execution_templates(&execution_templates);

    print_context_sections(decision, &payload["layers"]);
}

#[cfg(test)]
#[path = "decision_brief_output_tests.rs"]
mod tests;
