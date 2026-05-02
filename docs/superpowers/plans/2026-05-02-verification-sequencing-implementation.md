# Verification Sequencing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add machine-readable verification-first `execution_sequence` modes for missing and failing verification actions, then project and summarize them through decision and guidance payloads without changing the existing action enum or CLI text output.

**Architecture:** Extend recommendation-side sequencing so `recommend_project_action(...)` remains the single source of truth for all sequence modes. Reuse the existing `execution_sequence` field, keep repository-stabilization sequencing intact, and add only the minimal workspace summary counts needed for verification-first flows.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, Markdown docs

---

## File Structure

- Modify: `src/mcp/project_recommendation/sequencing.rs`
  - Extend the existing sequencing helper to emit missing-evidence and failing-evidence verification modes
- Modify: `src/mcp/project_recommendation.rs:120-340`
  - Compute project toolchain data once per recommendation and pass the extra sequencing inputs
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_sequence.rs`
  - Recommendation-level regression coverage for verification sequencing modes, command selection, empty-command fallback, and non-sequenced actions
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-16`
  - Register the new recommendation test module
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-190`
  - Add decision-brief regression coverage for verification-sequence projection
- Modify: `src/mcp/guidance_payload.rs:35-120,190-320`
  - Add execution-strategy summary counts for verification-first sequencing
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/verification_sequences.rs`
  - Guidance-layer summary coverage for verification sequencing counts
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-10`
  - Register the new workspace-guidance test module
- Modify: `docs/json-contracts.md:100-220`
  - Document verification sequencing modes and the new guidance summary fields
- Modify: `docs/mcp-tool-reference.md:500-590`
  - Document the new verification-sequencing fields in MCP responses

### Task 1: Recommendation Verification Sequencing

**Files:**
- Modify: `src/mcp/project_recommendation/sequencing.rs`
- Modify: `src/mcp/project_recommendation.rs:120-340`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_sequence.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-16`

- [ ] **Step 1: Write the failing recommendation tests**

```rust
use super::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn rust_project_root() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    dir
}

fn monitoring_state(root: &std::path::Path) -> ProjectGuidanceState {
    ProjectGuidanceState {
        id: "demo".to_string(),
        status: "monitoring".to_string(),
        root_path: root.to_path_buf(),
        total_files: 20,
        accessed_files: 8,
        unused_files: 6,
        latest_snapshot_captured_at: Some(fresh_ts()),
        latest_activity_at: Some(fresh_ts()),
        latest_verification_at: Some(fresh_ts()),
    }
}

fn clean_repo_risk() -> serde_json::Value {
    json!({
        "status": "available",
        "risk_level": "low",
        "is_dirty": false,
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": false
    })
}

#[test]
fn recommend_project_action_emits_missing_verification_sequence_from_toolchain_commands() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(&monitoring_state(root.path()), &clean_repo_risk(), &[]);

    assert_eq!(
        recommendation["recommended_next_action"],
        "run_verification_before_high_risk_changes"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "run_project_verification_then_resume",
            "current_phase": "verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test"],
            "resume_conditions": [
                "required_verification_recorded",
                "verification_evidence_fresh"
            ]
        })
    );
}

#[test]
fn recommend_project_action_emits_failing_verification_sequence_before_repo_stabilization() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &monitoring_state(root.path()),
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 1,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "failed".to_string(),
            command: "cargo test -p api".to_string(),
            exit_code: Some(101),
            summary: Some("test failure".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "review_failing_verification"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": [
                "no_failing_verification_runs",
                "verification_evidence_fresh"
            ]
        })
    );
}

#[test]
fn recommend_project_action_allows_empty_verification_command_lists() {
    let root = TempDir::new().unwrap();
    let recommendation = recommend_project_action(&monitoring_state(root.path()), &clean_repo_risk(), &[]);

    assert_eq!(
        recommendation["execution_sequence"]["verification_commands"],
        json!([])
    );
}

#[test]
fn recommend_project_action_keeps_null_sequence_for_non_sequenced_review_actions() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &monitoring_state(root.path()),
        &clean_repo_risk(),
        &[
            VerificationRun {
                id: 1,
                kind: "test".to_string(),
                status: "passed".to_string(),
                command: "cargo test".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
            VerificationRun {
                id: 2,
                kind: "lint".to_string(),
                status: "passed".to_string(),
                command: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
            VerificationRun {
                id: 3,
                kind: "build".to_string(),
                status: "passed".to_string(),
                command: "cargo check".to_string(),
                exit_code: Some(0),
                summary: Some("ok".to_string()),
                source: "cli".to_string(),
                started_at: None,
                finished_at: fresh_ts(),
            },
        ],
    );

    assert_eq!(recommendation["execution_sequence"], Value::Null);
}
```

