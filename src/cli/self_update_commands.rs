use std::path::Path;

use clap::Subcommand;

use crate::core::self_update::{run_self_update_build, self_update_status};
use crate::error::OpenDogError;

#[derive(Subcommand)]
pub(super) enum SelfUpdateCommand {
    /// Show whether the OpenDog release binary needs to be rebuilt
    Status {
        /// Explicit OpenDog source tree, for example /opt/claude/opendog
        #[arg(long)]
        source: String,
        /// Print machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Run `cargo build --release` in the explicit OpenDog source tree
    Build {
        /// Explicit OpenDog source tree, for example /opt/claude/opendog
        #[arg(long)]
        source: String,
        /// Print machine-readable JSON after build
        #[arg(long)]
        json: bool,
    },
}

pub(super) fn cmd_self_update(command: SelfUpdateCommand) -> Result<(), OpenDogError> {
    match command {
        SelfUpdateCommand::Status { source, json } => cmd_status(&source, json),
        SelfUpdateCommand::Build { source, json } => cmd_build(&source, json),
    }
}

fn cmd_status(source: &str, json_output: bool) -> Result<(), OpenDogError> {
    let current_exe = std::env::current_exe()?;
    let status = self_update_status(Path::new(source), current_exe)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("OpenDog self-update status:");
        println!("  mode: WSL/Linux shell maintenance command");
        println!("  source: {}", status.source_path);
        println!("  current_exe: {}", status.current_exe);
        println!("  release_binary: {}", status.release_binary);
        println!("  release_binary_exists: {}", status.release_binary_exists);
        println!("  release_binary_mtime: {:?}", status.release_binary_mtime);
        println!("  source_latest_mtime: {:?}", status.source_latest_mtime);
        println!("  needs_rebuild: {}", status.needs_rebuild);
        println!(
            "  restart_required_for_mcp: {}",
            status.restart_required_for_mcp
        );
        println!("  boundary: does not kill MCP hosts or edit host MCP config");
        for step in &status.next_steps {
            println!("  next: {}", step);
        }
    }
    Ok(())
}

fn cmd_build(source: &str, json_output: bool) -> Result<(), OpenDogError> {
    let result = run_self_update_build(Path::new(source))?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("OpenDog self-update build:");
        println!("  source: {}", result.source_path);
        println!("  command: {}", result.command);
        println!("  status: {}", result.status);
        println!("  exit_code: {:?}", result.exit_code);
        println!("  release_binary: {}", result.release_binary);
        println!(
            "  restart_required_for_mcp: {}",
            result.restart_required_for_mcp
        );
        println!("  boundary: did not kill MCP hosts or edit host MCP config");
        for step in &result.next_steps {
            println!("  next: {}", step);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn self_update_cli_accepts_status_and_build_commands() {
        assert!(super::super::Cli::try_parse_from([
            "opendog",
            "self-update",
            "status",
            "--source",
            "/opt/claude/opendog",
        ])
        .is_ok());
        assert!(super::super::Cli::try_parse_from([
            "opendog",
            "self-update",
            "build",
            "--source",
            "/opt/claude/opendog",
            "--json",
        ])
        .is_ok());
    }
}
