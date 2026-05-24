pub mod output;

mod config_commands;
mod governance_commands;
mod guidance_commands;
mod project_commands;
mod report_commands;
mod self_update_commands;
mod verification_commands;

use crate::core::project::ProjectManager;
use crate::core::retention::{CleanupScope, ProjectDataCleanupRequest};
use crate::core::verification::{ExecuteVerificationInput, RecordVerificationInput};
use crate::error::OpenDogError;
use clap::{Parser, Subcommand};

use self::config_commands::{cmd_config, ConfigCommand};
use self::report_commands::{cmd_report, ReportCommand};
use self::self_update_commands::{cmd_self_update, SelfUpdateCommand};

#[derive(Subcommand)]
enum GovernanceCommand {
    /// Create a new governance lane
    CreateLane {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        lane_id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Insert or update a governance node
    UpsertNode {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        lane_id: String,
        #[arg(long)]
        node_id: String,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        evidence_refs: Option<String>,
        #[arg(long)]
        artifact_refs: Option<String>,
        #[arg(long)]
        reported_git_head: Option<String>,
        #[arg(long)]
        suggested_next: Option<String>,
        #[arg(long)]
        forbidden_scope: Option<String>,
        #[arg(long)]
        external_anchors: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show governance state for a project
    Show {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        lane_id: Option<String>,
        #[arg(long)]
        node_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Close a governance lane (complete, defer, or delete)
    CloseLane {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        lane_id: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Parser)]
#[command(
    name = "opendog",
    version,
    about = "Multi-project file monitor for AI workflows"
)]
enum Cli {
    /// Register an existing project root with OPENDOG
    #[command(alias = "create")]
    Register {
        /// Unique project identifier
        #[arg(short, long)]
        id: String,
        /// Absolute path to project root directory
        #[arg(short, long)]
        path: String,
    },
    /// Trigger a file scan for a project
    Snapshot {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Start monitoring a project (blocks until Ctrl+C)
    Start {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Stop a daemon-managed monitor for a project
    Stop {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Show or mutate OPENDOG configuration defaults and project overrides
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Export project evidence rows to portable JSON or CSV files
    Export {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        format: String,
        #[arg(long, default_value = "stats")]
        view: String,
        #[arg(long)]
        output: String,
        #[arg(long, default_value_t = 5)]
        min_access_count: i64,
    },
    /// Remove retained OPENDOG project evidence selectively
    CleanupData {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        older_than_days: Option<i64>,
        #[arg(long)]
        keep_snapshot_runs: Option<usize>,
        #[arg(long)]
        vacuum: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Query comparative and time-windowed analytics
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Check or rebuild the OpenDog release binary from an explicit source tree
    SelfUpdate {
        #[command(subcommand)]
        command: SelfUpdateCommand,
    },
    /// Manage governance lanes and nodes for a project
    Governance {
        #[command(subcommand)]
        command: GovernanceCommand,
    },
    /// Run as stdio MCP server (for AI clients)
    Mcp,
    /// Show usage statistics for a project
    Stats {
        /// Project identifier
        #[arg(short, long)]
        id: String,
        /// Optional row classification filter: all, source, infrastructure, backup, or project.
        #[arg(long, default_value = "all")]
        path_classification: String,
    },
    /// List never-accessed files (unused candidates)
    Unused {
        /// Project identifier
        #[arg(short, long)]
        id: String,
        /// Optional row classification filter: all, source, infrastructure, backup, or project.
        #[arg(long, default_value = "all")]
        path_classification: String,
    },
    /// List all registered projects
    List,
    /// Show workspace-level AI guidance for what to inspect or verify next
    AgentGuidance {
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 5)]
        top: usize,
        #[arg(long)]
        json: bool,
    },
    /// Show a single AI-facing decision envelope with next action, entrypoints, and 8-layer workspace/project signals
    DecisionBrief {
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 5)]
        top: usize,
        #[arg(long)]
        json: bool,
    },
    /// Show mock and hardcoded-data risk candidates for a project
    DataRisk {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        candidate_type: Option<String>,
        #[arg(long)]
        min_review_priority: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Show workspace-wide mock and hardcoded-data risk overview across projects
    WorkspaceDataRisk {
        #[arg(long)]
        candidate_type: Option<String>,
        #[arg(long)]
        min_review_priority: Option<String>,
        #[arg(long, default_value_t = 20)]
        project_limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Record the latest test/lint/build result for a project
    RecordVerification {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        status: String,
        #[arg(long)]
        command: String,
        #[arg(long)]
        exit_code: Option<i64>,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long, default_value = "cli")]
        source: String,
        #[arg(long)]
        started_at: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show latest recorded test/lint/build results for a project
    Verification {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Execute a test/lint/build command inside the project root and record the result
    RunVerification {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        command: String,
        #[arg(long, default_value = "cli")]
        source: String,
        #[arg(long)]
        json: bool,
    },
    /// Delete a project and all its data
    Delete {
        /// Project identifier
        #[arg(short, long)]
        id: String,
    },
    /// Run as background daemon (for systemd)
    Daemon,
}

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
            GovernanceCommand::CreateLane { id, lane_id, title, description, json } => {
                governance_commands::cmd_create_lane(&pm, &id,
                    crate::core::governance::CreateLaneInput { lane_id, title, description }, json)
            }
            GovernanceCommand::UpsertNode { id, lane_id, node_id, state, summary,
                evidence_refs, artifact_refs, reported_git_head, suggested_next,
                forbidden_scope, external_anchors, json } => {
                let parse_json_list = |s: Option<String>| -> Option<Vec<String>> {
                    s.and_then(|v| serde_json::from_str(&v).ok())
                };
                let parse_json_value = |s: Option<String>| -> Option<serde_json::Value> {
                    s.and_then(|v| serde_json::from_str(&v).ok())
                };
                governance_commands::cmd_upsert_node(&pm, &id,
                    crate::core::governance::UpsertNodeInput {
                        node_id, lane_id, state, summary,
                        evidence_refs: parse_json_list(evidence_refs),
                        artifact_refs: parse_json_list(artifact_refs),
                        reported_git_head,
                        suggested_next,
                        forbidden_scope: parse_json_list(forbidden_scope),
                        external_anchors: parse_json_value(external_anchors),
                    }, json)
            }
            GovernanceCommand::Show { id, lane_id, node_id, json } => {
                governance_commands::cmd_show(&pm, &id,
                    crate::core::governance::GetGovernanceStateInput { lane_id, node_id }, json)
            }
            GovernanceCommand::CloseLane { id, lane_id, action, json } => {
                governance_commands::cmd_close_lane(&pm, &id,
                    crate::core::governance::CloseLaneInput { lane_id, action }, json)
            }
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

fn print_error(error: &OpenDogError) {
    for line in format_error_lines(error) {
        eprintln!("{}", line);
    }
}

fn format_error_lines(error: &OpenDogError) -> Vec<String> {
    match error {
        OpenDogError::DaemonControlUnavailable => vec![
            "Error: daemon appears to be running but the control socket is unavailable."
                .to_string(),
            format!(
                "Hint: check {}, remove a stale socket if needed, or restart `opendog daemon`.",
                crate::config::daemon_socket_path().display()
            ),
            format!(
                "Hint: if the daemon is wedged, inspect {} and restart the daemon cleanly.",
                crate::config::daemon_pid_path().display()
            ),
        ],
        _ => vec![format!("Error: {}", error)],
    }
}

#[cfg(test)]
mod tests {
    use super::format_error_lines;
    use crate::error::OpenDogError;
    use crate::guidance::trim_agent_guidance_payload;
    use serde_json::json;

    #[test]
    fn daemon_control_unavailable_error_includes_recovery_hints() {
        let lines = format_error_lines(&OpenDogError::DaemonControlUnavailable);
        let text = lines.join("\n");

        assert!(text.contains("control socket"));
        assert!(text.contains(".opendog/data/daemon.sock"));
        assert!(text.contains(".opendog/data/daemon.pid"));
        assert!(text.contains("restart `opendog daemon`"));
    }

    #[test]
    fn trim_agent_guidance_payload_limits_priority_lists() {
        let mut payload = json!({
            "guidance": {
                "project_recommendations": [{"project_id":"a"},{"project_id":"b"}],
                "layers": {
                    "execution_strategy": {
                        "project_recommendations": [{"project_id":"a"},{"project_id":"b"}]
                    },
                    "multi_project_portfolio": {
                        "priority_candidates": [{"project_id":"a"},{"project_id":"b"}],
                        "attention_queue": [{"project_id":"a"},{"project_id":"b"}],
                        "project_overviews": [{"project_id":"a"},{"project_id":"b"}]
                    }
                }
            }
        });

        trim_agent_guidance_payload(&mut payload, 1);

        assert_eq!(
            payload["guidance"]["project_recommendations"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            payload["guidance"]["layers"]["execution_strategy"]["project_recommendations"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            payload["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
