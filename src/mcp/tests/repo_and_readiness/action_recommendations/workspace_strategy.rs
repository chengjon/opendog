use super::*;

#[test]
fn workspace_strategy_profile_prefers_shell_when_repo_is_unstable() {
    let profile = workspace_strategy_profile(2, 2, false, true, 0);

    assert_eq!(
        profile["global_strategy_mode"],
        json!("stabilize_before_modify")
    );
    assert_eq!(profile["preferred_primary_tool"], json!("shell"));
}
