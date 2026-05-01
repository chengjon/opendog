# Repo Stabilization Sequencing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a machine-readable `execution_sequence` for `stabilize_repository_state`, project it into decision and guidance payloads, and document the new sequencing contract without changing the existing action enum or CLI text output.

**Architecture:** Keep sequencing logic as a small recommendation-side helper under `src/mcp/project_recommendation/`, then treat recommendation output as the source of truth for both `decision_brief` and `agent_guidance` summaries. Reuse the current forced-action path in `eligibility.rs`; do not widen action-trigger semantics or build a general workflow engine.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, Markdown docs

---

## File Structure

- Create: `src/mcp/project_recommendation/sequencing.rs`
  - Owns the minimal `execution_sequence` projection for `stabilize_repository_state`
- Modify: `src/mcp/project_recommendation.rs:160-420`
  - Imports the helper and emits `execution_sequence` in every recommendation payload branch
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_stabilization_sequence.rs`
  - Recommendation-level regression tests for positive and null sequencing cases
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-12`
  - Registers the new recommendation test module
- Modify: `src/mcp/workspace_decision.rs:118-242`
  - Projects `decision.execution_sequence` from the selected top recommendation
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-106`
  - Adds decision-brief assertions for execution-sequence projection
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs:70-78`
  - Adds explicit `execution_sequence: null` to the baseline recommendation fixture
- Modify: `src/mcp/guidance_payload.rs:49-90,194-295`
  - Adds workspace execution-strategy sequencing summary helper and fields
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/execution_sequences.rs`
  - Guidance-layer summary coverage for repo stabilization sequencing
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-8`
  - Registers the new workspace-guidance test module
- Modify: `docs/json-contracts.md:149-214`
  - Documents `decision.execution_sequence` and execution-strategy sequencing summary fields
- Modify: `docs/mcp-tool-reference.md:497-577`
  - Documents the new machine-readable sequencing fields in MCP responses

### Task 1: Recommendation Execution Sequence

**Files:**
- Create: `src/mcp/project_recommendation/sequencing.rs`
- Modify: `src/mcp/project_recommendation.rs:160-420`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_stabilization_sequence.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-12`

- [ ] **Step 1: Write the failing recommendation tests**

```rust
use super::*;
use serde_json::Value;

#[test]
fn recommend_project_action_emits_repo_stabilization_sequence() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 4,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 2,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(
        recommendation["execution_sequence"],
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
    );
}

