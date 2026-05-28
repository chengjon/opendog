---
title: "Fix agent-guidance UTF-8 boundary panic"
id: "TASK-20260509-agent-guidance-utf8-panic"
status: completed
owner: "codex"
priority: high
phase_hint: "Phase 6 guidance hardening"
ft_ids_touched:
  - FT-03.02.02
  - FT-03.07.01
  - FT-03.08.01
why_these_ft_ids:
  - "FT-03.02.02 owns agent guidance and next-step strategy output; the guidance route must not panic on valid UTF-8 project files."
  - "FT-03.07.01 owns boundaries and blind spots; failure handling must distinguish parser limits from attribution evidence quality."
  - "FT-03.08.01 owns the mock-detection path where the UTF-8 byte-boundary panic was observed."
requirement_ids:
  - STRAT-02
  - STRAT-04
  - BOUND-03
  - BOUND-04
  - MOCK-01
  - MOCK-05
  - MOCK-10
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not reopen or alter the accepted `fix-fd-attribution` scanner baseline."
  - "Do not change `/proc/<pid>/fd` attribution semantics."
  - "Do not broaden the fix into unrelated guidance ranking or mock-detection heuristic changes."
verification_plan:
  - "Reproduce the panic with `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog agent-guidance --project mystocks-fd --json` or an equivalent UTF-8 markdown fixture."
  - "Add regression coverage for non-ASCII markdown content so string slicing respects char boundaries."
  - "Run `cargo test`."
  - "Run `cargo clippy --all-targets --all-features -- -D warnings`."
  - "Run `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Regression test demonstrating non-ASCII markdown no longer panics."
  - "CLI/MCP guidance output remains schema-compatible."
  - "Verification command output recorded in the task completion note."
---

## Goal

Make `agent-guidance` resilient to UTF-8 markdown content so guidance generation fails gracefully or completes normally instead of panicking on byte-index slicing.

## Capability Scope

- `FT-03.02.02`
- `FT-03.07.01`
- `FT-03.08.01`

## Requirement Scope

This task hardens guidance and mock-detection text processing. It does not change scanner attribution, usage statistics, or the accepted fd attribution governance baseline.

## Change Plan

1. Reproduce the observed panic with a focused fixture or the isolated `mystocks-fd` validation state.
2. Identify byte-index slicing in `src/mcp/mock_detection.rs` and replace it with char-boundary-safe truncation or slicing.
3. Add regression coverage using non-ASCII markdown text.
4. Verify CLI/MCP guidance output remains contract-compatible.

## Guardrails

- Keep this task independent from archived change `openspec/changes/archive/2026-05-28-fix-fd-attribution` and the current `fd-attribution` OpenSpec contract.
- Do not modify `src/core/scanner.rs` unless a new OpenSpec change is approved.
- Preserve existing mock-detection categories and ranking behavior unless the regression fix requires a minimal parser-safety adjustment.

## Verification

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/validate_planning_governance.py`

## Completion Criteria

- The UTF-8 boundary panic has a regression test.
- `agent-guidance --json` no longer panics on non-ASCII markdown content.
- The fix is documented as separate from fd attribution closure.

## Completion Note

Implemented a char-boundary-safe preview slice in `src/mcp/mock_detection.rs` and added regression coverage with non-ASCII markdown content.

Verification evidence:

- `cargo test detect_mock_data_report_handles_non_ascii_preview_boundaries` passes after reproducing the original panic.
- `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog agent-guidance --project mystocks-fd --json` exits 0 and returns schema-shaped guidance JSON.
