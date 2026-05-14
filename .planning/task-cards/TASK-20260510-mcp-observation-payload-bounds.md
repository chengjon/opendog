---
title: "Bound MCP observation payloads for large repositories"
id: "TASK-20260510-mcp-observation-payload-bounds"
status: completed
owner: "codex"
priority: high
phase_hint: "Phase 6 MCP usability hardening"
ft_ids_touched:
  - FT-01.04.02
  - FT-02.02.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-01.04.02 owns unused and hotspot views; large repositories need bounded result views instead of unbounded file arrays."
  - "FT-02.02.01 owns MCP reporting; MCP clients need payload contracts that survive 50K-file repositories."
  - "FT-03.07.01 owns evidence boundaries; bounded responses must clearly state truncation, totals, and follow-up commands."
requirement_ids:
  - STAT-06
  - STAT-07
  - MCP-06
  - MCP-07
  - BOUND-01
  - BOUND-04
interface_surfaces:
  - mcp
  - cli
non_goals:
  - "Do not change scanner attribution semantics."
  - "Do not change SQLite schema unless a separate storage task requires it."
  - "Do not remove full CLI visibility for operators who intentionally need exhaustive output."
verification_plan:
  - "Reproduce the `mystocks` large-payload behavior with `get_stats` and `get_unused_files` on a 50K-file project."
  - "Add MCP parameter coverage for `limit` and, if implemented, pagination or directory/type filters."
  - "Assert default MCP responses are bounded and include total counts plus truncation metadata."
  - "Run CLI/MCP contract tests for stats and unused payloads."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Regression test showing bounded MCP stats output for large result sets."
  - "Regression test showing bounded MCP unused output for large result sets."
  - "Updated MCP docs describing default limits and how to request more detail."
---

## Goal

Make MCP `get_stats` and `get_unused_files` usable on large repositories by returning summary-first, bounded payloads by default while preserving explicit access to detailed rows.

## Evidence Source

`/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md` records that:

- `mystocks` has a 50,087-file snapshot.
- MCP `get_stats` closes the connection after release rebuild.
- MCP `get_unused_files` can produce about 5.9M characters and is too large for normal AI consumption.
- CLI equivalents succeed, which points to payload shape and transport limits rather than core stats logic.

## Change Plan

1. Define bounded MCP defaults for stats and unused-file responses.
2. Add `limit` semantics and truncation metadata to MCP payloads.
3. Keep response summaries authoritative even when file arrays are truncated.
4. Update docs and tests so AI clients know when to switch to CLI or request narrower views.

## Guardrails

- Preserve backward-compatible top-level fields where possible.
- Do not silently hide truncation; expose totals and returned counts.
- Keep this separate from daemon socket response integrity work.

## Completion Criteria

- Large-repo MCP stats and unused calls no longer close the connection under default parameters.
- Returned payloads include enough metadata for AI clients to understand bounded results.
- CLI remains available for exhaustive operator inspection.

## Completion Note

MCP stats and unused payloads now default to 50 returned file rows and expose `result_window` metadata with `total_count`, `returned_count`, `limit`, and `truncated`. The MCP tool schemas accept an optional `limit` parameter for callers that need a different bound.

Verification evidence:

- `cargo test payload_contracts::analysis_payloads` passes with default-bound and explicit-limit regression coverage.
- `docs/mcp-tool-reference.md` documents `limit` and `result_window` for `get_stats` and `get_unused_files`.
- `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py` pass.
- `cargo build --release` completed so MCP hosts pointing at `/opt/claude/opendog/target/release/opendog` can use the bounded payload implementation.
