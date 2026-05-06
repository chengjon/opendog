# Review Candidate Signals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Status note (2026-05-06):** Implemented in the current repo and re-verified during the Phase 6 audit. This checklist remains archival and was not retro-backfilled in the aggregated dirty `master` worktree.

**Goal:** Add machine-readable review-candidate signals for cleanup and refactor review paths, then reuse the same candidate vocabulary in `stats` and `unused` guidance without changing the action enum or expanding `decision_brief` / `agent_guidance`.

**Architecture:** Keep action selection recommendation-owned under `src/mcp/project_recommendation/`, add a small recommendation-level `review_focus`, and introduce a shared `src/mcp/review_candidates.rs` helper so `stats_guidance(...)` and `unused_guidance(...)` stop hand-assembling divergent candidate payloads. Reuse existing `detect_mock_data_report(...)` output instead of rescanning file content. Recommendation-level `review_focus` stays limited to reachable review-family metadata because stale snapshot/activity states are observation-first and preempt review actions in the current cascade.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, Markdown docs

---

## File Structure

- Create: `src/mcp/review_candidates.rs`
- Modify: `src/mcp/mod.rs`
- Modify: `src/mcp/project_recommendation.rs:120-470`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/review_focus.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-20`
- Modify: `src/mcp/project_guidance/stats_unused/stats.rs:1-220`
- Modify: `src/mcp/project_guidance/stats_unused/unused.rs:1-170`
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs:1-240`
- Modify: `docs/json-contracts.md:90-230`
- Modify: `docs/mcp-tool-reference.md:500-610`

### Task 1: Recommendation-Level Review Focus

**Files:**
- Modify: `src/mcp/project_recommendation.rs:120-470`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/review_focus.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs:1-20`

- [ ] **Step 1: Write the failing recommendation review-focus tests**

```rust
use super::*;
use serde_json::json;
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
        unused_files: 4,
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
fn recommend_project_action_emits_hot_file_review_focus() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            unused_files: 0,
            ..base_state(root.path())
        },
        &json!({
            "status": "available",
            "risk_level": "high",
            "is_dirty": true,
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true
        }),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "inspect_hot_files");
    assert_eq!(
        recommendation["review_focus"],
        json!({
            "candidate_family": "hot_file",
            "candidate_basis": ["highest_access_activity", "activity_present"],
            "candidate_risk_hints": ["repo_risk_elevated"]
        })
    );
}

#[test]
fn recommend_project_action_emits_unused_review_focus() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &base_state(root.path()),
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "review_unused_files");
    assert_eq!(
        recommendation["review_focus"],
        json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": []
        })
    );
}

