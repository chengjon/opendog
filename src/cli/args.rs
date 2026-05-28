use clap::Parser;

use super::config_commands::ConfigCommand;
use super::governance_commands::GovernanceCommand;
use super::report_commands::ReportCommand;
use super::self_update_commands::SelfUpdateCommand;

#[derive(Parser)]
#[command(
    name = "opendog",
    version,
    about = "Multi-project file monitor for AI workflows"
)]
pub(super) enum Cli {
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
