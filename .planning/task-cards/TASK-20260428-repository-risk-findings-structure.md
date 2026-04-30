---
title: "Structure repository risk findings for AI consumption"
id: "TASK-20260428-repository-risk-findings-structure"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 hardening slice for repository risk summaries and decision support"
ft_ids_touched:
  - FT-03.02.01
  - FT-03.02.02
  - FT-03.07.01
why_these_ft_ids:
  - "FT-03.02.01 owns repository risk summaries and should expose machine-readable findings instead of only free-form reason strings."
  - "FT-03.02.02 owns next-step sequencing, so decision payloads should surface the top repository risk item directly."
  - "FT-03.07.01 owns evidence boundaries and should keep direct git observations distinguishable from derived advice."
requirement_ids:
  - RISK-01
  - RISK-02
  - RISK-03
  - RISK-04
  - STRAT-04
  - EVID-01
  - EVID-02
  - EVID-03
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not add new git probes or shell dependencies."
  - "Do not replace project-native verification or shell truth with OPENDOG heuristics."
  - "Do not redesign the whole decision brief; keep the slice focused on structured repository risk."
verification_plan:
  - "Run targeted Rust tests for repository risk findings and decision-brief propagation."
  - "Run `cargo test`."
  - "Run `python3 scripts/validate_task_cards.py` and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "New unit tests for repository risk finding structure and top-risk propagation"
  - "Updated JSON-contract and MCP/AI docs for structured repo risk fields"
  - "Validated task-card and planning-governance output"
---

## Goal

Make repository risk summaries easier for AI to consume by exposing structured findings, severity, priority, confidence, and the highest-priority repository risk item.

## Capability Scope

- `FT-03.02.01`
- `FT-03.02.02`
- `FT-03.07.01`

## Requirement Scope

This card hardens the current repository risk summary shape. It does not add new repository inspection sources.

## Change Plan

1. Turn repository risk signals into structured `risk_findings` with severity, priority, confidence, and direct evidence.
2. Expose `highest_priority_finding` and severity counts in `repo_status_risk`.
3. Propagate the primary repository risk finding into decision-facing payloads and docs.

## Guardrails

- Keep `risk_reasons` for backward compatibility while adding structured findings.
- Do not claim repo risk findings are anything other than git-derived observations plus bounded interpretation.
- Do not broaden the slice into unrelated cleanup or data-risk ranking logic.

## Verification

- targeted Phase 6 repository-risk tests
- `cargo test`
- governance validators

## Completion Criteria

- `repo_status_risk` exposes machine-readable findings and a top finding
- `decision.risk_profile` can surface the primary repository risk item
- docs explain the new fields without changing OPENDOG’s authority boundary

## Completion Note

Repository risk now exposes structured `risk_findings`, `highest_priority_finding`, and `finding_counts` instead of relying only on free-form reason strings.

Those fields are now available to downstream AI and operator flows through:

- `project_overviews[*].repo_status_risk.*`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.repo_risk_findings`
- `decision.risk_profile.repo_risk_finding_counts`

This keeps repository-risk reasoning machine-readable while preserving the existing `risk_reasons` compatibility field.