- [ ] **Step 2: Run the targeted recommendation tests and confirm they fail**

Run: `cargo test verification_sequence --lib`

Expected: FAIL because `execution_sequence_for_recommendation(...)` only knows how to emit the repository-stabilization mode today, so the two verification-sequence assertions will still see `null`.

- [ ] **Step 3: Extend recommendation sequencing with verification modes**

```rust
// src/mcp/project_recommendation/sequencing.rs
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

fn commands_from_array(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.as_str())
        .map(|item| item.to_string())
        .collect()
}

fn latest_failing_command(runs: &[VerificationRun]) -> Option<String> {
    runs.iter()
        .find(|run| run.status != "passed" && !run.command.trim().is_empty())
        .map(|run| run.command.trim().to_string())
}

fn repo_stabilization_sequence(repo_risk: &Value) -> Value {
    let operation_active = repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);

    if !operation_active {
        return Value::Null;
    }

    json!({
        "mode": "shell_stabilize_then_resume",
        "current_phase": "stabilize",
        "resume_with": "refresh_guidance_after_repo_stable",
        "stability_checks": ["git status", "git diff"],
        "resume_conditions": [
            "operation_states_cleared",
            "conflicted_count_zero"
        ]
    })
}

fn missing_verification_sequence(project_toolchain: &Value) -> Value {
    let mut verification_commands =
        commands_from_array(project_toolchain, "recommended_test_commands");
    if verification_commands.is_empty() {
        verification_commands =
            commands_from_array(project_toolchain, "recommended_build_commands");
    }

    json!({
        "mode": "run_project_verification_then_resume",
        "current_phase": "verify",
        "resume_with": "refresh_guidance_after_verification",
        "verification_commands": verification_commands,
        "resume_conditions": [
            "required_verification_recorded",
            "verification_evidence_fresh"
        ]
    })
}

fn failing_verification_sequence(
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
    let verification_commands = latest_failing_command(verification_runs)
        .map(|command| vec![command])
        .filter(|commands| !commands.is_empty())
        .unwrap_or_else(|| commands_from_array(project_toolchain, "recommended_test_commands"));

    json!({
        "mode": "resolve_failing_verification_then_resume",
        "current_phase": "repair_and_verify",
        "resume_with": "refresh_guidance_after_verification",
        "verification_commands": verification_commands,
        "resume_conditions": [
            "no_failing_verification_runs",
            "verification_evidence_fresh"
        ]
    })
}

pub(crate) fn execution_sequence_for_recommendation(
    forced_action: Option<&str>,
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
    match forced_action {
        Some("review_failing_verification") => {
            failing_verification_sequence(verification_runs, project_toolchain)
        }
        Some("run_verification_before_high_risk_changes") => {
            missing_verification_sequence(project_toolchain)
        }
        Some("stabilize_repository_state") => repo_stabilization_sequence(repo_risk),
        _ => Value::Null,
    }
}
```

```rust
// src/mcp/project_recommendation.rs
let project_commands = detect_project_commands(&project.root_path);
let project_toolchain = project_toolchain_layer(&project.root_path);
let verification_layer = verification_status_layer(verification_runs);
// ...
let execution_sequence = execution_sequence_for_recommendation(
    eligibility.forced_action,
    repo_risk,
    verification_runs,
    &project_toolchain,
);
```

```rust
// src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs
#[path = "project_recommendations/verification_sequence.rs"]
mod verification_sequence;
```

- [ ] **Step 4: Re-run the targeted recommendation tests and confirm they pass**

Run: `cargo test verification_sequence --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 1**

```bash
git add src/mcp/project_recommendation.rs \
        src/mcp/project_recommendation/sequencing.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_sequence.rs
git commit -m "feat: add verification execution sequences"
```

### Task 2: Decision-Brief Verification Sequence Coverage

**Files:**
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-190`

- [ ] **Step 1: Add the decision-brief regression test**

