use clap::Subcommand;

use crate::config::{ConfigPatch, ProjectConfigPatch};
use crate::core::project::ProjectManager;
use crate::error::OpenDogError;

mod handlers;

use handlers::{
    cmd_config_reload, cmd_config_set_global, cmd_config_set_project, cmd_config_show,
    parse_retention_policy_json,
};

#[derive(Subcommand)]
pub(super) enum ConfigCommand {
    /// Show effective configuration for one project, or global defaults when no project is supplied
    Show {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Update per-project override fields
    SetProject {
        #[arg(short, long)]
        id: String,
        #[arg(
            long = "ignore-pattern",
            conflicts_with_all = [
                "add_ignore_patterns",
                "remove_ignore_patterns",
                "inherit_ignore_patterns"
            ]
        )]
        ignore_patterns: Vec<String>,
        #[arg(
            long = "add-ignore-pattern",
            conflicts_with = "inherit_ignore_patterns"
        )]
        add_ignore_patterns: Vec<String>,
        #[arg(
            long = "remove-ignore-pattern",
            conflicts_with = "inherit_ignore_patterns"
        )]
        remove_ignore_patterns: Vec<String>,
        #[arg(
            long = "process",
            conflicts_with_all = [
                "add_process_whitelist",
                "remove_process_whitelist",
                "inherit_process_whitelist"
            ]
        )]
        process_whitelist: Vec<String>,
        #[arg(long = "add-process", conflicts_with = "inherit_process_whitelist")]
        add_process_whitelist: Vec<String>,
        #[arg(long = "remove-process", conflicts_with = "inherit_process_whitelist")]
        remove_process_whitelist: Vec<String>,
        #[arg(
            long,
            conflicts_with_all = [
                "ignore_patterns",
                "add_ignore_patterns",
                "remove_ignore_patterns"
            ]
        )]
        inherit_ignore_patterns: bool,
        #[arg(
            long,
            conflicts_with_all = [
                "process_whitelist",
                "add_process_whitelist",
                "remove_process_whitelist"
            ]
        )]
        inherit_process_whitelist: bool,
        #[arg(long = "retention-policy-json", conflicts_with = "inherit_retention")]
        retention_policy_json: Option<String>,
        #[arg(long, conflicts_with = "retention_policy_json")]
        inherit_retention: bool,
        #[arg(long)]
        json: bool,
    },
    /// Update global default configuration
    SetGlobal {
        #[arg(
            long = "ignore-pattern",
            conflicts_with_all = ["add_ignore_patterns", "remove_ignore_patterns"]
        )]
        ignore_patterns: Vec<String>,
        #[arg(long = "add-ignore-pattern")]
        add_ignore_patterns: Vec<String>,
        #[arg(long = "remove-ignore-pattern")]
        remove_ignore_patterns: Vec<String>,
        #[arg(
            long = "process",
            conflicts_with_all = ["add_process_whitelist", "remove_process_whitelist"]
        )]
        process_whitelist: Vec<String>,
        #[arg(long = "add-process")]
        add_process_whitelist: Vec<String>,
        #[arg(long = "remove-process")]
        remove_process_whitelist: Vec<String>,
        #[arg(long = "retention-policy-json")]
        retention_policy_json: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Re-apply persisted configuration to a running daemon-managed monitor
    Reload {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
}

pub(super) fn cmd_config(pm: &ProjectManager, command: ConfigCommand) -> Result<(), OpenDogError> {
    match command {
        ConfigCommand::Show { id, json } => cmd_config_show(pm, id, json),
        ConfigCommand::SetProject {
            id,
            ignore_patterns,
            add_ignore_patterns,
            remove_ignore_patterns,
            process_whitelist,
            add_process_whitelist,
            remove_process_whitelist,
            inherit_ignore_patterns,
            inherit_process_whitelist,
            retention_policy_json,
            inherit_retention,
            json,
        } => {
            let retention = parse_retention_policy_json(retention_policy_json)?;
            cmd_config_set_project(
                pm,
                &id,
                ProjectConfigPatch {
                    ignore_patterns: if ignore_patterns.is_empty() {
                        None
                    } else {
                        Some(ignore_patterns)
                    },
                    add_ignore_patterns,
                    remove_ignore_patterns,
                    process_whitelist: if process_whitelist.is_empty() {
                        None
                    } else {
                        Some(process_whitelist)
                    },
                    retention,
                    add_process_whitelist,
                    remove_process_whitelist,
                    inherit_ignore_patterns,
                    inherit_process_whitelist,
                    inherit_retention,
                },
                json,
            )
        }
        ConfigCommand::SetGlobal {
            ignore_patterns,
            add_ignore_patterns,
            remove_ignore_patterns,
            process_whitelist,
            add_process_whitelist,
            remove_process_whitelist,
            retention_policy_json,
            json,
        } => {
            let retention = parse_retention_policy_json(retention_policy_json)?;
            cmd_config_set_global(
                pm,
                ConfigPatch {
                    ignore_patterns: if ignore_patterns.is_empty() {
                        None
                    } else {
                        Some(ignore_patterns)
                    },
                    add_ignore_patterns,
                    remove_ignore_patterns,
                    process_whitelist: if process_whitelist.is_empty() {
                        None
                    } else {
                        Some(process_whitelist)
                    },
                    retention,
                    add_process_whitelist,
                    remove_process_whitelist,
                },
                json,
            )
        }
        ConfigCommand::Reload { id, json } => cmd_config_reload(pm, &id, json),
    }
}

#[cfg(test)]
mod tests;
