# Action Prioritization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Status note (2026-05-06):** Implemented and re-verified. Checklist remains archival.

**Goal:** Rework project-level action prioritization so OPENDOG keeps the current action enum but chooses safer next actions, lowers confidence under mixed evidence, and emits more stable reasons across `recommend_project_action(...)`, `agent_guidance`, and `decision_brief`.

**Architecture:** Keep `recommend_project_action(...)` as the public facade, but split the ranking internals into three focused helpers: eligibility, scoring, and reasoning. Hard gating stays ahead of scoring, and consumer layers continue reading the same recommendation payload instead of inventing their own prioritization logic.

**Tech Stack:** Rust, `serde_json`, Cargo unit tests under `src/mcp/tests/`, existing MCP guidance and decision-brief payload fixtures.

---

## File Map

- Create: `src/mcp/project_recommendation/eligibility.rs`
- Create: `src/mcp/project_recommendation/scoring.rs`
- Create: `src/mcp/project_recommendation/reasoning.rs`
- Modify: `src/mcp/project_recommendation.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/prioritization_scoring.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/reason_stability.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_gates.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

No JSON contract or CLI docs change is expected in this batch.

### Task 1: Introduce Eligibility And Scoring Helpers

**Files:**
- Create: `src/mcp/project_recommendation/eligibility.rs`
- Create: `src/mcp/project_recommendation/scoring.rs`
- Modify: `src/mcp/project_recommendation.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/prioritization_scoring.rs`

- [ ] **Step 1: Register the new prioritization test module**

Update `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs`:

```rust
use super::*;

#[path = "project_recommendations/prioritization_scoring.rs"]
mod prioritization_scoring;
#[path = "project_recommendations/readiness_progression.rs"]
mod readiness_progression;
#[path = "project_recommendations/reason_stability.rs"]
mod reason_stability;
#[path = "project_recommendations/verification_gates.rs"]
mod verification_gates;
```

- [ ] **Step 2: Write the failing helper tests first**

Create `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/prioritization_scoring.rs`:

```rust
use super::*;
use crate::mcp::project_recommendation::eligibility::{
    determine_action_eligibility, GateLevel, RecommendationSignals,
};
use crate::mcp::project_recommendation::scoring::score_review_actions;

fn base_signals() -> RecommendationSignals {
    RecommendationSignals {
        cleanup_gate_level: GateLevel::Allow,
        refactor_gate_level: GateLevel::Allow,
        safe_for_cleanup: true,
        safe_for_refactor: true,
        cleanup_reason: "cleanup-ready".to_string(),
        refactor_reason: "refactor-ready".to_string(),
        monitoring_active: true,
        snapshot_available: true,
        activity_available: true,
        snapshot_stale: false,
        activity_stale: false,
        verification_missing: false,
        verification_stale: false,
        verification_failing: false,
        unused_files: 4,
    }
}

#[test]
fn determine_action_eligibility_blocks_hotspot_review_when_refactor_gate_is_blocked() {
    let mut signals = base_signals();
    signals.cleanup_gate_level = GateLevel::Caution;
    signals.refactor_gate_level = GateLevel::Blocked;
    signals.safe_for_refactor = false;
    signals.refactor_reason = "build evidence is missing".to_string();

    let eligibility = determine_action_eligibility(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": false,
            "risk_level": "low"
        }),
    );

    assert!(eligibility.cleanup_review_allowed);
    assert!(!eligibility.hotspot_review_allowed);
    assert_eq!(eligibility.forced_action, None);
}

#[test]
fn score_review_actions_penalizes_hotspot_review_more_than_unused_review_for_large_diff() {
    let signals = base_signals();
    let eligibility = determine_action_eligibility(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true,
            "changed_file_count": 18,
            "risk_level": "high"
        }),
    );

    let scores = score_review_actions(
        &signals,
        &json!({
            "operation_states": [],
            "conflicted_count": 0,
            "lockfile_anomalies": [],
            "large_diff": true,
            "changed_file_count": 18,
            "risk_level": "high"
        }),
        &eligibility,
    );

    assert_eq!(scores[0].action, "review_unused_files");
    assert!(scores[0].total > scores[1].total);
}
```

- [ ] **Step 3: Run the focused test and verify it fails to compile**

Run: `cargo test prioritization_scoring --lib`

Expected: FAIL with unresolved imports for `eligibility`, `scoring`, `GateLevel`, or `RecommendationSignals`.

- [ ] **Step 4: Add the helper modules and expose the internal types**

Create `src/mcp/project_recommendation/eligibility.rs`:

```rust
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GateLevel {
    Allow,
    Caution,
    Blocked,
}

