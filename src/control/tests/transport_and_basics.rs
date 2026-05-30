use super::*;

#[test]
fn handle_request_lists_no_monitors_initially() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::ListMonitors);

    match response {
        ControlResponse::Monitors { ids } => assert!(ids.is_empty()),
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn decode_control_response_reports_empty_daemon_response_as_integrity_error() {
    let error = decode_control_response(&[]).unwrap_err();

    match error {
        OpenDogError::DaemonResponseIntegrity(message) => {
            assert!(message.contains("without returning a response"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn decode_control_response_reports_truncated_daemon_response_as_integrity_error() {
    let error = decode_control_response(br#"{"DecisionBrief":{"payload":"#).unwrap_err();

    match error {
        OpenDogError::DaemonResponseIntegrity(message) => {
            assert!(message.contains("incomplete JSON response"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn map_connect_error_marks_missing_socket_as_daemon_unavailable() {
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    assert!(matches!(
        map_connect_error_with_liveness(error, false),
        OpenDogError::DaemonUnavailable
    ));
}

#[test]
fn map_connect_error_marks_live_daemon_without_socket_as_control_unavailable() {
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    assert!(matches!(
        map_connect_error_with_liveness(error, true),
        OpenDogError::DaemonControlUnavailable
    ));
}
