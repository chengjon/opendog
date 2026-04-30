# Verification Soft Gates Design

Date: 2026-04-30
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen OPENDOG's verification and evidence workflow so cleanup and refactor guidance becomes more precise, more explainable, and more consistent across MCP and CLI surfaces without hard-blocking execution.

The target outcome is a soft-gate model that:

- keeps existing commands and payloads compatible
- improves `safe_for_cleanup` and `safe_for_refactor`
- distinguishes missing, failing, and stale evidence explicitly
- gives actionable next steps instead of generic blocker text

## Capability Scope

FT IDs touched:

- `FT-03.03.01` Record and reason over verification evidence
- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.01.01` Explain readiness and evidence gaps
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `EVID-01..04`
- `STRAT-01..04`
- `OBS-02`
- `BOUND-01..04`

## Current Problem

Verification readiness is already surfaced, but the logic is spread across multiple layers:

- `verification_status_layer(...)`
- `project_readiness_reasons(...)`
- `project_overview(...)`
- `recommend_project_action(...)`
- stats / unused guidance paths
- `build_constraints_boundaries_layer(...)`

Today the system can say "safe" or "blocked", but it does not expose a strong enough shared explanation model. Missing evidence, stale evidence, and failing evidence are not represented with enough structure, so downstream guidance has to infer too much.

## Design

### 1. Single Source of Truth

`verification_status_layer(...)` becomes the only layer allowed to compute verification gate status directly.

It will continue returning the current compatibility fields:

- `safe_for_cleanup`
- `safe_for_refactor`
- `cleanup_blockers`
- `refactor_blockers`
- `safe_for_cleanup_reason`
- `safe_for_refactor_reason`

It will also add a new structured section:

```json
"gate_assessment": {
  "cleanup": {
    "allowed": false,
    "level": "blocked",
    "required_kinds": ["test"],
    "advisory_kinds": ["lint", "build"],
    "missing_kinds": ["test"],
    "failing_kinds": [],
    "stale_kinds": [],
    "reasons": ["No recorded test evidence is available."],
    "next_steps": ["Run a project-appropriate test command and record the result."]
  },
  "refactor": {
    "allowed": false,
    "level": "blocked",
    "required_kinds": ["test", "build"],
    "advisory_kinds": ["lint"],
    "missing_kinds": ["build"],
    "failing_kinds": [],
    "stale_kinds": [],
    "reasons": ["Build evidence is missing for refactor readiness."],
    "next_steps": ["Run a project-appropriate build command and refresh verification evidence."]
  }
}
```

All existing top-level gate booleans and reason fields become derived values from this structure.

Compatibility derivation is explicit:

- verification-layer `safe_for_cleanup` and `safe_for_refactor` remain `true` when the corresponding gate is `caution` or `allow`
- verification-layer `cleanup_blockers` and `refactor_blockers` remain blocker-only compatibility fields; advisory-only caution reasons do not appear there
- project-level `safe_for_cleanup` and `safe_for_refactor` remain `true` when verification is not blocked and repo-risk adds no extra blockers
- the new `level` field is the only place that distinguishes `caution` from `allow`

### 2. Gate Rules

`cleanup` gate:

- required kinds: `test`
- advisory kinds: `lint`, `build`
- `blocked` when required evidence is missing, stale, or failing
- `caution` when required evidence is fresh and passing but advisory evidence is missing or stale
- `allow` when required evidence is fresh and passing and no failing latest runs exist

`refactor` gate:

- required kinds: `test`, `build`
- advisory kinds: `lint`
- `blocked` when any required kind is missing, stale, or failing
- `caution` when required kinds are fresh and passing but advisory evidence is missing or stale
- `allow` when required kinds are fresh and passing and no failing latest runs exist

This preserves soft gating: `level` changes recommendations and explanations, but does not reject command execution.

Staleness keeps the current implementation threshold from `verification_is_stale()` in `src/mcp/observation.rs`. This design reuses the existing freshness windows and does not change them.

### 3. Consumer Order

Consumers must stop recomputing verification readiness ad hoc.

1. `verification_status_layer(...)`
   Computes `gate_assessment`.
2. `project_readiness_reasons(...)`
   Remains the shared combiner for repo-risk plus blocker-only verification compatibility fields. It must consume `gate_assessment`-derived compatibility values rather than recreating verification rules.
3. `project_overview(...)`
   Combines verification gates with repo-risk gates into project-level readiness fields.
4. `recommend_project_action(...)`
   Uses gate level to choose between recovery, caution, or proceed flows.
5. `stats_guidance(...)` and `unused_guidance(...)`
   Consume the shared readiness combiner instead of inventing local gate logic.
6. `build_constraints_boundaries_layer(...)`
   Continues emitting blocker arrays, but must route through the shared readiness combiner instead of assembling separate verification logic.
7. `workspace_verification_evidence_layer(...)`, `agent_guidance`, and `decision_brief`
   Aggregate and explain; do not create new verification rules.

## Compatibility

This is a non-breaking enhancement:

- no existing CLI or MCP command becomes a hard failure because of gate status
- all existing payload fields remain present
- new structure is additive
- old fields stay semantically stable because they are derived from the new gate model

## Implementation Order

1. Upgrade `verification_status_layer(...)` and related helpers to emit `gate_assessment`.
2. Add contract tests for missing, stale, failing, caution, and allow cases.
3. Route `project_readiness_reasons(...)`, `project_overview(...)`, and `recommend_project_action(...)` through the new gate structure.
4. Update stats / unused guidance and `build_constraints_boundaries_layer(...)` to consume the shared readiness combiner.
5. Extend workspace aggregation and decision-brief summaries with gate-based rollups.

## Testing Plan

Add and update tests to prove:

- no verification runs yields `blocked`
- passing fresh `test` enables cleanup but not refactor if `build` is missing
- stale required evidence blocks the relevant gate
- passing required evidence plus missing advisory evidence yields `caution`
- `caution` keeps legacy `safe_for_* == true` while exposing the stricter gate level separately
- advisory-only `caution` does not populate legacy `*_blockers`
- any failing latest verification blocks affected gates
- old compatibility fields match the new `gate_assessment` result
- recommendation and brief layers do not contradict verification gates

Validation commands for implementation:

- `cargo test repo_and_readiness`
- `cargo test guidance_basics`
- `cargo test portfolio_commands`
- `cargo test`
- `python3 scripts/validate_planning_governance.py` if planning docs change further

## Non-Goals

- no hard-blocking CLI or MCP command execution
- no new requirement family
- no changes to verification storage schema unless later implementation proves it necessary
- no automatic repository mutation based on verification results

## Risks and Controls

- Risk: duplicated gate logic remains in higher layers.
  Control: only `verification_status_layer(...)` computes verification gate state.

- Risk: stale, missing, and failing evidence remain conflated.
  Control: expose `missing_kinds`, `failing_kinds`, and `stale_kinds` separately.

- Risk: existing consumers break on payload shape changes.
  Control: keep all current top-level readiness fields and derive them from the new gate model.

- Risk: recommendations and readiness summaries diverge.
  Control: `recommend_project_action(...)` consumes gate output rather than re-deriving it.

- Risk: `caution` silently changes legacy readiness semantics.
  Control: keep `safe_for_*` compatible with current required-gate behavior and reserve `level` for the new distinction.

## Expected Outcome

After this work, OPENDOG will still be advisory and non-destructive, but its verification-based guidance will be materially sharper:

- safer cleanup and refactor readiness signals
- clearer explanation of why work is blocked, cautioned, or allowed
- better next-step suggestions when evidence is incomplete
- consistent verification reasoning across project, workspace, and decision-brief layers
