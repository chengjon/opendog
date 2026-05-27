# Review: IMPLEMENTATION_SUMMARY_2026-05-26.md

**Type**: Markdown implementation summary / codebase verification
**Perspective**: Completeness and consistency audit
**Date**: 2026-05-26
**Reviewer**: Codex

## Resolution Update

The issues identified in this review have now been addressed in the current working tree.

- Clippy gate: fixed. `cargo clippy --all-targets --all-features -- -D warnings` now passes.
- F-6 documentation surface: fixed. `docs/opendog-feature-introduction.md` now names `get_guidance(detail = "decision")` and keeps `opendog decision-brief` as the CLI equivalent.
- F-3-R1 report SQL LIMIT: fixed. Time-window reports now fetch `limit + 1`, compute `truncated` from the extra row, then truncate output to the requested limit.
- F-2A/F-2B verification trust: fixed. Stored verification status now treats passed summaries containing suspicious error/failure text as `caution`, exposes `suspicious_pass_signals`, and includes `suspicious_summary_kinds` in gate assessment.
- Pipeline operator detection: fixed. The detector now covers no-space forms such as `cmd|tail`, `cmd&&echo`, and `cmd||true`.
- Structural hygiene: fixed. `cargo fmt --check`, `git diff --check`, and `python3 scripts/validate_planning_governance.py` now pass.

