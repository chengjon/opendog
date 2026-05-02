# Observation Sequencing Design

Date: 2026-05-02
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.02.02` so OPENDOG expresses observation-first execution order as stable machine-readable data instead of relying only on `reason` prose or human-readable `recommended_flow`.

The target is intentionally narrow:

- keep the current `recommended_next_action` enum unchanged
- keep existing `reason`, `recommended_flow`, and `strategy_mode` intact
- reuse the existing `execution_sequence` field instead of introducing a second sequencing surface
- make it explicit when monitoring, snapshot, or fresh activity must happen before broader OPENDOG-guided review resumes

This is sequencing hardening, not a workflow engine.

## Capability Scope

FT IDs touched:

- `FT-03.01.01` Explain readiness and evidence gaps
- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.04.01` Rank projects by attention and evidence quality
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `OBS-01..04`
- `STRAT-01..04`
- `BOUND-03..04`

`FT-03.01.01` remains the source of truth for observation readiness state. This batch consumes existing snapshot/activity/monitoring facts rather than widening observation collection or redefining freshness rules.

## Current Problem

OPENDOG already selects observation-first actions when evidence is too thin, but the follow-up order is still too implicit.

Current weaknesses:

- the system can recommend `start_monitor`, `take_snapshot`, or `generate_activity_then_stats`, but not as stable machine-readable sequences
- downstream AI consumers must infer when to return to OPENDOG after observation work completes
- `reason` explains why evidence is insufficient, but not how to execute the bootstrap or refresh loop
- workspace guidance can summarize missing observation signals, but cannot yet summarize how many projects are blocked specifically on monitor start, snapshot refresh, or activity generation

This is most visible when a project is still bootstrapping observation coverage and recommendation payloads make consumers parse prose instead of following a stable resume contract.

## Design

### 1. Keep The Public Action Contract Stable

This work does not change:

- `recommended_next_action`
- existing `strategy_mode` values
- current schema versions
- current `reason`
- current `recommended_flow`

It also does not change CLI text output in this batch.

The outward change is narrower: recommendation, guidance, and decision payloads should expose a stable execution-order contract when observation work must happen before broader review resumes.

### 2. Reuse The Existing Sequence Field

Continue using the existing recommendation-level field:

- `execution_sequence`

Observation sequencing should not introduce a parallel `observation_sequence` field or a generic workflow DSL.

Preferred shape:

```json
{
  "execution_sequence": {
    "mode": "start_monitor_then_resume",
    "current_phase": "enable_monitoring",
    "resume_with": "refresh_guidance_after_observation",
    "observation_steps": ["start_monitor", "generate_real_project_activity"],
    "resume_conditions": [
      "monitoring_active",
      "activity_evidence_recorded"
    ]
  }
}
```

Field meaning:

- `mode`
  - names the sequencing pattern
- `current_phase`
  - tells the consumer which phase must happen now
- `resume_with`
  - tells the consumer to return to OPENDOG for a fresh recommendation after observation work completes
- `observation_steps`
  - lists the minimum OPENDOG or shell-side observation steps that should happen before resuming guidance
- `resume_conditions`
  - stable machine-readable criteria for when observation work is complete enough to re-enter OPENDOG guidance

This batch intentionally uses `observation_steps` rather than `stability_checks` or `verification_commands`. The naming difference is deliberate:

- `stability_checks` are shell truth checks for repository state
- `verification_commands` are project-native commands that create fresh validation evidence
- `observation_steps` are the minimal monitoring, snapshot, or activity steps needed to bootstrap fresh observation evidence

Consumers should branch on `mode` and then read the mode-appropriate step field rather than assuming all sequence modes use identical key names.

### 3. Three Observation Sequence Modes

This batch should support exactly three observation sequence modes.

#### Monitor-start mode

When the selected action is:

- `start_monitor`

emit:

- `mode = "start_monitor_then_resume"`
- `current_phase = "enable_monitoring"`
- `resume_with = "refresh_guidance_after_observation"`

