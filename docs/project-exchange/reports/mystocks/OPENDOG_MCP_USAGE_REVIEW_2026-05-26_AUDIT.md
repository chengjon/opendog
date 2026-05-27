# Review: OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md

**Type**: Markdown usage review / feasibility audit
**Perspective**: OPENDOG maintainer triage
**Date**: 2026-05-26
**Reviewer**: Codex
**Shared issue**: `ODX-20260526-mcp-usage-review-hardening`

## Summary

The MyStocks review is technically useful and should be treated as real product feedback, not just project-local usage notes. The strongest items are the daemon/schema compatibility failure, verification pipeline trust, and large-database report/cleanup behavior; all three map to current OPENDOG implementation gaps. The data-risk and capability-surface feedback is also valid, but should be split into smaller P1/P2 documentation and prioritization tasks rather than treated as one broad redesign.

This audit did not rerun the 10GB MyStocks database workload. It verifies the source review against the current OPENDOG code and docs.

This file has also been integrated with `USAGE_REVIEW_AUDIT_RESPONSE.md`. The integrated position keeps the original technical findings, but narrows several remedies toward smaller, higher-yield changes: enrich existing diagnostics instead of adding a new doctor command, reuse existing path classification before designing a new taxonomy, and treat auto-generated capability matrices as useful but not urgent.

## Verified

- Source review: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md`
- Runtime MCP inventory has 26 tools and does not expose `get_decision_brief`; the decision envelope is intentionally routed through `get_guidance(detail=decision)`.
- `verify_deletion_plan` is exposed in MCP, but there is no same-name CLI command.
- Current `get_build_info` exposes binary `version`, `git_hash`, `build_time`, binary path, and `needs_rebuild`; it does not expose daemon/client/schema compatibility fields.
- Current verification execution runs shell commands through `sh -lc` and records the shell exit code and text summary; it does not flag pipeline masking or suspicious passed output.
- Current report and cleanup flows have bounded payloads in some places, but heavy database scans can still occur before any bounded output is produced.
- Current path classification for stats/unused is broad (`source`, `infrastructure`, `backup`, `project`), while data-risk uses a separate classifier that does not specifically classify `.claude` agent config or skill reference paths.

## Integrated Response Adjustments

The response document accepts the main audit findings, but refines implementation scope:

- **F-1 accepted**: schema compatibility diagnostics stay P0, with a small implementation path: enrich `SchemaMigration` messages, expose supported schema version and daemon status in `get_build_info`, and attach `daemon_restart_required` to schema-migration error payloads.
- **F-2 accepted in phases**: verification trust should preserve raw recorded status, then add suspicious-pipeline metadata first; later gate/guidance behavior can consume that metadata.
- **F-3 accepted as two slices**: report protection should start with SQL-level `LIMIT`; cleanup protection should separately add estimate-first dry-run behavior.
- **F-4 accepted with narrower scope**: data-risk should reuse `core::file_classification::classify_file_path` for infrastructure paths before introducing any larger taxonomy.
- **F-5 accepted as regression coverage**: current advisory/deletion boundary is correct and should be protected by tests, not redesigned.
- **F-6 accepted as documentation cleanup**: `get_decision_brief` should remain non-public MCP surface; docs should say decision brief is `get_guidance(detail=decision)`.
- **F-7 partially accepted**: OPENDOG can report server/daemon/config diagnostics, but `host_tools_visible` requires AI-host participation and should remain a checklist item.

The response explicitly defers or rejects these broader suggestions for now: automatic `host_tools_visible` detection, a completely new unified path taxonomy, an auto-generated capability matrix as an urgent item, and a standalone `opendog doctor mcp` command.

## Findings

### HIGH: daemon/schema mismatch feedback is valid and should be P0

The source review reports MCP failures for `get_unused_files`, `get_verification_status`, and `get_data_risk_candidates`, with `project database schema version 6 is newer than supported version 4`, while same-class CLI commands succeeded. That points to an old daemon or remote-control process serving a newer project DB.

Evidence:

- Source report: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:119`
- Source report error: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:128`
- Source recommendation: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:252`
- Current build-info payload lacks daemon/client/schema fields: [src/mcp/payloads/config_payloads.rs](/opt/claude/opendog/src/mcp/payloads/config_payloads.rs:12)
- Storage migration rejects newer DB schemas but the remote-control error is not enriched with restart advice: [src/storage/migrations.rs](/opt/claude/opendog/src/storage/migrations.rs:33)