#[test]
fn recommend_project_action_keeps_review_focus_null_when_stale_snapshot_preempts_review() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            latest_snapshot_captured_at: Some(stale_ts()),
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "take_snapshot");
    assert_eq!(
        recommendation.get("review_focus"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn recommend_project_action_keeps_review_focus_null_when_stale_activity_preempts_review() {
    let root = rust_project_root();
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            unused_files: 0,
            latest_activity_at: Some(stale_ts()),
            ..base_state(root.path())
        },
        &clean_repo_risk(),
        &passing_runs(),
    );

    assert_eq!(recommendation["recommended_next_action"], "generate_activity_then_stats");
    assert_eq!(
        recommendation.get("review_focus"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn recommend_project_action_keeps_review_focus_null_for_non_review_actions() {
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
        recommendation.get("review_focus"),
        Some(&serde_json::Value::Null)
    );
}
```

- [ ] **Step 2: Run the targeted recommendation review-focus tests and confirm they fail**

Run: `cargo test review_focus --lib`

Expected: FAIL because recommendation payloads do not yet include `review_focus`.

- [ ] **Step 3: Add recommendation-level review focus**

```rust
// src/mcp/project_recommendation.rs
fn review_focus_for_action(
    selected_action: &str,
    repo_risk: &Value,
) -> Value {
    match selected_action {
        "inspect_hot_files" => {
            let mut risk_hints = Vec::new();
            if repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
                || repo_risk["large_diff"].as_bool().unwrap_or(false)
            {
                risk_hints.push("repo_risk_elevated");
            }
            json!({
                "candidate_family": "hot_file",
                "candidate_basis": ["highest_access_activity", "activity_present"],
                "candidate_risk_hints": risk_hints,
            })
        }
        "review_unused_files" => json!({
            "candidate_family": "unused_candidate",
            "candidate_basis": ["zero_recorded_access", "snapshot_present"],
            "candidate_risk_hints": [],
        }),
        _ => Value::Null,
    }
}
```

```rust
// src/mcp/project_recommendation.rs
let attach_review_focus = |mut payload: Value| {
    let selected_action = payload["recommended_next_action"]
        .as_str()
        .unwrap_or_default();
    payload["review_focus"] = review_focus_for_action(selected_action, repo_risk);
    payload
};

let attach_recommendation_metadata = |payload: Value| {
    let payload = attach_execution_sequence(payload);
    attach_review_focus(payload)
};

// replace every `attach_execution_sequence(json!({ ... }))`
// with `attach_recommendation_metadata(json!({ ... }))`
```

```rust
// src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs
#[path = "project_recommendations/review_focus.rs"]
mod review_focus;
```

- [ ] **Step 4: Run the targeted recommendation review-focus tests and confirm they pass**

Run: `cargo test review_focus --lib`

Expected: PASS with 6 tests passing.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/project_recommendation.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/review_focus.rs
git commit -m "feat: add review focus signals"
```

### Task 2: Shared Candidate Helper

**Files:**
- Create: `src/mcp/review_candidates.rs`
- Modify: `src/mcp/mod.rs`

- [ ] **Step 1: Add the shared candidate helper module**

```rust
// src/mcp/review_candidates.rs
use serde_json::{json, Value};

pub(crate) struct CandidateFreshness {
    pub(crate) snapshot_stale: bool,
    pub(crate) activity_stale: bool,
}

pub(crate) struct ReviewCandidateContext<'a> {
    pub(crate) mock_summary: &'a Value,
    pub(crate) freshness: CandidateFreshness,
    pub(crate) repo_risk: &'a Value,
}

fn candidate_basis_for(
    kind: &str,
    mock_summary: &Value,
    file_path: &str,
) -> Vec<&'static str> {
    let mut basis = match kind {
        "hot_file" => vec!["highest_access_activity", "activity_present"],
        _ => vec!["zero_recorded_access", "snapshot_present"],
    };

    if mock_summary["mock_data_candidates"]
        .as_array()
        .map(|items| items.iter().any(|item| item["file_path"] == file_path))
        .unwrap_or(false)
    {
        basis.push("mock_data_overlap");
    }
    if mock_summary["hardcoded_data_candidates"]
        .as_array()
        .map(|items| items.iter().any(|item| item["file_path"] == file_path))
        .unwrap_or(false)
    {
        basis.push("hardcoded_data_overlap");
    }

    basis
}

fn candidate_risk_hints_for(
    kind: &str,
    freshness: CandidateFreshness,
    repo_risk: &Value,
) -> Vec<&'static str> {
    let mut risk_hints = Vec::new();
    if kind == "hot_file" && freshness.activity_stale {
        risk_hints.push("activity_evidence_stale");
    }
    if kind == "unused_candidate" && freshness.snapshot_stale {
        risk_hints.push("snapshot_evidence_stale");
    }
    if kind == "hot_file"
        && (repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
            || repo_risk["large_diff"].as_bool().unwrap_or(false))
    {
        risk_hints.push("repo_risk_elevated");
    }
    risk_hints
}

pub(crate) fn build_review_candidate(
    kind: &str,
    file_path: &str,
    priority: &str,
    reason: &str,
    suggested_commands: Vec<String>,
    context: ReviewCandidateContext<'_>,
) -> Value {
    json!({
        "kind": kind,
        "file_path": file_path,
        "reason": reason,
        "suggested_commands": suggested_commands,
        "candidate_basis": candidate_basis_for(kind, context.mock_summary, file_path),
        "candidate_risk_hints": candidate_risk_hints_for(kind, context.freshness, context.repo_risk),
        "candidate_priority": priority,
    })
}
```

```rust
// src/mcp/mod.rs
mod review_candidates;
```

- [ ] **Step 2: Run a compile-only check to catch module and visibility errors early**

Run: `cargo check`

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/mcp/mod.rs src/mcp/review_candidates.rs
git commit -m "refactor: add shared review candidate helper"
```

### Task 3: Stats Guidance Candidate Signals

**Files:**
- Modify: `src/mcp/project_guidance/stats_unused/stats.rs:1-220`
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs:1-240`

- [ ] **Step 1: Write the failing stats guidance tests**

```rust
use super::*;
use std::fs;
use tempfile::TempDir;

fn stats_entry(path: &str, access_count: i64, modification_count: i64) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count,
        estimated_duration_ms: 1000,
        modification_count,
        last_access_time: None,
        first_seen_time: None,
    }
}

fn summary_with_activity() -> ProjectSummary {
    ProjectSummary {
        total_files: 4,
        accessed_files: 2,
        unused_files: 2,
    }
}

fn passing_verification_runs() -> Vec<VerificationRun> {
    vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]
}

#[test]
fn stats_guidance_marks_hot_file_candidate_as_primary_with_basis() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src/main.rs"),
        "const MOCK_USER: &str = \"demo data\";\n",
    )
    .unwrap();
    fs::write(dir.path().join("src/old.rs"), "pub fn old() {}\n").unwrap();

    let value = stats_guidance(
        dir.path(),
        &summary_with_activity(),
        &[
            stats_entry("src/main.rs", 12, 3),
            stats_entry("src/old.rs", 0, 0),
        ],
        &passing_verification_runs(),
    );

    assert_eq!(value["file_recommendations"][0]["candidate_priority"], "primary");
    assert_eq!(
        value["file_recommendations"][0]["candidate_basis"],
        json!(["highest_access_activity", "activity_present", "mock_data_overlap"])
    );
    assert_eq!(value["file_recommendations"][0]["candidate_risk_hints"], json!([]));
}

#[test]
fn stats_guidance_marks_companion_unused_candidate_as_secondary() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.path().join("src/old.rs"), "pub fn old() {}\n").unwrap();

    let value = stats_guidance(
        dir.path(),
        &summary_with_activity(),
        &[
            stats_entry("src/main.rs", 12, 3),
            stats_entry("src/old.rs", 0, 0),
        ],
        &passing_verification_runs(),
    );

    assert_eq!(value["file_recommendations"][1]["candidate_priority"], "secondary");
    assert_eq!(
        value["file_recommendations"][1]["candidate_basis"],
        json!(["zero_recorded_access", "snapshot_present"])
    );
}
```

- [ ] **Step 2: Run the targeted stats guidance tests and confirm they fail**

Run: `cargo test stats_guidance_marks_ --lib`

Expected: FAIL because candidate objects do not yet include `candidate_priority`, `candidate_basis`, or `candidate_risk_hints`.

- [ ] **Step 3: Rebuild stats guidance candidates through the shared helper**

```rust
// src/mcp/project_guidance/stats_unused/stats.rs
use crate::mcp::review_candidates::{
    build_review_candidate, CandidateFreshness, ReviewCandidateContext,
};

let freshness = CandidateFreshness {
    snapshot_stale: false,
    activity_stale: false,
};

let mut file_recommendations = Vec::new();
file_recommendations.push(build_review_candidate(
    "hot_file",
    &hottest.file_path,
    "primary",
    "This file currently has the highest observed access activity.",
    vec![
        format!("rg \"{}\" .", hottest.file_path),
        "git diff".to_string(),
        project_commands[0].clone(),
    ],
    ReviewCandidateContext {
        mock_summary: &mock_summary,
        freshness,
        repo_risk: &repo_risk,
    },
));

if let Some(unused_candidate) = entries.iter().find(|e| e.access_count == 0) {
    file_recommendations.push(build_review_candidate(
        "unused_candidate",
        &unused_candidate.file_path,
        "secondary",
        "This file appears in the snapshot but has no recorded accesses yet.",
        vec![
            format!("rg \"{}\" .", unused_candidate.file_path),
            "git grep <symbol>".to_string(),
            project_commands[0].clone(),
        ],
        ReviewCandidateContext {
            mock_summary: &mock_summary,
            freshness,
            repo_risk: &repo_risk,
        },
    ));
}
```

Use the existing recommendation and readiness logic as-is. Do not alter `safe_for_cleanup`, `safe_for_refactor`, or the current `cleanup_refactor_candidates` parent fields in this task. In this batch, stats guidance should emit overlap-aware basis fields and priority fields; freshness-style risk hints remain recommendation-owned because this guidance path does not currently receive timestamps.

- [ ] **Step 4: Run the targeted stats guidance tests and confirm they pass**

Run: `cargo test stats_guidance_marks_ --lib`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/project_guidance/stats_unused/stats.rs \
  src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs
git commit -m "feat: enrich stats review candidates"
```

### Task 4: Unused Guidance Candidate Signals

**Files:**
- Modify: `src/mcp/project_guidance/stats_unused/unused.rs:1-170`
- Modify: `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs:1-260`

- [ ] **Step 1: Write the failing unused guidance tests**

```rust
use super::*;
use std::fs;
use tempfile::TempDir;

fn unused_entry(path: &str) -> StatsEntry {
    StatsEntry {
        file_path: path.to_string(),
        size: 10,
        file_type: "rs".to_string(),
        access_count: 0,
        estimated_duration_ms: 0,
        modification_count: 0,
        last_access_time: None,
        first_seen_time: None,
    }
}

fn unused_passing_verification_runs() -> Vec<VerificationRun> {
    vec![VerificationRun {
        id: 1,
        kind: "test".to_string(),
        status: "passed".to_string(),
        command: "cargo test".to_string(),
        exit_code: Some(0),
        summary: None,
        source: "cli".to_string(),
        started_at: None,
        finished_at: fresh_ts(),
    }]
}

#[test]
fn unused_guidance_marks_first_candidate_as_primary() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/old.rs"), "pub fn old() {}\n").unwrap();
    fs::write(
        dir.path().join("src/legacy.rs"),
        "const CUSTOMER_EMAIL: &str = \"demo@example.com\";\nconst INVOICE_NO: &str = \"INV-1\";\nconst STREET: &str = \"Main Street\";\n",
    )
    .unwrap();

    let value = unused_guidance(
        dir.path(),
        &[unused_entry("src/old.rs"), unused_entry("src/legacy.rs")],
        &unused_passing_verification_runs(),
    );

    assert_eq!(value["file_recommendations"][0]["candidate_priority"], "primary");
    assert_eq!(
        value["file_recommendations"][0]["candidate_basis"],
        json!(["zero_recorded_access", "snapshot_present"])
    );
}

#[test]
fn unused_guidance_marks_overlap_in_candidate_basis() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src/legacy.rs"),
        "const CUSTOMER_EMAIL: &str = \"demo@example.com\";\nconst INVOICE_NO: &str = \"INV-1\";\nconst STREET: &str = \"Main Street\";\n",
    )
    .unwrap();

    let value = unused_guidance(
        dir.path(),
        &[unused_entry("src/legacy.rs")],
        &unused_passing_verification_runs(),
    );

    assert_eq!(
        value["file_recommendations"][0]["candidate_basis"],
        json!(["zero_recorded_access", "snapshot_present", "hardcoded_data_overlap"])
    );
}

