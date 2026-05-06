# External Truth Boundary Tightening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a thin read-only `external_truth_boundary` projection that tells AI consumers when the top project must switch from OPENDOG guidance to direct repository or project-native verification truth.

**Architecture:** Reuse existing `repo_truth_gaps`, `mandatory_shell_checks`, and `execution_sequence` to build one shared top-project boundary projection. Attach it once under `guidance.layers.execution_strategy`, then mirror that same value into `decision.external_truth_boundary` instead of recomputing a second path.

**Tech Stack:** Rust 2021, `serde_json`, Cargo unit/integration tests, Markdown docs

---

## File Structure

- Create: `src/mcp/constraints/external_truth.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-external-truth-boundary-tightening-design.md`

### Task 1: Write The Failing External-Truth Boundary Tests

**Files:**
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Add the workspace-guidance red test for combined repo-state and verification triggers**

Append this test to `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`:

```rust
#[test]
fn agent_guidance_exposes_external_truth_boundary_for_repo_state_and_verification() {
    let value = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "review_failing_verification",
            "reason": "Test evidence is failing.",
            "confidence": "high",
            "recommended_flow": ["Repair the failing verification before broader review."],
            "repo_truth_gaps": ["working_tree_conflicted"],
            "mandatory_shell_checks": ["git status", "git diff"],
            "execution_sequence": {
                "mode": "resolve_failing_verification_then_resume",
                "current_phase": "repair_and_verify",
                "resume_with": "refresh_guidance_after_verification",
                "verification_commands": ["cargo test -p api"],
                "resume_conditions": ["no_failing_verification_runs", "verification_evidence_fresh"]
            }
        })],
        &[json!({
            "project_id": "demo",
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "Verification is failing.",
            "safe_for_refactor_reason": "Verification is failing.",
            "verification_evidence": {
                "status": "available",
                "failing_runs": [{"kind":"test","status":"failed"}]
            },
            "repo_status_risk": {
                "status": "available",
                "risk_level": "medium",
                "is_dirty": true,
                "operation_states": [],
                "risk_findings": [],
                "highest_priority_finding": null
            },
            "mock_data_summary": {
                "hardcoded_candidate_count": 0,
                "mock_candidate_count": 0,
                "data_risk_focus": {
                    "primary_focus": "none",
                    "priority_order": ["hardcoded", "mixed", "mock"],
                    "basis": []
                }
            },
            "storage_maintenance": {
                "maintenance_candidate": false,
                "approx_reclaimable_bytes": 0,
                "reclaim_ratio": 0.0
            },
            "project_toolchain": {
                "project_type": "rust",
                "recommended_test_commands": ["cargo test"],
                "recommended_lint_commands": ["cargo clippy --all-targets --all-features -- -D warnings"],
                "recommended_build_commands": ["cargo check"]
            },
            "observation": {
                "coverage_state": "ready",
                "freshness": {
                    "snapshot": { "status": "fresh" },
                    "activity": { "status": "fresh" },
                    "verification": { "status": "fresh" }
                }
            }
        })],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["external_truth_boundary"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "mode": "must_switch_to_external_truth",
            "repo_state_required": true,
            "verification_required": true,
            "triggers": ["working_tree_conflicted", "failing_verification_repair_required"],
            "minimum_external_checks": ["git status", "git diff", "cargo test -p api"],
            "summary": "Top project needs direct repository and verification truth before broader changes."
        })
    );
}
```

- [ ] **Step 2: Add the three decision-brief red tests**

Append these tests to `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`:

```rust
#[test]
fn decision_brief_payload_exposes_external_truth_boundary_for_verification_only() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "run_verification_before_high_risk_changes",
        "reason": "Verification evidence is missing.",
        "confidence": "high",
        "recommended_flow": ["Run project verification before broader edits."],
        "execution_sequence": {
            "mode": "run_project_verification_then_resume",
            "current_phase": "verify",
            "resume_with": "refresh_guidance_after_verification",
            "verification_commands": ["cargo test"],
            "resume_conditions": ["verification_evidence_fresh"]
        },
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
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
        brief["decision"]["external_truth_boundary"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "mode": "must_switch_to_external_truth",
            "repo_state_required": false,
            "verification_required": true,
            "triggers": ["verification_run_required"],
            "minimum_external_checks": ["cargo test"],
            "summary": "Top project needs fresh project-native verification truth before broader changes."
        })
    );
}

#[test]
fn decision_brief_payload_keeps_not_git_repository_advisory_for_external_truth_boundary() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_unused_files",
        "reason": "Unused candidates should be reviewed first.",
        "confidence": "medium",
        "recommended_flow": ["Review the unused candidates before cleanup."],
        "execution_sequence": Value::Null,
        "repo_truth_gaps": ["not_git_repository"],
        "mandatory_shell_checks": []
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
        brief["decision"]["external_truth_boundary"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "mode": "opendog_guidance_can_continue",
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": "Current top recommendation can continue under OPENDOG guidance until a repository or verification boundary is reached."
        })
    );
}

#[test]
fn decision_brief_payload_marks_external_truth_boundary_absent_when_no_priority_project() {
    let agent_guidance = agent_guidance_payload(0, 0, &[], &[], &[], &[]);

    let brief = decision_brief_payload(
        MCP_DECISION_BRIEF_V1,
        "workspace",
        None,
        1,
        &agent_guidance,
        None,
    );

    assert_eq!(
        brief["layers"]["execution_strategy"]["external_truth_boundary"],
        json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "mode": Value::Null,
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": Value::Null
        })
    );
    assert_eq!(
        brief["decision"]["external_truth_boundary"],
        brief["layers"]["execution_strategy"]["external_truth_boundary"]
    );
}
```

