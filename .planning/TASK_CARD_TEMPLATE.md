# Task Card Template

Use this template for future execution tickets, implementation cards, or focused planning slices.

This template is function-tree-first: capability impact must be declared before implementation detail.

Store concrete cards under `.planning/task-cards/`.
Preferred governance check: `python3 scripts/validate_planning_governance.py`.
Task-card-only check: `python3 scripts/validate_task_cards.py`.

---
title: "<short task title>"
id: "TASK-YYYYMMDD-<slug>"
status: proposed
owner: "<human-or-agent>"
priority: medium
phase_hint: "<roadmap phase or ad hoc>"
ft_ids_touched:
  - FT-<leaf-id>
why_these_ft_ids:
  - "<why this task changes that capability>"
requirement_ids:
  - "<REQ or family id>"
lifecycle_impact:
  - ft_id: FT-<leaf-id>
    from: designing
    to: in_progress
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "<what this task explicitly will not change>"
verification_plan:
  - "<tests, manual checks, or gate commands>"
evidence_outputs:
  - "<docs, payload fields, logs, screenshots, test output>"
---

## Goal

State the user-visible or business-capability outcome, not the implementation mechanics.

## Capability Scope

- `FT IDs touched`: required
- `why these FT IDs`: required
- If a task cannot name at least one `FT-*` leaf node, it is not ready for execution

## Requirement Scope

List exact requirement IDs when known.

If a task spans a requirement family, say so explicitly and keep the scope bounded.

## Change Plan

1. `<step>`
2. `<step>`
3. `<step>`

## Guardrails

- Do not silently expand scope outside the listed `FT-*` leaf nodes
- Do not introduce new business capability without updating `.planning/FUNCTION_TREE.md`
- Do not mark capability progress complete without matching verification evidence

## Verification

- Required commands:
  - `<command>`
- Required review evidence:
  - `<evidence>`

## Completion Criteria

- capability behavior changed as intended
- mapped requirement scope remains accurate
- verification evidence recorded
- any lifecycle changes are reflected back into `.planning/FUNCTION_TREE.md`

## Mapping Notes

### Minimal required fields

- `ft_ids_touched`
- `why_these_ft_ids`
- `verification_plan`

### Recommended fields

- `requirement_ids`
- `lifecycle_impact`
- `interface_surfaces`
- `evidence_outputs`
