# Config Incremental Editing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit config list add/remove operations for global and project config updates without changing the existing overwrite semantics.

**Architecture:** Extend `ConfigPatch` and `ProjectConfigPatch` with incremental fields, keep list-merge logic centralized in `src/config/patching.rs`, and thread the richer patch objects through CLI parsing, `ProjectManager`, and the daemon control protocol. Validate behavior with unit tests for merge semantics, CLI tests for conflict rules and help visibility, and integration tests for inherited project materialization and daemon parity.

**Tech Stack:** Rust 2021, `clap`, `serde`, `cargo test`, `cargo clippy`, OPENDOG integration tests

---

## File Structure

- Modify: `src/config.rs`
  - Add incremental fields to `ConfigPatch` and `ProjectConfigPatch`
  - Expand config-facing unit tests
- Modify: `src/config/patching.rs`
  - Normalize incremental inputs
  - Apply overwrite/add/remove semantics in one place
- Modify: `src/cli/config_commands.rs`
  - Add `--add-*` / `--remove-*` flags
  - Add clap conflict rules and help text
  - Build richer patch objects
- Modify: `src/cli/mod.rs`
  - Add parser/help regression tests for config command conflicts
- Modify: `src/core/project.rs`
  - Materialize effective project config before incremental edits on inherited fields
- Modify: `src/control/protocol.rs`
  - Extend inline daemon request fields for incremental config operations
- Modify: `src/control/client/config_ops.rs`
  - Serialize new inline protocol fields from `ConfigPatch` / `ProjectConfigPatch`
- Modify: `src/control/request_handler.rs`
  - Reconstruct richer patch structs from daemon requests
- Modify: `src/control/tests.rs`
  - Cover daemon-side global incremental handling
- Modify: `tests/integration_test/storage_project_snapshot.rs`
  - Cover inherited project override materialization
- Modify: `tests/integration_test/daemon_process_cli.rs`
  - Cover daemon-backed CLI incremental updates

### Task 1: Write The Failing Config And CLI Tests

**Files:**
- Modify: `src/config.rs`
- Modify: `src/cli/mod.rs`
- Test: `src/config.rs`
- Test: `src/cli/mod.rs`

- [ ] **Step 1: Add failing patch-semantics tests in `src/config.rs`**

```rust
#[test]
fn config_patch_empty_detection_counts_incremental_fields() {
    assert!(!ConfigPatch {
        add_ignore_patterns: vec!["logs".to_string()],
        ..Default::default()
    }
    .is_empty());
    assert!(!ProjectConfigPatch {
        remove_process_whitelist: vec!["claude".to_string()],
        ..Default::default()
    }
    .is_empty());
}

#[test]
fn config_patch_supports_incremental_add_and_remove() {
    let updated = apply_global_config_patch(
        &ProjectConfig {
            ignore_patterns: vec!["dist".to_string(), "target".to_string()],
            process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        },
        ConfigPatch {
            ignore_patterns: None,
            process_whitelist: None,
            add_ignore_patterns: vec!["logs".to_string(), "target".to_string()],
            remove_ignore_patterns: vec!["dist".to_string()],
            add_process_whitelist: vec!["roo".to_string()],
            remove_process_whitelist: vec!["claude".to_string()],
        },
    );

    assert_eq!(
        updated.ignore_patterns,
        vec!["target".to_string(), "logs".to_string()]
    );
    assert_eq!(
        updated.process_whitelist,
        vec!["codex".to_string(), "roo".to_string()]
    );
}

#[test]
fn project_config_patch_supports_incremental_override_edits() {
    let updated = apply_project_config_patch(
        &ProjectConfigOverrides {
            ignore_patterns: Some(vec!["dist".to_string(), "target".to_string()]),
            process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        },
        &ProjectConfig {
            ignore_patterns: vec!["dist".to_string(), "target".to_string()],
            process_whitelist: vec!["claude".to_string(), "codex".to_string()],
        },
        ProjectConfigPatch {
            add_ignore_patterns: vec!["logs".to_string()],
            remove_process_whitelist: vec!["claude".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(
        updated.ignore_patterns,
        Some(vec![
            "dist".to_string(),
            "target".to_string(),
            "logs".to_string()
        ])
    );
    assert_eq!(updated.process_whitelist, Some(vec!["codex".to_string()]));
}
```

