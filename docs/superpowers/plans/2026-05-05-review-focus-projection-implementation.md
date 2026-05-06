# Review Focus Projection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a thin read-only `review_focus_projection` under workspace execution strategy and mirror the current top-project `review_focus` into the decision payload.

**Architecture:** Reuse the already-selected top project recommendation as the single source of truth. Build one shared projection helper under `src/mcp/constraints/`, attach its output once under `guidance.layers.execution_strategy`, then mirror only the nested `review_focus` value into `decision.review_focus` instead of recomputing logic on the decision path.

**Tech Stack:** Rust 2021, `serde_json`, Cargo unit tests, Markdown docs

---

## File Structure

- Create: `src/mcp/constraints/review_focus.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-review-focus-projection-design.md`

### Task 1: Write The Failing Review-Focus Projection Tests

**Files:**
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Add the workspace-guidance red test for hot-file review focus**

Append this test to `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`:

```rust
#[test]
fn agent_guidance_exposes_review_focus_projection_for_hot_file_review() {
    let value = agent_guidance_payload(
        1,
        1,
        &["demo".to_string()],
        &["demo".to_string()],
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "inspect_hot_files",
            "reason": "Recent activity shows concentrated hotspots.",
            "confidence": "medium",
            "recommended_flow": ["Inspect the hottest files before broader cleanup or refactor work."],
            "review_focus": {
                "candidate_family": "hot_file",
                "candidate_basis": ["highest_access_activity", "activity_present"],
                "candidate_risk_hints": ["repo_risk_elevated"]
            },
            "repo_truth_gaps": [],
            "mandatory_shell_checks": [],
            "execution_sequence": Value::Null
        })],
        &[json!({
            "project_id": "demo",
            "safe_for_cleanup": false,
            "safe_for_refactor": false,
            "safe_for_cleanup_reason": "Repository risk remains elevated.",
            "safe_for_refactor_reason": "Repository risk remains elevated.",
            "verification_evidence": {
                "status": "available",
                "failing_runs": []
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
                "vacuum_candidate": false,
                "approx_db_size_bytes": 0,
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
        value["guidance"]["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "review_focus": {
                "candidate_family": "hot_file",
                "candidate_basis": ["highest_access_activity", "activity_present"],
                "candidate_risk_hints": ["repo_risk_elevated"]
            }
        })
    );
}
```

- [ ] **Step 2: Add the three decision-brief red tests**

Append these tests to `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`:

```rust
#[test]
fn decision_brief_payload_mirrors_review_focus_for_unused_review() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "review_unused_files",
        "reason": "Unused candidates should be reviewed first.",
        "confidence": "medium",
        "recommended_flow": ["Review the unused candidates before cleanup."],
        "review_focus": {
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        },
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "execution_sequence": Value::Null
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
        brief["decision"]["review_focus"],
        json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        })
    );
    assert_eq!(
        brief["decision"]["review_focus"],
        brief["layers"]["execution_strategy"]["review_focus_projection"]["review_focus"]
    );
}

#[test]
fn decision_brief_payload_keeps_review_focus_null_for_non_review_action() {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": "take_snapshot",
        "reason": "Snapshot evidence is stale.",
        "confidence": "high",
        "recommended_flow": ["Refresh the snapshot before review work."],
        "review_focus": Value::Null,
        "repo_truth_gaps": [],
        "mandatory_shell_checks": [],
        "execution_sequence": {
            "mode": "refresh_snapshot_then_resume",
            "current_phase": "snapshot",
            "resume_with": "refresh_guidance_after_snapshot",
            "observation_steps": ["take_snapshot"],
            "resume_conditions": ["snapshot_evidence_fresh"]
        }
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
        brief["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "available",
            "source": "top_priority_project",
            "source_project_id": "demo",
            "review_focus": Value::Null
        })
    );
    assert_eq!(brief["decision"]["review_focus"], Value::Null);
}

#[test]
fn decision_brief_payload_marks_review_focus_projection_absent_when_no_priority_project() {
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
        brief["layers"]["execution_strategy"]["review_focus_projection"],
        json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "review_focus": Value::Null
        })
    );
    assert_eq!(brief["decision"]["review_focus"], Value::Null);
}
```

- [ ] **Step 3: Run the focused projection tests and confirm they fail**

Run:

```bash
cargo test review_focus_projection --lib
```

Expected:

- FAIL because `guidance.layers.execution_strategy.review_focus_projection` does not exist yet
- FAIL because `decision.review_focus` does not exist yet

### Task 2: Implement The Shared Review-Focus Projection

**Files:**
- Create: `src/mcp/constraints/review_focus.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Create the shared top-project review-focus helper**

Create `src/mcp/constraints/review_focus.rs` with:

```rust
use serde_json::{json, Value};

pub(crate) fn review_focus_projection_for_top_project(top_recommendation: Option<&Value>) -> Value {
    let Some(recommendation) = top_recommendation else {
        return json!({
            "status": "no_priority_project",
            "source": Value::Null,
            "source_project_id": Value::Null,
            "review_focus": Value::Null
        });
    };

    json!({
        "status": "available",
        "source": "top_priority_project",
        "source_project_id": recommendation["project_id"].clone(),
        "review_focus": recommendation["review_focus"].clone(),
    })
}
```

- [ ] **Step 2: Export the helper through the existing constraints surface**

Update `src/mcp/constraints.rs` near the top:

```rust
mod external_truth;
mod repo_truth;
mod review_focus;

