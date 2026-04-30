---
title: "Add portable usage export surfaces"
id: "TASK-20260427-portable-usage-export"
status: completed
owner: "codex"
priority: medium
phase_hint: "Executed as cross-phase hardening for Phase 3/4 surfaces"
ft_ids_touched:
  - FT-01.04.03
why_these_ft_ids:
  - "FT-01.04.03 owns reusable export of usage evidence into machine-portable formats."
requirement_ids:
  - EXPORT-01
  - EXPORT-02
lifecycle_impact:
  - ft_id: FT-01.04.03
    from: designing
    to: shipped
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not add scheduled export jobs or remote sinks."
  - "Do not redesign the entire guidance contract; keep scope on exportable analytics evidence."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Add tests that verify stable JSON fields and deterministic CSV columns for exported project statistics."
  - "Inspect sample export outputs for stats, unused, and hotspot evidence."
evidence_outputs:
  - "Governance validation output"
  - "Sample JSON export artifact"
  - "Sample CSV export artifact"
---

## Goal

Allow users and AI systems to carry OPENDOG analytics evidence into review, archival, or downstream automation workflows without scraping terminal output.

## Capability Scope

- `FT-01.04.03`

## Requirement Scope

This card covers:

- `EXPORT-01`
- `EXPORT-02`

Initial delivery should stay focused on project statistics and closely related evidence rows.

## Change Plan

1. Define the stable export contract for project statistics and file-level evidence.
2. Add JSON and CSV export paths with clear file naming and column rules.
3. Verify exported output remains deterministic enough for later AI or user review workflows.

## Guardrails

- Do not scope-creep into time-window analytics or snapshot comparison in this card.
- Do not make exported fields depend on presentation-only CLI formatting.
- Do not add write destinations other than explicit user-requested export paths.

## Verification

- `python3 scripts/validate_planning_governance.py`
- export contract tests
- manual inspection of sample JSON and CSV outputs

## Completion Criteria

- JSON and CSV exports cover the intended requirement scope
- exported schemas are explicit and reusable
- task-card and requirement mappings remain valid