- [ ] **Step 2: Add failing parser and help tests in `src/cli/mod.rs`**

```rust
#[test]
fn config_cli_rejects_mixed_overwrite_and_incremental_ignore_flags() {
    let error = Cli::try_parse_from([
        "opendog",
        "config",
        "set-project",
        "--id",
        "demo",
        "--ignore-pattern",
        "logs",
        "--add-ignore-pattern",
        "tmp",
    ])
    .unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn config_cli_rejects_inherit_and_incremental_process_flags() {
    let error = Cli::try_parse_from([
        "opendog",
        "config",
        "set-project",
        "--id",
        "demo",
        "--inherit-process-whitelist",
        "--remove-process",
        "claude",
    ])
    .unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn config_cli_help_lists_incremental_flags() {
    use clap::CommandFactory;

    let mut command = Cli::command();
    let mut help = Vec::new();
    command.write_long_help(&mut help).unwrap();
    let text = String::from_utf8(help).unwrap();

    assert!(text.contains("--add-ignore-pattern"));
    assert!(text.contains("--remove-ignore-pattern"));
    assert!(text.contains("--add-process"));
    assert!(text.contains("--remove-process"));
}
```

- [ ] **Step 3: Run focused tests to verify they fail**

Run: `cargo test config_patch --lib`
Expected: FAIL because incremental fields and the new `apply_project_config_patch(...)` signature do not exist yet.

Run: `cargo test config_cli --lib`
Expected: FAIL because the new config flags are not defined and no clap conflict rules exist yet.

### Task 2: Implement Patch Semantics And CLI Construction

**Files:**
- Modify: `src/config.rs`
- Modify: `src/config/patching.rs`
- Modify: `src/cli/config_commands.rs`
- Test: `src/config.rs`
- Test: `src/cli/mod.rs`

- [ ] **Step 1: Extend the patch structs in `src/config.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ConfigPatch {
    #[serde(default)]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub process_whitelist: Option<Vec<String>>,
    #[serde(default)]
    pub add_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub remove_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub add_process_whitelist: Vec<String>,
    #[serde(default)]
    pub remove_process_whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfigPatch {
    #[serde(default)]
    pub ignore_patterns: Option<Vec<String>>,
    #[serde(default)]
    pub process_whitelist: Option<Vec<String>>,
    #[serde(default)]
    pub add_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub remove_ignore_patterns: Vec<String>,
    #[serde(default)]
    pub add_process_whitelist: Vec<String>,
    #[serde(default)]
    pub remove_process_whitelist: Vec<String>,
    #[serde(default)]
    pub inherit_ignore_patterns: bool,
    #[serde(default)]
    pub inherit_process_whitelist: bool,
}
```

- [ ] **Step 2: Centralize overwrite/add/remove logic in `src/config/patching.rs`**

