# Verification Sequencing Design

Date: 2026-05-02
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.02.02` so OPENDOG expresses verification-first execution order as stable machine-readable data instead of relying only on `reason` prose or human-readable `recommended_flow`.

The target is intentionally narrow:

- keep the current `recommended_next_action` enum unchanged
- keep existing `reason`, `recommended_flow`, and `strategy_mode` intact
- reuse the existing `execution_sequence` field instead of creating a second sequencing surface
- make it explicit when project-native verification must happen before OPENDOG-guided review resumes

This is sequencing hardening, not a workflow engine.

## Capability Scope

FT IDs touched:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.03.01` Record and reason over verification evidence
- `FT-03.04.01` Aggregate and prioritize across projects

Primary requirement families:

- `STRAT-01..04`
- `EVID-01..04`

`FT-03.03.01` remains the source of truth for verification evidence state. This batch consumes existing verification facts rather than broadening evidence collection or redefining freshness rules.

## Current Problem

OPENDOG already chooses verification-first actions when evidence is missing or failing, but the follow-up order is still too implicit.

Current weaknesses:

- the system can recommend `run_verification_before_high_risk_changes`, but not as a stable machine-readable sequence
- the system can recommend `review_failing_verification`, but downstream consumers still infer repair and rerun order from prose
- AI consumers must guess which project-native commands to run before asking OPENDOG for refreshed guidance
- workspace guidance can summarize verification readiness, but cannot yet summarize how many projects must stop and verify before broader review

This is most visible when cleanup or refactor review is blocked by evidence state, but recommendation payloads still make consumers parse text to understand the next execution phase.

## Design

### 1. Keep The Public Action Contract Stable

This work does not change:

- `recommended_next_action`
- existing `strategy_mode` values
- current schema versions
- current `reason`
- current `recommended_flow`

It also does not change CLI text output in this batch.

The outward change is narrower: recommendation, guidance, and decision payloads should expose a stable execution-order contract when verification must happen before broader code review resumes.

### 2. Reuse The Existing Sequence Field

Continue using the existing recommendation-level field:

- `execution_sequence`

Verification sequencing should not introduce a parallel `verification_sequence` or a generic workflow DSL.

Preferred shape:

