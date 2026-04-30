---
title: "Stabilize workspace portfolio attention scoring"
id: "TASK-20260428-workspace-portfolio-attention-scoring"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 hardening slice for cross-project prioritization and execution sequencing"
ft_ids_touched:
  - FT-03.04.01
  - FT-03.02.02
  - FT-03.07.01
why_these_ft_ids:
  - "FT-03.04.01 owns cross-project prioritization and should rank projects by actual attention urgency rather than incidental signal volume."
  - "FT-03.02.02 owns next-step sequencing, so project recommendations must sort by action urgency and evidence quality."
  - "FT-03.07.01 owns explicit boundaries, so priority ordering must expose machine-readable reasons instead of opaque tuple sorting."
requirement_ids:
  - PORT-01
  - PORT-02
  - PORT-03
  - PORT-04
  - STRAT-02
  - STRAT-04
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not add a new MCP tool or a new capability family."
  - "Do not change project-level evidence collection; only change how existing evidence is prioritized and explained."
  - "Do not turn workspace ranking into authority; repo-native verification and shell truth still win."
verification_plan:
  - "Run targeted Rust tests covering portfolio ranking and decision-brief payloads."
  - "Run `cargo test`."
  - "Run `python3 scripts/validate_task_cards.py` and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Updated unit tests proving failing verification and repo instability outrank lower-risk cleanup candidates"
  - "Updated integration coverage for machine-readable attention score fields"
  - "Aligned JSON-contract and AI-playbook docs for workspace priority reasoning"
---

## Goal

Make workspace-level project ranking explicit, stable, and machine-readable so AI can understand why one project deserves attention before another.

## Capability Scope

- `FT-03.04.01`
- `FT-03.02.02`
- `FT-03.07.01`

## Requirement Scope

This card hardens ranking and explanation logic for already shipped Phase 6 guidance surfaces. It does not introduce new evidence sources.

## Change Plan

1. Replace opaque workspace sorting with explicit attention scoring based on action urgency, repo risk, evidence gaps, and risky data-review signals.
2. Expose machine-readable attention score, attention band, and ranking reasons in priority candidates and workspace portfolio summaries.
3. Keep decision-brief and CLI summaries aligned with the same ranking explanation contract.

## Guardrails

- Do not optimize ranking only for mock/hardcoded counts.
- Do not hide stale or missing evidence behind a single generic confidence label.
- Do not remove raw project overview fields that downstream consumers may already use.

## Verification

- targeted Phase 6 ranking tests
- `cargo test`
- governance validators

## Completion Criteria

- workspace priority ordering reflects attention urgency before heuristic candidate volume
- priority candidates and attention queue expose machine-readable score and reasons
- decision surfaces remain advisory and boundary-aware

## Completion Note

Workspace ranking now uses a shared attention-scoring model that combines action urgency, repository risk, evidence freshness/coverage, and risky data-review signals.

The same machine-readable attention fields are now exposed across:

- `guidance.layers.multi_project_portfolio.priority_candidates[*]`
- `guidance.layers.multi_project_portfolio.attention_queue[*]`
- `guidance.layers.multi_project_portfolio.project_overviews[*]`
- `decision.signals.attention_*`

This keeps portfolio ordering explainable for both AI and human operators without introducing a new MCP tool or changing evidence collection semantics.