- [ ] **Step 3: Run the focused boundary tests and confirm they fail**

Run:

```bash
cargo test external_truth_boundary --lib
```

Expected:

- FAIL because `external_truth_boundary` does not exist yet under `guidance.layers.execution_strategy`
- FAIL because `decision.external_truth_boundary` does not exist yet

### Task 2: Implement The Thin External-Truth Projection

**Files:**
- Create: `src/mcp/constraints/external_truth.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Create the shared top-project boundary helper**

Create `src/mcp/constraints/external_truth.rs` with:

```rust
use serde_json::{json, Value};

use super::string_array_field;

fn push_once(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn repo_state_triggers_for(recommendation: &Value) -> Vec<String> {
    string_array_field(recommendation, "repo_truth_gaps")
        .into_iter()
        .filter(|gap| {
            matches!(
                gap.as_str(),
                "repository_mid_operation"
                    | "working_tree_conflicted"
                    | "dependency_state_requires_repo_review"
                    | "git_metadata_unavailable"
            )
        })
        .collect()
}

fn verification_trigger_for(recommendation: &Value) -> Option<String> {
    match recommendation["execution_sequence"]["mode"].as_str().unwrap_or_default() {
        "run_project_verification_then_resume" => Some("verification_run_required".to_string()),
        "resolve_failing_verification_then_resume" => {
            Some("failing_verification_repair_required".to_string())
        }
        _ => None,
    }
}

fn verification_commands_for(recommendation: &Value) -> Vec<String> {
    recommendation["execution_sequence"]["verification_commands"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn external_truth_boundary_for_top_project(
    top_recommendation: Option<&Value>,
) -> Value {
    let Some(recommendation) = top_recommendation else {
        return json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "mode": Value::Null,
            "repo_state_required": false,
            "verification_required": false,
            "triggers": [],
            "minimum_external_checks": [],
            "summary": Value::Null
        });
    };

    let repo_triggers = repo_state_triggers_for(recommendation);
    let verification_trigger = verification_trigger_for(recommendation);
    let repo_state_required = !repo_triggers.is_empty();
    let verification_required = verification_trigger.is_some();

    let mut triggers = repo_triggers.clone();
    if let Some(trigger) = &verification_trigger {
        triggers.push(trigger.clone());
    }

    let mut minimum_external_checks = Vec::new();
    for command in string_array_field(recommendation, "mandatory_shell_checks") {
        push_once(&mut minimum_external_checks, &command);
    }
    for command in verification_commands_for(recommendation) {
        push_once(&mut minimum_external_checks, &command);
    }

    let mode = if repo_state_required || verification_required {
        "must_switch_to_external_truth"
    } else {
        "opendog_guidance_can_continue"
    };

    let summary = match (repo_state_required, verification_required) {
        (true, true) => {
            "Top project needs direct repository and verification truth before broader changes."
        }
        (true, false) => {
            "Top project needs direct repository truth before broader changes."
        }
        (false, true) => {
            "Top project needs fresh project-native verification truth before broader changes."
        }
        (false, false) => {
            "Current top recommendation can continue under OPENDOG guidance until a repository or verification boundary is reached."
        }
    };

    json!({
        "status": "available",
        "source": "top_priority_project",
        "source_project_id": recommendation["project_id"].clone(),
        "mode": mode,
        "repo_state_required": repo_state_required,
        "verification_required": verification_required,
        "triggers": triggers,
        "minimum_external_checks": minimum_external_checks,
        "summary": summary,
    })
}
```

- [ ] **Step 2: Export the helper through the existing constraints surface**

Update `src/mcp/constraints.rs` near the top:

```rust
mod external_truth;
mod repo_truth;

pub(crate) use self::external_truth::external_truth_boundary_for_top_project;
pub(crate) use self::repo_truth::repo_truth_gap_projection;
```

Update `src/mcp/mod.rs` imports:

```rust
use self::constraints::{
    build_constraints_boundaries_layer, common_boundary_hints,
    external_truth_boundary_for_top_project, project_readiness_snapshot,
};
```

- [ ] **Step 3: Attach the projection to `guidance.layers.execution_strategy`**

In `src/mcp/guidance_payload.rs`, extend the import list:

```rust
use super::{
    agent_guidance_recommended_flow, base_guidance_layers, build_constraints_boundaries_layer,
    default_shell_verification_commands, external_truth_boundary_for_top_project,
    sort_project_recommendations, storage_maintenance_layer, workspace_portfolio_layer,
    workspace_strategy_profile, workspace_toolchain_layer, workspace_verification_evidence_layer,
};
```

Then, after `sorted_project_recommendations` is available and before building the execution-strategy JSON, add:

```rust
let external_truth_boundary =
    external_truth_boundary_for_top_project(sorted_project_recommendations.first());
```

Then add it into the execution-strategy layer:

```rust
"external_truth_boundary": external_truth_boundary.clone(),
```

The surrounding shape should read like:

```rust
value["guidance"]["layers"]["execution_strategy"] = json!({
    "status": "available",
    "recommended_flow": recommended_flow,
    "project_recommendations": sorted_project_recommendations,
    "global_strategy_mode": workspace_strategy["global_strategy_mode"].clone(),
    "preferred_primary_tool": workspace_strategy["preferred_primary_tool"].clone(),
    "preferred_secondary_tool": workspace_strategy["preferred_secondary_tool"].clone(),
    "evidence_priority": workspace_strategy["evidence_priority"].clone(),
    "risk_strategy_coupling": risk_strategy_coupling.clone(),
    "external_truth_boundary": external_truth_boundary.clone(),
    // existing fields unchanged
});
```

- [ ] **Step 4: Mirror the same projection into `decision.external_truth_boundary`**

In `src/mcp/workspace_decision.rs`, inside the `"decision"` object, add:

```rust
"external_truth_boundary": layers["execution_strategy"]["external_truth_boundary"].clone(),
```

Place it next to the current boundary-related fields:

```rust
"repo_truth_gaps": top_candidate["repo_truth_gaps"].clone(),
"mandatory_shell_checks": top_candidate["mandatory_shell_checks"].clone(),
"external_truth_boundary": layers["execution_strategy"]["external_truth_boundary"].clone(),
"execution_sequence": top_candidate["execution_sequence"].clone(),
```

Do not compute a second boundary path in `decision_brief_payload(...)`.

- [ ] **Step 5: Run the focused boundary tests and verify they pass**

Run:

```bash
cargo test external_truth_boundary --lib
```

Expected:

- PASS for the workspace-guidance combined-trigger test
- PASS for the verification-only decision test
- PASS for the `not_git_repository` advisory-only test
- PASS for the no-priority decision + layer parity test

### Task 3: Update Docs And Run Full Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-external-truth-boundary-tightening-design.md`
- Test: project root verification commands

- [ ] **Step 1: Document the new boundary field in the contract docs**

In `docs/json-contracts.md`, add these field families to the guidance and decision sections:

```text
guidance.layers.execution_strategy.external_truth_boundary
guidance.layers.execution_strategy.external_truth_boundary.{mode,repo_state_required,verification_required}
guidance.layers.execution_strategy.external_truth_boundary.{triggers,minimum_external_checks}
decision.external_truth_boundary
decision.external_truth_boundary.{mode,repo_state_required,verification_required}
decision.external_truth_boundary.{triggers,minimum_external_checks}
```

Add a note in the explanatory-field guidance:

```text
`external_truth_boundary` is a read-only projection for the current top project. It is derived from existing `repo_truth_gaps`, `mandatory_shell_checks`, and verification execution-sequence fields, and it tells the AI when OPENDOG guidance must yield to direct repository or project-native verification truth.
```

In `docs/mcp-tool-reference.md`, add the same field family under useful response fields for:

- `get_guidance(detail = "summary")`
- `get_guidance(detail = "decision")`

Document it as:

- read-only
- top-project only
- not a new scheduler or detection engine

- [ ] **Step 2: Mark the design spec as implemented after code and verification land**

Update `docs/superpowers/specs/2026-05-05-external-truth-boundary-tightening-design.md`:

```text
Status: approved for implementation (2026-05-05)
```

to:

```text
Status: implemented and verified (2026-05-05)
```

- [ ] **Step 3: Run formatting**

Run:

```bash
cargo fmt --check
```

Expected: PASS

- [ ] **Step 4: Run the full Rust test suite**

Run:

```bash
cargo test
```

Expected: PASS

- [ ] **Step 5: Run the lint gate**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS

- [ ] **Step 6: Run planning governance validation**

Run:

```bash
python3 scripts/validate_planning_governance.py
```

Expected: PASS

- [ ] **Step 7: Summarize the final changed-file set**

The finished batch should stay limited to:

```text
docs/json-contracts.md
docs/mcp-tool-reference.md
docs/superpowers/specs/2026-05-05-external-truth-boundary-tightening-design.md
src/mcp/constraints/external_truth.rs
src/mcp/constraints.rs
src/mcp/guidance_payload.rs
src/mcp/mod.rs
src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs
src/mcp/workspace_decision.rs
```
