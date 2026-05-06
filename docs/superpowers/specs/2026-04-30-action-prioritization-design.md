# Action Prioritization Design

Date: 2026-04-30
Status: implemented and verified (2026-05-06)
Scope: Phase 6 selective deepening

## Goal

Strengthen project-level action prioritization so OPENDOG chooses safer next actions when verification evidence is weak, repository state is unstable, or observation freshness is not strong enough to support cleanup or refactor review.

The target outcome is narrower than a new capability family:

- keep the current `recommended_next_action` enum unchanged
- improve internal action ordering and weighting
- make recommendation reasons more stable and more explainable
- keep `agent_guidance` and `decision_brief` aligned because they consume the same recommendation result

## Capability Scope

FT IDs touched:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.03.01` Record and reason over verification evidence
- `FT-03.05.01` Surface cleanup and refactor candidates
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `STRAT-01..04`
- `EVID-01..04`
- `CLEAN-01..04`
- `RISK-01..04`
- `OBS-02..04`
- `BOUND-01..04`

## Current Problem

`recommend_project_action(...)` already uses verification, repository risk, and observation signals, but the effective prioritization model is still branch-heavy and too implicit.

The current weaknesses are:

- verification `caution` versus `blocked` does not influence action ordering strongly enough
- unstable repository state can still leave modification-adjacent review actions too close to the front
- freshness and evidence-quality signals influence choice, but their effect is not structured enough to produce stable explanations
- reason strings are generated inline inside action branches, so the same underlying fact pattern can produce uneven wording across recommendation surfaces

This shows up most clearly when OPENDOG must choose between `review_unused_files` and `inspect_hot_files`. The system has the right facts, but the ranking logic needs to become more explicit and more reusable.

## Design

### 1. Keep The Public Action Contract Stable

This work does not change the visible action vocabulary.

It keeps:

- the existing `recommended_next_action` enum
- the existing `strategy_mode` values
- the same MCP tool surfaces
- the same schema versions

It does not add hard blocking to CLI or MCP execution.

The only intended outward changes are:

- safer action choice under mixed evidence
- sharper `confidence`
- more stable `reason`

### 2. Two-Stage Action Selection

Action selection should stop behaving like one long `if/else` chain and instead become a two-stage model:

1. hard gating
2. lightweight scoring among eligible actions

This preserves clear short-circuit behavior for obviously unsafe states while still allowing finer prioritization when multiple actions remain valid.

### 3. Hard Gating Order

Hard gating order is fixed and must remain explicit:

1. verification gate
2. repository risk / operation state
3. observation readiness and freshness

#### Verification gate

Verification remains the strongest signal.

Rules:

- failing or uncertain verification short-circuits to `review_failing_verification`
- if both cleanup and refactor review are blocked by required evidence gaps, short-circuit to `run_verification_before_high_risk_changes`
- if cleanup is allowed or caution but refactor is blocked, `review_unused_files` remains eligible while `inspect_hot_files` is removed from the candidate set
- if refactor is allowed or caution but cleanup is blocked, `inspect_hot_files` remains eligible while `review_unused_files` is removed from the candidate set

`caution` is not a block, but it must lower later confidence and scoring.

#### Repository risk and operation state

Repository instability stays below verification in priority, but still outranks freshness-based cleanup or refactor review.

Rules:

- merge, rebase, cherry-pick, or bisect states short-circuit to `stabilize_repository_state`
- other high-risk repository findings do not always force a short-circuit, but they apply stronger penalties to `inspect_hot_files` than to `review_unused_files`

This asymmetry is intentional: hotspot inspection is closer to code modification flow than unused-file review.

#### Observation readiness and freshness

Observation gaps remain short-circuit states before file-review actions:

- not monitored -> `start_monitor`
- no useful snapshot baseline -> `take_snapshot`
- insufficient or stale activity evidence for hotspot judgment -> `generate_activity_then_stats`

Freshness should only participate in scoring after these stronger readiness failures are ruled out.

### 4. Lightweight Scoring Model

Scoring only compares actions that survive hard gating.

The main comparison target is still:

- `review_unused_files`
- `inspect_hot_files`

The model should not try to produce a global numeric truth. It only needs enough structure to make ranking explainable and testable.

Recommended scoring inputs:

- verification gate level
- repository risk level and specific operation-state findings
- snapshot freshness
- activity freshness
- evidence quality needed by the candidate action

Recommended weighting:

- verification has the highest weight
- repository risk has the second-highest weight
- observation freshness has the third-highest weight

Action-specific penalties:

- stale snapshot penalizes `review_unused_files` more strongly
- stale activity penalizes `inspect_hot_files` more strongly
- repository instability penalizes `inspect_hot_files` more strongly
- verification `caution` penalizes both actions, but refactor-oriented review should receive the larger penalty

`blocked` actions do not get scored. They are filtered out during eligibility.

### 5. Internal Responsibility Split

Keep `recommend_project_action(...)` as the public facade, but move the internals into focused helpers under `src/mcp/project_recommendation/`.

Suggested structure:

- `eligibility.rs`
- `scoring.rs`
- `reasoning.rs`
- `mod.rs`

Responsibilities:

#### `eligibility`

Inputs:

- verification readiness snapshot
- repo risk layer
- observation and freshness facts

Outputs:

- forced action, if short-circuiting applies
- per-action eligibility state
- elimination reasons for ineligible actions

#### `scoring`

Inputs:

- eligible actions only
- normalized signal facts

Outputs:

- total action score
- score breakdown by factor

The breakdown is internal, but it should be stable enough for tests.

#### `reasoning`

Inputs:

- selected action
- runner-up action when present
- dominant gate and score breakdown
- eliminated alternatives

Outputs:

- `reason`
- `confidence`

#### `mod`

Responsible only for assembling the current recommendation payload shape.

This separation keeps payload formatting from re-owning ranking rules.

### 6. Stable Reason Generation

Reason generation should become deterministic for the same fact pattern.

Required explanation order:

1. dominant constraint
2. why the chosen action is safer or more appropriate now
3. why the obvious alternative was not selected

Dominant constraint priority is fixed:

- verification gate
- repository risk
- observation freshness

This order matters because otherwise `agent_guidance` and `decision_brief` can describe the same recommendation from different angles and look inconsistent.

The reasoning helper should prefer consistent nouns over branch-specific prose:

- `verification evidence`
- `repository state`
- `snapshot baseline`
- `activity evidence`
- `cleanup review`
- `refactor review`

### 7. Confidence Rules

`confidence` should no longer behave like a mostly static action label.

Recommended interpretation:

- `high`
  - short-circuit state is obvious and supported by a single dominant signal
  - examples: failing verification, active rebase, missing snapshot

- `medium`
  - action is selected after gating and scoring, or any relevant gate is only `caution`
  - this should be the normal result for mixed but still actionable evidence

- `low`
  - optional for now
  - only needed if future implementation finds cases where two actions remain nearly tied and evidence is materially ambiguous

This design does not require introducing `low` immediately if existing surfaces and tests are cleaner with `high|medium`.

### 8. Consumer Behavior

`agent_guidance` and `decision_brief` remain consumers, not decision-makers.

They should:

- reuse the updated recommendation payload
- surface the stabilized `reason`
- preserve the selected `recommended_next_action`
- avoid layering their own alternative prioritization rules on top

This keeps the recommendation chain single-sourced:

1. shared evidence and readiness helpers compute facts
2. `recommend_project_action(...)` chooses the action
3. `agent_guidance` and `decision_brief` present that result

## Testing Plan

Testing should cover three layers.

### 1. Internal ranking unit tests

Add focused tests for:

- verification `caution` versus `blocked`
- cleanup eligible while refactor is blocked
- refactor eligible while cleanup is blocked
- repo operation state suppressing file-review actions
- stale snapshot penalizing cleanup review
- stale activity penalizing hotspot inspection

### 2. Recommendation behavior tests

Update and extend `recommend_project_action(...)` tests to prove:

- verification gates outrank repo risk and freshness
- repo risk outranks freshness
- `review_unused_files` wins when cleanup evidence is acceptable but refactor evidence is not
- `inspect_hot_files` is held back when repository instability is present
- `reason` includes both the winning explanation and the losing alternative rationale
- `confidence` drops to `medium` for caution-based wins

### 3. Consumer consistency tests

Update `agent_guidance` and `decision_brief` contract tests to prove:

- both surfaces reuse the same chosen action
- both surfaces expose compatible explanation language
- no schema regressions are introduced

Validation commands during implementation:

- `cargo test repo_and_readiness`
- `cargo test guidance_basics`
- `cargo test portfolio_commands`
- `cargo test`
- `python3 scripts/validate_planning_governance.py` if planning docs change

## Non-Goals

- no new `recommended_next_action` values
- no hard-blocking command execution
- no CLI text-output rewrite in this batch
- no new MCP tool
- no new verification storage model
- no project-level portfolio reorder work beyond what is necessary to consume the chosen action consistently

## Risks And Controls

- Risk: ranking rules remain spread across payload layers.
  Control: keep action choice inside `recommend_project_action(...)` and focused helper modules only.

- Risk: verification `caution` still behaves too much like `allow`.
  Control: use `caution` as a scoring penalty and confidence cap even when it is not a hard block.

- Risk: repository instability is treated too generically.
  Control: give operation-state findings explicit short-circuit authority and stronger penalties for refactor-adjacent review.

- Risk: explanations still drift between guidance surfaces.
  Control: centralize reason generation and keep consumer layers read-only.

- Risk: the scoring model becomes too opaque.
  Control: keep it lightweight, deterministic, and covered by direct unit tests.

## Expected Outcome

After this work, OPENDOG should make project-level next-action choices that are more conservative when evidence is weak, more precise when cleanup and refactor signals diverge, and more consistent across guidance surfaces.

Specifically:

- verification weakness suppresses riskier review actions earlier
- repository instability stops refactor-adjacent review from surfacing too aggressively
- freshness gaps affect the right action in the right direction
- `agent_guidance` and `decision_brief` tell the same story because they consume the same stabilized recommendation result