#[test]
fn unused_guidance_marks_later_candidates_as_secondary() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/old.rs"), "pub fn old() {}\n").unwrap();
    fs::write(dir.path().join("src/legacy.rs"), "pub fn legacy() {}\n").unwrap();

    let value = unused_guidance(
        dir.path(),
        &[unused_entry("src/old.rs"), unused_entry("src/legacy.rs")],
        &unused_passing_verification_runs(),
    );

    assert_eq!(value["file_recommendations"][1]["candidate_priority"], "secondary");
}
```

- [ ] **Step 2: Run the targeted unused guidance tests and confirm they fail**

Run: `cargo test unused_guidance_marks_ --lib`

Expected: FAIL because unused candidate objects do not yet include the new machine-readable fields.

- [ ] **Step 3: Rebuild unused guidance candidates through the shared helper**

```rust
// src/mcp/project_guidance/stats_unused/unused.rs
use crate::mcp::review_candidates::{
    build_review_candidate, CandidateFreshness, ReviewCandidateContext,
};

let freshness = CandidateFreshness {
    snapshot_stale: false,
    activity_stale: false,
};

let file_recommendations: Vec<Value> = unused_entries
    .iter()
    .take(3)
    .enumerate()
    .map(|(idx, entry)| {
        build_review_candidate(
            "unused_candidate",
            &entry.file_path,
            if idx == 0 { "primary" } else { "secondary" },
            "This file has not been observed as accessed in the current snapshot window.",
            vec![
                format!("rg \"{}\" .", entry.file_path),
                "git grep <symbol>".to_string(),
                project_commands[0].clone(),
            ],
            ReviewCandidateContext {
                mock_summary: &mock_summary,
                freshness,
                repo_risk: &repo_risk,
            },
        )
    })
    .collect();
