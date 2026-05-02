# Observation Sequencing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add machine-readable observation-first `execution_sequence` modes for `start_monitor`, `take_snapshot`, and `generate_activity_then_stats`, then summarize and document them without changing the action enum or CLI text output.

**Architecture:** Keep sequencing logic recommendation-owned under `src/mcp/project_recommendation/`, switch the discriminator from `forced_action` to the selected `recommended_next_action`, and reuse the existing `execution_sequence` field. Leave `decision_brief` as a passive projector and add only minimal workspace count summaries for observation bootstrap paths.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, Markdown docs

---

## File Structure

- Modify: `src/mcp/project_recommendation/sequencing.rs`
- Modify: `src/mcp/project_recommendation.rs:120-420`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/observation_sequence.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-16`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-220`
- Modify: `src/mcp/guidance_payload.rs:40-140,210-340`
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/observation_sequences.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-10`
- Modify: `docs/json-contracts.md:140-260`
- Modify: `docs/mcp-tool-reference.md:500-620`

### Task 1: Recommendation Observation Sequencing

**Files:**
- Modify: `src/mcp/project_recommendation/sequencing.rs`
- Modify: `src/mcp/project_recommendation.rs:120-420`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/observation_sequence.rs`
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

fn base_state(root: &std::path::Path) -> ProjectGuidanceState {
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

fn passing_runs() -> Vec<VerificationRun> {
    vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: Some("ok".to_string()),
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]
}

#[test]
fn recommend_project_action_emits_start_monitor_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            status: "stopped".to_string(),
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            latest_verification_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &[],
    );

    assert_eq!(recommendation["recommended_next_action"], "start_monitor");
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "start_monitor_then_resume",
            "current_phase": "enable_monitoring",
            "resume_with": "refresh_guidance_after_observation",
            "observation_steps": ["start_monitor", "generate_real_project_activity"],
            "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
        })
    );
}

#[test]
fn recommend_project_action_emits_snapshot_refresh_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "take_snapshot");
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "refresh_snapshot_then_resume",
            "current_phase": "snapshot",
            "resume_with": "refresh_guidance_after_snapshot",
            "observation_steps": ["take_snapshot"],
            "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
        })
    );
}

#[test]
fn recommend_project_action_emits_activity_generation_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            accessed_files: 0,
            latest_activity_at: None,
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "generate_activity_then_stats"
    );
    assert_eq!(
        recommendation["execution_sequence"],
        json!({
            "mode": "generate_activity_then_resume",
            "current_phase": "generate_activity",
            "resume_with": "refresh_guidance_after_activity",
            "observation_steps": ["generate_real_project_activity", "refresh_stats"],
            "resume_conditions": ["activity_evidence_recorded", "activity_evidence_fresh"]
        })
    );
}

#[test]
fn recommend_project_action_keeps_repo_stabilization_ahead_of_observation_sequence() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            status: "stopped".to_string(),
            total_files: 0,
            accessed_files: 0,
            unused_files: 0,
            latest_snapshot_captured_at: None,
            latest_activity_at: None,
            latest_verification_at: None,
            ..base_state(root.path())
        },
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": ["rebase"],
            "conflicted_count": 1,
            "lockfile_anomalies": [],
            "large_diff": false
        }),
        &[],
    );

    assert_eq!(
        recommendation["recommended_next_action"],
        "stabilize_repository_state"
    );
    assert_eq!(
        recommendation["execution_sequence"]["mode"],
        "shell_stabilize_then_resume"
    );
}
```

- [ ] **Step 2: Run the targeted recommendation tests and confirm they fail**

Run: `cargo test observation_sequence --lib`

Expected: FAIL because observation actions still return `execution_sequence = null`.

- [ ] **Step 3: Extend recommendation sequencing with observation modes**

```rust
// src/mcp/project_recommendation/sequencing.rs
use crate::storage::queries::VerificationRun;
use serde_json::{json, Value};

fn monitor_start_sequence() -> Value {
    json!({
        "mode": "start_monitor_then_resume",
        "current_phase": "enable_monitoring",
        "resume_with": "refresh_guidance_after_observation",
        "observation_steps": ["start_monitor", "generate_real_project_activity"],
        "resume_conditions": [
            "monitoring_active",
            "activity_evidence_recorded"
        ]
    })
}

fn snapshot_refresh_sequence() -> Value {
    json!({
        "mode": "refresh_snapshot_then_resume",
        "current_phase": "snapshot",
        "resume_with": "refresh_guidance_after_snapshot",
        "observation_steps": ["take_snapshot"],
        "resume_conditions": [
            "snapshot_available",
            "snapshot_evidence_fresh"
        ]
    })
}