```json
{
  "execution_sequence": {
    "mode": "run_project_verification_then_resume",
    "current_phase": "verify",
    "resume_with": "refresh_guidance_after_verification",
    "verification_commands": ["cargo test"],
    "resume_conditions": [
      "required_verification_recorded",
      "verification_evidence_fresh"
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
  - tells the consumer to return to OPENDOG for a fresh recommendation after verification work completes
- `verification_commands`
  - lists the project-native commands the consumer should run before resuming OPENDOG review
- `resume_conditions`
  - stable machine-readable criteria for when verification work is complete enough to re-enter OPENDOG guidance

Unlike repository-stabilization mode, this batch intentionally uses `verification_commands` rather than `stability_checks`. The naming difference is deliberate:

- `stability_checks` are advisory shell truth checks for repository state
- `verification_commands` are project-native execution commands that produce new verification evidence

Consumers should branch on `mode` and then read the mode-appropriate command field rather than assuming all sequence modes use identical command-key names.

### 3. Two Verification Sequence Modes

This batch should support exactly two verification sequence modes.

#### Missing-evidence mode

When the selected action is:

- `run_verification_before_high_risk_changes`

emit:

- `mode = "run_project_verification_then_resume"`
- `current_phase = "verify"`
- `resume_with = "refresh_guidance_after_verification"`

Preferred resume conditions:

- `required_verification_recorded`
- `verification_evidence_fresh`

This mode means "run the missing project-native verification and then refresh guidance." It does not require every verification dimension to be globally green before OPENDOG can reassess.
If the fresh verification result still fails, that is still a valid resume point for this mode because the next recommendation pass may then pivot to `review_failing_verification`.

#### Failing-evidence mode

When the selected action is:

- `review_failing_verification`

emit:

- `mode = "resolve_failing_verification_then_resume"`
- `current_phase = "repair_and_verify"`
- `resume_with = "refresh_guidance_after_verification"`

Preferred resume conditions:

- `no_failing_verification_runs`
- `verification_evidence_fresh`

This mode means "inspect the failure, repair, rerun, then refresh guidance." It should remain distinct from the missing-evidence path so consumers can distinguish "evidence absent" from "evidence actively failing."
Here `no_failing_verification_runs` refers to the current latest verification evidence snapshot, not the entire historical run log.

### 4. Sequencing Must Reuse Existing Recommendation Logic

`execution_sequence` remains a structured explanation of an already-selected action. It is not a second decision engine.

Verification sequence generation should only appear when existing recommendation logic already selects:

- `run_verification_before_high_risk_changes`
- `review_failing_verification`

Do not create new sequence-only action paths based on raw evidence state. Recommendation eligibility and reasoning continue to own action selection.

Implementation-wise, this batch should keep sequencing ownership in the recommendation layer. That can be done either by extending `execution_sequence_for_recommendation(...)` with verification inputs or by introducing a sibling helper under `src/mcp/project_recommendation/` that composes with the existing sequencing module. The important boundary is ownership, not the exact helper split.

### 5. Sequence Priority Must Match The Existing Eligibility Cascade

Sequence priority must remain explicit and must match the current action-selection cascade in `eligibility.rs`:

1. failing verification
2. missing or stale verification
3. repository stabilization
4. non-sequenced actions in this batch

That means:

- if recommendation eligibility selects `review_failing_verification`, emit failing-verification sequencing even when repository instability also exists
- if recommendation eligibility selects `run_verification_before_high_risk_changes`, emit missing-evidence sequencing
- emit `shell_stabilize_then_resume` only when `stabilize_repository_state` is the selected action after higher-priority verification checks have not already won

This batch preserves the current recommendation ordering. Reordering repository stabilization ahead of verification gates would be a separate design change, not an implicit side effect of adding sequence payloads.

If recommendation eligibility selects:

- `recommended_next_action = stabilize_repository_state`

then OPENDOG should keep the existing:

- `mode = "shell_stabilize_then_resume"`

and should not also attach a verification sequence in the same recommendation payload.

This keeps sequence generation aligned with the action already selected and avoids introducing stacked sequence semantics in this batch.

### 6. Verification Command Selection

`verification_commands` should come from existing project-native command sources rather than from new inference logic.

Rules:

- for `run_verification_before_high_risk_changes`
  - use `project_toolchain.recommended_test_commands`
  - if empty, fall back to `project_toolchain.recommended_build_commands`
- for `review_failing_verification`
  - prefer the command from the most recent failing verification run
  - if no recorded failing-run command is available, fall back to `project_toolchain.recommended_test_commands`

If neither test nor build commands are available for the missing-evidence path, or no failing-run command plus no test command is available for the failing-evidence path, `verification_commands` may be an empty array. Do not fall back to unknown-profile shell/search commands such as `git status`; those remain generic exploration commands, not verification steps.

Preserve stable ordering, remove duplicates, and keep the list minimal. This batch does not expand toolchain detection or try to emit every possible lint/build/test command.

### 7. Output Surfaces

This batch should update three surfaces.

#### Single-project recommendation

`recommend_project_action(...)` becomes the source of truth for:

- `execution_sequence`

Preferred behavior:

- sequence object for the two verification-first actions
- existing repository-stabilization sequence remains unchanged
- explicit `null` for all other actions

#### Decision brief

`decision_brief` should consume and expose:

- `decision.execution_sequence`

This field should read from the selected highest-priority recommendation and remain `null` when that selected project does not currently require sequencing.

#### Workspace execution strategy

`agent_guidance.layers.execution_strategy` should expose compact verification sequencing summaries:

- `projects_requiring_verification_run: u64`
- `projects_requiring_failing_verification_repair: u64`

These are counts only. Unlike repository stabilization, this batch does not need workspace-level project ID lists because project-specific command details remain available in `guidance.project_recommendations[*].execution_sequence`.

### 8. Compatibility With Existing Fields

Existing fields remain valid and should not be redefined:

- recommendation `reason`
- recommendation `recommended_flow`
- recommendation `suggested_commands`
- guidance verification-readiness summaries
- decision `risk_profile`

Compatibility rule:

- `reason` explains why verification must happen now
- `execution_sequence` explains which commands to run and when to return to OPENDOG
- `strategy_mode` explains the high-level strategy choice, while `execution_sequence` adds ordered phase and resume semantics for machine consumers

These roles are complementary and must not drift apart.

### 9. Non-Goals

This batch does not:

- introduce a general workflow state machine
- change action ordering outside the two existing verification-first actions
- stack verification and repository-stabilization sequences in one payload
- broaden verification evidence collection
- broaden toolchain detection
- change CLI text output
- expose per-project verification sequence registries in workspace summary layers

## Testing

Add or update tests at three levels.

### 1. Recommendation behavior coverage

Verify:

- `run_verification_before_high_risk_changes` emits `mode = run_project_verification_then_resume`
- `review_failing_verification` emits `mode = resolve_failing_verification_then_resume`
- missing-evidence sequences project stable `verification_commands` and resume conditions
- failing-evidence sequences prefer the most recent recorded failing command when available
- failing-verification sequencing still wins when repository instability also exists
- missing or stale verification still wins over repository stabilization when the current eligibility cascade selects it
- non-sequenced actions keep `execution_sequence = null`

### 2. Decision integration coverage

Verify:

- `decision.execution_sequence` is projected from the selected recommendation
- repository-stabilization and verification sequencing modes both serialize correctly through the decision envelope
- existing `reason`, `risk_profile`, and `entrypoints` remain intact

### 3. Guidance aggregation coverage

Verify:

- `projects_requiring_verification_run` counts projects whose selected action is `run_verification_before_high_risk_changes`
- `projects_requiring_failing_verification_repair` counts projects whose selected action is `review_failing_verification`
- `projects_requiring_repo_stabilization` remains intact
- execution-strategy summaries do not expand into full per-project workflow duplication

## Expected Outcome

After this batch:

- AI consumers can distinguish "run project-native verification now" from "refresh OPENDOG guidance later" without parsing prose
- failing-verification repair paths become as machine-readable as missing-verification paths
- `agent_guidance` and `decision_brief` stay aligned because they consume the same recommendation fact
- OPENDOG remains advisory-first while becoming clearer about verification-first sequencing under evidence gaps
