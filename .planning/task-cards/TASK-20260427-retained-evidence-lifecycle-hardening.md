---
title: "Formalize retained-evidence lifecycle and storage hygiene"
id: "TASK-20260427-retained-evidence-lifecycle-hardening"
status: completed
owner: "codex"
priority: medium
phase_hint: "Executed as retrospective hardening for Phase 6 retained-evidence lifecycle"
ft_ids_touched:
  - FT-01.04.05
why_these_ft_ids:
  - "FT-01.04.05 owns selective cleanup of retained OPENDOG evidence plus storage-hygiene signals such as reclaimable bytes, optimize, and VACUUM guidance."
requirement_ids:
  - RET-01
  - RET-02
  - RET-03
  - RET-04
  - RET-05
  - RET-06
lifecycle_impact:
  - ft_id: FT-01.04.05
    from: designing
    to: shipped
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not delete or rewrite project source files."
  - "Do not make `VACUUM` automatic or implicit."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Run Rust tests that cover cleanup scope behavior, dry-run previews, storage metrics, and baseline-preservation safety rules."
  - "Inspect representative CLI/MCP cleanup payloads and guidance/decision-brief storage-maintenance fields."
evidence_outputs:
  - "Governance validation output"
  - "Retention and storage-metrics test output"
  - "Example cleanup and storage-maintenance payloads"
---

## Goal

Let users and AI agents review, prune, and compact retained OPENDOG evidence safely so long-lived multi-project deployments can manage storage without confusing that work with source-code cleanup.

## Capability Scope

- `FT-01.04.05`

## Requirement Scope

This card covers the complete retained-evidence lifecycle family:

- `RET-01`
- `RET-02`
- `RET-03`
- `RET-04`
- `RET-05`
- `RET-06`

## Change Plan

1. Define selective retained-evidence cleanup by scope, including dry-run previews before mutation.
2. Preserve current snapshot baseline and aggregate file-usage evidence while pruning historical activity, verification, or snapshot rows.
3. Expose storage metrics, optimize/VACUUM outcomes, and storage-maintenance guidance through daemon, CLI, and MCP surfaces.

## Guardrails

- Do not expand this card into repo-file cleanup or destructive source-code operations.
- Do not silently compact the database; maintenance actions must stay explicit and auditable.
- Do not remove evidence needed for current snapshot baseline or stable aggregate file statistics.

## Verification

- `python3 scripts/validate_planning_governance.py`
- retained-evidence cleanup and storage-metrics Rust tests
- CLI/MCP payload review for cleanup and storage-maintenance outputs

## Completion Criteria

- retained OPENDOG evidence can be previewed and pruned by bounded scope
- storage-hygiene signals are explicit, auditable, and separate from source cleanup
- snapshot baseline and aggregate evidence safety rules remain intact
- requirement and FT mappings remain valid