```rust
#[test]
fn decision_brief_payload_projects_selected_verification_sequence() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_failing_verification",
        "reason": "Test evidence is failing.",
        "confidence": "high",
        "recommended_flow": ["Repair the failing verification before broader review."],
        "execution_sequence": {
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
        },
        "repo_truth_gaps": ["working_tree_conflicted"],
        "mandatory_shell_checks": ["git status", "git diff"]
    });
    let agent_guidance = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        std::slice::from_ref(&recommendation),
        std::slice::from_ref(&project_overview),
    );

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "project",
        Some("demo"),
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["decision"]["execution_sequence"],
        json!({
            "mode": "resolve_failing_verification_then_resume",
            "current_phase": "repair_and_verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test -p api"],
            "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
        })
    );
}
```

- [ ] **Step 2: Run the targeted decision-brief test**

Run: `cargo test decision_brief_payload_projects_selected_verification_sequence --lib`

Expected: PASS after Task 1, because `workspace_decision.rs` already clones `top_candidate["execution_sequence"]` generically.

- [ ] **Step 3: Preserve the generic decision projection**

```rust
// src/mcp/workspace_decision.rs
"execution_sequence": top_candidate["execution_sequence"].clone(),
```

No production change should be required here. If the new test fails, stop and inspect `workspace_decision.rs` before proceeding.

- [ ] **Step 4: Commit Task 2**

```bash
git add src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
git commit -m "test: cover verification sequence decision projection"
```

### Task 3: Guidance Verification Sequence Summaries

**Files:**
- Modify: `src/mcp/guidance_payload.rs:35-120,190-320`
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/verification_sequences.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-10`

- [ ] **Step 1: Write the failing guidance-summary test**

```rust
use super::*;

#[test]
fn agent_guidance_summarizes_verification_sequences() {
    let value = agent_guidance_payload(
        3,
        3,
        &[
            "missing".to_string(),
            "failing".to_string(),
            "stabilizing".to_string(),
        ],
        &[],
        &[
            json!({
                "project_id": "missing",
                "recommended_next_action": "run_verification_before_high_risk_changes",
                "reason": "Verification evidence is missing.",
                "confidence": "medium",
                "recommended_flow": ["Run verification before risky changes."],
                "execution_sequence": {
                    "mode": "run_project_verification_then_resume",
                    "current_phase": "verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test"],
                    "resume_conditions": ["required_verification_recorded", "verification_evidence_fresh"]
                },
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            }),
            json!({
                "project_id": "failing",
                "recommended_next_action": "review_failing_verification",
                "reason": "Verification evidence is failing.",
                "confidence": "high",
                "recommended_flow": ["Repair the failing verification first."],
                "execution_sequence": {
                    "mode": "resolve_failing_verification_then_resume",
                    "current_phase": "repair_and_verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test -p api"],
                    "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
                },
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            }),
            json!({
                "project_id": "stabilizing",
                "recommended_next_action": "stabilize_repository_state",
                "reason": "Repository is mid-operation.",
                "confidence": "high",
                "recommended_flow": ["Stabilize the repository before broader code changes."],
                "execution_sequence": {
                    "mode": "shell_stabilize_then_resume",
                    "current_phase": "stabilize",
                    "resume_with": "refresh_guidance_after_repo_stable",
                    "stability_checks": ["git status", "git diff"],
                    "resume_conditions": ["operation_states_cleared", "conflicted_count_zero"]
                },
                "repo_truth_gaps": ["repository_mid_operation"],
                "mandatory_shell_checks": ["git status", "git diff"]
            })
        ],
        &[
            workspace_verification_overview(
                "missing",
                "not_recorded",
                "missing",
                &[],
                false,
                false,
            ),
            workspace_verification_overview(
                "failing",
                "available",
                "fresh",
                &[json!({"kind": "test", "status": "failed"})],
                false,
                false,
            ),
            workspace_verification_overview(
                "stabilizing",
                "available",
                "fresh",
                &[],
                false,
                false,
            ),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_verification_run"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_failing_verification_repair"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
}
```

- [ ] **Step 2: Run the targeted guidance-summary test and confirm it fails**

Run: `cargo test verification_sequences --lib`

Expected: FAIL because `agent_guidance_payload(...)` does not yet emit the two verification-sequencing summary fields.

- [ ] **Step 3: Add the guidance summary helper and fields**

```rust
// src/mcp/guidance_payload.rs
fn execution_strategy_verification_summary(project_recommendations: &[Value]) -> Value {
    let projects_requiring_verification_run = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "run_verification_before_high_risk_changes"
                && recommendation["execution_sequence"]["mode"]
                    == "run_project_verification_then_resume"
        })
        .count() as u64;

    let projects_requiring_failing_verification_repair = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "review_failing_verification"
                && recommendation["execution_sequence"]["mode"]
                    == "resolve_failing_verification_then_resume"
        })
        .count() as u64;

    json!({
        "projects_requiring_verification_run": projects_requiring_verification_run,
        "projects_requiring_failing_verification_repair":
            projects_requiring_failing_verification_repair,
    })
}
```

```rust
let verification_summary =
    execution_strategy_verification_summary(&sorted_project_recommendations);
