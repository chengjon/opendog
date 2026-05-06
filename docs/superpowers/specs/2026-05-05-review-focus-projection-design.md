# Review Focus Projection Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.05.01` by projecting the already-existing cleanup/refactor `review_focus` signal into OPENDOG's unified guidance and decision-consumption surfaces.

This slice is intentionally narrow:

- reuse the current top-project `review_focus` exactly as it already exists
- add one thin read-only projection under workspace execution strategy
- mirror the same review-focus value into the decision payload
- keep file-level `candidate_*` fields where they already live
- keep project action selection, scoring, and recommendation ordering unchanged
- keep CLI text output unchanged

This is a consumer-surface projection, not a new candidate engine and not a ranking rewrite.

## Capability Scope

FT IDs touched:

- `FT-03.05.01` Surface cleanup and refactor candidates

Consumer-side effects only:

- `FT-03.02.02` Recommend next-step execution strategy

Primary requirement family:

- `CLEAN-01..04`

Consumed supporting semantics only:

- `STRAT-01..04`

This batch deepens an already-shipped Phase 6 signal. It does not broaden file candidate detection or add a new aggregation family.

## Current Problem

OPENDOG already exposes two useful review-oriented signals:

- project recommendation `review_focus`
- file-level candidate metadata such as `candidate_basis`, `candidate_risk_hints`, and `candidate_priority`

But the unified AI-facing entry points still have a gap:

- `get_guidance(detail = "summary")` does not expose a dedicated top-level review-focus projection
- `get_guidance(detail = "decision")` does not expose the top review-focus signal directly inside `decision`

That means downstream AI consumers still have to inspect:

- `guidance.project_recommendations[0].review_focus`

and infer whether the current top recommendation is steering toward:

- hotspot review
- unused-file review
- or no review-family focus at all

This is not a detection problem. The signal already exists. The missing piece is a thin, stable projection on the same unified surfaces where recent Phase 6 hardening already placed:

- `risk_strategy_coupling`
- `external_truth_boundary`
- attention batching

## Design

### 1. Add One Thin Read-Only Review-Focus Projection

Add:

- `guidance.layers.execution_strategy.review_focus_projection`

and mirror the current top-project review-focus value into:

- `decision.review_focus`

The execution-strategy projection is the structured source. The decision field is a convenience mirror for the most common AI-consumption path.

### 2. Reuse Only The Existing Top-Project `review_focus`

This slice must not introduce new review-focus derivation logic.

Rules:

- source only from the current top recommendation
- do not inspect lower-priority project recommendations
- do not infer review focus from file candidates
- do not recompute review family from `recommended_next_action`
- do not add workspace-level review-focus counts or distributions

The projection exists only to surface the current top-project review intent more directly.

### 3. Preferred Projection Shape

`guidance.layers.execution_strategy.review_focus_projection` should use:

```json
{
  "status": "available",
  "source": "top_priority_project",
  "source_project_id": "demo",
  "review_focus": {
    "candidate_family": "hot_file",
    "candidate_basis": ["highest_access_activity", "activity_present"],
    "candidate_risk_hints": ["repo_risk_elevated"]
  }
}
```

Field meaning:

- `status`
  - projection state
- `source`
  - where the projection came from
- `source_project_id`
  - current top project
- `review_focus`
  - the existing top-project `review_focus` object, unchanged

This object is explanatory only. It does not change strategy selection, candidate ranking, or cleanup/refactor permission.

### 4. Decision Payload Mirrors Only The Review-Focus Value

`decision.review_focus` should mirror only:

- `guidance.layers.execution_strategy.review_focus_projection.review_focus`

Do not mirror the whole projection envelope into `decision`.

Rationale:

- `decision` should stay compact
- `decision.target_project_id` already carries most of the context needed by the consumer
- the full projection metadata remains available under `layers.execution_strategy`

### 5. Empty And Non-Review Behavior

This slice must distinguish three states:

1. top project exists and has review focus
- `status = "available"`
- `review_focus` is the current object

2. top project exists but selected action is not a cleanup/refactor review action
- `status = "available"`
- `review_focus = null`

3. no top project exists
- `status = "no_priority_project"`
- `source = null`
- `source_project_id = null`
- `review_focus = null`

Preferred empty shape:

```json
{
  "status": "no_priority_project",
  "source": null,
  "source_project_id": null,
  "review_focus": null
}
```

This keeps the projection explicit without pretending a review focus exists when no project is currently selected.

### 6. Preserve Existing `review_focus` Vocabulary

This batch must not change the current `review_focus` schema or vocabulary.

Do not add:

- new `candidate_family` values
- new `candidate_basis` labels
- new `candidate_risk_hints`
- file previews
- candidate counts
- candidate lists

Current supported behavior remains:

- review actions may expose `hot_file` or `unused_candidate`
- non-review actions keep `review_focus = null`

### 7. Keep File-Level Candidate Detail Out Of The Unified Surface

This slice must not project `file_recommendations[*]` into:

- `guidance.layers.execution_strategy`
- `decision`

Reason:

- file-level candidate detail already belongs to `stats` and `unused` guidance
- projecting candidate previews into unified AI entry points would widen the surface and blur the current layering
- the user explicitly requested the smallest useful projection

This batch is about family-level review intent, not candidate payload expansion.

## Implementation Shape

Primary implementation files:

- `src/mcp/constraints/review_focus.rs`
- `src/mcp/constraints.rs`
- `src/mcp/mod.rs`
- `src/mcp/guidance_payload.rs`
- `src/mcp/workspace_decision.rs`

Primary test files:

- `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

Primary doc files for the implementation batch:

- `docs/json-contracts.md`
- `docs/mcp-tool-reference.md`

Preferred helper shape:

- `review_focus_projection_for_top_project(top_recommendation: Option<&Value>) -> Value`

Rules:

- accept only the already-selected top recommendation
- copy `review_focus` as-is
- do not inspect file candidates
- do not recompute action class
- mirror into `decision.review_focus` rather than creating a second logic path

## Test Plan

Keep tests narrow and contract-shaped.

Required cases:

1. summary guidance: top project exposes `hot_file` review focus
2. decision brief: top project exposes `unused_candidate` review focus
3. non-review action: projection exists but `review_focus = null`
4. no-priority state: projection reports `no_priority_project` and `decision.review_focus = null`

Assertions should focus on:

- field presence
- `status`
- `source_project_id`
- unchanged `review_focus` structure
- decision mirror parity
- explicit null behavior for non-review and no-priority states

## Non-Goals

This slice must not:

- change `recommended_next_action`
- change project recommendation ordering
- change `review_focus_for_action(...)`
- change file-level candidate ranking
- add candidate previews to `decision`
- add workspace review-focus aggregation
- change CLI text rendering
- change existing `candidate_*` fields
- broaden cleanup/refactor capability scope outside the current review-family signal

## Acceptance Criteria

Implementation is complete when:

1. `guidance.layers.execution_strategy.review_focus_projection` exists
2. `decision.review_focus` exists
3. both fields derive from the same current top-project `review_focus`
4. non-review top actions keep projection `status = "available"` with `review_focus = null`
5. no-priority state returns explicit projection null-state metadata
6. no file-level candidate data is added to unified guidance or decision payloads
7. no recommendation logic, ranking logic, or CLI text behavior changes are introduced
