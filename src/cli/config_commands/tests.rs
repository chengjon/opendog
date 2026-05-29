use clap::{error::ErrorKind, CommandFactory, Parser};

#[test]
fn config_cli_rejects_mixed_overwrite_and_incremental_ignore_flags() {
    let result = super::super::Cli::try_parse_from([
        "opendog",
        "config",
        "set-project",
        "--id",
        "demo",
        "--ignore-pattern",
        "logs",
        "--add-ignore-pattern",
        "tmp",
    ]);
    let error = match result {
        Ok(_) => panic!("expected clap to reject mixed overwrite and incremental ignore flags"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}

#[test]
fn config_cli_rejects_inherit_and_incremental_process_flags() {
    let result = super::super::Cli::try_parse_from([
        "opendog",
        "config",
        "set-project",
        "--id",
        "demo",
        "--inherit-process-whitelist",
        "--remove-process",
        "claude",
    ]);
    let error = match result {
        Ok(_) => panic!("expected clap to reject inherit and incremental process flags"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}

#[test]
fn config_cli_help_lists_incremental_flags() {
    let mut command = super::super::Cli::command();
    command.build();
    let config = command
        .find_subcommand_mut("config")
        .expect("config subcommand should exist");
    config.build();
    let set_project = config
        .find_subcommand_mut("set-project")
        .expect("config set-project subcommand should exist");
    let mut help = Vec::new();
    set_project.write_long_help(&mut help).unwrap();
    let help = String::from_utf8(help).unwrap();

    assert!(help.contains("--add-ignore-pattern"));
    assert!(help.contains("--remove-ignore-pattern"));
    assert!(help.contains("--add-process"));
    assert!(help.contains("--remove-process"));
    assert!(help.contains("--retention-policy-json"));
    assert!(help.contains("--inherit-retention"));
}

#[test]
fn config_cli_rejects_inherit_and_retention_policy_json() {
    let result = super::super::Cli::try_parse_from([
        "opendog",
        "config",
        "set-project",
        "--id",
        "demo",
        "--inherit-retention",
        "--retention-policy-json",
        "{}",
    ]);
    let error = match result {
        Ok(_) => panic!("expected clap to reject inherit and retention policy JSON"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
}

#[test]
fn retention_policy_json_accepts_partial_policy_with_defaults() {
    let policy =
        super::parse_retention_policy_json(Some(r#"{"activity_rows_threshold":42}"#.to_string()))
            .unwrap()
            .unwrap();

    assert_eq!(policy.activity_rows_threshold, 42);
    assert_eq!(policy.keep_snapshot_runs, 20);
}