let repo_truth_summary = execution_strategy_repo_truth_summary(&sorted_project_recommendations);
let stabilization_summary =
    execution_strategy_stabilization_summary(&sorted_project_recommendations);
```

```rust
"projects_requiring_verification_run":
    verification_summary["projects_requiring_verification_run"].clone(),
"projects_requiring_failing_verification_repair":
    verification_summary["projects_requiring_failing_verification_repair"].clone(),
"projects_requiring_repo_stabilization":
    stabilization_summary["projects_requiring_repo_stabilization"].clone(),
```

```rust
// src/mcp/tests/guidance_basics/workspace_guidance.rs
#[path = "workspace_guidance/verification_sequences.rs"]
mod verification_sequences;
```

- [ ] **Step 4: Re-run the targeted guidance-summary test and confirm it passes**

Run: `cargo test verification_sequences --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 3**

```bash
git add src/mcp/guidance_payload.rs \
        src/mcp/tests/guidance_basics/workspace_guidance.rs \
        src/mcp/tests/guidance_basics/workspace_guidance/verification_sequences.rs
git commit -m "feat: summarize verification sequences in guidance"
```

### Task 4: Contract Docs And Full Verification

**Files:**
- Modify: `docs/json-contracts.md:100-220`
- Modify: `docs/mcp-tool-reference.md:500-590`

- [ ] **Step 1: Update the JSON contract reference**

```md
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy.projects_requiring_verification_run`
- `guidance.layers.execution_strategy.projects_requiring_failing_verification_repair`
- `decision.execution_sequence`

When `decision.recommended_next_action = run_verification_before_high_risk_changes`, read
`decision.execution_sequence.verification_commands` and refresh OPENDOG guidance only after
verification evidence has been recorded again.

When `decision.recommended_next_action = review_failing_verification`, read
`decision.execution_sequence.verification_commands` to repair and rerun the failing project-native
verification before broader cleanup or refactor review.
```

- [ ] **Step 2: Update the MCP tool reference**

```md
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy.projects_requiring_verification_run`
- `guidance.layers.execution_strategy.projects_requiring_failing_verification_repair`
- `decision.execution_sequence`

Treat verification-mode `execution_sequence` as machine-readable ordering metadata.
`strategy_mode` still names the high-level strategy, while `execution_sequence` tells the consumer
which project-native verification commands to run and when to refresh OPENDOG guidance afterward.
```

- [ ] **Step 3: Run formatting, compile, focused tests, full tests, and governance validation**

Run:

```bash
cargo fmt --check
cargo check
cargo test verification_sequence --lib
cargo test decision_brief_payload_projects_selected_verification_sequence --lib
cargo test verification_sequences --lib
cargo test
python3 scripts/validate_planning_governance.py
```

Expected:

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- targeted tests: PASS
- `cargo test`: PASS
- governance validation: PASS

- [ ] **Step 4: Commit Task 4**

```bash
git add docs/json-contracts.md docs/mcp-tool-reference.md
git commit -m "docs: document verification sequencing"
```

## Self-Review

- Spec coverage: Task 1 implements the new recommendation-side verification modes and command selection; Task 2 verifies decision projection; Task 3 adds workspace execution-strategy counts; Task 4 documents the contract and runs the full verification set.
- Placeholder scan: No `TODO`, `TBD`, or implied "fill this in later" steps remain.
- Type consistency: `execution_sequence` remains `Value::Null` or an object; workspace summary fields stay `u64`; verification command lists stay `string[]` and do not reuse the repository-stabilization `stability_checks` key.
