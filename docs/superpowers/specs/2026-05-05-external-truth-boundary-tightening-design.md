# External Truth Boundary Tightening Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.07.01` by making OPENDOG more explicit about when AI consumers must stop relying on OPENDOG-only guidance and switch to direct external truth sources.

This slice is intentionally narrow:

- add one small read-only `external_truth_boundary` projection
- cover only `repo-state` and `verification` boundary triggers
- reuse only existing machine-readable signals already present in guidance and decision payloads
- keep recommendation ordering, scoring, action selection, and execution sequencing unchanged
- keep CLI text output unchanged in this batch

This is authority-boundary tightening, not a new detection engine and not a recommendation rewrite.

## Capability Scope

FT IDs touched:

- `FT-03.07.01` Constraints and Boundaries
- `FT-03.02.02` AI Execution Strategy Suggestions
- `FT-03.03.01` Verification and Evidence Layer

Primary requirement families:

- `BOUND-01..04`
- `STRAT-01..04`
- `EVID-01..04`

## Current Problem

OPENDOG already exposes the underlying ingredients for authority boundaries:

- `repo_truth_gaps`
- `mandatory_shell_checks`
- `execution_sequence`
- verification status and gate fields
- `constraints_boundaries`
- `entrypoints.selection_reasons`

But the AI still has to infer the most important question for the current top project:

- should I keep following OPENDOG guidance right now
- or have I reached a point where direct repository or project-native verification truth is mandatory first

That gap is most visible in two cases:

- repository truth is unstable or incomplete, but the AI must manually combine `repo_truth_gaps` and `mandatory_shell_checks`
- verification must run or be repaired before broader edits, but the AI must infer that this is now a hard external-truth boundary rather than just a suggested next step

The needed improvement is a thin projection, not more detection.

## Design

### 1. Add One Small Read-Only Boundary Projection

Add a new read-only field:

- `guidance.layers.execution_strategy.external_truth_boundary`

Add the same projection at the decision layer:

- `decision.external_truth_boundary`

Both fields must describe the same top-priority project boundary state. The decision payload should not recompute separate logic.

Preferred shape:

```json
{
  "status": "available",
  "source": "top_priority_project",
  "source_project_id": "demo",
  "mode": "must_switch_to_external_truth",
  "repo_state_required": true,
  "verification_required": true,
  "triggers": [
    "repository_mid_operation",
    "verification_run_required"
  ],
  "minimum_external_checks": [
    "git status",
    "git diff",
    "cargo test"
  ],
  "summary": "Top project needs direct repository and verification truth before broader changes."
}
```

This object exists only to project current authority boundaries for the top project. It does not grant or deny actions, and it does not replace the underlying evidence fields.

### 2. Use Only The Current Top-Priority Project

This slice must not introduce workspace-wide aggregation logic.

Rules:

- `guidance.layers.execution_strategy.external_truth_boundary` uses only the current top recommendation
- `decision.external_truth_boundary` mirrors the same top-project projection
- do not inspect second or later priority candidates
- do not emit project counts or distributions

This keeps the new field aligned with the current recommendation surface and avoids turning it into a second ranking layer.

### 3. Cover Only Repo-State And Verification Triggers

This slice covers exactly two external-truth families:

- `repo-state`
- `verification`

Do not include:

- symbol/search boundaries such as `rg` or `git grep`
- cleanup/data-risk advisory-only review signals
- new observation-freshness trigger families beyond what is already encoded in current action sequencing

### 4. Repo-State Trigger Rules

Set `repo_state_required = true` when the top project's existing `repo_truth_gaps` includes any of:

- `repository_mid_operation`
- `working_tree_conflicted`
- `dependency_state_requires_repo_review`
- `git_metadata_unavailable`

Set `repo_state_required = false` otherwise.

`not_git_repository` must remain advisory-only in this batch:

- keep it in `repo_truth_gaps`
- do not escalate it into `repo_state_required = true`

Rationale:

- `not_git_repository` means git truth is not applicable, not that git must be consulted before proceeding
- this preserves the current "non-git boundary stays advisory" behavior

When `repo_state_required = true`, populate repo-side checks from the existing `mandatory_shell_checks`.

### 5. Verification Trigger Rules

Set `verification_required = true` only when the top project's existing `execution_sequence.mode` is:

- `run_project_verification_then_resume`
- `resolve_failing_verification_then_resume`

