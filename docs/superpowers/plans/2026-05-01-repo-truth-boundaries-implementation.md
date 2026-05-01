# Repo Truth Boundaries Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add machine-readable repository-truth boundary fields to project recommendations, decision briefs, and workspace execution-strategy summaries without changing action enums, CLI text output, or existing boundary fields.

**Architecture:** Keep repo-truth gap derivation as a standalone helper under the existing `constraints` module, then let `recommend_project_action(...)` consume that helper and project the normalized fields outward. Decision and guidance layers should read the recommendation output instead of re-deriving gap logic, while legacy `blind_spots`, `requires_shell_verification`, and `reason` fields remain intact.

**Tech Stack:** Rust, `serde_json`, Cargo unit tests under `src/mcp/tests/`, operator docs in `docs/json-contracts.md` and `docs/mcp-tool-reference.md`.

---

## File Map

- Create: `src/mcp/constraints/repo_truth.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/project_recommendation.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Modify: `src/mcp/tests/repo_and_readiness/repo_risk_layers.rs`
- Create: `src/mcp/tests/repo_and_readiness/repo_risk_layers/repo_truth_projection.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_truth_boundaries.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`

### Task 1: Add Standalone Repo-Truth Gap Projection Helper

**Files:**
- Create: `src/mcp/constraints/repo_truth.rs`
- Modify: `src/mcp/constraints.rs`
- Modify: `src/mcp/tests/repo_and_readiness/repo_risk_layers.rs`
- Create: `src/mcp/tests/repo_and_readiness/repo_risk_layers/repo_truth_projection.rs`

- [ ] **Step 1: Register a focused repo-truth projection test module**

Update `src/mcp/tests/repo_and_readiness/repo_risk_layers.rs`:

```rust
use super::*;

#[path = "repo_risk_layers/git_status_inputs.rs"]
mod git_status_inputs;
#[path = "repo_risk_layers/repo_risk_findings.rs"]
mod repo_risk_findings;
#[path = "repo_risk_layers/repo_truth_projection.rs"]
mod repo_truth_projection;
#[path = "repo_risk_layers/workspace_portfolio.rs"]
mod workspace_portfolio;
```

- [ ] **Step 2: Write the failing helper tests first**

Create `src/mcp/tests/repo_and_readiness/repo_risk_layers/repo_truth_projection.rs`:

```rust
use super::*;
use crate::mcp::constraints::repo_truth_gap_projection;

#[test]
fn repo_truth_gap_projection_maps_repo_truth_blind_spots() {
    let (gaps, checks) = repo_truth_gap_projection(&json!({
        "status": "error",
        "operation_states": ["rebase"],
        "conflicted_count": 2,
        "lockfile_anomalies": ["package-lock.json without package.json"],
        "large_diff": true
    }));

    assert_eq!(
        gaps,
        vec![
            "git_metadata_unavailable".to_string(),
            "repository_mid_operation".to_string(),
            "working_tree_conflicted".to_string(),
            "dependency_state_requires_repo_review".to_string(),
        ]
    );
    assert_eq!(checks, vec!["git status".to_string(), "git diff".to_string()]);
}

#[test]
fn repo_truth_gap_projection_keeps_non_git_projects_out_of_mandatory_git_checks() {
    let (gaps, checks) = repo_truth_gap_projection(&json!({
        "status": "not_git_repository",
        "operation_states": [],
        "conflicted_count": 0,
        "lockfile_anomalies": [],
        "large_diff": true
    }));

    assert_eq!(gaps, vec!["not_git_repository".to_string()]);
    assert!(checks.is_empty());
}
```

- [ ] **Step 3: Run the focused test and verify it fails**

Run: `cargo test repo_truth_projection --lib`

Expected: FAIL with unresolved import or missing `repo_truth_gap_projection`.

- [ ] **Step 4: Add the standalone helper and re-export it through `constraints`**

Create `src/mcp/constraints/repo_truth.rs`:

```rust
use serde_json::Value;

