# Repo Stabilization Sequencing Design

Date: 2026-05-02
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.02.02` so OPENDOG expresses the repository-stabilization execution order as stable machine-readable data instead of relying only on `reason` prose or human-readable `recommended_flow`.

The target is intentionally narrow:

- keep the current `recommended_next_action` enum unchanged
- keep existing `reason`, `recommended_flow`, and `strategy_mode` intact
- add a minimal sequence object for `stabilize_repository_state`
- make it explicit when shell stabilization must happen before OPENDOG-guided review resumes

This is sequencing hardening, not a new workflow engine.

## Capability Scope

FT IDs touched:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.04.01` Aggregate and prioritize across projects
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `STRAT-01..04`
- `BOUND-03..04`
- `RISK-01..04`

`RISK-01..04` remains a consumed dependency from `FT-03.02.01`, not an ownership expansion in this batch. This design reuses existing repository-risk output rather than broadening risk collection.

## Current Problem

OPENDOG already short-circuits to `stabilize_repository_state` when repository state is unsafe, but the follow-up order is still too implicit.

Current weaknesses:

- the system tells the consumer to stabilize the repository, but not in a stable machine-readable sequence
- downstream AI consumers must infer that shell work comes first and OPENDOG refresh comes later
- `reason` explains the current blocker, but does not cleanly encode when to resume OPENDOG guidance
- workspace guidance can summarize unstable repositories, but cannot yet summarize the specific "stabilize first, then resume" pattern

This is most visible when `operation_states` force `stabilize_repository_state` and conflicted paths coexist inside the same unstable repository state. The action choice is correct, but the execution order after that choice is still too prose-heavy.

## Design

### 1. Keep The Public Action Contract Stable

This work does not change:

- `recommended_next_action`
- existing `strategy_mode` values
- current schema versions
- current `reason`
- current `recommended_flow`

It also does not change CLI text output in this batch.

The intended outward change is narrower: recommendation, guidance, and decision payloads should expose a stable execution-order contract when repository stabilization is mandatory.

### 2. Add A Minimal Sequence Field

Add one new recommendation-level field:

- `execution_sequence`

This field is a structured supplement, not a replacement for `reason` or `recommended_flow`.

Preferred shape:

