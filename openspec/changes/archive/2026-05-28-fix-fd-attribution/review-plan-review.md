# Review: review-plan.md

**Type**: md / plan | **Perspective**: auto (completeness + consistency + feasibility) | **Date**: 2026-05-09 | **Updated**: 2026-05-28 | **Reviewer**: Claude

---

## Executive Summary

This review originally found that `review-plan.md` was technically accurate but read like a forward-looking plan after implementation was already complete. The current `review-plan.md` now addresses those review notes by documenting current state, downstream consumer checks, evidence-bound acceptance criteria, OpenSpec success output, and the post-implementation approval boundary.

The Evidence Verification, Checklist Results, Findings, and Scoring sections below preserve the original 2026-05-09 review snapshot. Use the Post-Remediation Status section for the current disposition.

## Post-Remediation Status [2026-05-28]

| Original review item | Current status | Current evidence |
|----------------------|----------------|------------------|
| Plan read as future work although implementation was complete | Resolved | `review-plan.md` now begins with `Current State` and `Governance Outcome` sections. |
| Missing downstream-consumer regression verification | Resolved | `review-plan.md` Proposed Work step 7 covers `opendog stats`, `opendog unused`, `opendog report window`, and `get_guidance` / `opendog agent-guidance` contract compatibility. |
| `openspec validate` acceptance criterion lacked expected output | Resolved | Acceptance criteria now state that `openspec validate fix-fd-attribution` must exit 0 and report `Change 'fix-fd-attribution' is valid`. |
| Procfs fd-stability assumption was not scoped | Resolved | Scan-cycle deduplication now explicitly scopes `(pid, fd)` identity to one scan cycle and does not claim cross-cycle fd stability. |
| Approval gate was ambiguous after implementation | Resolved | The Approval Gate now states that project-owner approval is required before any further implementation or behavioral edits. |

## Document Metadata

| Field | Value |
|-------|-------|
| Source | openspec/changes/fix-fd-attribution/review-plan.md |
| File Type | md |
| Doc Type | plan |
| Sections | 8 (Goal, Scope, Proposed Work, Acceptance Criteria, Risks, Risk Controls, Out Of Scope, Approval Gate) |
| Referenced Files | 4 found / 0 missing |
| Referenced Symbols | 5 found / 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `openspec/changes/fix-fd-attribution/` | yes | directory with 7 artifacts (.openspec.yaml, proposal.md, design.md, spec.md, tasks.md, review-plan.md) |
| `src/core/scanner.rs` | yes | `src/core/scanner.rs` (201 lines, uncommitted diff: +75/-18) |
| `src/core/monitor.rs` | yes | `src/core/monitor.rs` (imports `FileSighting`, writes `file_sightings`/`file_stats` tables) |
| `FIELD_NOTES.md` | yes | project root, 302 lines, includes completed validation results at lines 272-298 |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `FileSighting` | yes | `src/core/scanner.rs:7` (struct definition) |
| `file_sightings` | yes | SQLite table at `src/storage/schema.rs:46` |
| `file_stats` | yes | SQLite table at `src/storage/schema.rs:36` |
| `access_count` | yes | Column in `file_stats`, used in 100+ locations across stats, reports, export |
| `mark_fd_seen` | yes | `src/core/scanner.rs:133` (extracted helper) |
| `resolve_snapshot_relative_file_path` | yes | `src/core/scanner.rs:137` (extracted helper) |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| Directory fd produces identical `access_count` across files | confirmed | FIELD_NOTES.md OBS-003: all 31 `.claude/` files showed exactly 446 accesses and 1,349,000ms duration |
| Tasks already implemented | confirmed | tasks.md: all 11 items checked `[x]`; scanner.rs diff shows +75/-18 with directory exclusion, fd dedup, and tests |
| `mystocks` large-repo validation completed | confirmed | FIELD_NOTES.md lines 283-293: `.py` got 4 accesses/9000ms, `.vue` got 1 access/0ms -- independent counts |
| `openspec` CLI available | confirmed | `/root/.nvm/versions/node/v24.7.0/bin/openspec` |
| `/opt/claude/mystocks_spec` exists for regression | confirmed | directory exists on disk |
| All 240 tests pass | confirmed | `cargo test`: 240 passed, 4 suites, 3.20s |