```rust
fn apply_string_list_patch(
    current: &[String],
    replace: Option<Vec<String>>,
    add: Vec<String>,
    remove: Vec<String>,
) -> Vec<String> {
    let mut values = replace.unwrap_or_else(|| current.to_vec());

    if !remove.is_empty() {
        let removed: HashSet<_> = remove.into_iter().collect();
        values.retain(|value| !removed.contains(value));
    }

    for value in add {
        if !values.contains(&value) {
            values.push(value);
        }
    }

    values
}

impl ConfigPatch {
    pub fn is_empty(&self) -> bool {
        self.ignore_patterns.is_none()
            && self.process_whitelist.is_none()
            && self.add_ignore_patterns.is_empty()
            && self.remove_ignore_patterns.is_empty()
            && self.add_process_whitelist.is_empty()
            && self.remove_process_whitelist.is_empty()
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = Some(normalize_string_list(ignore_patterns));
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = Some(normalize_string_list(process_whitelist));
        }
        self.add_ignore_patterns = normalize_string_list(self.add_ignore_patterns);
        self.remove_ignore_patterns = normalize_string_list(self.remove_ignore_patterns);
        self.add_process_whitelist = normalize_string_list(self.add_process_whitelist);
        self.remove_process_whitelist = normalize_string_list(self.remove_process_whitelist);
        self
    }
}

impl ProjectConfigPatch {
    pub fn is_empty(&self) -> bool {
        self.ignore_patterns.is_none()
            && self.process_whitelist.is_none()
            && self.add_ignore_patterns.is_empty()
            && self.remove_ignore_patterns.is_empty()
            && self.add_process_whitelist.is_empty()
            && self.remove_process_whitelist.is_empty()
            && !self.inherit_ignore_patterns
            && !self.inherit_process_whitelist
    }

    pub fn normalized(mut self) -> Self {
        if let Some(ignore_patterns) = self.ignore_patterns.take() {
            self.ignore_patterns = Some(normalize_string_list(ignore_patterns));
        }
        if let Some(process_whitelist) = self.process_whitelist.take() {
            self.process_whitelist = Some(normalize_string_list(process_whitelist));
        }
        self.add_ignore_patterns = normalize_string_list(self.add_ignore_patterns);
        self.remove_ignore_patterns = normalize_string_list(self.remove_ignore_patterns);
        self.add_process_whitelist = normalize_string_list(self.add_process_whitelist);
        self.remove_process_whitelist = normalize_string_list(self.remove_process_whitelist);
        self
    }
}

pub fn apply_global_config_patch(current: &ProjectConfig, patch: ConfigPatch) -> ProjectConfig {
    let patch = patch.normalized();

    ProjectConfig {
        ignore_patterns: apply_string_list_patch(
            &current.ignore_patterns,
            patch.ignore_patterns,
            patch.add_ignore_patterns,
            patch.remove_ignore_patterns,
        ),
        process_whitelist: apply_string_list_patch(
            &current.process_whitelist,
            patch.process_whitelist,
            patch.add_process_whitelist,
            patch.remove_process_whitelist,
        ),
    }
}

pub fn apply_project_config_patch(
    current: &ProjectConfigOverrides,
    effective: &ProjectConfig,
    patch: ProjectConfigPatch,
) -> ProjectConfigOverrides {
    let patch = patch.normalized();
    let ProjectConfigPatch {
        ignore_patterns,
        process_whitelist,
        add_ignore_patterns,
        remove_ignore_patterns,
        add_process_whitelist,
        remove_process_whitelist,
        inherit_ignore_patterns,
        inherit_process_whitelist,
    } = patch;

    let ignore_patterns = if inherit_ignore_patterns {
        None
    } else if let Some(ignore_patterns) = ignore_patterns {
        Some(ignore_patterns)
    } else if add_ignore_patterns.is_empty() && remove_ignore_patterns.is_empty() {
        current.ignore_patterns.clone()
    } else {
        let base = current
            .ignore_patterns
            .as_deref()
            .unwrap_or(&effective.ignore_patterns);
        Some(apply_string_list_patch(
            base,
            None,
            add_ignore_patterns,
            remove_ignore_patterns,
        ))
    };

    let process_whitelist = if inherit_process_whitelist {
        None
    } else if let Some(process_whitelist) = process_whitelist {
        Some(process_whitelist)
    } else if add_process_whitelist.is_empty() && remove_process_whitelist.is_empty() {
        current.process_whitelist.clone()
    } else {
        let base = current
            .process_whitelist
            .as_deref()
            .unwrap_or(&effective.process_whitelist);
        Some(apply_string_list_patch(
            base,
            None,
            add_process_whitelist,
            remove_process_whitelist,
        ))
    };

    ProjectConfigOverrides {
        ignore_patterns,
        process_whitelist,
    }
}
```

- [ ] **Step 3: Add config CLI flags, help text, and patch construction in `src/cli/config_commands.rs`**

