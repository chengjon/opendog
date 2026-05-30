use super::*;

#[test]
fn handle_request_stop_reports_not_running_when_missing() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::StopMonitor {
        id: "demo".to_string(),
    });

    match response {
        ControlResponse::Stopped { was_running, .. } => assert!(!was_running),
        other => panic!("unexpected response: {:?}", other),
    }
}

#[test]
fn start_monitor_is_idempotent() {
    let (_dir, mut controller) = test_controller();
    let first = controller.start_monitor("demo").unwrap();
    let second = controller.start_monitor("demo").unwrap();

    assert!(!first.already_running);
    assert!(second.already_running);
    controller.stop_all();
}

#[test]
fn handle_request_start_returns_error_for_unknown_project() {
    let (_dir, mut controller) = test_controller();
    let response = controller.handle_request(ControlRequest::StartMonitor {
        id: "missing".to_string(),
    });

    match response {
        ControlResponse::Error { message } => assert!(message.contains("not found")),
        other => panic!("unexpected response: {:?}", other),
    }
}
