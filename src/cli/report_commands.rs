use clap::Subcommand;

use crate::contracts::{CLI_SNAPSHOT_COMPARE_V1, CLI_TIME_WINDOW_REPORT_V1, CLI_USAGE_TRENDS_V1};
use crate::control::DaemonClient;
use crate::core::project::ProjectManager;
use crate::core::report::{self, ReportWindow};
use crate::error::OpenDogError;
use crate::mcp::{snapshot_comparison_payload, time_window_report_payload, usage_trends_payload};

use super::output;

#[derive(Subcommand)]
pub(super) enum ReportCommand {
    /// Show activity statistics for one time window (24h, 7d, 30d)
    Window {
        #[arg(short, long)]
        id: String,
        #[arg(long, default_value = "24h")]
        window: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Compare two snapshot runs, or the latest two when run ids are omitted
    Compare {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        base_run_id: Option<i64>,
        #[arg(long)]
        head_run_id: Option<i64>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Show bucketed usage trends for one time window (24h, 7d, 30d)
    Trend {
        #[arg(short, long)]
        id: String,
        #[arg(long, default_value = "7d")]
        window: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

pub(super) fn cmd_report(pm: &ProjectManager, command: ReportCommand) -> Result<(), OpenDogError> {
    match command {
        ReportCommand::Window {
            id,
            window,
            limit,
            json,
        } => cmd_report_window(pm, &id, &window, limit, json),
        ReportCommand::Compare {
            id,
            base_run_id,
            head_run_id,
            limit,
            json,
        } => cmd_report_compare(pm, &id, base_run_id, head_run_id, limit, json),
        ReportCommand::Trend {
            id,
            window,
            limit,
            json,
        } => cmd_report_trend(pm, &id, &window, limit, json),
    }
}

fn cmd_report_window(
    pm: &ProjectManager,
    id: &str,
    window: &str,
    limit: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let window = ReportWindow::parse(window)?;
    let daemon = DaemonClient::new();
    let report = match daemon.get_time_window_report(id, window, limit) {
        Ok(report) => report,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            report::get_time_window_report(&db, window, limit)?
        }
        Err(error) => return Err(error),
    };

    if json_output {
        let payload = time_window_report_payload(CLI_TIME_WINDOW_REPORT_V1, id, &report);
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_time_window_report(id, &report);
    }
    Ok(())
}

fn cmd_report_compare(
    pm: &ProjectManager,
    id: &str,
    base_run_id: Option<i64>,
    head_run_id: Option<i64>,
    limit: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let daemon = DaemonClient::new();
    let comparison = match daemon.compare_snapshots(id, base_run_id, head_run_id, limit) {
        Ok(comparison) => comparison,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            match (base_run_id, head_run_id) {
                (None, None) => report::compare_latest_snapshots(&db, limit)?,
                (Some(base_run_id), Some(head_run_id)) => {
                    report::compare_snapshot_runs(&db, base_run_id, head_run_id, limit)?
                }
                _ => {
                    return Err(OpenDogError::InvalidInput(
                        "base_run_id and head_run_id must be provided together".to_string(),
                    ));
                }
            }
        }
        Err(error) => return Err(error),
    };

    if json_output {
        let payload = snapshot_comparison_payload(CLI_SNAPSHOT_COMPARE_V1, id, &comparison);
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_snapshot_comparison(id, &comparison);
    }
    Ok(())
}

fn cmd_report_trend(
    pm: &ProjectManager,
    id: &str,
    window: &str,
    limit: usize,
    json_output: bool,
) -> Result<(), OpenDogError> {
    let window = ReportWindow::parse(window)?;
    let daemon = DaemonClient::new();
    let report = match daemon.get_usage_trends(id, window, limit) {
        Ok(report) => report,
        Err(OpenDogError::DaemonUnavailable) => {
            let db = pm.open_project_db(id)?;
            report::get_usage_trend_report(&db, window, limit)?
        }
        Err(error) => return Err(error),
    };

    if json_output {
        let payload = usage_trends_payload(CLI_USAGE_TRENDS_V1, id, &report);
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        output::print_usage_trends(id, &report);
    }
    Ok(())
}
