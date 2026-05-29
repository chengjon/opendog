use opendog::contracts::{
    CLI_RECORD_VERIFICATION_V1, CLI_RUN_VERIFICATION_V1, CLI_VERIFICATION_STATUS_V1,
};
use std::path::Path;

use super::super::run_cli_json;

pub(super) fn assert_verification_json_outputs(home: &Path) {
    let record_verification = run_cli_json(
        home,
        &[
            "record-verification",
            "--id",
            "risky",
            "--kind",
            "test",
            "--status",
            "passed",
            "--command",
            "cargo test",
            "--exit-code",
            "0",
            "--summary",
            "all good",
            "--json",
        ],
    );
    assert_eq!(
        record_verification["schema_version"].as_str(),
        Some(CLI_RECORD_VERIFICATION_V1)
    );
    assert_eq!(
        record_verification["recorded"]["kind"].as_str(),
        Some("test")
    );
    assert_eq!(
        record_verification["recorded"]["status"].as_str(),
        Some("passed")
    );

    let verification_status = run_cli_json(home, &["verification", "--id", "risky", "--json"]);
    assert_eq!(
        verification_status["schema_version"].as_str(),
        Some(CLI_VERIFICATION_STATUS_V1)
    );
    assert_eq!(verification_status["project_id"], "risky");
    assert_eq!(
        verification_status["verification"]["latest_runs"][0]["kind"].as_str(),
        Some("test")
    );

    let run_verification = run_cli_json(
        home,
        &[
            "run-verification",
            "--id",
            "clean",
            "--kind",
            "test",
            "--command",
            "printf ok-from-verification",
            "--json",
        ],
    );
    assert_eq!(
        run_verification["schema_version"].as_str(),
        Some(CLI_RUN_VERIFICATION_V1)
    );
    assert_eq!(
        run_verification["executed"]["run"]["status"].as_str(),
        Some("passed")
    );
    assert!(run_verification["executed"]["stdout_tail"]
        .as_str()
        .unwrap_or("")
        .contains("ok-from-verification"));
}