```

Keep the existing parent-layer readiness and blocker fields unchanged. In this batch, unused guidance should emit overlap-aware basis fields and priority fields; it does not invent new freshness timestamps locally.

- [ ] **Step 4: Run the targeted unused guidance tests and confirm they pass**

Run: `cargo test unused_guidance_marks_ --lib`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/project_guidance/stats_unused/unused.rs \
  src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs
git commit -m "feat: enrich unused review candidates"
```

### Task 5: Contract Docs

**Files:**
- Modify: `docs/json-contracts.md:90-230`
- Modify: `docs/mcp-tool-reference.md:500-610`

- [ ] **Step 1: Update JSON contract docs**

Add the new fields under cleanup/refactor candidate guidance and recommendation payloads:

```md
- `guidance.project_recommendations[*].review_focus`
- `file_recommendations[*].candidate_basis`
- `file_recommendations[*].candidate_risk_hints`
- `file_recommendations[*].candidate_priority`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_basis`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_risk_hints`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_priority`
```

Add a short note explaining:

- `review_focus` names the review family selected by recommendation
- `candidate_basis` gives positive review reasons
- `candidate_risk_hints` gives advisory environment caveats
- exact gate state remains on the parent layer

- [ ] **Step 2: Update MCP tool reference**

Document the same fields in:

- `get_agent_guidance`
- `get_stats`
- `get_unused_files`

Keep the note concise: these are candidate-review aids, not deletion/refactor permission fields.

- [ ] **Step 3: Commit**

```bash
git add docs/json-contracts.md docs/mcp-tool-reference.md
git commit -m "docs: document review candidate signals"
```

### Task 6: Final Verification

**Files:**
- Verify only; no new files

- [ ] **Step 1: Run formatting verification**

Run: `cargo fmt --check`

Expected: PASS.

- [ ] **Step 2: Run compile verification**

Run: `cargo check`

Expected: PASS.

- [ ] **Step 3: Run focused recommendation tests**

Run: `cargo test review_focus --lib`

Expected: PASS.

- [ ] **Step 4: Run focused guidance tests**

Run: `cargo test stats_guidance_marks_ --lib`

Expected: PASS.

Run: `cargo test unused_guidance_marks_ --lib`

Expected: PASS.

- [ ] **Step 5: Run the broader guidance regression tests**

Run: `cargo test stats_and_unused --lib`

Expected: PASS.

- [ ] **Step 6: Run the full test suite**

Run: `cargo test`

Expected: PASS.

- [ ] **Step 7: Run planning governance validation**

Run: `python3 scripts/validate_planning_governance.py`

Expected: PASS.

- [ ] **Step 8: Confirm the worktree is clean**

Run: `git status --short`

Expected: no output.
