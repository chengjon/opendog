# Toolchain Edge-Confidence Tightening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tighten `docs_only` and `unknown` confidence semantics so trusted docs-only repositories stop appearing in low-confidence workspace review while unknown repositories remain conservatively low-confidence.

**Architecture:** Keep the change inside `src/mcp/toolchain.rs`, add one small `docs_only_profile()` helper, reuse the existing trusted-confidence aggregation rule, and verify behavior through focused project-toolchain and workspace-aggregation tests before touching production logic.

**Tech Stack:** Rust 2021, `cargo test`, `cargo clippy`, OPENDOG Phase 6 guidance/toolchain tests

---

### Task 1: Write The Failing Edge-Confidence Tests

**Files:**
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

- [x] **Step 1: Add failing docs-only and unknown profile tests**

```rust
#[test]
fn docs_only_profile_moves_into_medium_high_confidence() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("mkdocs.yml"), "site_name: Demo\n").unwrap();
    std::fs::write(dir.path().join("docs/index.md"), "# Demo\n").unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("docs_only"));
    assert_eq!(value["confidence"], json!("medium-high"));
    assert_eq!(value["recommended_test_commands"], json!([]));
    assert_eq!(value["recommended_lint_commands"], json!([]));
    assert_eq!(value["recommended_build_commands"], json!([]));
    assert_eq!(
        value["recommended_search_commands"],
        json!(["rg \"<pattern>\" docs README.md"])
    );
}

#[test]
fn unknown_profile_stays_low_with_current_fallback_commands() {
    let dir = TempDir::new().unwrap();

    let value = project_toolchain_layer(dir.path());
    assert_eq!(value["project_type"], json!("unknown"));
    assert_eq!(value["confidence"], json!("low"));
    assert_eq!(value["recommended_test_commands"], json!([]));
    assert_eq!(value["recommended_lint_commands"], json!([]));
    assert_eq!(value["recommended_build_commands"], json!([]));
    assert_eq!(
        value["recommended_search_commands"],
        json!(["rg \"<pattern>\" .", "git diff", "git status"])
    );
}
```

- [x] **Step 2: Add failing workspace aggregation test for trusted docs-only**

```rust
#[test]
fn workspace_toolchain_aggregation_treats_docs_only_as_trusted() {
    let value = agent_guidance_payload(
        2,
        1,
        &["docs".to_string()],
        &[],
        &[
            json!({
                "project_id": "docs",
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
            workspace_toolchain_overview("docs", "docs_only", "medium-high", &[], &[], &[]),
            workspace_toolchain_overview("mystery", "unknown", "low", &[], &[], &[]),
        ],
    );

    let layer = &value["guidance"]["layers"]["project_toolchain"];
    assert!(!layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "docs"));
    assert!(layer["low_confidence_projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["project_id"] == "mystery"));
}
```

- [x] **Step 3: Run tests to verify the new cases fail**

Run: `cargo test toolchain -- --nocapture`
Expected: FAIL because `docs_only` still uses `medium`, and the new docs-only trust expectation is not yet implemented.

### Task 2: Implement Minimal Edge-Confidence Tightening

**Files:**
- Modify: `src/mcp/toolchain.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

- [x] **Step 1: Add a dedicated docs-only profile helper**

```rust
fn docs_only_profile() -> ProjectToolchainProfile {
    ProjectToolchainProfile {
        project_type: "docs_only".to_string(),
        confidence: "medium-high",
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec!["rg \"<pattern>\" docs README.md".to_string()],
    }
}
```

- [x] **Step 2: Reuse that helper inside `detect_project_profile(...)`**

```rust
} else if docs_only_marker_exists(root) {
    docs_only_profile()
} else {
    unknown_profile()
}
```

- [x] **Step 3: Keep `unknown_profile()` and trusted-aggregation semantics otherwise unchanged**

```rust
fn unknown_profile() -> ProjectToolchainProfile {
    ProjectToolchainProfile {
        project_type: "unknown".to_string(),
        confidence: "low",
        test_commands: vec![],
        lint_commands: vec![],
        build_commands: vec![],
        search_commands: vec![
            "rg \"<pattern>\" .".to_string(),
            "git diff".to_string(),
            "git status".to_string(),
        ],
    }
}
```

- [x] **Step 4: Run focused tests to verify they pass**

Run: `cargo test toolchain -- --nocapture`
Expected: PASS for docs-only confidence, unknown fallback stability, and workspace low-confidence aggregation behavior.

### Task 3: Full Verification And Governance

**Files:**
- Modify: `docs/superpowers/specs/2026-05-05-toolchain-edge-confidence-design.md`
- Modify: `docs/superpowers/plans/2026-05-05-toolchain-edge-confidence-implementation.md`
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
- docs/superpowers/specs/2026-05-05-toolchain-edge-confidence-design.md
- docs/superpowers/plans/2026-05-05-toolchain-edge-confidence-implementation.md
```
