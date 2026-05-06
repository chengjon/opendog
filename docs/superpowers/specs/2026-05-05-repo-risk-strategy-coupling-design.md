# Repository Risk Strategy Coupling Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.02.01` by projecting existing repository-risk summaries directly into workspace strategy output so AI consumers can see which repository risk is shaping the current strategy posture without changing any project-level decision logic.

This slice is intentionally narrow:

- add only read-only coupling metadata under `guidance.layers.execution_strategy`
- keep repository-risk detection logic unchanged
- keep project-level `recommend_project_action(...)` logic unchanged
- keep MCP/CLI command lists unchanged
- keep existing payload structure stable and only add backward-compatible fields
- reuse existing `repo_status_risk.highest_priority_finding` and `risk_findings`

This is strategy explanation hardening, not strategy re-decision.

## Capability Scope

FT IDs touched:

- `FT-03.02.01` Repository Risk Summaries
- consumer-side effect for `FT-03.02.02` AI Execution Strategy Suggestions

Primary requirement families:

- `RISK-01..04`
- `STRAT-01`
- `STRAT-04`
- `EVID-02..03`

## Current Problem

Current Phase 6 output already exposes:

- structured repository risk findings in `project_overviews[*].repo_status_risk`
- workspace strategy mode and recommended flow in `guidance.layers.execution_strategy`
- project-level reasons and action sequencing

But the coupling is still indirect.

The AI currently has to infer:

- which repository risk finding matters most for the current workspace strategy
- which project that finding came from
- how that finding affects shell-first vs OPENDOG-first behavior

That means repository risk and strategy are both available, but the relationship between them is still mostly implicit.

## Design

### 1. Add A Thin Workspace Coupling Object

Add one read-only field:

- `guidance.layers.execution_strategy.risk_strategy_coupling`

This object exists only to explain how the current workspace strategy relates to the most strategy-relevant repository risk already present in the workspace view.

Proposed shape:

```json
{
  "status": "coupled",
  "source": "primary_repo_risk_finding",
  "source_project_id": "demo",
  "recommended_next_action": "stabilize_repository_state",
  "strategy_mode": "stabilize_before_modify",
  "preferred_primary_tool": "shell",
  "primary_repo_risk_finding": {
    "kind": "repository_operation_in_progress",
    "severity": "high",
    "priority": "immediate",
    "confidence": "high",
    "summary": "Repository is mid-operation: rebase."
  },
  "summary": "Top repository risk keeps the workspace in stabilize-before-modify mode and shell-first handling."
}
```

Fallback shape when no repository risk finding is available:

```json
{
  "status": "no_repo_risk_signal",
  "source": null,
  "source_project_id": null,
  "recommended_next_action": "<existing action>",
  "strategy_mode": "<existing mode>",
  "preferred_primary_tool": "<existing tool>",
  "primary_repo_risk_finding": null,
  "summary": null
}
```

### 2. Derive Coupling From Existing Top-Path Workspace Context

This slice must not create new ranking logic.

The coupling source should be derived from the already selected workspace top path:

- start from the first sorted project recommendation
- find the matching project overview
- reuse that overview's `repo_status_risk.highest_priority_finding`
- reuse the already computed workspace `global_strategy_mode`
- reuse the already computed workspace `preferred_primary_tool`

This keeps the coupling aligned with current workspace prioritization instead of inventing a separate repository-risk ranking system.

### 3. Keep The Coupling Explanatory Only

`risk_strategy_coupling` must not:

- override `global_strategy_mode`
- override `preferred_primary_tool`
- alter `recommended_next_action`
- inject new command suggestions
- add detection heuristics

It is a read-only explanation layer, not a strategy authority layer.

### 4. Tighten `recommended_flow` First-Step Wording When Coupling Exists

When `risk_strategy_coupling.status = "coupled"`, the first item in `guidance.recommended_flow` should mention the active repository-risk summary in a concise way.

Rules:

- change only the first step
- keep the existing step order
- keep the existing action-specific flow structure
- explain how the current risk reinforces the current strategy
- avoid claiming the risk changed the action-selection logic

Example direction:

- before:
  - `Start with project 'demo' because repository state is mid-operation and must be stabilized first.`
- after:
  - `Start with project 'demo' because repository state is mid-operation and must be stabilized first; top repository risk: Repository is mid-operation: rebase.`

For non-stabilization actions with a coupled repository risk:

- keep the action reason first
- append a short clause showing the active repository-risk boundary

### 5. Let Decision Summary Reuse The Same Tightened First Step

`decision.summary` already reads from the first `guidance.recommended_flow` step.

This slice should keep that reuse model:

- do not add a new decision-only coupling field
- let the updated first-step wording naturally tighten `decision.summary`

That preserves one source of truth for the human-readable top-line strategy explanation.

## Implementation Shape

Primary files:

- `src/mcp/guidance_payload.rs`
- `src/mcp/strategy.rs`

Likely helper additions:

- a small workspace-level helper in `guidance_payload.rs` that builds `risk_strategy_coupling`
- a small formatter in `strategy.rs` that applies coupling text only to the first recommended-flow step

Rules:

- do not touch `src/mcp/project_recommendation.rs` decision logic
- do not touch `src/mcp/repo_risk/findings.rs` detection logic
- do not add new command or tool lists
- do not add config or operator knobs

## Test Strategy

Stay inside existing workspace/decision guidance tests.

Primary test files:

- `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

Coverage goals:

1. `execution_strategy.risk_strategy_coupling` is emitted when the top workspace path has a primary repository risk finding
2. the coupling object carries the matched project id, strategy mode, preferred primary tool, and cloned primary finding
3. `guidance.recommended_flow[0]` mentions the coupled repository risk summary without changing the rest of the flow shape
4. `decision.summary` inherits the tightened first-step wording through the existing shared flow source

## Non-Goals

Do not:

- change project-level action scoring or selection
- change workspace ranking or attention scoring
- add repository-risk counts or extra dashboard fields beyond the thin coupling object
- redesign CLI presentation
- widen this slice into repo-truth, verification, or data-risk changes
