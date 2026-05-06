# Toolchain Confidence Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Status note (2026-05-06):** The implementation in this repository is already landed. The checklist below is backfilled to match the current code/test state plus fresh verification rerun on 2026-05-06; the original red-phase failure output was not re-executed in this session because the worktree already contained the finished implementation.

**Goal:** Refine `mixed_workspace` and `mono_repo` confidence labels so existing toolchain outputs better reflect signal trust without changing detection scope, commands, or payload shape.

**Architecture:** Keep all logic inside `src/mcp/toolchain.rs`, add small confidence helpers that reuse existing marker functions, and tighten workspace aggregation to treat `medium-high` as trusted. Drive the change with focused project-toolchain and workspace-aggregation tests before editing production code.

**Tech Stack:** Rust 2021, `cargo test`, OPENDOG Phase 6 guidance/toolchain tests

---

### Task 1: Write The Failing Toolchain Confidence Tests

**Files:**
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

- [x] **Step 1: Add failing tests for the five confidence scenarios**

```rust
#[test]
fn mixed_workspace_without_workspace_corroboration_stays_medium() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"demo","version":"1.0.0"}"#,
    )
    .unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mixed_workspace"));
    assert_eq!(value["confidence"], json!("medium"));
}

#[test]
fn mixed_workspace_with_workspace_corroboration_becomes_medium_high() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{"name":"demo","private":true,"workspaces":["apps/*"]}"#,
    )
    .unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mixed_workspace"));
    assert_eq!(value["confidence"], json!("medium-high"));
}

#[test]
fn generic_mono_repo_with_only_workspace_marker_becomes_low_confidence() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("pnpm-workspace.yaml"), "packages:\n  - apps/*\n").unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("mono_repo"));
    assert_eq!(value["confidence"], json!("low"));
}
```

- [x] **Step 2: Add failing workspace aggregation test for trusted `medium-high`**

```rust
#[test]
fn workspace_toolchain_aggregation_treats_medium_high_as_trusted() {
    let value = agent_guidance_payload(
        2,
        1,
        &["hybrid".to_string()],
        &[],
        &[
            json!({
                "project_id": "hybrid",
                "recommended_next_action": "inspect_hot_files",
                "reason": "Activity exists.",
                "confidence": "medium",
                "recommended_flow": ["Inspect the hottest files first."]
            }),
            json!({
                "project_id": "mystery",
                "recommended_next_action": "take_snapshot",
                "reason": "Needs baseline.",
                "confidence": "low",
                "recommended_flow": ["Take a snapshot first."]
            }),
        ],
        &[
            workspace_toolchain_overview(
                "hybrid",
                "mixed_workspace",
                "medium-high",
                &["cargo test", "npm test"],
                &[],
                &[],
            ),
            workspace_toolchain_overview("mystery", "unknown", "low", &[], &[], &[]),
        ],
    );

    let layer = &value["guidance"]["layers"]["project_toolchain"];
    assert!(!layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "hybrid"));
}
```

- [x] **Step 3: Run tests to verify the new cases fail**

Run: `cargo test toolchain -- --nocapture`
Expected: FAIL because `mixed_workspace` and generic `mono_repo` still use coarse `medium`, and workspace aggregation still treats every non-`high` project as low-confidence.

### Task 2: Implement Confidence Refinement In `toolchain.rs`

**Files:**
- Modify: `src/mcp/toolchain.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

- [x] **Step 1: Add minimal confidence helpers**

```rust
fn workspace_signal_present(root: &Path) -> bool {
    cargo_toml_has_workspace(root) || node_workspace_marker_exists(root) || file_exists(root, "go.work")
}

fn mixed_workspace_confidence(root: &Path, stacks: &[&'static str]) -> &'static str {
    if stacks.len() > 1 && workspace_signal_present(root) {
        "medium-high"
    } else {
        "medium"
    }
}

fn generic_mono_repo_confidence(stacks: &[&'static str]) -> &'static str {
    if stacks.is_empty() {
        "low"
    } else {
        "medium"
    }
}

fn toolchain_confidence_is_trusted(confidence: &str) -> bool {
    matches!(confidence, "high" | "medium-high")
}
```

- [x] **Step 2: Wire `mixed_workspace` and generic `mono_repo` through those helpers**

```rust
fn mixed_workspace_profile(root: &Path, stacks: &[&'static str]) -> ProjectToolchainProfile {
    // existing command merge stays unchanged
    ProjectToolchainProfile {
        project_type: "mixed_workspace".to_string(),
        confidence: mixed_workspace_confidence(root, stacks),
        test_commands,
        lint_commands,
        build_commands,
        search_commands,
    }
}

ProjectToolchainProfile {
    project_type: "mono_repo".to_string(),
    confidence: generic_mono_repo_confidence(stacks),
    test_commands: vec![],
    lint_commands: vec![],
    build_commands: vec![],
    search_commands: vec![
        "rg \"<pattern>\" .".to_string(),
        "git diff".to_string(),
        "git status".to_string(),
    ],
}
```

- [x] **Step 3: Tighten workspace aggregation trust filtering**

```rust
if (!toolchain_confidence_is_trusted(confidence)) || project_type == "unknown" {
    low_confidence_projects.push(json!({
        "project_id": project_id,
        "project_type": project_type,
        "confidence": confidence,
    }));
}
```

- [x] **Step 4: Run the focused tests to verify they pass**

Run: `cargo test toolchain -- --nocapture`
Expected: PASS for the new project-toolchain and workspace-aggregation confidence cases.

### Task 3: Full Verification And Governance

**Files:**
- Modify: `docs/superpowers/specs/2026-05-05-toolchain-confidence-design.md`
- Modify: `docs/superpowers/plans/2026-05-05-toolchain-confidence-implementation.md`
- Verify: repo-wide Rust and planning checks

- [x] **Step 1: Run formatting**

Run: `cargo fmt --check`
Expected: PASS

- [x] **Step 2: Run full Rust regression**

Run: `cargo test`
Expected: PASS

- [x] **Step 3: Run lint gate**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

- [x] **Step 4: Run governance validation**

Run: `python3 scripts/validate_planning_governance.py`
Expected: PASS

- [x] **Step 5: Summarize changed files and verification evidence**

```text
- src/mcp/toolchain.rs
- src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs
- src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs
- docs/superpowers/specs/2026-05-05-toolchain-confidence-design.md
- docs/superpowers/plans/2026-05-05-toolchain-confidence-implementation.md
```
