use super::error_output::format_error_lines;
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

#[test]
fn format_error_lines_generic_error_is_single_line() {
    let lines = format_error_lines(&OpenDogError::ProjectNotFound("my-proj".into()));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("Error:"));
    assert!(lines[0].contains("my-proj"));
}

#[test]
fn format_error_lines_invalid_input_shows_message() {
    let lines = format_error_lines(&OpenDogError::InvalidInput("bad value".into()));
    assert!(lines[0].contains("bad value"));
}

#[test]
fn format_error_lines_daemon_unavailable_is_generic() {
    let lines = format_error_lines(&OpenDogError::DaemonUnavailable);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("Error:"));
}

#[test]
fn format_error_lines_database_error_includes_detail() {
    let lines = format_error_lines(&OpenDogError::Database(
        rusqlite::Error::InvalidParameterName("x".into()),
    ));
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("Error:"));
    assert!(lines[0].contains("x"));
}