## Checklist Results

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Goal, Scope, Proposed Work, Acceptance Criteria, Risks, Risk Controls, Out Of Scope, Approval Gate all present |
| C2 | Edge cases | FAIL | Symlink-to-directory case not discussed. Code handles it via `canonicalize` + `metadata.is_file()`, but the plan does not mention this edge case or how the fix handles it |
| C3 | Implicit assumptions | FAIL | Plan assumes procfs exposes stable fd numbers but design.md raises this as an open question (line 56). Plan does not address the assumption or resolve the open question |
| C4 | Acceptance criteria | PASS | All 6 criteria are objectively testable and specific |
| C5 | Missing roles/stakeholders | FAIL | Approval Gate (line 103) says "reviewed and explicitly approved" but does not name who reviews or who approves |

### Consistency

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | fd, FileSighting, access_count, file_sightings, file_stats used consistently throughout and match codebase |
| N2 | Naming conventions | PASS | Follows project Rust snake_case conventions; file paths match actual locations |
| N3 | Formatting | PASS | Consistent numbered lists, dash lists, header hierarchy |
| N4 | Cross-references | PASS | All referenced files, artifacts, and code paths exist and resolve correctly |
| N5 | Style consistency | FAIL | Plan is written entirely in future/imperative tense but describes work that is already complete; tense is internally consistent but inconsistent with actual project state |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | Directory vs file classification implemented via `metadata.is_file()`; tested and validated |
| F2 | Dependency availability | PASS | `procfs` 0.18 provides `FDTarget` and fd number; `tempfile` for test fixtures |
| F3 | Timeline realism | PASS | All work already completed; 240 tests pass |
| F4 | Resource constraints | PASS | Single-file change (scanner.rs) with targeted downstream verification |
| F5 | Rollback plan | FAIL | design.md mentions "revert the scanner change" but review-plan does not include rollback steps in Proposed Work or Acceptance Criteria |

## Findings

### Critical Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Proposed Work (lines 26-67) | Plan describes all work in future tense when implementation is already complete | Reviewer may treat this as a forward-looking plan rather than a post-implementation review document, leading to duplicate work or confusion about what remains | tasks.md shows all 11 items checked `[x]`; scanner.rs diff shows completed implementation; FIELD_NOTES.md records completed large-repo validation; `cargo test` passes 240/240 | Add a "Current State" section at the top noting that all tasks are implemented and this plan serves as a review/audit checklist rather than a forward-looking work plan |

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Proposed Work step 6 (lines 52-63) | No downstream-consumer regression verification | Stats, unused-file detection, reports, and guidance all consume `file_sightings`/`file_stats`. The plan does not specify how to verify these consumers still produce correct results after the scanner change | Document lists "Expected downstream consumers" at line 21 but Proposed Work only covers scanner-level tests and a single large-repo validation. No steps verify `get_stats`, `get_unused_files`, time-window reports, or guidance outputs remain consistent | Add step: "Verify downstream consumer outputs (stats, unused, reports, guidance) produce consistent results with pre-fix data" |
| 2 | Acceptance Criteria line 77 | `openspec validate fix-fd-attribution` listed without expected output | Reviewer cannot determine pass/fail without knowing what successful validation looks like | openspec CLI exists at `/root/.nvm/versions/node/v24.7.0/bin/openspec`, but the plan does not specify whether validation should exit 0, produce specific output, or have other success indicators | Add expected exit code and output summary for the openspec validate command |
| 3 | Acceptance Criteria (all) | Criteria do not distinguish between "pass in theory" and "pass with evidence" | Acceptance criteria say what should be true but not what evidence proves it | FIELD_NOTES.md contains validation evidence but acceptance criteria do not reference it as the evidence source. Document links at lines 66-67 mention updating FIELD_NOTES but acceptance criteria do not require reading it | Add evidence references to each criterion (e.g., "Directory fds do not create per-file sightings -- verified by scanner unit test `resolve_snapshot_relative_file_path_ignores_directory_targets`") |
| 4 | Implicit assumption (line 39) | Plan states dedup key is `(pid, fd)` without addressing procfs fd stability | design.md line 56 raises "Does procfs expose enough fd identity to dedupe exactly by fd number in all supported environments?" as an open question. The review plan does not resolve or acknowledge this uncertainty | design.md Open Questions section raises the concern; scanner.rs implementation uses `info.fd` from procfs directly; plan proceeds without addressing whether fd numbers are stable across scan cycles or environments | Either resolve the open question (fd numbers are stable within a single scan cycle per `/proc` semantics) or document the assumption explicitly with its scope limitation |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | Acceptance Criteria line 79 | `cargo clippy --all-targets --all-features -- -D warnings` may fail on unrelated code | clippy with `-D warnings` treats all warnings as errors across the entire project, not just scanner.rs changes. A pre-existing warning in an unrelated module would block acceptance | Consider scoping to `cargo clippy -p opendog -- -D warnings` or noting that clippy must pass on the changed files specifically |
| 2 | Approval Gate line 103 | No named approver or approval process | "Do not proceed to implementation" but implementation is already done. The gate reads as a pre-implementation control applied post-implementation | Update to reflect actual approval workflow: who reviews, what constitutes approval, and whether post-implementation retrospective approval is acceptable |
| 3 | Scope line 17 | "Primary implementation area: src/core/scanner.rs" but diff shows only scanner.rs changed | Accurate but could note that no other files required modification, which validates the design decision to keep changes at the scanner boundary | Consider adding a "Blast Radius Verified" note confirming no downstream source changes were needed |