#[test]
fn recommend_project_action_keeps_null_sequence_for_non_stabilization_actions() {
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 6,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &json!({
            "status": "not_git_repository",
            "risk_level": "low",
            "is_dirty": false,
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[VerificationRun {
            id: 1,
            kind: "test".to_string(),
            status: "passed".to_string(),
            command: "cargo test".to_string(),
            exit_code: Some(0),
            summary: Some("ok".to_string()),
            source: "cli".to_string(),
            started_at: None,
            finished_at: fresh_ts(),
        }],
    );

    assert_eq!(recommendation.get("execution_sequence"), Some(&Value::Null));
}
```

- [ ] **Step 2: Run the targeted tests and confirm they fail**

Run: `cargo test repo_stabilization_sequence --lib`

Expected: FAIL because `recommend_project_action(...)` does not yet emit `execution_sequence` for the stabilization branch and does not explicitly emit `null` for non-stabilization branches.

- [ ] **Step 3: Implement the minimal sequencing helper and payload field**

```rust
// src/mcp/project_recommendation/sequencing.rs
use serde_json::{json, Value};

pub(crate) fn execution_sequence_for_recommendation(
    forced_action: Option<&str>,
    repo_risk: &Value,
) -> Value {
    let operation_active = repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false);

    if forced_action != Some("stabilize_repository_state") || !operation_active {
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
```

```rust
// src/mcp/project_recommendation.rs
pub(crate) mod sequencing;

use self::sequencing::execution_sequence_for_recommendation;

let execution_sequence =
    execution_sequence_for_recommendation(eligibility.forced_action, repo_risk);

// add to every json! payload branch
"execution_sequence": execution_sequence.clone(),
```

- [ ] **Step 4: Re-run the targeted tests and confirm they pass**

Run: `cargo test repo_stabilization_sequence --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 1**

```bash
git add src/mcp/project_recommendation.rs \
        src/mcp/project_recommendation/sequencing.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_stabilization_sequence.rs
git commit -m "feat: add repo stabilization execution sequence"
```

### Task 2: Decision Brief Projection

**Files:**
- Modify: `src/mcp/workspace_decision.rs:118-242`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-106`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs:70-78`

- [ ] **Step 1: Extend the decision-brief fixture and add the failing assertion**

```rust
// fixtures.rs
pub(super) fn demo_recommendation() -> serde_json::Value {
    json!({
        "project_id": "demo",
        "recommended_next_action": "review_failing_verification",
        "reason": "Test evidence is failing.",
        "confidence": "high",
        "repo_truth_gaps": ["working_tree_conflicted"],
        "mandatory_shell_checks": ["git status", "git diff"],
        "execution_sequence": null
    })
}
```

```rust
// decision_brief_envelope.rs
assert_eq!(brief["decision"]["execution_sequence"], Value::Null);
```

- [ ] **Step 2: Run the decision-brief test and confirm it fails**

Run: `cargo test decision_brief_payload_exposes_unified_entry_envelope --lib`

Expected: FAIL because `decision_brief_payload(...)` does not yet copy `execution_sequence` into `decision`.

- [ ] **Step 3: Project execution sequence from the selected recommendation**

```rust
// src/mcp/workspace_decision.rs
"decision": json!({
    "summary": ...,
    "recommended_next_action": recommended_next_action,
    "reason": top_candidate["reason"].clone(),
    "repo_truth_gaps": top_candidate["repo_truth_gaps"].clone(),
    "mandatory_shell_checks": top_candidate["mandatory_shell_checks"].clone(),
    "execution_sequence": top_candidate["execution_sequence"].clone(),
    "target_project_id": target_project_id,
    ...
})
```

- [ ] **Step 4: Re-run the decision-brief test and confirm it passes**

Run: `cargo test decision_brief_payload_exposes_unified_entry_envelope --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 2**

```bash
git add src/mcp/workspace_decision.rs \
        src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs \
        src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs
git commit -m "feat: project stabilization sequence into decision brief"
```

### Task 3: Workspace Execution-Strategy Summary

**Files:**
- Modify: `src/mcp/guidance_payload.rs:49-90,194-295`
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/execution_sequences.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-8`

- [ ] **Step 1: Write the failing guidance summary test**

```rust
use super::*;
use serde_json::Value;

#[test]
fn agent_guidance_summarizes_repo_stabilization_sequences() {
    let value = agent_guidance_payload(
        2,
        2,
        &["demo".to_string(), "steady".to_string()],
        &[],
        &[
            json!({
                "project_id": "demo",
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
            }),
            json!({
                "project_id": "steady",
                "recommended_next_action": "review_unused_files",
                "reason": "Unused-file evidence is strong enough to review.",
                "confidence": "medium",
                "recommended_flow": ["Inspect unused-file candidates first."],
                "execution_sequence": Value::Null,
                "repo_truth_gaps": [],
                "mandatory_shell_checks": []
            })
        ],
        &[
            json!({
                "project_id": "demo",
                "safe_for_cleanup": false,
                "safe_for_refactor": false,
                "verification_evidence": { "status": "available", "failing_runs": [] },
                "repo_status_risk": { "status": "available", "risk_level": "high", "is_dirty": true, "operation_states": ["rebase"] },
                "mock_data_summary": { "hardcoded_candidate_count": 0, "mock_candidate_count": 0 },
                "storage_maintenance": { "maintenance_candidate": false, "vacuum_candidate": false, "approx_reclaimable_bytes": 0, "approx_db_size_bytes": 0 },
                "project_toolchain": { "project_type": "rust", "recommended_test_commands": ["cargo test"], "recommended_lint_commands": ["cargo clippy"], "recommended_build_commands": ["cargo check"] },
                "observation": { "coverage_state": "ready", "freshness": { "snapshot": { "status": "fresh" }, "activity": { "status": "fresh" }, "verification": { "status": "fresh" } } }
            }),
            json!({
                "project_id": "steady",
                "safe_for_cleanup": true,
                "safe_for_refactor": true,
                "verification_evidence": { "status": "available", "failing_runs": [] },
                "repo_status_risk": { "status": "available", "risk_level": "low", "is_dirty": false, "operation_states": [] },
                "mock_data_summary": { "hardcoded_candidate_count": 0, "mock_candidate_count": 0 },
                "storage_maintenance": { "maintenance_candidate": false, "vacuum_candidate": false, "approx_reclaimable_bytes": 0, "approx_db_size_bytes": 0 },
                "project_toolchain": { "project_type": "rust", "recommended_test_commands": ["cargo test"], "recommended_lint_commands": ["cargo clippy"], "recommended_build_commands": ["cargo check"] },
                "observation": { "coverage_state": "ready", "freshness": { "snapshot": { "status": "fresh" }, "activity": { "status": "fresh" }, "verification": { "status": "fresh" } } }
            })
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["repo_stabilization_priority_projects"],
        json!(["demo"])
    );
}
```

- [ ] **Step 2: Run the targeted guidance test and confirm it fails**

Run: `cargo test execution_sequences --lib`

Expected: FAIL because `agent_guidance_payload(...)` does not yet emit the sequencing summary fields.

- [ ] **Step 3: Add the sequencing summary helper and execution-strategy fields**

```rust
fn execution_strategy_stabilization_summary(project_recommendations: &[Value]) -> Value {
    let mut project_ids = Vec::new();

    for recommendation in project_recommendations {
        if recommendation["recommended_next_action"] == "stabilize_repository_state"
            && !recommendation["execution_sequence"].is_null()
        {
            if let Some(project_id) = recommendation["project_id"].as_str() {
                project_ids.push(project_id.to_string());
            }
        }
    }

    json!({
        "projects_requiring_repo_stabilization": project_ids.len() as u64,
        "repo_stabilization_priority_projects": project_ids,
    })
}
```

```rust
let stabilization_summary =
    execution_strategy_stabilization_summary(&sorted_project_recommendations);

"projects_requiring_repo_stabilization":
    stabilization_summary["projects_requiring_repo_stabilization"].clone(),
"repo_stabilization_priority_projects":
    stabilization_summary["repo_stabilization_priority_projects"].clone(),
```

- [ ] **Step 4: Re-run the targeted guidance test and confirm it passes**

Run: `cargo test execution_sequences --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 3**

```bash
git add src/mcp/guidance_payload.rs \
        src/mcp/tests/guidance_basics/workspace_guidance.rs \
        src/mcp/tests/guidance_basics/workspace_guidance/execution_sequences.rs
git commit -m "feat: summarize repo stabilization sequences in guidance"
```

### Task 4: Contract Docs And Full Verification

**Files:**
- Modify: `docs/json-contracts.md:149-214`
- Modify: `docs/mcp-tool-reference.md:497-577`

- [ ] **Step 1: Update the JSON contract reference**

```md
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy.projects_requiring_repo_stabilization`
- `guidance.layers.execution_strategy.repo_stabilization_priority_projects`
- `decision.execution_sequence`

Read `decision.execution_sequence` when `decision.recommended_next_action = stabilize_repository_state`; it tells the consumer to stabilize in shell first and refresh OPENDOG guidance after repository state is stable again.
```

- [ ] **Step 2: Update the MCP tool reference**

```md
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy.projects_requiring_repo_stabilization`
- `guidance.layers.execution_strategy.repo_stabilization_priority_projects`
- `decision.execution_sequence`

Treat `execution_sequence` as machine-readable ordering metadata. `strategy_mode` still names the high-level strategy, while `execution_sequence` describes shell-first stabilization and OPENDOG resume order.
```

- [ ] **Step 3: Run formatting, compile, focused tests, full tests, and governance validation**

Run:

```bash
cargo fmt --check
cargo check
cargo test repo_stabilization_sequence --lib
cargo test decision_brief_payload_exposes_unified_entry_envelope --lib
cargo test execution_sequences --lib
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
git commit -m "docs: document repo stabilization sequencing"
```

## Self-Review

- Spec coverage: Task 1 implements the recommendation-side `execution_sequence`; Task 2 projects it into `decision_brief`; Task 3 adds the `execution_strategy` summaries; Task 4 documents the new contract fields and runs full verification.
- Placeholder scan: No `TODO`, `TBD`, or implied "fill this in later" steps remain.
- Type consistency: `execution_sequence` stays a recommendation/decision object or explicit `null`; workspace summary types are fixed as `u64` plus `string[]`, matching the reviewed spec.