```json
{
  "execution_sequence": {
    "mode": "shell_stabilize_then_resume",
    "current_phase": "stabilize",
    "resume_with": "refresh_guidance_after_repo_stable",
    "stability_checks": ["git status", "git diff"],
    "resume_conditions": [
      "operation_states_cleared",
      "conflicted_count_zero"
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
  - tells the consumer to return to OPENDOG for a fresh recommendation after stabilization
- `stability_checks`
  - the smallest shell-check set required before treating the repository as stabilized
- `resume_conditions`
  - stable machine-readable criteria for when shell stabilization is complete enough to re-enter OPENDOG guidance

This batch should keep the field narrow and explicit instead of introducing a general workflow DSL.

### 3. Sequence Triggering Must Reuse Existing Recommendation Logic

`execution_sequence` is not a second decision engine.

It should only appear when existing recommendation logic already forces:

- `recommended_next_action = stabilize_repository_state`

That means sequence generation must consume:

- existing `repo_status_risk` facts
- existing recommendation eligibility output
- existing verification and observation readiness context only where already needed by the recommendation layer

The new field explains the sequencing implications of an existing forced action. It does not add a second path for choosing actions.

### 4. Sequence Semantics

The sequence model is fixed for this batch:

1. shell stabilization
2. repository-stability recheck
3. resume OPENDOG guidance

This means OPENDOG should not guess the post-stabilization cleanup or refactor action in advance. After shell stabilization completes, the consumer should request fresh guidance again.

Recommended values:

- `mode = "shell_stabilize_then_resume"`
- `current_phase = "stabilize"`
- `resume_with = "refresh_guidance_after_repo_stable"`

This keeps the contract small while still making order explicit.

### 5. Trigger Conditions

Generate `execution_sequence.mode = shell_stabilize_then_resume` only when repository instability already has short-circuit authority through recommendation eligibility.

In practice, this means cases such as:

- `operation_states` is non-empty

Current code already forces `stabilize_repository_state` through recommendation eligibility when `operation_states` is non-empty. This batch should preserve that trigger shape rather than widening the forced-action criteria.

Do not create sequence objects for ordinary caution-level repository risk.

This boundary matters because the field is meant to describe mandatory execution order, not every possible advisory preference.

### 6. Resume Conditions

Start with the smallest stable resume-condition set:

- `operation_states_cleared`
- `conflicted_count_zero`

Do not include `large_diff_false` in this batch.

Rationale:

- `operation_states` and conflicts directly represent repository instability
- `large_diff` is a risk-severity signal, not proof that stabilization is incomplete

`conflicted_count_zero` is a resume condition, not a standalone trigger in this batch. A project can enter the sequence because `operation_states` forced stabilization, while conflicts still remain part of the minimum "safe to resume OPENDOG" check.

This keeps "repository is stable enough to resume guidance" separate from "repository is low risk."

### 7. Stability Checks

`stability_checks` should stay fixed and minimal:

- `git status`
- `git diff`

Rules:

- preserve stable ordering
- do not duplicate commands
- do not add project-native test commands in this field

Project-native verification remains important, but it belongs to the next recommendation pass after repository stabilization, not to the minimum shell truth needed to confirm repository state.

These are consumer-facing advisory commands, not a promise that OPENDOG uses the exact same internal probe arguments. Internal git collection can continue using more specific commands such as porcelain or git-path inspection while this field stays concise and operator-readable.

### 8. Output Surfaces

This batch should update three surfaces.

#### Single-project recommendation

`recommend_project_action(...)` becomes the source of truth for:

- `execution_sequence`

Preferred behavior:

- sequence object when `stabilize_repository_state` is forced
- explicit `null` otherwise

Using `null` keeps the machine contract stable and avoids forcing consumers to branch on field presence.

#### Decision brief

`decision_brief` should consume and expose the same structured sequencing fact:

- `decision.execution_sequence`

This field should read from the selected highest-priority recommendation and remain `null` when that selected project does not currently require repository stabilization. This lets downstream consumers read the final decision envelope without reopening recommendation internals.

#### Workspace execution strategy

`agent_guidance.layers.execution_strategy` should expose compact sequencing summaries:

- `projects_requiring_repo_stabilization: u64`
- `repo_stabilization_priority_projects: string[]`

`projects_requiring_repo_stabilization` is a count that matches the existing execution-strategy summary style. `repo_stabilization_priority_projects` is an ordered list of project IDs aligned with existing portfolio priority order so AI consumers know which repositories need shell-first stabilization next. This layer is summary-only. It should not replicate every per-project sequence object in full.

### 9. Compatibility With Existing Fields

Existing fields remain valid and should not be redefined:

- recommendation `reason`
- recommendation `recommended_flow`
- recommendation `suggested_commands`
- guidance `when_to_use_shell`
- decision `risk_profile`

Compatibility rule:

- `reason` explains why repository stabilization is currently mandatory
- `execution_sequence` explains how to execute and how to resume OPENDOG afterward
- `strategy_mode` explains the high-level strategy choice, while `execution_sequence` adds ordered phase and resume semantics for machine consumers

These roles are complementary and must not drift apart.

### 10. Non-Goals

This batch does not:

- introduce a general workflow state machine
- change action ordering outside the already forced `stabilize_repository_state` path
- add new git probes
- change CLI text output
- expose per-project sequence details everywhere in workspace payloads
- turn OPENDOG into the final authority for repository truth

## Testing

Add or update tests at three levels.

### 1. Recommendation behavior coverage

Verify:

- repository mid-operation still returns `stabilize_repository_state`
- `execution_sequence.mode` is `shell_stabilize_then_resume`
- `stability_checks` is exactly `["git status", "git diff"]`
- `resume_conditions` contains:
  - `operation_states_cleared`
  - `conflicted_count_zero`
- non-stabilization actions keep `execution_sequence = null`

### 2. Decision integration coverage

Verify:

- `decision.execution_sequence` is projected from the selected recommendation
- existing `reason`, `risk_profile`, and `entrypoints` remain intact

### 3. Guidance aggregation coverage

Verify:

- `projects_requiring_repo_stabilization` counts projects whose selected action is `stabilize_repository_state`
- `repo_stabilization_priority_projects` stays aligned with existing portfolio priority order
- execution-strategy summaries do not expand into full per-project workflow payload duplication

## Expected Outcome

After this batch:

- AI consumers can distinguish "stabilize now" from "resume guidance later" without parsing prose
- repository-stabilization order is expressed as a stable machine-readable contract
- `agent_guidance` and `decision_brief` stay aligned because they consume the same recommendation fact
- OPENDOG remains advisory-first while becoming clearer about shell-first sequencing under repository instability