fn activity_generation_sequence() -> Value {
    json!({
        "mode": "generate_activity_then_resume",
        "current_phase": "generate_activity",
        "resume_with": "refresh_guidance_after_activity",
        "observation_steps": ["generate_real_project_activity", "refresh_stats"],
        "resume_conditions": [
            "activity_evidence_recorded",
            "activity_evidence_fresh"
        ]
    })
}

pub(crate) fn execution_sequence_for_recommendation(
    selected_action: &str,
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value {
    match selected_action {
        "review_failing_verification" => {
            failing_verification_sequence(verification_runs, project_toolchain)
        }
        "run_verification_before_high_risk_changes" => {
            missing_verification_sequence(project_toolchain)
        }
        "stabilize_repository_state" => repo_stabilization_sequence(repo_risk),
        "start_monitor" => monitor_start_sequence(),
        "take_snapshot" => snapshot_refresh_sequence(),
        "generate_activity_then_stats" => activity_generation_sequence(),
        _ => Value::Null,
    }
}
```

```rust
// src/mcp/project_recommendation.rs
let attach_execution_sequence = |mut payload: Value| {
    let selected_action = payload["recommended_next_action"]
        .as_str()
        .unwrap_or_default();
    payload["execution_sequence"] = execution_sequence_for_recommendation(
        selected_action,
        repo_risk,
        verification_runs,
        &project_toolchain,
    );
    payload
};

// wrap every existing branch payload with attach_execution_sequence(...)
// and use the branch's literal selected action string as `recommended_next_action`.

if eligibility.forced_action == Some("review_failing_verification") {
    attach_execution_sequence(json!({
        "project_id": project.id,
        "recommended_next_action": "review_failing_verification",
        "recommended_flow": [
            "Inspect the latest failing or uncertain verification evidence first.",
            "Use shell diff and project-native verification commands to stabilize the project.",
            "Return to cleanup or refactor review only after verification is passing again."
        ]
    }))
} else if eligibility.forced_action == Some("stabilize_repository_state") {
    attach_execution_sequence(json!({
        "project_id": project.id,
        "recommended_next_action": "stabilize_repository_state",
        "recommended_flow": [
            "Stabilize the repository before broader code changes.",
            "Use git status and diff to understand the in-progress operation.",
            "Only return to OPENDOG-guided cleanup or review after the repository state is stable."
        ]
    }))
}
```

Apply the same wrapper to the remaining existing branches, using these literal `recommended_next_action` values:

- `start_monitor`
- `take_snapshot`
- `generate_activity_then_stats`
- `run_verification_before_high_risk_changes`
- `review_unused_files`
- `inspect_hot_files`

```rust
// src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs
#[path = "project_recommendations/observation_sequence.rs"]
mod observation_sequence;
```

- [ ] **Step 4: Re-run the targeted recommendation tests and confirm they pass**

Run: `cargo test observation_sequence --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 1**

```bash
git add src/mcp/project_recommendation.rs \
        src/mcp/project_recommendation/sequencing.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
        src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/observation_sequence.rs
git commit -m "feat: add observation execution sequences"
```

### Task 2: Decision Brief Observation Regression Coverage

**Files:**
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs:1-220`

- [ ] **Step 1: Add decision-brief regression tests for observation modes**

```rust
fn assert_decision_sequence(
    action: &str,
    sequence: serde_json::Value,
    monitoring_count: usize,
    monitored_projects: &[String],
) {
    let project_overview = fixtures::demo_project_overview();
    let recommendation = json!({
        "project_id": "demo",
        "recommended_next_action": action,
        "reason": action,
        "confidence": "medium",
        "recommended_flow": [action],
        "execution_sequence": sequence.clone(),
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    });
    let agent_guidance = agent_guidance_payload(
        1,
        monitoring_count,
        monitored_projects,
        &[],
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

    assert_eq!(brief["decision"]["execution_sequence"], sequence);
}

#[test]
fn decision_brief_payload_projects_selected_start_monitor_sequence() {
    assert_decision_sequence(
        "start_monitor",
        json!({
            "mode": "start_monitor_then_resume",
            "current_phase": "enable_monitoring",
            "resume_with": "refresh_guidance_after_observation",
            "observation_steps": ["start_monitor", "generate_real_project_activity"],
            "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
        }),
        0,
        &[],
    );
}

#[test]
fn decision_brief_payload_projects_selected_snapshot_sequence() {
    assert_decision_sequence(
        "take_snapshot",
        json!({
            "mode": "refresh_snapshot_then_resume",
            "current_phase": "snapshot",
            "resume_with": "refresh_guidance_after_snapshot",
            "observation_steps": ["take_snapshot"],
            "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
        }),
        1,
        &["demo".to_string()],
    );
}
```

- [ ] **Step 2: Run the targeted decision-brief tests**

Run: `cargo test decision_brief_payload_projects_selected_ --lib`

Expected: PASS; `decision_brief_payload(...)` already projects `top_candidate["execution_sequence"]` generically.

- [ ] **Step 3: Commit Task 2**

```bash
git add src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
git commit -m "test: cover observation sequence decision projection"
```

### Task 3: Guidance Observation Sequence Summaries

**Files:**
- Modify: `src/mcp/guidance_payload.rs:40-140,210-340`
- Create: `src/mcp/tests/guidance_basics/workspace_guidance/observation_sequences.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance.rs:1-10`

- [ ] **Step 1: Write the failing guidance summary test**

```rust
use super::*;

fn recommendation(project_id: &str, action: &str, sequence: serde_json::Value) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "recommended_next_action": action,
        "reason": action,
        "confidence": "medium",
        "recommended_flow": [action],
        "execution_sequence": sequence,
        "repo_truth_gaps": [],
        "mandatory_shell_checks": []
    })
}

