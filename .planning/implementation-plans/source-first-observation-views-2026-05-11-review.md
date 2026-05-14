# Review: source-first-observation-views-2026-05-11.md

**Type**: .md / plan (implementation) | **Perspective**: auto (completeness + feasibility + architecture + consistency) | **Date**: 2026-05-11 | **Reviewer**: Claude

---

## Executive Summary

This implementation plan is well-structured with precise file/symbol references that match the live codebase, a sound presentation-layer filter architecture that avoids touching storage or scanner code, and clear backward-compatible defaults. Two medium-severity gaps need attention before implementation: the guidance contract promises filter-aware wording in `workspace_observation` but no task step covers it, and the existing boundary text in stats/unused guidance modules is partially present already, so Task 5 should clarify that it replaces/expands existing strings rather than adding new ones.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | `.planning/implementation-plans/source-first-observation-views-2026-05-11.md` |
| File Type | .md |
| Doc Type | plan (implementation) |
| Sections | 9 (Goal, Architecture, Evidence Basis, Interface Contract, File Structure, Tasks 1-6, Final Verification, Retest Handoff) |
| Referenced Files | 15 found / 0 missing |
| Referenced Symbols | 17 found / 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/core/file_classification.rs` | yes | verified |
| `src/mcp/params.rs` | yes | verified |
| `src/mcp/analysis_handlers.rs` | yes | verified |
| `src/mcp/payloads/analysis_payloads.rs` | yes | verified |
| `src/mcp/project_guidance/stats_unused/stats.rs` | yes | verified |
| `src/mcp/project_guidance/stats_unused/unused.rs` | yes | verified |
| `src/cli/mod.rs` | yes | verified |
| `src/cli/project_commands.rs` | yes | verified |
| `src/cli/output/project_output.rs` | yes | verified |
| `src/mcp/tests/payload_contracts/analysis_payloads.rs` | yes | verified |
| `src/mcp/tests/tool_surface.rs` | yes | verified |
| `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs` | yes | verified |
| `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md` | yes | verified |
| `docs/project-exchange/issues/INDEX.md` | yes | verified |
| `.planning/task-cards/TASK-20260511-source-first-observation-views.md` | yes | verified |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `FilePathClassification` | yes | `src/core/file_classification.rs:2` |
| `classify_file_path` | yes | `src/core/file_classification.rs:27` |
| `ObservationRowsParams` | yes | `src/mcp/params.rs:21` |
| `handle_get_stats` | yes | `src/mcp/analysis_handlers.rs:17` |
| `handle_get_unused_files` | yes | `src/mcp/analysis_handlers.rs:62` |
| `stats_payload_with_limit` | yes | `src/mcp/payloads/analysis_payloads.rs:33` |
| `unused_files_payload_with_limit` | yes | `src/mcp/payloads/analysis_payloads.rs:100` |
| `observation_result_window` | yes | `src/mcp/payloads/analysis_payloads.rs:147` |
| `classification_summary` | yes | `src/mcp/payloads/analysis_payloads.rs:156` |
| `versioned_project_payload` | yes | `src/contracts.rs:63` |
| `StatsEntry` | yes | `src/storage/queries/stats.rs:18` |
| `cmd_stats` | yes | `src/cli/project_commands.rs:162` |
| `cmd_unused` | yes | `src/cli/project_commands.rs:180` |
| `print_stats` | yes | `src/cli/output/project_output.rs:23` |
| `print_unused` | yes | `src/cli/output/project_output.rs:58` |
| `DaemonClient::get_stats` | yes | `src/control/client/report_ops.rs:9` |
| `DaemonClient::get_unused_files` | yes | `src/control/client/report_ops.rs:22` |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| `ObservationRowsParams` has `id` and `limit` only | confirmed | `src/mcp/params.rs:21-26`: struct has exactly those two fields |
| Handlers pass `(id, limit)` to payload builders | confirmed | `src/mcp/analysis_handlers.rs:29,50,74,93`: calls match signature |
| `FilePathClassification` has four variants | confirmed | `src/core/file_classification.rs:2-7`: Source, Infrastructure, Backup, Project |
| `observation_result_window` takes 3 params | confirmed | `src/mcp/payloads/analysis_payloads.rs:147`: `(total_count, returned_count, limit)` |
| `versioned_project_payload` accepts iterator of pairs | confirmed | `src/contracts.rs:63`: `IntoIterator<Item = (&'static str, Value)>` |
| Existing boundary text "very brief file accesses" exists | confirmed | `src/mcp/project_guidance/stats_unused/stats.rs:206` |
| Existing boundary text "not proof that a file is safe to delete" exists | confirmed | `src/mcp/project_guidance/stats_unused/unused.rs:150` |
| CLI `Stats`/`Unused` variants have only `id` field | confirmed | `src/cli/mod.rs:96-106` |
| `cmd_stats`/`cmd_unused` take `(pm, id)` | confirmed | `src/cli/project_commands.rs:162,180` |
| `validate_task_cards.py` and `validate_planning_governance.py` exist | confirmed | Both found in `scripts/` |
| Daemon client has stats/unused methods | confirmed | `src/control/client/report_ops.rs:9,22` |

## Checklist Results

### Architecture

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | PASS | Clean separation: filter enum in core, params in mcp, handlers bridge, payload builders filter and format, output prints. No cross-concern leakage. |
| A2 | Data flow | PASS | Filter flows top-down: MCP request -> params -> handler parses -> payload builder filters -> output. CLI mirrors the same path. |
| A3 | Coupling | PASS | New `FilePathClassificationFilter` is a separate enum from `FilePathClassification`. `matches()` method bridges them without inheritance. |
| A4 | Interface contracts | PASS | Request/response contracts explicit in "Proposed Interface Contract". Default `all` preserves current behavior. New `result_window.path_classification` and `filtered_unused_count` specified. |
| A5 | Scalability | PASS (noted) | Filter applied at presentation layer after full dataset retrieval. Acceptable per approval boundary (no storage changes). For 50k-file projects, daemon transfers full set; filter trims client-side. |
| A6 | Terminology consistency | PASS | `path_classification` used uniformly across MCP params, CLI args, result-window fields, and docs. |
| A7 | Backward compatibility | PASS | Default `all` = current behavior. `classification_summary` stays full-set. `unused_count` stays unfiltered. New fields are additive. |
| A8 | Implementation surface precision | PASS | Every task names exact files, current signatures, and proposed changes with code snippets. |
| A9 | Named entities verified | PASS | All 15 files and 17 symbols confirmed in live codebase. |

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Goal, Architecture, Evidence Basis, Interface Contract, File Structure, 6 tasks, Final Verification, Retest Handoff. |
| C2 | Edge cases | FAIL | No specification for empty-filter-result behavior (zero rows match the selected classification). Retest handoff mentions "empty bounded set with clear metadata" but no contract defines what that looks like. |
| C3 | Implicit assumptions | FAIL (minor) | Task 3 Step 5 builds a `Vec<(&str, Value)>` to conditionally add `filtered_unused_count`. Current code uses array literal tuples passed directly to `versioned_project_payload`. The plan does not explicitly note this structural change from array literal to Vec builder. |
| C4 | Acceptance criteria | PASS | Each task has compile/test expectations. Retest handoff gives concrete verification commands with expected outcomes. |
| C5 | Missing roles/stakeholders | PASS | Evidence basis cites mystocks calibration. Retest handoff identifies mystocks as tester. |

### Consistency

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | `path_classification` consistent throughout. "source-first" used in title and goal only. |
| N2 | Naming conventions | PASS | `FilePathClassificationFilter` follows existing `FilePathClassification` naming. Method names `parse`/`as_str`/`matches` follow precedent. |
| N3 | Formatting | PASS | Uniform heading hierarchy, code blocks, task/step structure. |
| N4 | Cross-references | FAIL | "Proposed Interface Contract" states `guidance.layers.workspace_observation should state the selected filter when one is active` but Task 5 covers only blind spot wording, not filter-aware guidance text. No task step implements this contract requirement. |
| N5 | Style consistency | PASS | Uniform technical writing throughout. |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | Presentation-layer filter is lowest-risk approach. No storage, schema, or scanner changes. |
| F2 | Dependency availability | PASS | All referenced crates already in use. No new dependencies. |
| F3 | Timeline realism | PASS | 6 well-scoped tasks with compile checkpoints. No external blockers. |
| F4 | Resource constraints | N/A | Single-developer context assumed. |
| F5 | Rollback plan | PASS | Default `all` makes feature invisible if code is reverted. No schema migration needed. |

## Findings

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Task 5 vs Interface Contract | `workspace_observation` filter-awareness contract is not implemented by any task step. The "Proposed Interface Contract" section states guidance should state the active filter, but Task 5 only covers transient-read blind spots. | Guidance payloads will not fulfill the stated contract when a non-default filter is active. | **Codebase**: `src/mcp/project_guidance/stats_unused/stats.rs:138` and `unused.rs:86` build `workspace_observation` layer without any filter field. **Document**: Interface Contract line 70 says "guidance.layers.workspace_observation should state the selected filter when one is active"; Task 5 Steps 1-4 make no mention of adding filter awareness to guidance. | Add a step to Task 5 (or a new Task 5.5) that passes `path_filter` into `stats_guidance` and `unused_guidance` signatures, and injects a `path_classification_filter` field into the `workspace_observation` layer when non-default. Alternatively, move filter awareness into Task 3 where the payload is built, injecting it alongside the guidance call. |
| 2 | Task 5 Steps 1-2 | Plan says "ensure blind spots include" wording that partially already exists. Current `stats.rs:206` has `"Sampling-based monitoring may miss very brief file accesses."` and `unused.rs:150` has `"Lack of observed access is not proof that a file is safe to delete."`. The plan proposes expanded replacements but doesn't state this is a replacement. | Implementer may add duplicate boundary strings instead of replacing existing ones, or may not realize the existing strings need updating. | **Codebase**: `stats.rs:206` contains `"very brief file accesses"` (note "accesses" not "reads"). `unused.rs:150` contains `"not proof that a file is safe to delete"`. **Document**: Task 5 Step 1 proposes `"very brief file reads, including MCP host or AI assistant reads that open and close source files quickly."` (different from existing "accesses" wording). Step 2 proposes `"access_count=0 means OPENDOG did not observe an open descriptor; it is not proof that the file was never read or is safe to delete."` (expansion of existing text). | Clarify that Steps 1-2 replace existing boundary strings at `stats.rs:206` and `unused.rs:150`, not append new ones. Update step text to: "Replace the existing boundary string at line 206 with:" or "Expand the existing boundary text to include:". |
| 3 | Task 3 Step 5 | Conditional `filtered_unused_count` field requires restructuring the payload from an array literal to a `Vec` builder. This is a mechanical change but not shown in the code snippets. | Implementer must infer the Vec-builder pattern. Current code passes `[(...) -> impl IntoIterator]` directly to `versioned_project_payload`; the conditional field requires collecting into a `Vec<(&str, Value)>` first. | **Codebase**: `src/mcp/payloads/analysis_payloads.rs:120-136` uses an array literal `[(...) -> impl IntoIterator]` passed directly to `versioned_project_payload`. **Document**: Task 3 Step 5 says "building a small `Vec<(&str, Value)>` if needed" but doesn't show the full restructured code. | Add a code snippet showing the Vec-builder pattern for `unused_files_payload_with_limit`. Example: `let mut fields: Vec<(&str, Value)> = vec![("unused_count", json!(unused.len())), ...]; if path_filter != All { fields.push(("filtered_unused_count", json!(filtered_count))); } versioned_project_payload(V1, id, fields)` |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | Task 3 Step 6 | Test dataset only asserts `source` filter behavior. Adding assertions for `infrastructure` or `project` filter would improve coverage. | **Document**: Task 3 Step 6 assertions only test `path_classification=source`. No assertions for `infrastructure`, `backup`, or `project` filters. **Codebase**: `src/mcp/tests/payload_contracts/analysis_payloads.rs` has infrastructure/source/backup entries in existing tests (lines 220-228) demonstrating the pattern for multi-classification assertions. | Add at least one assertion for `path_classification=infrastructure` returning only `.claude/settings.json`, and verify `result_window.total_count` matches the filtered subset. |
| 2 | Task 2 Step 3 | Plan says "Update calls to `stats_payload_with_limit` and `unused_files_payload_with_limit` to include `path_filter`" but handlers have two code paths each (daemon-backed and direct). Both paths need updating. | **Codebase**: `src/mcp/analysis_handlers.rs:29` and `:50` both call `stats_payload_with_limit`; `:74` and `:93` both call `unused_files_payload_with_limit`. **Document**: Task 2 Step 3 doesn't explicitly call out the dual-path update. | Add a note that all four call sites in `analysis_handlers.rs` must be updated (lines 29, 50, 74, 93). |
| 3 | Task 5 Step 3 | Test assertions use substrings `very brief file reads` and `not proof` / `safe to delete`. The existing `unused.rs:150` already contains `not proof` and `safe to delete`, so the unused-side assertions would pass even without changes. | **Codebase**: `unused.rs:150`: `"Lack of observed access is not proof that a file is safe to delete."` contains both `not proof` and `safe to delete`. **Document**: Task 5 Step 3 proposes asserting these substrings. | Either: (a) accept that existing text already satisfies the unused assertion, or (b) use more specific substrings that distinguish the expanded wording from the current text. |

## Strengths

- **Evidence-grounded**: The plan opens with calibration data from a real project, explains why the fix is view/filter work and not scanner work, and cites authoritative local evidence files.
- **Precise implementation surface**: Every file path, function signature, and code change is specified with enough detail that an implementer can work task-by-task without exploratory reads.
- **Backward-compatible defaults**: Default `all` filter, preserved `classification_summary` from full set, and unchanged `unused_count` mean zero risk of breaking existing consumers.
- **Sound architecture**: Presentation-layer filtering avoids the single biggest risk (changing storage or scanner code) while delivering the value.
- **Checkpoint structure**: Each task ends with a compile or test checkpoint, making it easy to catch regressions early.
- **Clear approval boundary**: Explicitly lists what the plan does NOT change, preventing scope creep.

## Detailed Recommendations

1. **Add filter-awareness to guidance (Medium #1)**: The interface contract promises `workspace_observation` will state the active filter. Add a `path_filter: Option<FilePathClassificationFilter>` parameter to `stats_guidance` and `unused_guidance`. When non-default, inject a `path_classification_filter` field into the `workspace_observation` layer. This can be a sub-step of Task 3 or Task 5.

2. **Clarify Task 5 as replacement (Medium #2)**: Change Step 1 and Step 2 wording from "ensure blind spots include" to "replace the existing boundary string with". This prevents duplicate entries and makes the intent unambiguous.

3. **Show Vec-builder pattern (Medium #3)**: Task 3 Step 5 should include a complete code snippet for `unused_files_payload_with_limit` showing the transition from array literal to `Vec<(&str, Value)>` builder with conditional `filtered_unused_count`.

4. **Expand test assertions (Low #1)**: Add one more test case in Task 3 Step 6 asserting `path_classification=infrastructure` returns only infrastructure-classified rows and has correct `total_count`.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 5 | All 17 symbols and 15 files verified present. Function signatures match. Architecture is sound. |
| Completeness | 3 | Missing guidance filter-awareness task step. Edge case (empty filtered result) not specified. |
| Codebase Alignment | 5 | Every file path, function name, and type matches the live codebase. |
| Actionability | 4 | Tasks are well-scoped with checkpoints. Three minor ambiguities need clarification before implementation. |
| Terminology Consistency | 5 | `path_classification` used uniformly. Enum variant names match existing classification. |
| **Overall** | **4.4** | |

## Verdict

APPROVE_WITH_NOTES

The plan is technically sound and precisely grounded in the codebase. The three medium findings are clarifications, not architectural flaws: (1) the guidance contract promises filter awareness that no task step delivers, (2) Task 5 should state it replaces existing boundary text rather than appending, and (3) the conditional Vec-builder pattern should be shown explicitly. Address these before implementation and the plan is ready to execute.
