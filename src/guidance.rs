use crate::config::ProjectInfo;
use crate::core::project::ProjectManager;
use crate::mcp::{
    agent_guidance_payload, build_governance_layer, collect_project_guidance_context,
    collect_workspace_data_risk_summaries, decision_brief_payload,
    workspace_data_risk_overview_payload, ProjectGuidanceData,
};
use crate::storage::queries::{self, StatsEntry};
use serde_json::Value;

pub(crate) fn trim_agent_guidance_payload(payload: &mut Value, top: usize) {
    if let Some(items) = payload["guidance"]["project_recommendations"].as_array_mut() {
        items.truncate(top);
    }
    if let Some(items) = payload["guidance"]["layers"]["execution_strategy"]
        ["project_recommendations"]
        .as_array_mut()
    {
        items.truncate(top);
    }
    if let Some(items) = payload["guidance"]["layers"]["multi_project_portfolio"]
        ["priority_candidates"]
        .as_array_mut()
    {
        items.truncate(top);
    }
    if let Some(items) =
        payload["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"].as_array_mut()
    {
        items.truncate(top);
    }
    if let Some(items) =
        payload["guidance"]["layers"]["multi_project_portfolio"]["project_overviews"].as_array_mut()
    {
        items.truncate(top);
    }
}

pub(crate) fn guidance_notes(monitored_projects: &[String]) -> Vec<String> {
    if monitored_projects.is_empty() {
        vec![
            "No projects are currently marked as monitoring; start one before relying on activity stats."
                .to_string(),
        ]
    } else {
        vec![format!(
            "Currently monitored projects: {}",
            monitored_projects.join(", ")
        )]
    }
}

pub(crate) fn load_project_guidance_data(
    pm: &ProjectManager,
    project: &ProjectInfo,
) -> ProjectGuidanceData {
    let mut data = ProjectGuidanceData::default();

    if let Ok(db) = pm.open_project_db(&project.id) {
        if let Ok(summary) = crate::core::stats::get_summary(&db) {
            data.total_files = summary.total_files;
            data.accessed_files = summary.accessed_files;
            data.unused_files = summary.unused_files;
        }
        data.stats_entries = crate::core::stats::get_stats(&db).unwrap_or_default();
        data.verification_runs = crate::mcp::latest_verification_runs_for_project(&db);
        data.latest_snapshot_captured_at = queries::list_snapshot_runs(&db, 1)
            .ok()
            .and_then(|runs| runs.into_iter().next().map(|run| run.captured_at));
    }

    data
}

pub(crate) fn build_agent_guidance_for_projects<F>(
    pm: &ProjectManager,
    projects: &[ProjectInfo],
    top: usize,
    load_project_state: F,
) -> Value
where
    F: FnMut(&ProjectInfo) -> ProjectGuidanceData,
{
    let (monitored_projects, recommendations, project_overviews) =
        collect_project_guidance_context(projects, load_project_state);
    let notes = guidance_notes(&monitored_projects);

    let governance = {
        let dbs: Vec<(String, crate::storage::database::Database)> = projects
            .iter()
            .filter_map(|p| pm.open_project_db(&p.id).ok().map(|db| (p.id.clone(), db)))
            .collect();
        let refs: Vec<(&String, &crate::storage::database::Database)> =
            dbs.iter().map(|(id, db)| (id, db)).collect();
        build_governance_layer(&refs)
    };

    let mut payload = agent_guidance_payload(
        projects.len(),
        monitored_projects.len(),
        &monitored_projects,
        &notes,
        &recommendations,
        &project_overviews,
        governance,
    );
    trim_agent_guidance_payload(&mut payload, top.max(1));
    payload
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_decision_brief_for_projects<F, G>(
    pm: &ProjectManager,
    schema_version: &str,
    scope: &str,
    selected_project_id: Option<&str>,
    projects: &[ProjectInfo],
    top: usize,
    load_project_state: F,
    load_workspace_entries: G,
) -> Value
where
    F: FnMut(&ProjectInfo) -> ProjectGuidanceData,
    G: FnMut(&ProjectInfo) -> Vec<StatsEntry>,
{
    let agent_guidance = build_agent_guidance_for_projects(pm, projects, top, load_project_state);
    let pm_ref = pm;
    let mut summaries =
        collect_workspace_data_risk_summaries(projects, "all", "low", load_workspace_entries, |project_id: &str| {
            pm_ref.open_project_db(project_id).ok()
        });
    summaries.truncate(top.max(1));
    let workspace_data_guidance = workspace_data_risk_overview_payload(&summaries, projects.len());

    decision_brief_payload(
        schema_version,
        scope,
        selected_project_id,
        top.max(1),
        &agent_guidance,
        Some(&workspace_data_guidance),
    )
}