Recommendation:

Add a small compatibility envelope to daemon/MCP errors and `get_build_info`: current binary version/build metadata, supported schema version, daemon connectivity/status, and `daemon_restart_required` when schema migration errors imply an old daemon. The response narrows the first implementation to message enrichment plus `get_build_info` diagnostics; deeper client-vs-daemon binary comparison can follow if the first slice is insufficient.

### HIGH: verification pipeline trust feedback is valid and should be P0

The review identifies commands like `npx vue-tsc --noEmit 2>&1 | tail -30`; without `pipefail`, the shell can report the exit status of `tail` rather than the failing verifier. OPENDOG currently runs user commands through `sh -lc`, then records `output.status.success()` as passed/failed and stores the summary. There is no trust layer that detects masked pipeline exits or passed output containing obvious error text.

Evidence:

- Source report command pattern: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:168`
- Source report observed suspicious pass: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:177`
- Source recommendation: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:180`
- Current shell execution: [src/core/verification.rs](/opt/claude/opendog/src/core/verification.rs:130)
- Current pass/fail recording: [src/core/verification.rs](/opt/claude/opendog/src/core/verification.rs:142)
- Current verification evidence exposes latest runs and gates, but not trust flags: [src/mcp/verification_evidence.rs](/opt/claude/opendog/src/mcp/verification_evidence.rs:69)

Recommendation:

Implement this in two phases. Phase A should add low-risk detection fields while preserving raw status: `pipeline_operators_detected`, `exit_code_masked_possible`, and `suspicious_pass_signals`. These can be stored either as new columns or encoded in existing summary-compatible output, but should not rewrite `status`. Phase B should make `get_verification_status` and guidance gates consume those fields by lowering trust from `trusted` to `caution` and recommending rerun commands without pipelines.

### HIGH: large-database report/cleanup feedback is valid, with implementation-specific root causes

The MyStocks review reports a project DB around 8.29GB plus WAL around 2.68GB and 15-second timeouts across `report window`, `report trend`, `report compare`, and all `cleanup-data --dry-run` scopes. Current OPENDOG already has bounded result windows for stats/unused and summaries in report payloads, but heavy SQL work still happens before payload bounding.

Evidence:

- Source DB size and timeout report: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:139`
- Source recommendation: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:156`
- Time-window report runs multiple `COUNT`/`COUNT(DISTINCT)` queries, then builds grouped file maps before truncating: [src/core/report/time_window.rs](/opt/claude/opendog/src/core/report/time_window.rs:24)
- Access/modification grouped queries do not apply SQL `LIMIT` before returning rows: [src/core/report/time_window.rs](/opt/claude/opendog/src/core/report/time_window.rs:110)
- Cleanup dry-run still counts full retained-evidence scopes: [src/core/retention/executor.rs](/opt/claude/opendog/src/core/retention/executor.rs:34)

Recommendation:

Split this into two tasks. R-1 should add SQL-level `LIMIT` to report grouped queries before considering broader timeout machinery; this is the smallest high-value protection for large stores. R-2 should add cleanup estimate-first behavior, especially for snapshot scopes: start with cheap counts, use a threshold, and return an `estimate_mode` such as `full` or `scope_counts_only` before expensive detailed counting.

### MEDIUM: data-risk noise feedback is valid, but should reuse a unified classification model

The source review reports MyStocks data-risk hits mostly under `.claude` config and skill reference docs. Current stats/unused classification treats `.claude`, `.cursor`, `.agents`, and `.zread` as infrastructure, but data-risk has a separate classifier with only `generated_artifact`, `test_only`, `runtime_shared`, `documentation`, and `unknown`. A `.claude/settings.json` path can fall through to `unknown`, and mock candidates outside test/generated paths default to high review priority.

Evidence:

- Source noisy candidates: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:190`
- Source recommendation: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:206`
- Broad file classifier already knows `.claude` as infrastructure: [src/core/file_classification.rs](/opt/claude/opendog/src/core/file_classification.rs:65)
- Data-risk path classifier lacks agent/tooling categories: [src/mcp/mock_detection.rs](/opt/claude/opendog/src/mcp/mock_detection.rs:117)
- Data-risk mock priority defaults non-test, non-generated candidates to high: [src/mcp/mock_detection.rs](/opt/claude/opendog/src/mcp/mock_detection.rs:398)

Recommendation:

Use the existing `core::file_classification::classify_file_path` inside data-risk first. If a path is classified as infrastructure, especially `.claude/`, `.cursor/`, `.agents/`, or similar tooling paths, data-risk should emit `path_classification = "infrastructure"` and lower default `review_priority` instead of letting it fall through to `unknown -> high`. A larger taxonomy can remain a future design topic if this minimal reuse does not reduce noise enough.

### MEDIUM: unused/deletion guidance is mostly already correct; preserve and harden it

The source review explicitly says OPENDOG correctly returned blocked cleanup/refactor gates, `destructive_change_recommended: false`, and `recommended_next_action: take_snapshot` for a dirty MyStocks worktree with weak access coverage and stale evidence. This is a positive validation of the current advisory boundary.

Evidence:

- Source stats: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:217`
- Source says current decision brief is reasonable: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:230`
- Source asks to keep emphasizing unused is not deletion permission: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:245`
- Current JSON contract already states retained-evidence cleanup does not delete source files: [docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md:386)

