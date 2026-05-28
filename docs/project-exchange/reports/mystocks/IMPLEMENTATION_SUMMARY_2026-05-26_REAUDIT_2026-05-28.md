# Re-audit: IMPLEMENTATION_SUMMARY_2026-05-26.md

**Date**: 2026-05-28
**Scope**: Re-check the implementation claims in `docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md` against the current codebase and continue with the next concrete remediation.

## Verdict

The implementation summary is materially accurate after code cross-check. All committed feature items have corresponding code, tests, or documentation evidence in the current tree.

One documentation regression was found during this re-audit: `docs/opendog-feature-introduction.md` still had one user-facing sentence that described `get_decision_brief` as an MCP entrypoint. That conflicted with the summary's F-6 claim that the decision brief is exposed through `get_guidance(detail = "decision")`. This re-audit remediated that sentence.

## Evidence Matrix

| Item | Summary claim | Re-audit result | Evidence |
|------|---------------|-----------------|----------|
| F-1 | Schema/daemon mismatch diagnostics | PASS | `src/storage/migrations.rs` contains restart advice; `src/mcp/payloads/config_payloads.rs` exposes `storage_schema_version`; commit `7156ac6` exists. |
| F-4 | Data-risk path classification noise reduction | PASS | Classification and filter terms are present in `src`; `candidate_type` and `min_review_priority` are wired through MCP params; commit `f14249e` exists. |
| F-2A | Verification pipeline trust detection Phase A | PASS | `src/core/verification.rs` exposes `command_contains_pipeline_operators`; verification evidence exposes `trust_level`, `exit_code_masked_possible`, and suspicious summary tests; commit `ab68fb4` exists. |
| F-6 | Documentation capability surface correction | FIXED IN THIS RE-AUDIT | `docs/opendog-feature-introduction.md` now says MCP users should call `get_guidance(detail = "decision")`; `opendog decision-brief` remains the CLI equivalent. |
| F-3-R1 | Report SQL `LIMIT` large-DB protection | PASS | Report/storage query paths contain `LIMIT`; limit behavior has code/test coverage; commit `563a569` exists. |
| F-5 | Advisory-boundary regression tests | PASS | Cleanup/refactor advisory gate terms and safe-for-cleanup coverage are present; commit `d88d7f1` exists. |
| F-7 | `daemon_running` and `opendog_home` diagnostics | PASS | `src/mcp/payloads/config_payloads.rs` exposes both fields; code/docs references are present; commit `0559af6` exists. |
| F-3-R2 | Cleanup estimate-first dry-run | PASS | `EstimateMode::ScopeCountsOnly`, `count_snapshot_runs`, and estimate-mode tests/surfaces are present; commit `b429b89` exists. |
| Retained evidence extension | Activity rollups, retention policy, cleanup | PASS | `activity_retention_days`, `get_activity_rollups`, activity cleanup, and storage retention runbook evidence are present; retained-evidence commits exist. |
| F-2B | Verification trust gate integration | PASS | `trust_level`, `suspicious_summary_kinds`, and gate assessment references are present in verification evidence; commit `7f25843` exists. |

## Remediation Performed

Updated `docs/opendog-feature-introduction.md`:

- Before: AI could use `get_decision_brief` as an MCP entrypoint.
- After: AI should call `get_guidance(detail = "decision")` as the MCP entrypoint, while `opendog decision-brief` remains the CLI entrypoint.

## Extended Documentation Scan

After the remediation, the current user-facing documentation set was scanned for remaining old public MCP tool names:

- `get_decision_brief`: 0 remaining matches in current user-facing docs.
- `get_agent_guidance`: 0 remaining matches in current user-facing docs.
- Remaining matches: historical architecture reviews, overdesign reviews, implementation plans, audit-response records, source-control-plane internals, and this re-audit's before/after explanation.

Those remaining matches are intentionally preserved as historical or internal-context records, not current public MCP tool guidance.

## MCP Surface Consistency Scan

The current code and user-facing docs were also checked for MCP surface count drift:

- `src/mcp/tool_inventory.rs` defines 27 public MCP tools in `MCP_TOOL_INVENTORY`.
- Current reference docs that claim a tool count (`docs/mcp-tool-reference.md`, `QUICKSTART.md`, `FUNCTION_TREE.md`, `CLAUDE.md`) still say 27 tools.
- Current reference docs that enumerate tools (`docs/mcp-tool-reference.md`, README, `QUICKSTART.md`, `FUNCTION_TREE.md`, `CLAUDE.md`) mention all 27 public tool names.
- Read-only Resources remain documented as `opendog://projects` and `opendog://project/{id}/verification`, matching the resource/template handlers.

No MCP surface documentation correction was needed in this pass.

This scan is now guarded by `scripts/validate_structural_hygiene.py`, which checks current MCP reference docs for:

- current tool-count drift,
- missing public MCP tool names,
- removed guidance tool names, and
- read-only Resource URI drift.

## Remaining Deferred Items

The implementation summary's explicitly deferred recommendations remain deferred:

| Deferred item | Re-audit position |
|---------------|-------------------|
| `host_tools_visible` automatic detection | Still outside OPENDOG's direct responsibility because host-tool visibility depends on the AI host. |
| New unified path classification system | No new evidence that it is needed immediately; existing `file_classification` reuse remains adequate. |
| Capability matrix automatic generation | Still useful long-term, but not required to close the MyStocks feedback hardening line. |
| Standalone `opendog doctor mcp` command | Current `get_build_info` diagnostics and checklist documentation remain the lighter-weight path. |

## Next Step Completed

The actionable next step from this re-audit was the F-6 documentation correction. It has been applied in this branch and should be verified with the standard repo gate before commit.