Preferred observation steps:

- `start_monitor`
- `generate_real_project_activity`

Preferred resume conditions:

- `monitoring_active`
- `activity_evidence_recorded`

This mode means "enable monitoring, let real workflow activity occur, then refresh guidance." Starting the monitor alone is not enough to resume cleanup or hotspot review.

#### Snapshot-refresh mode

When the selected action is:

- `take_snapshot`

emit:

- `mode = "refresh_snapshot_then_resume"`
- `current_phase = "snapshot"`
- `resume_with = "refresh_guidance_after_snapshot"`

Preferred observation steps:

- `take_snapshot`

Preferred resume conditions:

- `snapshot_available`
- `snapshot_evidence_fresh`

This mode covers both missing and stale snapshot cases. The goal is to re-establish a fresh baseline before OPENDOG reassesses the next action.

#### Activity-generation mode

When the selected action is:

- `generate_activity_then_stats`

emit:

- `mode = "generate_activity_then_resume"`
- `current_phase = "generate_activity"`
- `resume_with = "refresh_guidance_after_activity"`

Preferred observation steps:

- `generate_real_project_activity`
- `refresh_stats`

Preferred resume conditions:

- `activity_evidence_recorded`
- `activity_evidence_fresh`

This mode means "perform real project work until observation captures meaningful activity, then refresh guidance." It should stay distinct from monitor-start mode so consumers can distinguish "monitor absent" from "monitor active but no useful activity yet."

### 4. Sequencing Must Reuse Existing Recommendation Logic

`execution_sequence` remains a structured explanation of an already-selected action. It is not a second decision engine.

Observation sequence generation should only appear when existing recommendation logic already selects:

- `start_monitor`
- `take_snapshot`
- `generate_activity_then_stats`

Do not create new sequence-only action paths based on raw observation state. Recommendation eligibility and reasoning continue to own action selection.

Implementation-wise, this batch should keep sequencing ownership in the recommendation layer. `execution_sequence_for_recommendation(...)` is still the natural extension point, but its current discriminator is too narrow: today it mainly receives `forced_action`, while observation actions are selected later by direct state checks in `recommend_project_action(...)`.

To support observation sequencing cleanly, the sequencing helper should accept the selected `recommended_next_action` string, or an equivalent post-selection discriminator, rather than inspecting only `forced_action`. The helper match arms should then cover:

- existing verification sequencing modes
- existing repository-stabilization sequencing mode
- the three new observation sequencing modes

This keeps sequencing centralized without duplicating sequencing logic inside individual recommendation branches.

### 5. Sequence Priority Must Match The Existing Recommendation Cascade

Sequence priority must remain explicit and must match the current selected-action ordering in `recommend_project_action(...)`:

1. failing verification
2. repository stabilization
3. observation sequencing actions
4. missing or stale verification
5. non-sequenced review actions in this batch

That means:

- if recommendation selection yields `review_failing_verification`, emit failing-verification sequencing even when observation gaps also exist
- if recommendation selection yields `stabilize_repository_state`, emit repository-stabilization sequencing
- if recommendation selection yields `start_monitor`, `take_snapshot`, or `generate_activity_then_stats`, emit the matching observation sequence even when verification evidence is still missing or stale
- emit verification sequencing for `run_verification_before_high_risk_changes` only when higher-priority failing, repository-stabilization, and observation bootstrap actions have not already won

This batch preserves the current recommendation ordering. Reordering repository stabilization, observation bootstrap, or verification refresh relative to each other would be a separate design change, not an implicit side effect of adding sequence payloads.

This batch also does not introduce stacked sequences. Each recommendation emits at most one `execution_sequence` object.

### 6. Resume Conditions Must Align With Existing Observation Semantics

Resume conditions should stay tightly aligned with current observation helpers:

- `project_observation_layer(...)`
- `snapshot_is_stale(...)`
- `activity_is_stale(...)`

Rules:

- `start_monitor_then_resume`
  - `monitoring_active`
  - `activity_evidence_recorded`