pub(crate) use self::external_truth::external_truth_boundary_for_top_project;
pub(crate) use self::repo_truth::repo_truth_gap_projection;
pub(crate) use self::review_focus::review_focus_projection_for_top_project;
```

Update `src/mcp/mod.rs` imports:

```rust
use self::constraints::{
    build_constraints_boundaries_layer, common_boundary_hints,
    external_truth_boundary_for_top_project, project_readiness_snapshot,
    review_focus_projection_for_top_project,
};
```

- [ ] **Step 3: Attach the projection to `guidance.layers.execution_strategy`**

In `src/mcp/guidance_payload.rs`, extend the import list:

```rust
use super::{
    agent_guidance_recommended_flow, base_guidance_layers, build_constraints_boundaries_layer,
    default_shell_verification_commands, external_truth_boundary_for_top_project,
    review_focus_projection_for_top_project, sort_project_recommendations,
    storage_maintenance_layer, workspace_portfolio_layer, workspace_strategy_profile,
    workspace_toolchain_layer, workspace_verification_evidence_layer,
};
```

After `sorted_project_recommendations` is available, add:

```rust
let review_focus_projection =
    review_focus_projection_for_top_project(sorted_project_recommendations.first());
```

Then add it into the execution-strategy layer:

```rust
"review_focus_projection": review_focus_projection.clone(),
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
    "review_focus_projection": review_focus_projection.clone(),
    // existing fields unchanged
});
```

- [ ] **Step 4: Mirror only the nested review-focus value into the decision payload**

In `src/mcp/workspace_decision.rs`, inside the `"decision"` object, add:

```rust
"review_focus": layers["execution_strategy"]["review_focus_projection"]["review_focus"].clone(),
```

Place it near the other recommendation-explanation fields:

```rust
"reason": top_candidate["reason"].clone(),
"repo_truth_gaps": top_candidate["repo_truth_gaps"].clone(),
"mandatory_shell_checks": top_candidate["mandatory_shell_checks"].clone(),
"review_focus": layers["execution_strategy"]["review_focus_projection"]["review_focus"].clone(),
"external_truth_boundary": layers["execution_strategy"]["external_truth_boundary"].clone(),
"execution_sequence": top_candidate["execution_sequence"].clone(),
```

Do not mirror the whole projection envelope into `decision`.

- [ ] **Step 5: Run the focused projection tests and verify they pass**

Run:

```bash
cargo test review_focus_projection --lib
```

Expected:

- PASS for the summary hot-file projection test
- PASS for the decision unused-review mirror test
- PASS for the non-review null test
- PASS for the no-priority null-state test

### Task 3: Update Docs And Run Full Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-review-focus-projection-design.md`
- Test: project root verification commands

- [ ] **Step 1: Document the new projection in the contract docs**

Update `docs/json-contracts.md` in the guidance explanatory-field list to include:

```md
- `guidance.layers.execution_strategy.review_focus_projection`
```

Update the decision explanatory-field list to include:

```md
- `decision.review_focus`
```

Add one short note near the existing guidance-consumption rules:

```md
Read `guidance.layers.execution_strategy.review_focus_projection` when the current top recommendation is a cleanup or refactor review action and the AI needs a direct machine-readable summary of the current review family.
```

Add one short note near the decision-consumption rules:

```md
Read `decision.review_focus` when the AI needs the current review family without drilling into `project_recommendations[0]`; `null` means the top action is not a cleanup/refactor review action.
```

- [ ] **Step 2: Document the projection in the MCP tool reference**

Update `docs/mcp-tool-reference.md` under `get_guidance` useful response fields when `detail = "summary"`:

```md
- `guidance.layers.execution_strategy.review_focus_projection`
```

Update the `detail = "decision"` field list:

```md
- `decision.review_focus`
```

Add one short explanation:

```md
`guidance.layers.execution_strategy.review_focus_projection` is a read-only top-project projection of the existing recommendation `review_focus`. It does not expose file-level candidate previews and does not change recommendation logic.
```

- [ ] **Step 3: Mark the spec as implemented and verified after code and tests are green**

Update the header of `docs/superpowers/specs/2026-05-05-review-focus-projection-design.md` from:

```md
Status: proposed
```

to:

```md
Status: implemented and verified (2026-05-05)
```

- [ ] **Step 4: Run formatting, tests, lint, and governance validation**

Run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
python3 scripts/validate_planning_governance.py
```

Expected:

- `cargo fmt --check`: PASS with no diff
- `cargo test`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `python3 scripts/validate_planning_governance.py`: PASS

### Task 4: Final Contract Sanity Check

**Files:**
- Test: `src/mcp/guidance_payload.rs`
- Test: `src/mcp/workspace_decision.rs`
- Test: docs and spec files above

- [ ] **Step 1: Confirm the final contract shape stays minimal**

Check that the final code and docs satisfy all of these:

```text
1. review_focus_projection lives only under guidance execution strategy.
2. decision mirrors only decision.review_focus, not the full projection envelope.
3. no file_recommendations preview is added to guidance or decision.
4. no new candidate_family, candidate_basis, or candidate_risk_hints values are introduced.
5. non-review actions keep status=available with review_focus=null.
6. no-priority stays explicit with status=no_priority_project.
```

- [ ] **Step 2: Update plan footer status after implementation completes**

Append this footer after all work is done:

```md
Status: approved for implementation (2026-05-05)

Implementation notes:
- Keep the implementation commutative with existing `review_focus` and `candidate_*` work; this slice is projection-only.
- Do not widen unified guidance payloads beyond family-level review intent.

Status: implemented and verified (2026-05-05)
```