#[test]
fn agent_guidance_summarizes_observation_sequences() {
    let value = agent_guidance_payload(
        5,
        3,
        &[
            "monitor".to_string(),
            "snapshot".to_string(),
            "activity".to_string(),
        ],
        &[],
        &[
            recommendation(
                "monitor",
                "start_monitor",
                json!({
                    "mode": "start_monitor_then_resume",
                    "current_phase": "enable_monitoring",
                    "resume_with": "refresh_guidance_after_observation",
                    "observation_steps": ["start_monitor", "generate_real_project_activity"],
                    "resume_conditions": ["monitoring_active", "activity_evidence_recorded"]
                }),
            ),
            recommendation(
                "snapshot",
                "take_snapshot",
                json!({
                    "mode": "refresh_snapshot_then_resume",
                    "current_phase": "snapshot",
                    "resume_with": "refresh_guidance_after_snapshot",
                    "observation_steps": ["take_snapshot"],
                    "resume_conditions": ["snapshot_available", "snapshot_evidence_fresh"]
                }),
            ),
            recommendation(
                "activity",
                "generate_activity_then_stats",
                json!({
                    "mode": "generate_activity_then_resume",
                    "current_phase": "generate_activity",
                    "resume_with": "refresh_guidance_after_activity",
                    "observation_steps": ["generate_real_project_activity", "refresh_stats"],
                    "resume_conditions": ["activity_evidence_recorded", "activity_evidence_fresh"]
                }),
            ),
            recommendation(
                "verify",
                "run_verification_before_high_risk_changes",
                json!({
                    "mode": "run_project_verification_then_resume",
                    "current_phase": "verify",
                    "resume_with": "refresh_guidance_after_verification",
                    "verification_commands": ["cargo test"],
                    "resume_conditions": ["required_verification_recorded", "verification_evidence_fresh"]
                }),
            ),
            json!({
                "project_id": "stabilize",
                "recommended_next_action": "stabilize_repository_state",
                "reason": "stabilize_repository_state",
                "confidence": "high",
                "recommended_flow": ["stabilize_repository_state"],
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
            workspace_verification_overview("monitor", "not_recorded", "missing", &[], false, false),
            workspace_verification_overview("snapshot", "available", "fresh", &[], false, false),
            workspace_verification_overview("activity", "available", "fresh", &[], false, false),
            workspace_verification_overview("verify", "not_recorded", "missing", &[], false, false),
            workspace_verification_overview("stabilize", "available", "fresh", &[], false, false),
        ],
    );

    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_monitor_start"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_snapshot_refresh"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_activity_generation"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_verification_run"],
        json!(1)
    );
    assert_eq!(
        value["guidance"]["layers"]["execution_strategy"]["projects_requiring_repo_stabilization"],
        json!(1)
    );
}
```

- [ ] **Step 2: Run the targeted guidance test and confirm it fails**

Run: `cargo test observation_sequences --lib`

Expected: FAIL because the three observation-sequence count fields do not exist yet.

- [ ] **Step 3: Add the workspace observation sequencing summary**

```rust
// src/mcp/guidance_payload.rs
fn execution_strategy_observation_summary(project_recommendations: &[Value]) -> Value {
    let projects_requiring_monitor_start = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "start_monitor"
                && recommendation["execution_sequence"]["mode"] == "start_monitor_then_resume"
        })
        .count() as u64;

    let projects_requiring_snapshot_refresh = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "take_snapshot"
                && recommendation["execution_sequence"]["mode"] == "refresh_snapshot_then_resume"
        })
        .count() as u64;

    let projects_requiring_activity_generation = project_recommendations
        .iter()
        .filter(|recommendation| {
            recommendation["recommended_next_action"] == "generate_activity_then_stats"
                && recommendation["execution_sequence"]["mode"] == "generate_activity_then_resume"
        })
        .count() as u64;

    json!({
        "projects_requiring_monitor_start": projects_requiring_monitor_start,
        "projects_requiring_snapshot_refresh": projects_requiring_snapshot_refresh,
        "projects_requiring_activity_generation": projects_requiring_activity_generation,
    })
}

