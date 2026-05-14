---
title: "Add source-first observation views"
id: "TASK-20260511-source-first-observation-views"
status: completed
owner: "unassigned"
priority: high
phase_hint: "Phase 6 observation quality hardening"
ft_ids_touched:
  - FT-01.04.02
  - FT-03.01.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-01.04.02 owns hotspot and unused views; source-first filtering is the smallest fix for infrastructure-dominated outputs."
  - "FT-03.01.01 owns evidence gaps; guidance should explain when source access evidence is absent because reads are too transient for fd sampling."
  - "FT-03.07.01 owns authority boundaries; outputs must not imply access_count=0 means source files were not read or are safe to delete."
requirement_ids:
  - STAT-06
  - STAT-07
  - OBS-02
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not change `/proc/<pid>/fd` scanner attribution semantics."
  - "Do not globally hide or ignore `.claude/` infrastructure evidence."
  - "Do not reopen fixed Case H or Case I."
  - "Do not claim source files are unused solely because access_count remains zero."
verification_plan:
  - "Add or update contract tests showing stats/unused can expose source-focused rows without losing classification_summary and result_window metadata."
  - "Verify guidance text distinguishes transient-read invisibility from scanner failure."
  - "Run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`."
  - "Run `python3 scripts/validate_task_cards.py` and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "mystocks source-signal calibration evidence in `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`."
  - "Shared issue entry `ODX-20260511-source-signal-observation-calibration`."
  - "Implementation plan: `.planning/implementation-plans/source-first-observation-views-2026-05-11.md`."
  - "Updated MCP/CLI examples in `QUICKSTART.md`, `docs/mcp-tool-reference.md`, and `docs/json-contracts.md`."
  - "Verification passed: `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, planning governance checks, and `git diff --check`."
---

## Goal

Make OpenDog useful when AI-tool infrastructure dominates observed file activity by adding governed source-first observation affordances and clearer guidance boundaries.

## Evidence Source

The mystocks 2026-05-11 calibration showed that Claude Code Read operations close file descriptors too quickly for fd sampling, while sustained `.claude/` infrastructure reads remain visible and dominate hot stats.

This means the next improvement is not scanner attribution. It is presentation and guidance:

- let users and AI agents ask for source-focused observation slices
- keep infrastructure evidence available
- explain that `access_count=0` can mean "not observed by fd sampling", not "not read" or "safe to delete"

## Change Plan

1. Review current CLI/MCP stats and unused filtering parameters before changing interface shape. Completed in implementation plan.
2. Propose the minimal source-first view contract: default guidance wording, optional source/infrastructure filter, or both. Completed in implementation plan.
3. Add tests before implementation for bounded payload metadata, classification summaries, and source-focused rows. Completed.
4. Implement only after the proposed contract is reviewed. Completed.
5. Update `QUICKSTART.md`, MCP docs, and `CHANGELOG.md` if the public interface or guidance changes. Completed.

## Verification Evidence

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/validate_task_cards.py`
- `python3 scripts/validate_planning_governance.py`
- `git diff --check`

## Guardrails

- Preserve fixed fd-attribution baseline.
- Preserve raw/infrastructure evidence for users who need it.
- Keep large-repository payload bounds intact.
- Treat this as an observation-quality improvement, not a cleanup-safety guarantee.

## Completion Criteria

- Source-first observation path exists or a documented no-code decision explains why it was rejected.
- Guidance explicitly states transient-read blind spots.
- Regression tests cover bounded result windows and classification-aware output.
- mystocks can retest without relying on manual `.claude/` filtering.
