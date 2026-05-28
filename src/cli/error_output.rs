use crate::error::OpenDogError;

pub(super) fn print_error(error: &OpenDogError) {
    for line in format_error_lines(error) {
        eprintln!("{}", line);
    }
}

pub(super) fn format_error_lines(error: &OpenDogError) -> Vec<String> {
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
