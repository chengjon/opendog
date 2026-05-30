use super::*;

#[test]
fn request_start_monitor_round_trip() {
    let req = ControlRequest::StartMonitor {
        id: "sm".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::StartMonitor { .. }));
}

#[test]
fn request_stop_monitor_round_trip() {
    let req = ControlRequest::StopMonitor {
        id: "st".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::StopMonitor { .. }));
}

#[test]
fn request_take_snapshot_round_trip() {
    let req = ControlRequest::TakeSnapshot {
        id: "ts".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ControlRequest = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ControlRequest::TakeSnapshot { .. }));
}

// ---- ControlResponse round-trip tests ----