## Strengths

- **Accurate codebase references**: Every file path, function name, and data model reference resolves correctly against the live code. The plan clearly maps implementation area to specific source files.
- **Precise acceptance criteria**: Each criterion is objectively testable (directory fds do not produce sightings, regular fds still count, dedup works, independent counts observable, downstream contracts unchanged, verification commands pass).
- **Clear scope boundaries**: Out Of Scope section (lines 95-100) is explicit and prevents scope creep into default-ignore patterns, new stats views, or payload pagination.
- **Risk controls matched to risks**: Each risk in Risks (lines 83-87) has a corresponding control in Risk Controls (lines 90-93), and the controls align with the implementation (filter by metadata, keep non-path targets ignored, keep downstream contracts).
- **Evidence-grounded problem statement**: The Goal section accurately describes the bug mechanism and ties it to downstream impact on hotspots, unused candidates, reports, and guidance.

## Detailed Recommendations

1. **Add a "Current State" preamble.** Before the Goal section, add a brief status note: "All tasks described below are implemented. This plan serves as a review checklist for the completed `fix-fd-attribution` change. See `tasks.md` for per-task completion status and `FIELD_NOTES.md` for validation evidence." This prevents any reader from treating this as forward-looking work.

2. **Bind each acceptance criterion to evidence.** For example:
   - "Directory fds do not create per-file sightings" -> verified by `resolve_snapshot_relative_file_path_ignores_directory_targets` test in scanner.rs:185
   - "Regular file fds still create per-file sightings" -> verified by the same test's file-path assertion at scanner.rs:194
   - "One (pid, fd) contributes at most one sighting per scan cycle" -> verified by `mark_fd_seen_deduplicates_per_pid_and_fd` test at scanner.rs:175
   - "mystocks validation shows independent counts" -> verified by FIELD_NOTES.md lines 289-293

3. **Resolve the procfs fd-stability open question.** The design document raises whether fd numbers are stable enough for deduplication. The answer is yes within a single `/proc` scan cycle (fd numbers are assigned by the kernel and stable for the lifetime of the open file description). Document this resolution to close the open question.

4. **Add downstream consumer verification step.** After the scanner-level tests, add a step that runs `opendog stats`, `opendog unused`, and `opendog report window` against the test project and verifies the outputs are well-formed and consistent with pre-fix expectations.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 5 | All codebase references resolve; bug description matches observed behavior; fix approach matches actual implementation |
| Completeness | 3 | Missing downstream consumer verification steps; unresolved open question from design; symlink edge case not discussed |
| Codebase Alignment | 5 | Every file, function, table, and column reference matches the live codebase exactly |
| Actionability | 3 | Acceptance criteria lack evidence bindings; `openspec validate` has no expected output; approval gate has no named approver |
| Terminology Consistency | 5 | All terms used consistently and match codebase naming |
| **Overall** | **4.2** | |

## Verdict

**APPROVE_REMEDIATED**

The original review notes are retained for traceability, and the current `review-plan.md` now remediates the non-blocking documentation issues that led to `APPROVE_WITH_NOTES`.
