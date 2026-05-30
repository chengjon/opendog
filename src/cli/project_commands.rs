use crate::control::{
    CliProjectLifecycle, DaemonClient, DaemonProjectLifecycle, FallbackLifecycle,
};
use crate::core::project::ProjectManager;

mod cleanup;
mod export;
mod lifecycle;
mod observation;

pub(super) use cleanup::cmd_cleanup_data;
pub(super) use export::cmd_export;
pub(super) use lifecycle::{cmd_delete, cmd_list, cmd_register, cmd_snapshot, cmd_start, cmd_stop};
pub(super) use observation::{cmd_stats, cmd_unused};

pub(super) fn project_lifecycle(
    pm: &ProjectManager,
) -> FallbackLifecycle<DaemonProjectLifecycle<'static>, CliProjectLifecycle<'_>> {
    static DAEMON: std::sync::OnceLock<DaemonClient> = std::sync::OnceLock::new();
    let client = DAEMON.get_or_init(DaemonClient::new);
    FallbackLifecycle::new(
        DaemonProjectLifecycle::new(client),
        CliProjectLifecycle::new(pm),
    )
}