```rust
SetGlobal {
    #[arg(long = "ignore-pattern", conflicts_with_all = ["add_ignore_patterns", "remove_ignore_patterns"])]
    ignore_patterns: Vec<String>,
    #[arg(long = "add-ignore-pattern")]
    add_ignore_patterns: Vec<String>,
    #[arg(long = "remove-ignore-pattern")]
    remove_ignore_patterns: Vec<String>,
    #[arg(long = "process", conflicts_with_all = ["add_process_whitelist", "remove_process_whitelist"])]
    process_whitelist: Vec<String>,
    #[arg(long = "add-process")]
    add_process_whitelist: Vec<String>,
    #[arg(long = "remove-process")]
    remove_process_whitelist: Vec<String>,
    #[arg(long)]
    json: bool,
},

SetProject {
    #[arg(short, long)]
    id: String,
    #[arg(long = "ignore-pattern", conflicts_with_all = ["add_ignore_patterns", "remove_ignore_patterns", "inherit_ignore_patterns"])]
    ignore_patterns: Vec<String>,
    #[arg(long = "add-ignore-pattern", conflicts_with = "inherit_ignore_patterns")]
    add_ignore_patterns: Vec<String>,
    #[arg(long = "remove-ignore-pattern", conflicts_with = "inherit_ignore_patterns")]
    remove_ignore_patterns: Vec<String>,
    #[arg(long = "process", conflicts_with_all = ["add_process_whitelist", "remove_process_whitelist", "inherit_process_whitelist"])]
    process_whitelist: Vec<String>,
    #[arg(long = "add-process", conflicts_with = "inherit_process_whitelist")]
    add_process_whitelist: Vec<String>,
    #[arg(long = "remove-process", conflicts_with = "inherit_process_whitelist")]
    remove_process_whitelist: Vec<String>,
    #[arg(long, conflicts_with_all = ["ignore_patterns", "add_ignore_patterns", "remove_ignore_patterns"])]
    inherit_ignore_patterns: bool,
    #[arg(long, conflicts_with_all = ["process_whitelist", "add_process_whitelist", "remove_process_whitelist"])]
    inherit_process_whitelist: bool,
    #[arg(long)]
    json: bool,
},

ConfigPatch {
    ignore_patterns: (!ignore_patterns.is_empty()).then_some(ignore_patterns),
    process_whitelist: (!process_whitelist.is_empty()).then_some(process_whitelist),
    add_ignore_patterns,
    remove_ignore_patterns,
    add_process_whitelist,
    remove_process_whitelist,
}

ProjectConfigPatch {
    ignore_patterns: (!ignore_patterns.is_empty()).then_some(ignore_patterns),
    process_whitelist: (!process_whitelist.is_empty()).then_some(process_whitelist),
    add_ignore_patterns,
    remove_ignore_patterns,
    add_process_whitelist,
    remove_process_whitelist,
    inherit_ignore_patterns,
    inherit_process_whitelist,
}
```

- [ ] **Step 4: Run focused unit tests to verify they pass**

Run: `cargo test config_patch --lib`
Expected: PASS for incremental merge semantics and empty-patch detection.

Run: `cargo test config_cli --lib`
Expected: PASS for clap conflict validation and help-flag visibility.

### Task 3: Write The Failing Project And Daemon Regression Tests

**Files:**
- Modify: `tests/integration_test/storage_project_snapshot.rs`
- Modify: `src/control/tests.rs`
- Modify: `tests/integration_test/daemon_process_cli.rs`
- Test: `tests/integration_test.rs`
- Test: `src/control/tests.rs`

- [ ] **Step 1: Add a failing inherited-materialization test in `tests/integration_test/storage_project_snapshot.rs`**

```rust
#[test]
fn test_project_incremental_process_updates_materialize_inherited_defaults() {
    let (dir, mgr) = setup_manager();
    let project_dir = dir.path().join("myproject");
    ensure_dir(&project_dir);

    mgr.update_global_config(ConfigPatch {
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        ..Default::default()
    })
    .unwrap();
    mgr.create("cfg-test", &project_dir).unwrap();

    let updated = mgr
        .update_project_config(
            "cfg-test",
            ProjectConfigPatch {
                remove_process_whitelist: vec!["claude".to_string()],
                add_process_whitelist: vec!["roo".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(
        updated.project_overrides.process_whitelist,
        Some(vec!["codex".to_string(), "roo".to_string()])
    );
    assert_eq!(
        updated.effective.process_whitelist,
        vec!["codex".to_string(), "roo".to_string()]
    );
}
```

