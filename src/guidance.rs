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
    let mut summaries = collect_workspace_data_risk_summaries(
        projects,
        "all",
        "low",
        load_workspace_entries,
        |project_id: &str| pm_ref.open_project_db(project_id).ok(),
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- trim_agent_guidance_payload ---

    #[test]
    fn trim_agent_guidance_payload_truncates_project_recommendations() {
        let mut payload = json!({
            "guidance": {
                "project_recommendations": [
                    {"project_id": "a"},
                    {"project_id": "b"},
                    {"project_id": "c"},
                    {"project_id": "d"},
                    {"project_id": "e"},
                ],
                "layers": {
                    "execution_strategy": {
                        "project_recommendations": [
                            {"project_id": "a"},
                            {"project_id": "b"},
                            {"project_id": "c"},
                        ]
                    },
                    "multi_project_portfolio": {
                        "priority_candidates": [
                            {"project_id": "a"},
                            {"project_id": "b"},
                            {"project_id": "c"},
                            {"project_id": "d"},
                        ],
                        "attention_queue": [
                            {"project_id": "a"},
                            {"project_id": "b"},
                            {"project_id": "c"},
                            {"project_id": "d"},
                            {"project_id": "e"},
                        ],
                        "project_overviews": [
                            {"project_id": "a"},
                            {"project_id": "b"},
                            {"project_id": "c"},
                        ]
                    }
                }
            }
        });

        trim_agent_guidance_payload(&mut payload, 2);

        // project_recommendations should be truncated to 2
        let recs = payload["guidance"]["project_recommendations"]
            .as_array()
            .unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0]["project_id"], "a");
        assert_eq!(recs[1]["project_id"], "b");

        // execution_strategy project_recommendations truncated to 2
        let exec_recs = payload["guidance"]["layers"]["execution_strategy"]
            ["project_recommendations"]
            .as_array()
            .unwrap();
        assert_eq!(exec_recs.len(), 2);

        // priority_candidates truncated to 2
        let candidates = payload["guidance"]["layers"]["multi_project_portfolio"]
            ["priority_candidates"]
            .as_array()
            .unwrap();
        assert_eq!(candidates.len(), 2);

        // attention_queue truncated to 2
        let queue = payload["guidance"]["layers"]["multi_project_portfolio"]["attention_queue"]
            .as_array()
            .unwrap();
        assert_eq!(queue.len(), 2);

        // project_overviews truncated to 2
        let overviews = payload["guidance"]["layers"]["multi_project_portfolio"]
            ["project_overviews"]
            .as_array()
            .unwrap();
        assert_eq!(overviews.len(), 2);
    }

    #[test]
    fn trim_agent_guidance_payload_no_op_when_under_limit() {
        let mut payload = json!({
            "guidance": {
                "project_recommendations": [{"project_id": "a"}],
                "layers": {}
            }
        });
        trim_agent_guidance_payload(&mut payload, 5);
        let recs = payload["guidance"]["project_recommendations"]
            .as_array()
            .unwrap();
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn trim_agent_guidance_payload_zero_top() {
        let mut payload = json!({
            "guidance": {
                "project_recommendations": [{"project_id": "a"}, {"project_id": "b"}],
                "layers": {}
            }
        });
        trim_agent_guidance_payload(&mut payload, 0);
        let recs = payload["guidance"]["project_recommendations"]
            .as_array()
            .unwrap();
        assert_eq!(recs.len(), 0);
    }

    #[test]
    fn trim_agent_guidance_payload_missing_paths_is_no_op() {
        let mut payload = json!({"other_key": "value"});
        trim_agent_guidance_payload(&mut payload, 3);
        // Should not panic; payload unchanged except no truncation targets exist
        assert_eq!(payload["other_key"], "value");
    }

    // --- guidance_notes ---

    #[test]
    fn guidance_notes_empty_list_warns_no_monitoring() {
        let notes = guidance_notes(&[]);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("No projects are currently marked as monitoring"));
    }

    #[test]
    fn guidance_notes_single_project() {
        let notes = guidance_notes(&["myproject".to_string()]);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("myproject"));
        assert!(notes[0].contains("Currently monitored projects"));
    }

    #[test]
    fn guidance_notes_multiple_projects() {
        let notes = guidance_notes(&["alpha".to_string(), "beta".to_string(), "gamma".to_string()]);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("alpha"));
        assert!(notes[0].contains("beta"));
        assert!(notes[0].contains("gamma"));
        // Comma-separated
        assert!(notes[0].contains("alpha, beta, gamma"));
    }
}