let observation_summary =
    execution_strategy_observation_summary(&sorted_project_recommendations);

"execution_strategy": {
    "global_strategy_mode": workspace_strategy["global_strategy_mode"].clone(),
    "preferred_primary_tool": workspace_strategy["preferred_primary_tool"].clone(),
    "preferred_secondary_tool": workspace_strategy["preferred_secondary_tool"].clone(),
    "evidence_priority": workspace_strategy["evidence_priority"].clone(),
    "recommended_flow": workspace_strategy["recommended_flow"].clone(),
    "projects_requiring_monitor_start":
        observation_summary["projects_requiring_monitor_start"].clone(),
    "projects_requiring_snapshot_refresh":
        observation_summary["projects_requiring_snapshot_refresh"].clone(),
    "projects_requiring_activity_generation":
        observation_summary["projects_requiring_activity_generation"].clone(),
    "projects_requiring_verification_run":
        verification_summary["projects_requiring_verification_run"].clone(),
    "projects_requiring_failing_verification_repair":
        verification_summary["projects_requiring_failing_verification_repair"].clone(),
    "projects_requiring_repo_stabilization":
        stabilization_summary["projects_requiring_repo_stabilization"].clone(),
    "repo_stabilization_priority_projects":
        stabilization_summary["repo_stabilization_priority_projects"].clone(),
    "projects_with_repo_truth_gaps":
        repo_truth_summary["projects_with_repo_truth_gaps"].clone(),
    "repo_truth_gap_distribution":
        repo_truth_summary["repo_truth_gap_distribution"].clone(),
    "mandatory_shell_check_examples":
        repo_truth_summary["mandatory_shell_check_examples"].clone()
}
```

```rust
// src/mcp/tests/guidance_basics/workspace_guidance.rs
#[path = "workspace_guidance/observation_sequences.rs"]
mod observation_sequences;
```

- [ ] **Step 4: Re-run the targeted guidance test and confirm it passes**

Run: `cargo test observation_sequences --lib`

Expected: PASS

- [ ] **Step 5: Commit Task 3**

```bash
git add src/mcp/guidance_payload.rs \
        src/mcp/tests/guidance_basics/workspace_guidance.rs \
        src/mcp/tests/guidance_basics/workspace_guidance/observation_sequences.rs
git commit -m "feat: summarize observation sequences in guidance"
```

### Task 4: Contract Docs And Full Verification

**Files:**
- Modify: `docs/json-contracts.md:140-260`
- Modify: `docs/mcp-tool-reference.md:500-620`

- [ ] **Step 1: Update JSON contract documentation**

Document three new `decision.execution_sequence.mode` values:

- `start_monitor_then_resume`
- `refresh_snapshot_then_resume`
- `generate_activity_then_resume`

Document the new `guidance.layers.execution_strategy` count fields:

- `projects_requiring_monitor_start`
- `projects_requiring_snapshot_refresh`
- `projects_requiring_activity_generation`

- [ ] **Step 2: Update MCP tool reference documentation**

Add an observation-sequencing note under MCP recommendation/decision payloads:

1. read `execution_sequence.mode`
2. execute the listed `observation_steps`
3. wait until `resume_conditions` are satisfied
4. request fresh guidance again

- [ ] **Step 3: Run formatting, targeted tests, full tests, and governance validation**

Run:

```bash
cargo fmt --check
cargo check
cargo test observation_sequence --lib
cargo test decision_brief_payload_projects_selected_ --lib
cargo test observation_sequences --lib
cargo test
python3 scripts/validate_planning_governance.py
```

Expected:

- `cargo fmt --check`: PASS
- `cargo check`: PASS
- targeted test commands: PASS
- `cargo test`: PASS
- governance validation: PASS

- [ ] **Step 4: Commit Task 4**

```bash
git add docs/json-contracts.md \
        docs/mcp-tool-reference.md
git commit -m "docs: document observation sequencing"
```
