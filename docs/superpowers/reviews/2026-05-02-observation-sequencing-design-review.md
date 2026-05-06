# Review: Observation Sequencing Design

Spec: `docs/superpowers/specs/2026-05-02-observation-sequencing-design.md`
Date: 2026-05-02
Verdict: Solid design, one structural issue to resolve before implementation

---

## Strengths

### 1. High alignment with existing patterns

The three new sequence modes (monitor-start, snapshot-refresh, activity-generation) are a mechanical extension of the three modes already in `sequencing.rs:83-99` (`repo_stabilization_sequence`, `missing_verification_sequence`, `failing_verification_sequence`). The shape (`mode`, `current_phase`, `resume_with`, `resume_conditions`) is consistent.

### 2. Correct reuse of `execution_sequence`

The spec avoids introducing a parallel field. The current code in `sequencing.rs` already has the `execution_sequence` slot used by callers — the spec simply fills in cases that currently return `Value::Null`.

### 3. Priority ordering is correct

Section 5's cascade matches the actual priority in `eligibility.rs:42-70`: failing verification > missing verification > repo stabilization > observation actions. The existing code guarantees this because `determine_action_eligibility` excludes the first three, then observation actions are selected on remaining state.

### 4. Resume conditions align with existing helpers

The spec correctly references `project_observation_layer(...)`, `snapshot_is_stale(...)`, `activity_is_stale(...)` as the source of truth rather than redefining freshness semantics.

### 5. Workspace summary pattern is consistent

The three new count fields proposed in Section 7 (`projects_requiring_monitor_start`, etc.) follow the exact same pattern as `execution_strategy_stabilization_summary` and `execution_strategy_verification_summary` in `guidance_payload.rs:93-136`. A new `execution_strategy_observation_summary` function would fit naturally.

---

## Issues

### Critical: Function signature mismatch (Section 4)

The spec states:

> Extending `execution_sequence_for_recommendation(...)` is the most direct fit

But the current function signature is:

```rust
// sequencing.rs:83
pub(crate) fn execution_sequence_for_recommendation(
    forced_action: Option<&str>,   // only sees 3 forced-action strings or None
    repo_risk: &Value,
    verification_runs: &[VerificationRun],
    project_toolchain: &Value,
) -> Value
```

The problem: the three observation actions (`start_monitor`, `take_snapshot`, `generate_activity_then_stats`) are **not** selected by `forced_action`. They are selected by direct state checks in `project_recommendation.rs:266-354` (`project.status != "monitoring"`, `project.total_files == 0 || snapshot_stale`, `project.accessed_files == 0 || activity_stale`). When these branches fire, `forced_action` is `None`, so `execution_sequence_for_recommendation` returns `Null`.

To make the spec work, this function needs additional context — either the selected `recommended_next_action` string, or observation signals. The spec should state explicitly how to bridge this gap. Two clean options:

- **Option A**: Pass the selected action string (e.g. `"start_monitor"`) instead of only `forced_action`, and add the three new modes to the match arms.
- **Option B**: Call sequence generation locally from each observation-action branch in `recommend_project_action`, passing a new discriminator.

Option A is simpler and keeps sequencing centralized.

### Minor: Call site must move from top to per-branch

`recommend_project_action` currently calls `execution_sequence_for_recommendation` once at the top (line 180), then embeds the result via `.clone()` in each branch. Observation modes require the call to happen **after** action selection, because the sequence depends on which branch was chosen. The spec should note that the call site moves from "once at top" to "once per branch."

### Minor: Decision brief propagation is implicit

Section 7 states `decision_brief` should expose `decision.execution_sequence`, but doesn't detail the propagation mechanism. In the code, `decision_brief_payload` receives the full `agent_guidance` structure and extracts from it. This works, but only if the selected project's recommendation (inside `agent_guidance`) carries the new sequence objects. The spec should explicitly state that the decision brief needs no sequencing logic of its own — it reads what is already in the recommendation layer.

---

## Suggestions

1. **Before implementation planning**, add a short paragraph to Section 4 addressing the function signature issue. Suggested wording:

   > `execution_sequence_for_recommendation` currently receives `forced_action` as its primary discriminator. Observation actions are not forced actions — they are selected by direct state checks in `recommend_project_action`. To support observation sequences, the function should accept the selected `recommended_next_action` string (or equivalent discriminator) rather than inspecting only `forced_action`. The match arms should be extended to include the three new observation modes alongside the existing verification and repository modes.

2. **Note the call site move**: the function must be called inside each branch (after the action is known), not once before action selection.

3. **Consider renaming `generate_real_project_activity`** to something more precise — it currently reads like an operative command but is actually a semantic label meaning "do real work so the observer captures activity." Something like `await_real_activity` might be clearer, though this is low priority if the existing codebase already uses the current name consistently.

---

## Summary

The spec is precise, well-scoped, and consistent with the existing codebase architecture. The one critical gap is the mismatch between the spec's claim to extend `execution_sequence_for_recommendation` and the fact that this function's current signature lacks the context needed for observation modes. Resolving this (likely by passing the selected action string rather than only `forced_action`) is a prerequisite for implementation planning.