Recommendation:

Do not turn unused/orphan into destructive automation. Add regression tests that dirty worktree, stale verification, low activity coverage, and storage-maintenance flags keep deletion guidance blocked or human-confirmed.

### MEDIUM: CLI/MCP/documentation mismatch is partially valid

The `get_decision_brief` point should be treated as documentation ambiguity, not a missing MCP tool. Current tests intentionally reject legacy `get_decision_brief` as an exposed MCP alias, and the current MCP reference says to use `get_guidance(detail=decision)`. However, the codebase still contains control-plane `get_decision_brief`, CLI `decision-brief`, and JSON schema language about an MCP decision-brief schema, so readers can reasonably infer that a separate MCP tool may exist. The `verify_deletion_plan` asymmetry is real: it exists in MCP but there is no same-name CLI command.

Evidence:

- Source mismatch report: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:309`
- Runtime inventory exposes `get_guidance` and `verify_deletion_plan`, but no `get_decision_brief`: [src/mcp/tool_inventory.rs](/opt/claude/opendog/src/mcp/tool_inventory.rs:25)
- Tool-surface test intentionally excludes `get_decision_brief`: [src/mcp/tests/tool_surface.rs](/opt/claude/opendog/src/mcp/tests/tool_surface.rs:61)
- MCP docs route decision envelope through `get_guidance(detail=decision)`: [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md:395)

Recommendation:

Treat this as a documentation cleanup first. Clarify that MCP decision brief is `get_guidance(detail=decision)`, not a standalone `get_decision_brief` tool. Mark `verify_deletion_plan` as MCP-only unless a CLI wrapper is intentionally added. A generated capability matrix is useful long term, but the response correctly defers it as non-urgent.

### LOW: MCP host diagnostics is valid but partly outside OPENDOG's direct visibility

The source review distinguishes server startup, daemon health, and whether the current AI host actually exposes the tools. OPENDOG can diagnose server/daemon/config state, but it cannot fully know whether Codex, Claude, or another host has surfaced the tools inside a live session unless the host participates.

Evidence:

- Source diagnosis gap: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:100`
- Source suggested fields: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md:324`
- Current `get_build_info` is server-side build status, not host visibility diagnosis: [src/mcp/payloads/config_payloads.rs](/opt/claude/opendog/src/mcp/payloads/config_payloads.rs:12)

Recommendation:

Do not add a standalone doctor command in the first slice. Extend `get_build_info` or adjacent docs with `daemon_running`, `opendog_home`, binary path, and effective config guidance. Treat `host_tools_visible` as a host-side checklist item, because OPENDOG cannot prove tool visibility inside Codex/Claude without host cooperation.

## Proposed Triage

| Rank | Work Item | Source Priority | Integrated Scope |
|---|---|---|---|
| 1 | F-1 schema compatibility diagnostics | P0 | Enrich schema errors, expose supported schema and daemon status, add restart advice |
| 2 | F-4 data-risk path classification noise | P1 | Reuse `file_classification` for infrastructure paths and lower priority |
| 3 | F-2A verification trust Phase A | P0 | Record pipeline/suspicious-pass signals without changing raw status |
| 4 | F-6 documentation capability cleanup | P1 | Clarify decision brief routing and MCP-only deletion-plan verification |
| 5 | F-3-R1 report SQL limit | P1 | Add SQL-level `LIMIT` to report grouped queries |
| 6 | F-5 advisory-boundary regression tests | MEDIUM | Lock blocked cleanup/refactor behavior for dirty/stale/low-coverage cases |
| 7 | F-7 daemon status diagnostics + checklist | P2 | Add `daemon_running` style status and host-side setup checklist |
| 8 | F-3-R2 cleanup estimate-first | P1 | Add thresholded estimate-only dry-run mode for expensive scopes |
| 9 | F-2B verification trust gate integration | P0 | Make gates/guidance consume trust metadata |

## Verdict

Accept the review as actionable, with the response document's scope reductions. Do not implement it as one large feature batch. The correct next move is to create small task cards in the integrated priority order above, starting with schema compatibility diagnostics and data-risk infrastructure-path demotion.

## Implementation Follow-Up Status

Updated: 2026-05-27

All nine triaged work items have been reviewed after implementation. The accepted implementation scope is complete; the only intentionally scoped boundary is host-side MCP tool visibility under F-7, which OPENDOG cannot prove without cooperation from the external AI host.

| Rank | Work Item | Follow-Up Status | Evidence |
|---|---|---|---|
| 1 | F-1 schema compatibility diagnostics | Implemented | Commit `7156ac6` enriched schema migration errors with daemon/MCP restart advice and exposed supported schema version through build info. |
| 2 | F-4 data-risk path classification noise | Implemented | Commit `f14249e` demoted agent/config infrastructure paths in data-risk classification. |
| 3 | F-2A verification trust Phase A | Implemented | Commit `ab68fb4` added pipeline operator detection and suspicious pass signal analysis while preserving raw verification status. |
| 4 | F-6 documentation capability cleanup | Implemented | Commit `ffa6fcf` clarified decision brief routing; commits `11ea181` and `6b5a6d7` later synchronized rollup/retention docs and MCP tool counts. |
| 5 | F-3-R1 report SQL limit | Implemented | Commit `563a569` added SQL-level `LIMIT` protection to grouped report queries for large repositories. |
| 6 | F-5 advisory-boundary regression tests | Implemented | Commit `d88d7f1` added regression tests for blocked cleanup/refactor gates under stale/dirty/low-confidence evidence. |
| 7 | F-7 daemon status diagnostics + checklist | Implemented with scoped boundary | Commit `0559af6` added `daemon_running` and `opendog_home` diagnostics. Host-side `host_tools_visible` remains a checklist/documentation responsibility because OPENDOG cannot observe external MCP host UI/tool exposure directly. |
| 8 | F-3-R2 cleanup estimate-first | Implemented and extended | Commit `b429b89` added estimate-first cleanup dry-run behavior; commit `1efa8e2` extended the line into retained-evidence storage governance, retention policy, activity rollups, and vacuum-aware cleanup. |
| 9 | F-2B verification trust gate integration | Implemented | Commit `7f25843` integrated pipeline trust metadata into verification gate assessment and guidance. |

Additional validation:

- Commit `efdadb8` added the storage retention runbook and a mystocks dry-run record.
- Commits `f920fba`, `e7fd6bd`, and `010e6e0` record the mystocks 14-day dry-run, real retained-activity cleanup, rollup verification, WAL checkpoint, and persisted project-level 14-day retention policy.
- Current status: reviewed, implemented, documented, and validated against the mystocks project; no unresolved item remains in the accepted scope.