- `refresh_snapshot_then_resume`
  - `snapshot_available`
  - `snapshot_evidence_fresh`
- `generate_activity_then_resume`
  - `activity_evidence_recorded`
  - `activity_evidence_fresh`

Do not introduce new freshness semantics in this batch. This design reuses the existing definition of stale or missing observation evidence.

### 7. Output Surfaces

This batch should update three surfaces.

#### Single-project recommendation

`recommend_project_action(...)` becomes the source of truth for:

- `execution_sequence`

Preferred behavior:

- sequence object for the three observation-first actions
- existing repository-stabilization and verification sequences remain unchanged
- explicit `null` for all other actions

Because observation sequence generation depends on the already-selected action, the sequencing call site should move from "compute once before branch selection" to "compute after the selected action is known" inside `recommend_project_action(...)` or through an equivalent post-selection assembly step.

#### Decision brief

`decision_brief` should consume and expose:

- `decision.execution_sequence`

This field should read from the selected highest-priority recommendation and remain `null` when that selected project does not currently require sequencing.

`decision_brief` should not add sequencing logic of its own in this batch. It should continue to project the already-selected recommendation payload, including `execution_sequence`, into the decision envelope.

#### Workspace execution strategy

`agent_guidance.layers.execution_strategy` should expose compact observation sequencing summaries:

- `projects_requiring_monitor_start: u64`
- `projects_requiring_snapshot_refresh: u64`
- `projects_requiring_activity_generation: u64`

These are count-only fields. Per-project sequence details remain on `project_recommendations[*].execution_sequence`.

### 8. Compatibility With Existing Fields

Existing fields remain valid and should not be redefined:

- recommendation `reason`
- recommendation `recommended_flow`
- recommendation `suggested_commands`
- guidance `when_to_use_shell`
- decision `risk_profile`

Compatibility rule:

- `reason` explains why observation work is currently mandatory
- `execution_sequence` explains how to execute the observation bootstrap or refresh path and how to resume OPENDOG afterward
- `strategy_mode` explains the high-level strategy choice, while `execution_sequence` adds ordered phase and resume semantics for machine consumers

These roles are complementary and must not drift apart.

### 9. Non-Goals

This batch does not:

- add new recommendation actions
- change CLI wording or operator-facing prose flows
- add project-ID lists to workspace observation-sequence summaries
- create a generic workflow engine
- stack observation sequence objects with repository or verification sequences
- broaden monitoring, snapshot, or activity collection behavior beyond current helpers

## Testing Strategy

### Recommendation coverage

Add or extend recommendation tests that prove:

- `start_monitor` emits `mode = "start_monitor_then_resume"`
- `take_snapshot` emits `mode = "refresh_snapshot_then_resume"`
- `generate_activity_then_stats` emits `mode = "generate_activity_then_resume"`
- higher-priority failing-verification or repository-stabilization actions still suppress observation sequencing
- missing or stale verification does not suppress observation sequencing when an observation bootstrap action is the selected recommendation
- non-sequenced review actions still emit `execution_sequence = null`

### Decision coverage

Add or extend decision-brief tests that prove:

- `decision.execution_sequence` correctly mirrors observation sequencing on the selected project
- at least `start_monitor` and `take_snapshot` modes are covered explicitly

### Guidance coverage

Add or extend guidance tests that prove:

- `projects_requiring_monitor_start`
- `projects_requiring_snapshot_refresh`
- `projects_requiring_activity_generation`

are counted correctly without breaking existing sequencing summaries such as:

- `projects_requiring_repo_stabilization`
- `projects_requiring_verification_run`
- `projects_requiring_failing_verification_repair`

## Implementation Notes

- Keep sequencing derivation centralized under `src/mcp/project_recommendation/`.
- Reuse existing observation/readiness facts instead of recalculating observation state separately in guidance or decision layers.
- Update contract docs only after the recommendation, decision, and guidance payloads agree on the new sequence modes and counts.