- [ ] **Step 2: Add a failing daemon request test in `src/control/tests.rs`**

```rust
#[test]
fn handle_request_applies_incremental_global_config_updates() {
    let (_dir, mut controller) = test_controller();

    let response = controller.handle_request(ControlRequest::UpdateGlobalConfig {
        ignore_patterns: None,
        process_whitelist: Some(vec!["claude".to_string(), "codex".to_string()]),
        add_ignore_patterns: vec!["logs".to_string()],
        remove_ignore_patterns: vec!["node_modules".to_string()],
        add_process_whitelist: vec!["roo".to_string()],
        remove_process_whitelist: vec!["claude".to_string()],
    });

    match response {
        ControlResponse::GlobalConfigUpdated { result } => {
            assert!(result
                .global_defaults
                .ignore_patterns
                .contains(&"logs".to_string()));
            assert_eq!(
                result.global_defaults.process_whitelist,
                vec!["codex".to_string(), "roo".to_string()]
            );
        }
        other => panic!("unexpected response: {:?}", other),
    }
}
```

- [ ] **Step 3: Extend the daemon CLI smoke test with failing incremental update assertions**

```rust
let update_global_config = run_cli(
    home,
    &[
        "config",
        "set-global",
        "--process",
        "claude",
        "--process",
        "codex",
        "--ignore-pattern",
        "dist",
        "--json",
    ],
);
assert!(update_global_config.status.success(), "{:?}", update_global_config);

let update_project_config = run_cli(
    home,
    &[
        "config",
        "set-project",
        "--id",
        "demo",
        "--remove-process",
        "claude",
        "--add-process",
        "roo",
        "--add-ignore-pattern",
        "logs",
        "--json",
    ],
);
assert!(update_project_config.status.success(), "{:?}", update_project_config);

let update_project_config_json: Value =
    serde_json::from_slice(&update_project_config.stdout).unwrap();
assert_eq!(
    update_project_config_json["project_overrides"]["process_whitelist"],
    serde_json::json!(["codex", "roo"])
);
assert_eq!(
    update_project_config_json["effective"]["ignore_patterns"],
    serde_json::json!(["dist", "logs"])
);
```

- [ ] **Step 4: Run targeted integration tests and confirm they fail**

Run: `cargo test test_project_incremental_process_updates_materialize_inherited_defaults --test integration_test -- --nocapture`
Expected: FAIL because project updates still apply only overwrite semantics.

Run: `cargo test handle_request_applies_incremental_global_config_updates --lib`
Expected: FAIL because the daemon request enum does not carry incremental fields.

Run: `cargo test test_daemon_process_cli_smoke --test integration_test -- --nocapture`
Expected: FAIL because the CLI does not yet accept the new flags through the daemon path.

### Task 4: Implement Project Materialization And Daemon Parity

**Files:**
- Modify: `src/core/project.rs`
- Modify: `src/control/protocol.rs`
- Modify: `src/control/client/config_ops.rs`
- Modify: `src/control/request_handler.rs`
- Test: `tests/integration_test/storage_project_snapshot.rs`
- Test: `src/control/tests.rs`
- Test: `tests/integration_test/daemon_process_cli.rs`

- [ ] **Step 1: Materialize effective config before incremental project updates in `src/core/project.rs`**

