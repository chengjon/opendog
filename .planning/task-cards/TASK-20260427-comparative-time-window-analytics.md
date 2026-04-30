---
title: "Add comparative and time-windowed analytics"
id: "TASK-20260427-comparative-time-window-analytics"
status: completed
owner: "codex"
priority: medium
phase_hint: "Backlog candidate"
ft_ids_touched:
  - FT-01.04.04
why_these_ft_ids:
  - "FT-01.04.04 owns time-window summaries, snapshot comparison, and usage-trend reporting."
requirement_ids:
  - RPT-01
  - RPT-02
  - RPT-03
lifecycle_impact:
  - ft_id: FT-01.04.04
    from: designing
    to: shipped
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not build a visual dashboard."
  - "Do not promise predictive analytics; keep scope on historical comparison and summarization."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Add tests for time-window filters, snapshot comparison correctness, and trend-summary query behavior."
  - "Inspect representative CLI/MCP outputs for 24h, 7d, and snapshot-diff scenarios."
evidence_outputs:
  - "Governance validation output"
  - "Time-window query test output"
  - "Snapshot-diff example output"
---

## Goal

Turn raw usage evidence into comparative reporting so operators and AI can tell not just what is hot now, but what changed, what stayed cold, and which files are trending.

## Capability Scope

- `FT-01.04.04`

## Requirement Scope

This card covers:

- `RPT-01`
- `RPT-02`
- `RPT-03`

## Change Plan

1. Define the query model for time windows and trend aggregation.
2. Add snapshot comparison outputs that show adds, removals, and changed files clearly.
3. Expose concise trend and comparison outputs through CLI and MCP without overloading the base stats surface.

## Guardrails

- Do not hide approximation limits in time-based analytics; carry forward confidence and freshness caveats where needed.
- Do not intermingle export-file concerns with this card; use the dedicated export card for that.
- Do not mark trend outputs complete without verifying snapshot-diff correctness.

## Verification

- `python3 scripts/validate_planning_governance.py`
- time-window and comparison tests
- CLI/MCP sample output review

## Completion Criteria

- time-window analytics and snapshot comparison are exposed through supported interfaces
- trend outputs stay bounded by actual recorded evidence
- task-card and requirement mappings remain valid
