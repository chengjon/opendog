use crate::contracts::{CLI_DATA_RISK_V1, CLI_DECISION_BRIEF_V1, CLI_WORKSPACE_DATA_RISK_V1};
use crate::control::{DaemonClient, Guidance};
use crate::core::project::ProjectManager;
use crate::core::stats;
use crate::error::OpenDogError;
use crate::mcp::{
    normalize_candidate_type, normalize_min_review_priority, project_data_risk_payload,
    workspace_data_risk_payload,
};

use super::output;
use super::project_commands::project_lifecycle;

pub(super) fn cmd_agent_guidance(
    pm: &ProjectManager,
    project: Option<String>,
    top: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let guidance = svc.get_agent_guidance(project.as_deref(), top.max(1))?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&guidance)?);
    } else {
        output::print_agent_guidance(&guidance["guidance"]);
    }
    Ok(())
}

pub(super) fn cmd_decision_brief(
    pm: &ProjectManager,
    project: Option<String>,
    top: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let svc = project_lifecycle(pm);
    let payload = svc.get_decision_brief(CLI_DECISION_BRIEF_V1, project.as_deref(), top.max(1))?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_decision_brief(&payload);
    }
    Ok(())
}

pub(super) fn cmd_data_risk(
    pm: &ProjectManager,
    id: &str,
    candidate_type: Option<String>,
    min_review_priority: Option<String>,
    limit: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let candidate_type = normalize_candidate_type(candidate_type).map_err(|error| {
        OpenDogError::InvalidInput(error["error"].as_str().unwrap().to_string())
    })?;
    let min_review_priority =
        normalize_min_review_priority(min_review_priority).map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap().to_string())
        })?;
    let limit = limit.max(1);

    let daemon = DaemonClient::new();
    match daemon.get_data_risk_candidates(
        id,
        &candidate_type,
        &min_review_priority,
        limit,
        CLI_DATA_RISK_V1,
    ) {
        Ok(payload) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                output::print_data_risk(
                    id,
                    payload["candidate_type"].as_str().unwrap_or("all"),
                    payload["min_review_priority"].as_str().unwrap_or("low"),
                    &payload,
                    &payload["guidance"],
                );
            }
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let db = pm.open_project_db(id)?;
    let info = pm
        .get(id)?
        .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let entries = stats::get_stats(&db)?;
    let payload = project_data_risk_payload(
        CLI_DATA_RISK_V1,
        id,
        &candidate_type,
        &min_review_priority,
        limit,
        &info.root_path,
        &entries,
    );
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_data_risk(
            id,
            payload["candidate_type"].as_str().unwrap_or("all"),
            payload["min_review_priority"].as_str().unwrap_or("low"),
            &payload,
            &payload["guidance"],
        );
    }
    Ok(())
}

pub(super) fn cmd_workspace_data_risk(
    pm: &ProjectManager,
    candidate_type: Option<String>,
    min_review_priority: Option<String>,
    project_limit: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let candidate_type = normalize_candidate_type(candidate_type).map_err(|error| {
        OpenDogError::InvalidInput(error["error"].as_str().unwrap().to_string())
    })?;
    let min_review_priority =
        normalize_min_review_priority(min_review_priority).map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap().to_string())
        })?;
    let project_limit = project_limit.max(1);

    let daemon = DaemonClient::new();
    match daemon.get_workspace_data_risk_overview(
        &candidate_type,
        &min_review_priority,
        project_limit,
        CLI_WORKSPACE_DATA_RISK_V1,
    ) {
        Ok(payload) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                output::print_workspace_data_risk(
                    payload["candidate_type"].as_str().unwrap_or("all"),
                    payload["min_review_priority"].as_str().unwrap_or("low"),
                    project_limit,
                    payload["total_registered_projects"].as_u64().unwrap_or(0) as usize,
                    payload["matched_project_count"].as_u64().unwrap_or(0) as usize,
                    &payload["guidance"],
                );
            }
            return Ok(());
        }
        Err(OpenDogError::DaemonUnavailable) => {}
        Err(e) => return Err(e),
    }

    let projects = pm.list()?;
    let payload = workspace_data_risk_payload(
        CLI_WORKSPACE_DATA_RISK_V1,
        &projects,
        &candidate_type,
        &min_review_priority,
        project_limit,
        |project| {
            pm.open_project_db(&project.id)
                .ok()
                .and_then(|db| stats::get_stats(&db).ok())
                .unwrap_or_default()
        },
    );
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_workspace_data_risk(
            payload["candidate_type"].as_str().unwrap_or("all"),
            payload["min_review_priority"].as_str().unwrap_or("low"),
            project_limit,
            payload["total_registered_projects"].as_u64().unwrap_or(0) as usize,
            payload["matched_project_count"].as_u64().unwrap_or(0) as usize,
            &payload["guidance"],
        );
    }
    Ok(())
}