```rust
pub fn update_project_config(
    &self,
    id: &str,
    patch: ProjectConfigPatch,
) -> Result<ProjectConfigUpdateResult> {
    if patch.is_empty() {
        return Err(OpenDogError::InvalidInput(
            "project config patch must change at least one field".to_string(),
        ));
    }

    let info = self
        .get(id)?
        .ok_or_else(|| OpenDogError::ProjectNotFound(id.to_string()))?;
    let global_defaults = self.global_config()?;
    let effective_before = resolve_project_config(&global_defaults, &info.config);
    let updated_overrides = apply_project_config_patch(&info.config, &effective_before, patch);
    queries::update_project_config(&self.registry, id, &updated_overrides)?;
    let effective = resolve_project_config(&global_defaults, &updated_overrides);

    Ok(ProjectConfigUpdateResult {
        project_id: id.to_string(),
        global_defaults,
        project_overrides: updated_overrides,
        effective,
        reload: Default::default(),
    })
}
```

- [ ] **Step 2: Extend daemon request fields and patch reconstruction**

```rust
UpdateGlobalConfig {
    ignore_patterns: Option<Vec<String>>,
    process_whitelist: Option<Vec<String>>,
    add_ignore_patterns: Vec<String>,
    remove_ignore_patterns: Vec<String>,
    add_process_whitelist: Vec<String>,
    remove_process_whitelist: Vec<String>,
},
UpdateProjectConfig {
    id: String,
    ignore_patterns: Option<Vec<String>>,
    process_whitelist: Option<Vec<String>>,
    add_ignore_patterns: Vec<String>,
    remove_ignore_patterns: Vec<String>,
    add_process_whitelist: Vec<String>,
    remove_process_whitelist: Vec<String>,
    inherit_ignore_patterns: bool,
    inherit_process_whitelist: bool,
}
```

```rust
match self.send(ControlRequest::UpdateProjectConfig {
    id: id.to_string(),
    ignore_patterns: patch.ignore_patterns,
    process_whitelist: patch.process_whitelist,
    add_ignore_patterns: patch.add_ignore_patterns,
    remove_ignore_patterns: patch.remove_ignore_patterns,
    add_process_whitelist: patch.add_process_whitelist,
    remove_process_whitelist: patch.remove_process_whitelist,
    inherit_ignore_patterns: patch.inherit_ignore_patterns,
    inherit_process_whitelist: patch.inherit_process_whitelist,
})? {
```

```rust
ControlRequest::UpdateGlobalConfig {
    ignore_patterns,
    process_whitelist,
    add_ignore_patterns,
    remove_ignore_patterns,
    add_process_whitelist,
    remove_process_whitelist,
} => match self.update_global_config(ConfigPatch {
    ignore_patterns,
    process_whitelist,
    add_ignore_patterns,
    remove_ignore_patterns,
    add_process_whitelist,
    remove_process_whitelist,
}) {
```

- [ ] **Step 3: Run targeted project and daemon tests to verify they pass**

Run: `cargo test test_project_incremental_process_updates_materialize_inherited_defaults --test integration_test -- --nocapture`
Expected: PASS with persisted project override materialized from the inherited baseline.

Run: `cargo test handle_request_applies_incremental_global_config_updates --lib`
Expected: PASS with daemon request reconstruction preserving incremental fields.

Run: `cargo test test_daemon_process_cli_smoke --test integration_test -- --nocapture`
Expected: PASS with daemon-backed CLI add/remove operations producing the same effective config as direct mode.

### Task 5: Run Full Verification

**Files:**
- Verify: repo-wide Rust compilation, tests, and lint gates

- [ ] **Step 1: Check formatting**

Run: `cargo fmt --check`
Expected: PASS

- [ ] **Step 2: Run the full Rust test suite**

Run: `cargo test`
Expected: PASS

- [ ] **Step 3: Run the lint gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [ ] **Step 4: Validate planning governance**

Run: `python3 scripts/validate_planning_governance.py`
Expected: PASS

- [ ] **Step 5: Capture the final changed-file set for handoff**

```text
src/config.rs
src/config/patching.rs
src/cli/config_commands.rs
src/cli/mod.rs
src/core/project.rs
src/control/protocol.rs
src/control/client/config_ops.rs
src/control/request_handler.rs
src/control/tests.rs
tests/integration_test/storage_project_snapshot.rs
tests/integration_test/daemon_process_cli.rs
```