Latest verification:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all --quiet`: passed (`1738` lib tests, `31` integration tests).
- `git diff --check`: passed.
- `python3 scripts/validate_planning_governance.py`: passed (`122` requirements, `122` phase-mapped, `0` backlog, `20` completed task cards).

## Summary

The implementation summary is mostly grounded in real code: the nine listed commits are present, `cargo test --all --quiet` passes, and the claimed feature areas generally have corresponding implementation and tests. However, the summary overstates completion in a few important places. The current tree does not satisfy the claimed clippy gate, the F-6 documentation cleanup is incomplete, F-3-R1's new `truncated` flag is not reliable after SQL-level limiting, and verification trust still has residual gaps around suspicious passed summaries and operator detection coverage.

Verdict: **partially complete**. The planned hardening work is substantially implemented, but not cleanly complete until the issues below are addressed.

## Verification Performed

- `cargo test --all -- --list`: **1768 tests**, 0 benchmarks.
- `cargo test --all --quiet`: **passed**.
- `cargo clippy --all-targets --all-features -- -D warnings`: **failed** with 28 reported errors.
- Cross-checked the implementation summary against `src/`, `docs/`, recent git history, and the prior MyStocks audit/response documents.

## Findings

### HIGH: clippy gate is not clean despite summary claiming 0 warnings

The summary reports clippy warnings as `0`, but the current code fails `cargo clippy --all-targets --all-features -- -D warnings`.

Evidence:

- Claimed metric: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:175)
- Verification command failed with errors including:
  - [src/mcp/data_risk/guidance.rs](/opt/claude/opendog/src/mcp/data_risk/guidance.rs:170): unused `serde_json::json`
  - [src/core/report/usage_trend.rs](/opt/claude/opendog/src/core/report/usage_trend.rs:134): too many function arguments
  - [src/mcp/payloads/config_payloads.rs](/opt/claude/opendog/src/mcp/payloads/config_payloads.rs:13): too many function arguments
  - [src/mcp/payloads/analysis_payloads.rs](/opt/claude/opendog/src/mcp/payloads/analysis_payloads.rs:210): items after test module
  - [src/storage/schema.rs](/opt/claude/opendog/src/storage/schema.rs:256): `assert!(true)`

Impact:

The implementation cannot be considered delivery-gate clean. The summary should not claim clippy is clean until this command passes.

Recommendation:

Fix the clippy errors or revise the implementation summary to say tests pass but clippy is currently failing.

### MEDIUM: F-6 documentation cleanup is incomplete

F-6 says `docs/opendog-feature-introduction.md` was corrected to make clear that decision brief is routed through `get_guidance(detail=decision)`, not a standalone MCP tool. The document now states that correctly once, but later still says ``get_decision_brief` returns...`, which reintroduces the exact ambiguity F-6 was supposed to remove.

Evidence:

- Summary claim: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:76)
- Correct wording exists: [docs/opendog-feature-introduction.md](/opt/claude/opendog/docs/opendog-feature-introduction.md:69)
- Stale/ambiguous wording remains: [docs/opendog-feature-introduction.md](/opt/claude/opendog/docs/opendog-feature-introduction.md:86)

Impact:

Readers can still infer that `get_decision_brief` is a public MCP tool, which is the mismatch MyStocks reported.

Recommendation:

Replace the remaining `get_decision_brief` wording with `get_guidance(detail = "decision")` or explicitly describe it as an internal/control-plane route behind that MCP mode.

### MEDIUM: F-3-R1 SQL LIMIT exists, but `truncated` is not reliable

The implementation adds SQL `LIMIT ?3` to `access_counts` and `modification_counts`, and adds `TimeWindowReport.truncated`. However, `truncated` is computed after both SQL queries are already limited:

```rust
let truncated = files.len() > limit;
```

If a project has more than `limit` accessed files but no separate modification-only rows, `access_counts` returns exactly `limit`, `files.len() == limit`, and `truncated` incorrectly reports `false` even though more rows exist.

Evidence:

- Summary claim: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:88)
- SQL `LIMIT` added: [src/core/report/time_window.rs](/opt/claude/opendog/src/core/report/time_window.rs:112)
- Current `truncated` calculation: [src/core/report/time_window.rs](/opt/claude/opendog/src/core/report/time_window.rs:95)

Impact:

The large-DB protection is partly implemented, but consumers cannot trust the `truncated` flag as an indication that the result set was bounded.

Recommendation:

Fetch `limit + 1` rows per grouped query, or derive truncation from total distinct counts in the summary path. Then truncate to `limit` for output.

### MEDIUM: verification trust does not catch suspicious manually recorded passed summaries

The original feedback called out a run recorded as passed while the summary contained TypeScript errors. The current F-2A detects suspicious output only while executing a command and returns it in `ExecutedVerificationResult`; those signals are not persisted in `VerificationRun`. F-2B's `get_verification_status` trust layer re-detects only command pipeline operators from stored command strings, not suspicious text in stored summaries.

Evidence:

- F-2A summary says suspicious pass signals are detected: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:57)
- `ExecutedVerificationResult` has transient signal fields: [src/core/verification.rs](/opt/claude/opendog/src/core/verification.rs:27)
- `verification_status_layer` uses command pipeline detection, not summary text inspection: [src/mcp/verification_evidence.rs](/opt/claude/opendog/src/mcp/verification_evidence.rs:61)
- `latest_runs` trust fields are based on `command_contains_pipeline_operators(&run.command)`: [src/mcp/verification_evidence.rs](/opt/claude/opendog/src/mcp/verification_evidence.rs:76)

Impact:

A manually recorded or historical run with `status = "passed"` and a summary containing `error TS`, `Traceback`, `FAILED`, etc. can still be reported as `trust_level = "trusted"` if the command has no detected pipeline operator.

Recommendation:

Either persist suspicious pass signals when recording verification results, or make `verification_status_layer` inspect stored `run.summary` for the same suspicious patterns.

### LOW: F-2A summary says two public functions were added, but one is private

The summary says `src/core/verification.rs` added two public functions: `command_contains_pipeline_operators` and `detect_suspicious_pass_signals`. In code, only `command_contains_pipeline_operators` is public; `detect_suspicious_pass_signals` is private.

Evidence:

- Summary claim: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:63)
- Actual visibility: [src/core/verification.rs](/opt/claude/opendog/src/core/verification.rs:86)

Impact:

This is probably a documentation accuracy issue rather than a functional defect, unless downstream modules are expected to reuse the suspicious-signal detector.

Recommendation:

Change the summary to say one public helper and one private helper were added, or make `detect_suspicious_pass_signals` public if intended for MCP/status reuse.

### LOW: pipeline operator detection is narrower than the summary implies

The summary says `command_contains_pipeline_operators` detects `|`, `&&`, `||`, `2>/dev/null`, and `> /dev/null`. The implementation uses string patterns such as `" | "`, `"&& "`, and `"|| "`, so commands like `cmd|tail`, `cmd&&echo ok`, or `cmd||true` are not detected.

Evidence:

- Summary claim: [IMPLEMENTATION_SUMMARY_2026-05-26.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md:63)
- Actual patterns: [src/core/verification.rs](/opt/claude/opendog/src/core/verification.rs:86)

Impact:

The MyStocks example with spaces is covered, but the detector is not a general shell-operator detector.

Recommendation:

Expand tests to include no-space operator forms and either improve detection or narrow the summary wording.

## Implementation Checklist

| Item | Summary Status | Code Evidence | Review Status |
|---|---|---|---|
| F-1 schema diagnostics | Complete | Restart advice and `build_info.storage_schema_version` exist | Implemented |
| F-4 data-risk path classification | Complete | Infrastructure path classification and low priority exist | Implemented |
| F-2A verification trust Phase A | Complete | Detection fields exist in execution result and status views now reuse suspicious pass detection | Fixed after review |
| F-6 documentation surface | Complete | Stale `get_decision_brief` wording replaced with `get_guidance(detail = "decision")` | Fixed after review |
| F-3-R1 report SQL LIMIT | Complete | SQL fetches `limit + 1` and computes reliable `truncated` before output truncation | Fixed after review |
| F-5 advisory-boundary regression | Complete | Tests exist and full suite passes | Implemented |
| F-7 daemon diagnostics | Complete | `daemon_running` and `opendog_home` exist | Implemented |
| F-3-R2 cleanup estimate-first | Complete | `EstimateMode` and `count_snapshot_runs` exist | Implemented |
| F-2B verification trust gate | Complete | Pipeline and suspicious-summary caution gates exist | Fixed after review |

## Positive Verification

- Recent git history includes all nine listed implementation commits.
- `cargo test --all --quiet` passed.
- The test list reports 1768 tests, matching the summary.
- `schema_version`, `daemon_running`, `opendog_home`, infrastructure data-risk demotion, SQL-level limits, cleanup `estimate_mode`, and pipeline caution fields all have code evidence.

## Verdict

Original verdict: the implementation summary should have been revised from "all done and clean" to "core implementation present, tests passing, clippy and several completeness details still open."

Current verdict after fixes: the reviewed gaps are resolved in the working tree, and the repo gates listed in the resolution update pass. The remaining delivery consideration is process-level: the working tree includes a large rustfmt-only diff plus new split test files, so a commit or PR should call out formatting separately from behavior changes.
