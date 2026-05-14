---
title: "Classify infrastructure files in stats and unused views"
id: "TASK-20260510-infrastructure-file-classification"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 observation quality hardening"
ft_ids_touched:
  - FT-01.02.01
  - FT-01.04.02
  - FT-03.05.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-01.02.01 owns baseline inventory filtering; infrastructure directories may need default ignore or classification treatment."
  - "FT-01.04.02 owns hotspot and unused views; those views need source-vs-infrastructure separation to stay useful."
  - "FT-03.05.01 owns cleanup/refactor candidates; infrastructure files should not be presented as ordinary cleanup candidates without context."
  - "FT-03.07.01 owns evidence boundaries; output must state that unused means unobserved, not safe to delete."
requirement_ids:
  - SNAP-04
  - STAT-06
  - STAT-07
  - CLEAN-01
  - CLEAN-02
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not change scanner attribution semantics."
  - "Do not delete or rewrite project files."
  - "Do not make `.claude/` or similar directories globally invisible without an explicit compatibility decision."
verification_plan:
  - "Use `mystocks` evidence to measure infrastructure share in unused and hot-file outputs."
  - "Define whether infrastructure handling is default ignore, soft classification, or filterable view."
  - "Add tests for `.claude/`, `.amazonq/`, `.cursor/`, `.agents/`, `.zread/`, backup patterns, and source files."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Before/after stats for infrastructure vs source entries on `mystocks`."
  - "Updated CLI/MCP docs for infrastructure classification or filtering."
  - "Tests proving source files remain visible while infrastructure noise is controlled."
completion_notes:
  - "Implemented soft path classification for source, infrastructure, backup, and project files without changing scanner attribution or default ignore semantics."
  - "MCP stats and unused payloads now expose `classification_summary` and per-row `path_classification`."
  - "Unused guidance now prefers source-classified candidates before infrastructure noise while still keeping infrastructure entries visible and counted."
  - "CLI stats/unused output now displays path classification for operator inspection."
---

## Goal

Improve stats and unused-file usefulness by separating AI-tool infrastructure files from source-code files instead of treating every unobserved file as an equal cleanup/refactor candidate.

## Evidence Source

`/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md` records that:

- `unused` output is dominated by `.claude/`, `.amazonq/`, hooks, agent prompts, and other tool metadata.
- Hot-file results can be dominated by AI-tool config files.
- Users still need the boundary that `unused` means unobserved, not safe to delete.

## Change Plan

1. Decide whether infrastructure handling should be implemented as default ignore patterns, soft file classification, output filters, or a combination.
2. Add representative classification/ignore tests for common AI-tool directories and backup-file patterns.
3. Expose the distinction in CLI/MCP outputs without hiding evidence silently.
4. Validate on `mystocks` that source-code views are easier to inspect while infrastructure files remain explainable.

## Guardrails

- Prefer reversible classification/filtering over irreversible hiding unless the default-ignore decision is explicit.
- Keep cleanup/refactor recommendations non-destructive.
- Keep this independent from the accepted fd attribution baseline.

## Completion Criteria

- Stats and unused views can separate source and infrastructure entries.
- Guidance can explain infrastructure noise instead of treating it as ordinary source cleanup evidence.
- `mystocks` validation shows a measurable improvement in source-file signal visibility.

## Closure

This task is closed as a soft-classification implementation. Infrastructure files remain observable evidence; they are not globally hidden. Future changes that turn these classifications into default ignore behavior require an explicit compatibility decision.