Map these modes to new trigger labels:

- `run_project_verification_then_resume` -> `verification_run_required`
- `resolve_failing_verification_then_resume` -> `failing_verification_repair_required`

Set `verification_required = false` otherwise.

When `verification_required = true`, populate verification-side checks from:

- `execution_sequence.verification_commands`

Do not treat OPENDOG wrapper commands such as `opendog verification` as external truth checks in this new field. The goal is to point to project-native test/lint/build commands.

### 6. Mode Rules

`mode` must have only these active values:

- `must_switch_to_external_truth`
- `opendog_guidance_can_continue`

Rules:

- if `repo_state_required || verification_required`, set `mode = "must_switch_to_external_truth"`
- otherwise set `mode = "opendog_guidance_can_continue"`

This keeps the field machine-simple and avoids over-designing a new severity ladder.

### 7. Trigger And Minimum-Check Construction

`triggers` must be a stable ordered list.

Recommended order:

1. repo-state triggers, in the order they appear from the current projection rules
2. verification trigger, if present

`minimum_external_checks` must also be stable:

1. repo-side `mandatory_shell_checks`
2. verification-side `execution_sequence.verification_commands`

Deduplicate while preserving first appearance.

This field is meant to answer:

- what is the minimum external truth handoff before broader edits

It is not meant to list every optional follow-up command.

### 8. Summary Rules

Use concise summaries only.

Preferred summaries:

- repo-state only:
  `Top project needs direct repository truth before broader changes.`
- verification only:
  `Top project needs fresh project-native verification truth before broader changes.`
- both:
  `Top project needs direct repository and verification truth before broader changes.`
- neither:
  `Current top recommendation can continue under OPENDOG guidance until a repository or verification boundary is reached.`

### 9. Empty / No-Priority Behavior

If no top recommendation exists:

```json
{
  "status": "no_priority_project",
  "source": null,
  "source_project_id": null,
  "mode": null,
  "repo_state_required": false,
  "verification_required": false,
  "triggers": [],
  "minimum_external_checks": [],
  "summary": null
}
```

This keeps the field explicit without pretending a boundary exists when no project has been selected.

## Implementation Shape

Primary implementation files:

- `src/mcp/constraints/external_truth.rs`
- `src/mcp/constraints.rs`
- `src/mcp/guidance_payload.rs`
- `src/mcp/workspace_decision.rs`

Primary test files:

- `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

Primary doc files for the later implementation batch:

- `docs/json-contracts.md`
- `docs/mcp-tool-reference.md`

Recommended helper split:

- `external_truth_boundary_for_top_project(...)`
- one shared builder that accepts the already-selected top recommendation or decision inputs
- no second computation path in `decision_brief_payload(...)`

## Test Plan

Keep tests narrow and contract-shaped.

Required cases:

1. Workspace guidance: top project requires both repo-state and verification external truth
2. Decision brief: top project requires verification external truth only
3. Advisory-only edge: `not_git_repository` stays non-blocking and does not set `repo_state_required`
4. Empty / no-priority edge: `status = "no_priority_project"`

Assertions should focus on:

- field presence
- `mode`
- `repo_state_required`
- `verification_required`
- stable `triggers`
- stable deduplicated `minimum_external_checks`
- parity between execution-strategy and decision projections where both are present

## Non-Goals

This slice must not:

- add new repository detection logic
- add new verification detection logic
- change `recommended_next_action`
- change recommendation ordering or attention scoring
- change `execution_sequence`
- change `repo_truth_gaps`
- change `mandatory_shell_checks`
- change `entrypoints.next_mcp_tools`
- change `entrypoints.next_cli_commands`
- change CLI text rendering
- add symbol/search authority triggers such as `rg` or `git grep`

## Acceptance Criteria

Implementation is complete when:

1. `guidance.layers.execution_strategy.external_truth_boundary` exists
2. `decision.external_truth_boundary` exists
3. both fields are derived from the same top-priority project state
4. repo-state triggers only fire for the listed existing `repo_truth_gaps`
5. verification triggers only fire for the listed existing `execution_sequence.mode` values
6. `not_git_repository` remains advisory-only
7. `minimum_external_checks` is stable, deduplicated, and built only from existing repo and verification command sources
8. no recommendation logic, ranking logic, or CLI text output changes are introduced
