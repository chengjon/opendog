---
title: "Expand MCP regression coverage for reporting and guidance paths"
id: "TASK-20260510-mcp-regression-coverage-expansion"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 MCP reliability hardening"
ft_ids_touched:
  - FT-02.02.01
  - FT-03.03.01
  - FT-01.04.04
why_these_ft_ids:
  - "FT-02.02.01 owns the MCP tool surface, so regression coverage must protect the machine-facing contract."
  - "FT-03.03.01 owns verification evidence, which should be exercised by MCP execution-path tests."
  - "FT-01.04.04 owns comparative reporting, including snapshot comparison and time-window reporting surfaces that need coverage across CLI and MCP."
requirement_ids:
  - MCP-01
  - MCP-06
  - MCP-07
  - RPT-01
  - RPT-02
  - RPT-03
  - EVID-01
  - EVID-03
interface_surfaces:
  - mcp
  - cli
non_goals:
  - "Do not redesign the payload contract in this task."
  - "Do not add new feature behavior beyond test coverage and contract confirmation."
  - "Do not change scanner attribution or data-risk semantics."
verification_plan:
  - "Add regression coverage for `run_verification_command` and `compare_snapshots` with explicit run-id selection."
  - "Cover MCP success/error envelopes for report and guidance paths that already exist."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Passing focused regression tests for the targeted MCP paths."
  - "Any required doc updates to `docs/mcp-tool-reference.md` or `docs/json-contracts.md` if contract fields are clarified."
completion_notes:
  - "Expanded daemon/control integration coverage for explicit `compare_snapshots` base/head run-id selection."
  - "Added regression coverage for daemon-backed verification command execution and persistence, matching the path MCP reuses when the daemon is live."
  - "No payload contract or business behavior changes were required."
---

## Goal

Strengthen MCP confidence by adding regression coverage for the tool paths that the quantix report showed were useful but not yet fully covered by direct tests.

## Evidence Source

`docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md` highlights `run_verification_command` and explicit `compare_snapshots` parameters as uncovered but valuable paths.

## Change Plan

1. Add focused tests for explicit snapshot-run selection and verification command execution paths.
2. Add envelope assertions for successful and error responses where contract stability matters.
3. Keep the tests aligned with current CLI/MCP behavior instead of inventing new flows.

## Guardrails

- Test the existing contract; do not widen scope into new tool design.
- Keep CLI and MCP behavior aligned.
- Preserve machine-readable error contracts and versioned payload expectations.

## Completion Criteria

- Regression coverage exists for the missing high-value MCP/report paths.
- CLI and MCP remain aligned on the tested behavior.
- Verification evidence is recorded in the repo after implementation.

## Closure

This task is closed as a regression-coverage expansion. It protects existing compare-snapshot and verification-execution behavior without changing API contracts.
