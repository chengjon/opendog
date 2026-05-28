pub mod output;

mod args;
mod config_commands;
mod error_output;
mod governance_commands;
mod guidance_commands;
mod project_commands;
mod report_commands;
mod self_update_commands;
mod verification_commands;

use crate::core::project::ProjectManager;
use crate::core::retention::{CleanupScope, ProjectDataCleanupRequest};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};
use clap::Parser;

use self::args::Cli;
use self::config_commands::cmd_config;
use self::error_output::print_error;
use self::governance_commands::GovernanceCommand;
use self::report_commands::cmd_report;
use self::self_update_commands::cmd_self_update;

pub fn run() {
    let cli = Cli::parse();
    let pm = ProjectManager::new().unwrap_or_else(|e| {
        eprintln!("Error: failed to initialize — {}", e);
        std::process::exit(1);
    });

    let result = match cli {
        Cli::Register { id, path } => project_commands::cmd_register(&pm, &id, &path),
        Cli::Snapshot { id } => project_commands::cmd_snapshot(&pm, &id),
        Cli::Start { id } => project_commands::cmd_start(&pm, &id),
        Cli::Stop { id } => project_commands::cmd_stop(&id),
        Cli::Config { command } => cmd_config(&pm, command),
        Cli::Export {
            id,
            format,
            view,
            output,
            min_access_count,
        } => project_commands::cmd_export(&pm, &id, &format, &view, &output, min_access_count),
        Cli::CleanupData {
            id,
            scope,
            older_than_days,
            keep_snapshot_runs,
            vacuum,
            dry_run,
            json,
        } => CleanupScope::parse(&scope).and_then(|scope| {
            project_commands::cmd_cleanup_data(
                &pm,
                &id,
                ProjectDataCleanupRequest {
                    scope,
                    older_than_days,
                    keep_snapshot_runs,
                    vacuum,
                    dry_run,
                },
                json,
            )
        }),
        Cli::Report { command } => cmd_report(&pm, command),
        Cli::SelfUpdate { command } => cmd_self_update(command),
        Cli::Governance { command } => match command {
            GovernanceCommand::CreateLane {
                id,
                lane_id,
                title,
                description,
                json,
            } => governance_commands::cmd_create_lane(
                &pm,
                &id,
                crate::core::governance::CreateLaneInput {
                    lane_id,
                    title,
                    description,
                },
                json,
            ),
            GovernanceCommand::UpsertNode {
                id,
                lane_id,
                node_id,
                state,
                summary,
                evidence_refs,
                artifact_refs,
                reported_git_head,
                suggested_next,
                forbidden_scope,
                external_anchors,
                json,
            } => {
                let parse_json_list = |s: Option<String>| -> Option<Vec<String>> {
                    s.and_then(|v| serde_json::from_str(&v).ok())
                };
                let parse_json_value = |s: Option<String>| -> Option<serde_json::Value> {
                    s.and_then(|v| serde_json::from_str(&v).ok())
                };
                governance_commands::cmd_upsert_node(
                    &pm,
                    &id,
                    crate::core::governance::UpsertNodeInput {
                        node_id,
                        lane_id,
                        state,
                        summary,
                        evidence_refs: parse_json_list(evidence_refs),
                        artifact_refs: parse_json_list(artifact_refs),
                        reported_git_head,
                        suggested_next,
                        forbidden_scope: parse_json_list(forbidden_scope),
                        external_anchors: parse_json_value(external_anchors),
                    },
                    json,
                )
            }
            GovernanceCommand::Show {
                id,
                lane_id,
                node_id,
                active_only,
                json,
            } => governance_commands::cmd_show(
                &pm,
                &id,
                crate::core::governance::GetGovernanceStateInput {
                    lane_id,
                    node_id,
                    active_only,
                },
                json,
            ),
            GovernanceCommand::CloseLane {
                id,
                lane_id,
                action,
                json,
            } => governance_commands::cmd_close_lane(
                &pm,
                &id,
                crate::core::governance::CloseLaneInput { lane_id, action },
                json,
            ),
        },
        Cli::Mcp => {
            crate::mcp::run_stdio();
            return;
        }
        Cli::Stats {
            id,
            path_classification,
        } => project_commands::cmd_stats(&pm, &id, &path_classification),
        Cli::Unused {
            id,
            path_classification,
        } => project_commands::cmd_unused(&pm, &id, &path_classification),
        Cli::List => project_commands::cmd_list(&pm),
        Cli::AgentGuidance { project, top, json } => {
            guidance_commands::cmd_agent_guidance(&pm, project, top, json)
        }
        Cli::DecisionBrief { project, top, json } => {
            guidance_commands::cmd_decision_brief(&pm, project, top, json)
        }
        Cli::DataRisk {
            id,
            candidate_type,
            min_review_priority,
            limit,
            json,
        } => guidance_commands::cmd_data_risk(
            &pm,
            &id,
            candidate_type,
            min_review_priority,
            limit,
            json,
        ),
        Cli::WorkspaceDataRisk {
            candidate_type,
            min_review_priority,
            project_limit,
            json,
        } => guidance_commands::cmd_workspace_data_risk(
            &pm,
            candidate_type,
            min_review_priority,
            project_limit,
            json,
        ),
        Cli::RecordVerification {
            id,
            kind,
            status,
            command,
            exit_code,
            summary,
            source,
            started_at,
            json,
        } => verification_commands::cmd_record_verification(
            &pm,
            &id,
            RecordVerificationInput {
                kind,
                status,
                command,
                exit_code,
                summary,
                source,
                started_at,
            },
            json,
        ),
        Cli::Verification { id, json } => verification_commands::cmd_verification(&pm, &id, json),
        Cli::RunVerification {
            id,
            kind,
            command,
            source,
            json,
        } => verification_commands::cmd_run_verification(
            &pm,
            &id,
            ExecuteVerificationInput {
                kind,
                command,
                source,
            },
            json,
        ),
        Cli::Delete { id } => project_commands::cmd_delete(&pm, &id),
        Cli::Daemon => {
            crate::daemon::run();
            return;
        }
    };

    if let Err(e) = result {
        print_error(&e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests;