fn push_once(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

pub(crate) fn repo_truth_gap_projection(repo_risk: &Value) -> (Vec<String>, Vec<String>) {
    let mut gaps = Vec::new();
    let mut mandatory_shell_checks = Vec::new();
    let status = repo_risk["status"].as_str().unwrap_or("unknown");

    match status {
        "not_git_repository" => push_once(&mut gaps, "not_git_repository"),
        "error" => {
            push_once(&mut gaps, "git_metadata_unavailable");
            push_once(&mut mandatory_shell_checks, "git status");
        }
        _ => {}
    }

    if repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false)
    {
        push_once(&mut gaps, "repository_mid_operation");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    if repo_risk["conflicted_count"].as_u64().unwrap_or(0) > 0 {
        push_once(&mut gaps, "working_tree_conflicted");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    if repo_risk["lockfile_anomalies"]
        .as_array()
        .map(|items| !items.is_empty())
        .unwrap_or(false)
    {
        push_once(&mut gaps, "dependency_state_requires_repo_review");
        push_once(&mut mandatory_shell_checks, "git status");
        push_once(&mut mandatory_shell_checks, "git diff");
    }

    (gaps, mandatory_shell_checks)
}
```

Update `src/mcp/constraints.rs`:

```rust
mod repo_truth;

pub(crate) use self::repo_truth::repo_truth_gap_projection;
```

- [ ] **Step 5: Run the focused test and verify it passes**

Run: `cargo test repo_truth_projection --lib`

Expected: PASS with `2 passed; 0 failed`.

- [ ] **Step 6: Commit the helper batch**

```bash
git add src/mcp/constraints.rs \
        src/mcp/constraints/repo_truth.rs \
        src/mcp/tests/repo_and_readiness/repo_risk_layers.rs \
        src/mcp/tests/repo_and_readiness/repo_risk_layers/repo_truth_projection.rs
git commit -m "refactor: add repo truth gap projection"
```

### Task 2: Expose Repo-Truth Fields In Project Recommendations

**Files:**
- Modify: `src/mcp/project_recommendation.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_truth_boundaries.rs`

- [ ] **Step 1: Register a recommendation regression test module**

Update `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`:

```rust
use super::*;

#[path = "project_recommendations/prioritization_scoring.rs"]
mod prioritization_scoring;
#[path = "project_recommendations/readiness_progression.rs"]
mod readiness_progression;
#[path = "project_recommendations/reason_stability.rs"]
mod reason_stability;
#[path = "project_recommendations/repo_truth_boundaries.rs"]
mod repo_truth_boundaries;
#[path = "project_recommendations/verification_gates.rs"]
mod verification_gates;
```

- [ ] **Step 2: Write the failing recommendation tests first**

Create `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_truth_boundaries.rs`:

```rust
use super::*;

#[test]
fn recommend_project_action_exposes_repo_truth_gaps_for_mid_operation_repo() {
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

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("stabilize_repository_state")
    );
    assert_eq!(
        recommendation["repo_truth_gaps"],
        json!(["repository_mid_operation"])
    );
    assert_eq!(
        recommendation["mandatory_shell_checks"],
        json!(["git status", "git diff"])
    );
}

#[test]
fn recommend_project_action_keeps_non_git_boundary_advisory() {
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

    assert_eq!(
        recommendation["recommended_next_action"],
        json!("review_unused_files")
    );
    assert_eq!(
        recommendation["repo_truth_gaps"],
        json!(["not_git_repository"])
    );
    assert_eq!(recommendation["mandatory_shell_checks"], json!([]));
}
```

- [ ] **Step 3: Run the focused test and verify it fails**

Run: `cargo test repo_truth_boundaries --lib`

Expected: FAIL because the recommendation payload does not yet emit `repo_truth_gaps` or `mandatory_shell_checks`.

- [ ] **Step 4: Wire the new fields into recommendation output**

Update `src/mcp/project_recommendation.rs` near the existing readiness and gate-level setup:

```rust
let (repo_truth_gaps, mandatory_shell_checks) = repo_truth_gap_projection(repo_risk);
let repo_truth_gaps_json = json!(repo_truth_gaps);
let mandatory_shell_checks_json = json!(mandatory_shell_checks);
```

Import the helper directly from the sibling constraints module:

```rust
use super::constraints::repo_truth_gap_projection;

use super::{
    activity_is_stale, detect_mock_data_report, detect_project_commands,
    latest_activity_timestamp, latest_verification_timestamp, now_unix_secs,
    project_observation_layer, project_readiness_snapshot,
    project_storage_maintenance, project_toolchain_layer, repo_status_risk_layer,
    snapshot_is_stale, strategy_profile, verification_has_failures, verification_is_missing,
    verification_is_stale, verification_status_layer, ProjectGuidanceData,
    ProjectGuidanceState,
};
```

Add the new fields to every returned recommendation payload branch:

```rust
"repo_truth_gaps": repo_truth_gaps_json.clone(),
"mandatory_shell_checks": mandatory_shell_checks_json.clone(),
```

Do not change action selection, confidence, or existing reason strings in this task.

- [ ] **Step 5: Run the focused test and verify it passes**

Run: `cargo test repo_truth_boundaries --lib`

Expected: PASS with `2 passed; 0 failed`.

- [ ] **Step 6: Run the broader recommendation suite**

Run: `cargo test recommend_project_action_ --lib`

Expected: PASS with the existing recommendation regressions still green.

- [ ] **Step 7: Commit the recommendation batch**

```bash
git add src/mcp/project_recommendation.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/repo_truth_boundaries.rs
git commit -m "feat: expose repo truth boundary fields in recommendations"
```

### Task 3: Propagate Repo-Truth Fields Into Decision And Guidance Layers

**Files:**
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Write the failing guidance and decision assertions first**

Update `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs` so the synthetic recommendation already carries the new fields:

```rust
        &[json!({
            "project_id": "demo",
            "recommended_next_action": "review_failing_verification",
            "reason": "Test evidence is failing.",
            "confidence": "high",
            "recommended_flow": ["Inspect verification state before broader edits."],
            "repo_truth_gaps": ["working_tree_conflicted"],
            "mandatory_shell_checks": ["git status", "git diff"]
        })],
```

Add assertions:

```rust
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_with_repo_truth_gaps"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["repo_truth_gap_distribution"]
            ["working_tree_conflicted"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["mandatory_shell_check_examples"],
        json!(["git status", "git diff"])
    );
```

Update `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs` inside `demo_recommendation()`:

```rust
        "repo_truth_gaps": ["working_tree_conflicted"],
        "mandatory_shell_checks": ["git status", "git diff"],
```

Update `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`:

```rust
    assert_eq!(
        brief["decision"]["repo_truth_gaps"],
        json!(["working_tree_conflicted"])
    );
    assert_eq!(
        brief["decision"]["mandatory_shell_checks"],
        json!(["git status", "git diff"])
    );
```

- [ ] **Step 2: Run the focused integration tests and verify they fail**

Run:

- `cargo test workspace_advice --lib`
- `cargo test decision_brief_payload_exposes_unified_entry_envelope --lib`

Expected: FAIL because guidance aggregation and decision projection do not yet expose the new fields.

- [ ] **Step 3: Add guidance aggregation and decision projection**

Update `src/mcp/guidance_payload.rs` by adding a focused summary helper near the top of the file:

```rust
fn execution_strategy_repo_truth_summary(project_recommendations: &[Value]) -> Value {
    let mut projects_with_repo_truth_gaps = 0_u64;
    let mut gap_distribution = serde_json::Map::new();
    let mut mandatory_shell_check_examples = Vec::new();

    for recommendation in project_recommendations {
        let gaps = recommendation["repo_truth_gaps"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !gaps.is_empty() {
            projects_with_repo_truth_gaps += 1;
        }
        for gap in gaps {
            if let Some(key) = gap.as_str() {
                let next = gap_distribution
                    .get(key)
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0)
                    + 1;
                gap_distribution.insert(key.to_string(), json!(next));
            }
        }
        if let Some(checks) = recommendation["mandatory_shell_checks"].as_array() {
            for check in checks {
                if let Some(cmd) = check.as_str() {
                    if !mandatory_shell_check_examples
                        .iter()
                        .any(|item: &String| item == cmd)
                    {
                        mandatory_shell_check_examples.push(cmd.to_string());
                    }
                }
            }
        }
    }

    json!({
        "projects_with_repo_truth_gaps": projects_with_repo_truth_gaps,
        "repo_truth_gap_distribution": gap_distribution,
        "mandatory_shell_check_examples": mandatory_shell_check_examples,
    })
}
```

Merge the summary into `guidance["layers"]["execution_strategy"]` after `sorted_project_recommendations` is available:

```rust
    let repo_truth_summary =
        execution_strategy_repo_truth_summary(&sorted_project_recommendations);
```

```rust
        "projects_with_repo_truth_gaps": repo_truth_summary["projects_with_repo_truth_gaps"].clone(),
        "repo_truth_gap_distribution": repo_truth_summary["repo_truth_gap_distribution"].clone(),
        "mandatory_shell_check_examples": repo_truth_summary["mandatory_shell_check_examples"].clone(),
```

Update `src/mcp/workspace_decision.rs` inside the `decision` payload:

```rust
                    "repo_truth_gaps": top_candidate["repo_truth_gaps"].clone(),
                    "mandatory_shell_checks": top_candidate["mandatory_shell_checks"].clone(),
```

- [ ] **Step 4: Run the focused integration tests and verify they pass**

Run:

- `cargo test workspace_advice --lib`
- `cargo test decision_brief_payload_exposes_unified_entry_envelope --lib`

Expected: PASS for both tests.

- [ ] **Step 5: Run the broader guidance suites**

Run:

- `cargo test guidance_basics --lib`
- `cargo test repo_and_readiness --lib`

Expected: PASS with no regressions in existing guidance or decision payload tests.

- [ ] **Step 6: Commit the propagation batch**

```bash
git add src/mcp/guidance_payload.rs \
        src/mcp/workspace_decision.rs \
        src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs \
        src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs \
        src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
git commit -m "feat: propagate repo truth boundaries to guidance and decisions"
```

### Task 4: Update Public Docs And Run Final Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`

- [ ] **Step 1: Update JSON contract docs**

Update `docs/json-contracts.md` in the guidance and decision sections so the new fields are listed explicitly:

```md
- `guidance.project_recommendations[*].repo_truth_gaps`
- `guidance.project_recommendations[*].mandatory_shell_checks`
- `guidance.layers.execution_strategy.projects_with_repo_truth_gaps`
- `guidance.layers.execution_strategy.repo_truth_gap_distribution`
- `guidance.layers.execution_strategy.mandatory_shell_check_examples`
```

```md
- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`
```

Add a short compatibility note:

```md
Compatibility rule: `repo_truth_gaps` and `mandatory_shell_checks` are machine-readable boundary projections. Legacy `blind_spots`, `requires_shell_verification`, and human-readable `reason` fields remain available and unchanged.
```

- [ ] **Step 2: Update MCP tool reference docs**

Update `docs/mcp-tool-reference.md` in the `get_agent_guidance` and `get_decision_brief` sections:

```md
- `project_recommendations[*].repo_truth_gaps`
- `project_recommendations[*].mandatory_shell_checks`
- `layers.execution_strategy.projects_with_repo_truth_gaps`
- `layers.execution_strategy.repo_truth_gap_distribution`
- `layers.execution_strategy.mandatory_shell_check_examples`
```

```md
- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`
```

Add one short usage note:

```md
Read `repo_truth_gaps` before broad edits when repository truth is uncertain. Use `mandatory_shell_checks` as the minimum shell handoff set before treating OPENDOG guidance as sufficient.
```

- [ ] **Step 3: Run formatting, compile, tests, and governance validation**

Run:

- `cargo fmt --check`
- `cargo check`
- `cargo test guidance_basics --lib`
- `cargo test repo_and_readiness --lib`
- `cargo test`
- `python3 scripts/validate_planning_governance.py`

Expected:

- formatting passes with no diffs
- build succeeds
- targeted guidance and repo/readiness suites pass
- full unit and integration suite passes
- planning governance validator reports zero backlog and completed task cards unchanged

- [ ] **Step 4: Commit docs and verification-ready final state**

```bash
git add docs/json-contracts.md \
        docs/mcp-tool-reference.md
git commit -m "docs: document repo truth boundary fields"
```