impl GateLevel {
    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "allow" => Self::Allow,
            "caution" => Self::Caution,
            _ => Self::Blocked,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RecommendationSignals {
    pub(crate) cleanup_gate_level: GateLevel,
    pub(crate) refactor_gate_level: GateLevel,
    pub(crate) safe_for_cleanup: bool,
    pub(crate) safe_for_refactor: bool,
    pub(crate) cleanup_reason: String,
    pub(crate) refactor_reason: String,
    pub(crate) monitoring_active: bool,
    pub(crate) snapshot_available: bool,
    pub(crate) activity_available: bool,
    pub(crate) snapshot_stale: bool,
    pub(crate) activity_stale: bool,
    pub(crate) verification_missing: bool,
    pub(crate) verification_stale: bool,
    pub(crate) verification_failing: bool,
    pub(crate) unused_files: i64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EligibilityResult {
    pub(crate) forced_action: Option<&'static str>,
    pub(crate) cleanup_review_allowed: bool,
    pub(crate) hotspot_review_allowed: bool,
}

pub(crate) fn determine_action_eligibility(
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> EligibilityResult {
    if signals.verification_failing {
        return EligibilityResult {
            forced_action: Some("review_failing_verification"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }
    if signals.verification_missing || signals.verification_stale {
        return EligibilityResult {
            forced_action: Some("run_verification_before_high_risk_changes"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }
    if repo_risk["operation_states"]
        .as_array()
        .map(|states| !states.is_empty())
        .unwrap_or(false)
    {
        return EligibilityResult {
            forced_action: Some("stabilize_repository_state"),
            cleanup_review_allowed: false,
            hotspot_review_allowed: false,
        };
    }

    EligibilityResult {
        forced_action: None,
        cleanup_review_allowed: signals.monitoring_active
            && signals.snapshot_available
            && !signals.snapshot_stale
            && signals.safe_for_cleanup,
        hotspot_review_allowed: signals.monitoring_active
            && signals.snapshot_available
            && signals.activity_available
            && !signals.activity_stale
            && signals.safe_for_refactor,
    }
}
```

Create `src/mcp/project_recommendation/scoring.rs`:

```rust
use serde_json::Value;

use super::eligibility::{EligibilityResult, GateLevel, RecommendationSignals};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionScore {
    pub(crate) action: &'static str,
    pub(crate) total: i32,
}

pub(crate) fn score_review_actions(
    signals: &RecommendationSignals,
    repo_risk: &Value,
    eligibility: &EligibilityResult,
) -> Vec<ActionScore> {
    let mut scores = Vec::new();

    if eligibility.cleanup_review_allowed {
        let mut total = 100;
        if signals.cleanup_gate_level == GateLevel::Caution {
            total -= 20;
        }
        if signals.snapshot_stale {
            total -= 40;
        }
        scores.push(ActionScore {
            action: "review_unused_files",
            total,
        });
    }

    if eligibility.hotspot_review_allowed {
        let mut total = 100;
        if signals.refactor_gate_level == GateLevel::Caution {
            total -= 25;
        }
        if signals.activity_stale {
            total -= 40;
        }
        if repo_risk["large_diff"].as_bool().unwrap_or(false) {
            total -= 30;
        }
        if repo_risk["risk_level"].as_str().unwrap_or("low") == "high" {
            total -= 10;
        }
        scores.push(ActionScore {
            action: "inspect_hot_files",
            total,
        });
    }

    scores.sort_by(|a, b| b.total.cmp(&a.total));
    scores
}
```

Wire the modules at the top of `src/mcp/project_recommendation.rs`:

```rust
pub(crate) mod eligibility;
pub(crate) mod reasoning;
pub(crate) mod scoring;

use self::eligibility::{determine_action_eligibility, GateLevel, RecommendationSignals};
use self::scoring::score_review_actions;
```

- [ ] **Step 5: Run the focused test and commit the helper scaffolding**

Run: `cargo test prioritization_scoring --lib`

Expected: PASS

Commit:

```bash
git add src/mcp/project_recommendation.rs \
  src/mcp/project_recommendation/eligibility.rs \
  src/mcp/project_recommendation/scoring.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/prioritization_scoring.rs
git commit -m "refactor: add action prioritization helpers"
```

### Task 2: Rewire `recommend_project_action(...)` Around Gating And Scoring

**Files:**
- Modify: `src/mcp/project_recommendation.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_gates.rs`
- Modify: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs`

- [ ] **Step 1: Extend the regression tests with failing behavior checks**

Append to `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs`:

```rust
#[test]
fn recommend_project_action_lowers_hotspot_confidence_when_repo_risk_is_high() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "high",
        "is_dirty": true,
        "large_diff": true,
        "changed_file_count": 18,
        "conflicted_count": 0,
        "lockfile_anomalies": []
    });
    let recommendation = recommend_project_action(
        &ProjectGuidanceState {
            id: "demo".to_string(),
            status: "monitoring".to_string(),
            root_path: std::path::PathBuf::from("/tmp/demo"),
            total_files: 20,
            accessed_files: 8,
            unused_files: 0,
            latest_snapshot_captured_at: Some(fresh_ts()),
            latest_activity_at: Some(fresh_ts()),
            latest_verification_at: Some(fresh_ts()),
        },
        &repo_risk,
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
                command: "cargo clippy".to_string(),
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

    assert_eq!(recommendation["recommended_next_action"], json!("inspect_hot_files"));
    assert_eq!(recommendation["confidence"], json!("medium"));
}
```

- [ ] **Step 2: Run the focused tests and confirm at least one assertion fails**

Run: `cargo test recommend_project_action_ --lib`

Expected: FAIL because hotspot confidence is still `high` under high repo risk, or because the recommendation branch still bypasses the new scoring helper.

- [ ] **Step 3: Build the signal object and route review actions through helper-based selection**

Replace the middle of `recommend_project_action(...)` in `src/mcp/project_recommendation.rs` with:

```rust
    let signals = RecommendationSignals {
        cleanup_gate_level: GateLevel::from_str(cleanup_gate_level),
        refactor_gate_level: GateLevel::from_str(refactor_gate_level),
        safe_for_cleanup,
        safe_for_refactor,
        cleanup_reason: cleanup_reason.clone(),
        refactor_reason: refactor_reason.clone(),
        monitoring_active: project.status == "monitoring",
        snapshot_available: project.total_files > 0,
        activity_available: project.accessed_files > 0,
        snapshot_stale,
        activity_stale,
        verification_missing: verification_is_missing(verification_runs),
        verification_stale,
        verification_failing: verification_has_failures(verification_runs),
        unused_files: project.unused_files,
    };

    let eligibility = determine_action_eligibility(&signals, repo_risk);
    if eligibility.forced_action == Some("review_failing_verification") {
    } else if eligibility.forced_action == Some("stabilize_repository_state") {
    } else if eligibility.forced_action == Some("run_verification_before_high_risk_changes") {
    }

    let scores = score_review_actions(&signals, repo_risk, &eligibility);
    let best_review_action = scores
        .first()
        .map(|score| score.action)
        .unwrap_or("inspect_hot_files");
```

Then replace the final review branch condition only:

```rust
    } else if best_review_action == "review_unused_files" {
    } else {
    }
```

Use these guards in place of the current `verification_has_failures(...)`, repository operation-state predicate, `verification_is_missing(...) || verification_stale`, and `project.unused_files > 0` branch tests. In this task, keep the existing JSON payload bodies unchanged; only re-route which branch wins.

- [ ] **Step 4: Run the recommendation suite and commit the routing change**

Run: `cargo test repo_and_readiness --lib`

Expected: PASS

Commit:

```bash
git add src/mcp/project_recommendation.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_gates.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs
git commit -m "refactor: route project actions through priority gating"
```

### Task 3: Centralize Reason And Confidence Generation

**Files:**
- Create: `src/mcp/project_recommendation/reasoning.rs`
- Modify: `src/mcp/project_recommendation.rs`
- Create: `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/reason_stability.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [ ] **Step 1: Add failing reason-stability tests**

Create `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/reason_stability.rs`:

```rust
use super::*;

#[test]
fn recommend_project_action_reason_mentions_why_hotspot_review_lost() {
    let repo_risk = json!({
        "operation_states": [],
        "risk_level": "high",
        "is_dirty": true,
        "large_diff": true,
        "changed_file_count": 18,
        "conflicted_count": 0,
        "lockfile_anomalies": []
    });
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
        &repo_risk,
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
                command: "cargo clippy".to_string(),
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

    assert_eq!(recommendation["recommended_next_action"], json!("review_unused_files"));
    assert!(recommendation["reason"].as_str().unwrap().contains("hotspot review"));
    assert!(recommendation["reason"].as_str().unwrap().contains("repository state"));
}
```

Extend `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`:

```rust
    assert_eq!(
        value["guidance"]["project_recommendations"][0]["reason"],
        value["guidance"]["layers"]["execution_strategy"]["project_recommendations"][0]["reason"]
    );
```

Extend `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`:

```rust
    assert_eq!(
        brief["decision"]["reason"],
        json!("Test evidence is failing.")
    );
```

- [ ] **Step 2: Run the reason-focused tests and confirm failure**

Run: `cargo test reason_ --lib`

Expected: FAIL because `recommend_project_action(...)` still returns branch-local reasons that do not mention the suppressed alternative.

- [ ] **Step 3: Implement the shared reasoning helper and use it from both review branches**

Create `src/mcp/project_recommendation/reasoning.rs`:

```rust
use serde_json::Value;

use super::eligibility::{GateLevel, RecommendationSignals};
use super::scoring::ActionScore;

pub(crate) fn build_reason(
    selected: &ActionScore,
    runner_up: Option<&ActionScore>,
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> String {
    let dominant_constraint = if signals.cleanup_gate_level == GateLevel::Caution
        || signals.refactor_gate_level == GateLevel::Caution
    {
        "verification evidence"
    } else if repo_risk["risk_level"].as_str().unwrap_or("low") != "low"
        || repo_risk["large_diff"].as_bool().unwrap_or(false)
    {
        "repository state"
    } else if signals.snapshot_stale || signals.activity_stale {
        "observation freshness"
    } else {
        "current evidence"
    };

    let losing_action = runner_up.map(|score| score.action).unwrap_or("inspect_hot_files");
    let losing_label = if losing_action == "inspect_hot_files" {
        "hotspot review"
    } else {
        "cleanup review"
    };
    let winning_label = if selected.action == "inspect_hot_files" {
        "hotspot review"
    } else {
        "cleanup review"
    };

    format!(
        "Current {} makes {} the safer next step, and {} stays behind it for now.",
        dominant_constraint, winning_label, losing_label
    )
}

pub(crate) fn derive_confidence(
    selected: &ActionScore,
    signals: &RecommendationSignals,
    repo_risk: &Value,
) -> &'static str {
    if signals.verification_failing
        || repo_risk["operation_states"]
            .as_array()
            .map(|states| !states.is_empty())
            .unwrap_or(false)
    {
        "high"
    } else if selected.total >= 100
        && signals.cleanup_gate_level == GateLevel::Allow
        && signals.refactor_gate_level == GateLevel::Allow
        && repo_risk["risk_level"].as_str().unwrap_or("low") == "low"
    {
        "high"
    } else {
        "medium"
    }
}
```

In `src/mcp/project_recommendation.rs`, import and use the helper:

```rust
use self::reasoning::{build_reason, derive_confidence};
```

Then replace the branch-local `reason` and `confidence` assignments in the review branches with:

```rust
    let runner_up = scores.get(1);
    let selected_score = scores.first().expect("review scoring requires an eligible action");
    let shared_reason = build_reason(selected_score, runner_up, &signals, repo_risk);
    let shared_confidence = derive_confidence(selected_score, &signals, repo_risk);
```

And use:

```rust
            "reason": shared_reason,
            "confidence": shared_confidence,
```

- [ ] **Step 4: Run consumer-facing tests and commit the stabilized reasoning**

Run:

- `cargo test reason_stability --lib`
- `cargo test workspace_advice --lib`
- `cargo test decision_brief_payload_exposes_unified_entry_envelope --lib`

Expected: PASS

Commit:

```bash
git add src/mcp/project_recommendation.rs \
  src/mcp/project_recommendation/reasoning.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/reason_stability.rs \
  src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs \
  src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
git commit -m "refactor: stabilize recommendation reasoning"
```

### Task 4: Final Verification And Cleanup

- [ ] **Step 1: Run formatting and compilation checks**

Run:

- `cargo fmt --check`
- `cargo check`

Expected: both commands PASS

- [ ] **Step 2: Run the targeted recommendation and guidance suites**

Run:

- `cargo test repo_and_readiness --lib`
- `cargo test guidance_basics --lib`
- `cargo test portfolio_commands --lib`

Expected: PASS

- [ ] **Step 3: Run the full project verification sequence**

Run:

- `cargo test`
- `python3 scripts/validate_planning_governance.py`

Expected: PASS

- [ ] **Step 4: Commit the finished batch**

```bash
git add src/mcp/project_recommendation.rs \
  src/mcp/project_recommendation/eligibility.rs \
  src/mcp/project_recommendation/scoring.rs \
  src/mcp/project_recommendation/reasoning.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/prioritization_scoring.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/reason_stability.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_gates.rs \
  src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs \
  src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs \
  src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs \
  src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs
git commit -m "feat: refine project action prioritization"
```
