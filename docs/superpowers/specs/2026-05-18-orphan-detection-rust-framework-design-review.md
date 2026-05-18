# Review: 2026-05-18-orphan-detection-rust-framework-design.md

**Type**: .md / proposal | **Perspective**: completeness + consistency + feasibility + architecture | **Date**: 2026-05-18 | **Reviewer**: Claude

---

## Executive Summary

This proposal defines a well-structured, language-neutral orphan detection framework for OpenDog. The core model (Subject, Evidence Signal, Scanner Health, Classification) is cleanly separated from scanner-specific semantics, and the safety-first classification (blocked > review_required > remove_candidate) is sound. Cross-referencing against the live codebase confirms that the proposed module layout aligns with existing patterns and that claimed codebase dependencies (file classification, ignore rules, verification orchestration, MCP contract versioning) all exist. The main gaps are: missing `generated` path classification mapping, an ambiguous `required_scanners` contract, and no error contract specification for the two new MCP tools.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | docs/superpowers/specs/2026-05-18-orphan-detection-rust-framework-design.md |
| File Type | .md |
| Doc Type | proposal |
| Sections | 14 |
| Referenced Files | 8 proposed, 0 existing (all new paths) |
| Referenced Symbols | 4 codebase concepts verified, 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/core/orphan_detection/` | no | proposed new module — not yet created |
| `src/mcp/orphan_handlers.rs` | no | proposed new file |
| `src/mcp/orphan_payload.rs` | no | proposed new file; existing `src/mcp/payloads.rs` serves similar role |
| `src/storage/queries/orphan_detection.rs` | no | proposed new file |
| `src/control/*` | yes | `src/control/controller_queries.rs`, `src/control/request_handler.rs`, etc. |
| `src/core/file_classification.rs` | yes | `src/core/file_classification.rs` (verified) |
| `src/core/verification.rs` | yes | `src/core/verification.rs` (verified) |
| `src/contracts.rs` | yes | `src/contracts.rs` (verified) |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `classify_file_path` | yes | `src/core/file_classification.rs:72` |
| `FilePathClassification` | yes | `src/core/file_classification.rs:2` |
| `should_ignore_path` | yes | `src/config.rs` (used in `src/core/snapshot.rs:71`) |
| `verification-command orchestration` | yes | `src/core/verification.rs` (ExecuteVerificationInput, Command execution) |
| `versioned_payload` | yes | `src/contracts.rs:53` |
| `schema_version` pattern | yes | `src/contracts.rs` (30 MCP schema version constants) |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| "reuse OpenDog ignore rules" | confirmed | `should_ignore_path` in config, used in `src/core/snapshot.rs:103` |
| "classify paths with existing file classification rules" | confirmed | `classify_file_path()` at `src/core/file_classification.rs:72` |
| "reuse the existing verification-command orchestration style" | confirmed | `ExecuteVerificationInput` and `Command` usage in `src/core/verification.rs:20-24` |
| "JSON evidence blobs" approach | confirmed | existing pattern: `scanner_summary_json`, `warnings_json`, `errors_json` in `schema.rs` |
| "exclude infrastructure, generated, backup, and project-doc paths" | partially contradicted | `FilePathClassification` enum has Source, Infrastructure, Backup, Project — no `generated` variant (see Finding 3) |

## Checklist Results

### Architecture (A1-A9)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | PASS | Clear separation: core model, scanners, aggregation, classification, MCP handlers, persistence |
| A2 | Data flow | PASS | Explicit pipeline: Candidate Collector -> Scanners -> Evidence Signals -> Aggregation -> Classification |
| A3 | Coupling | PASS | Classifier sees only normalized signals + scanner health; scanner semantics stay external |
| A4 | Interface contracts | PASS | MCP tool inputs/outputs fully specified for both `scan_orphans` and `verify_deletion_plan` |
| A5 | Scalability | PASS | Phased approach with Phase 2 addressing external command orchestration, Phase 3 language packages |
| A6 | Terminology consistency | PASS | Terms (veto, blocked, review_required, remove_candidate, signal_kind) used consistently throughout |
| A7 | Backward compatibility | PASS | Non-Goals explicitly exclude code deletion; existing MCP surface unchanged |
| A8 | Implementation surface precision | FAIL | See Finding 1: module boundaries described but no file/function-level implementation spec |
| A9 | Named entities verified | PASS | All referenced existing codebase entities verified as present |

### Completeness (C1-C5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Goal, Non-Goals, Design Principle, Core Model, MCP Surface, Test Plan, Open Questions all present |
| C2 | Edge cases | FAIL | See Finding 2: no error contract for MCP tools |
| C3 | Implicit assumptions | FAIL | See Finding 3: assumes `generated` classification exists; Finding 4: `required_scanners` undefined |
| C4 | Acceptance criteria | PASS | Test Plan enumerates specific behavioral assertions for each classification level |
| C5 | Missing roles/stakeholders | N/A | Proposal-type doc; stakeholder identification is the OpenDog development team |

### Consistency (N1-N5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | "candidate", "signal", "veto", "scanner health" used uniformly |
| N2 | Naming conventions | PASS | Proposed file/function names follow existing OpenDog conventions (`*_handlers.rs`, `queries/*.rs`) |
| N3 | Formatting | PASS | Consistent heading hierarchy, code blocks, and bullet lists |
| N4 | Cross-references | PASS | Internal section references consistent; external references verified against codebase |
| N5 | Style consistency | PASS | Uniform technical writing style throughout |

### Feasibility (F1-F5)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | Hardest part (safety classification) is well-designed with explicit safety gates before confidence promotion |
| F2 | Dependency availability | PASS | All referenced crate dependencies (rusqlite, rmcp, walkdir) already in use |
| F3 | Timeline realism | PASS | Phased approach with Phase 1 scoped to Rust-only work; no timeline estimates given (appropriate for proposal) |
| F4 | Resource constraints | N/A | Proposal does not specify team size or timeline |
| F5 | Rollback plan | PASS | Phase 1 is MCP-only; can ship/withhold independently of existing surface |

15 items PASS, 3 items FAIL, 4 items N/A.

## Findings

### Critical Issues

None.

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Repository Fit | Proposed module layout uses `src/core/orphan_detection/` as a directory, but no sub-modules under `src/core/` currently use directories — all are single files (`snapshot.rs`, `monitor.rs`, `verification.rs`). The only sub-directory pattern is `src/storage/queries/`. | Implementation ambiguity: developer may create inconsistent module layout | Checked `src/core/mod.rs`: 11 flat module declarations, zero directory-based sub-modules. `src/storage/queries/` uses directory pattern with `mod.rs` re-exports. | Either (a) specify this as `src/core/orphan.rs` (single file, consistent with core pattern) with sub-modules only if line count warrants it, or (b) explicitly note that orphan_detection is the first core sub-directory and justify why it needs multiple files upfront. |
| 2 | MCP Surface | No error response contracts specified for `scan_orphans` or `verify_deletion_plan`. Existing MCP tools follow a clear error contract pattern (`versioned_project_error_payload`, `versioned_error_payload`). | Implementer must guess error shapes; risk of inconsistency with existing 20-tool surface | Checked `src/contracts.rs:53-79`: `versioned_project_error_payload` and `versioned_error_payload` are the standard error constructors. The doc's test plan mentions "missing project returns the existing project error contract" but the MCP surface section does not enumerate error responses. | Add an "Error Responses" subsection to each MCP tool specifying: (1) project-not-found error using existing contract, (2) validation errors for malformed input, (3) scanner health failure error. |
| 3 | Candidate Collector | States "exclude infrastructure, generated, backup, and project-doc paths" but `FilePathClassification` enum has no `generated` variant. Current classifications: Source, Infrastructure, Backup, Project. | Candidate collector's exclude logic cannot implement the `generated` exclusion as specified | Checked `src/core/file_classification.rs:2-7`: enum variants are `Source`, `Infrastructure`, `Backup`, `Project`. No `Generated` variant. The `classify_file_path` function at line 72 does not detect generated files. | Either (a) add a `Generated` variant to `FilePathClassification` as a prerequisite task, or (b) change the spec to say "exclude infrastructure and backup paths" and note that generated-file detection requires a separate enhancement to the classification system. |
| 4 | MCP Surface / `scan_orphans` | `required_scanners` is listed as an optional parameter but the contract for what makes a scanner "required" vs optional is never defined. The classification logic says "all required scanner health checks are acceptable" for `remove_candidate`, but there is no specification of how `required_scanners` is determined, validated, or defaulted. | Classification correctness depends on this concept; without it, the safety gate for `remove_candidate` is undefined | Grep for `required_scanner` found zero matches outside this document. No existing OpenDog config or data model defines scanner requirements per project. | Add a subsection defining: (1) default required scanner set, (2) how `required_scanners` parameter overrides defaults, (3) validation rules (e.g., must include at least one scanner, must reference known scanner names), (4) behavior when `required_scanners` is empty vs omitted. |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | MCP Surface | `verify_deletion_plan` response includes `required_verification_commands` but the doc states OpenDog will not delete files. The distinction between verification commands (existing concept) and deletion-safety verification is unclear. | `src/core/verification.rs` already has `ExecuteVerificationInput` for test/lint/build. The spec's Phase 2 says "keep scanner command kinds separate from test/lint/build verification" but `verify_deletion_plan` response mixes the two concepts. | Clarify whether `required_verification_commands` refers to existing test/lint/build commands or to a new deletion-safety verification category. If the former, explain the mapping. If the latter, define the new category. |
| 2 | Persistence | Proposed `orphan_candidates` table has `confidence REAL NOT NULL` but the confidence formula (`base_confidence * signal_density * freshness_factor`) is described as a scoring aid, not a stable value. Stored confidence may become misleading if the formula changes. | `src/storage/schema.rs` uses `SCHEMA_VERSION: u32 = 4` for migration tracking. No versioning strategy mentioned for orphan tables. | Consider adding `classification_version` or `formula_version` column, or note that confidence is point-in-time and should be recalculated on re-scan rather than trusted from history. |
| 3 | Open Questions | The fourth open question ("per-project protected path globs") partially overlaps with existing config infrastructure. `ProjectConfig` already has `ignore_patterns`; protected paths could be modeled similarly. | `src/cli/config_commands.rs:34-36` shows `ignore_patterns`/`add_ignore_patterns`/`remove_ignore_patterns` already exist in project config. | Note in the open question that the config mechanism exists; the question is whether to add a separate `protected_paths` field or reuse `ignore_patterns` semantics. |

## Strengths

- **Safety-first classification hierarchy**: blocked > review_required > remove_candidate with explicit safety gates before confidence promotion is well-designed and honest about uncertainty
- **Clean scanner abstraction**: normalized evidence signal contract decouples Rust core from language-specific scanner implementations, allowing independent evolution
- **Codebase alignment**: proposed module layout, file naming, and contract patterns follow existing OpenDog conventions (verified: `*_handlers.rs`, `payloads.rs`, `queries/*.rs`, `contracts.rs` versioning)
- **Honest about limitations**: Non-Goals section explicitly excludes language-specific AST analysis and code deletion; "approximate attribution only" ethos matches existing `/proc` scanning approach
- **Existing pattern reuse**: correctly identifies `file_classification`, `should_ignore_path`, verification-command orchestration, and JSON-blob persistence as reusable building blocks

## Recommendations

1. **Add error response contracts** for both MCP tools, following the existing `versioned_project_error_payload` / `versioned_error_payload` pattern from `src/contracts.rs`. This is the highest-impact gap because it directly affects implementation correctness.

2. **Resolve the `generated` classification gap** before Phase 1 implementation. Either extend `FilePathClassification` with a `Generated` variant (and update `classify_file_path`), or remove `generated` from the candidate collector exclusion list and track it as a follow-up.

3. **Define the `required_scanners` contract** with defaults, overrides, and validation rules. This is load-bearing for the `remove_candidate` classification safety gate.

4. **Clarify module structure**: if `src/core/orphan_detection/` is the first directory-based sub-module under `src/core/`, document why it needs multiple files at the outset (estimated line count, number of sub-modules). Otherwise, consider starting as `src/core/orphan.rs` and splitting only when warranted.

5. **Add schema version constant** for orphan persistence tables (following `SCHEMA_VERSION: u32 = 4` pattern in `schema.rs`) to support future migrations.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | Core model and classification logic are sound; one factual gap (generated classification) |
| Completeness | 3 | Error contracts and required_scanners definition missing; otherwise thorough |
| Codebase Alignment | 5 | Proposed patterns match existing conventions precisely; all codebase references verified |
| Actionability | 4 | Clear implementation phases with specific deliverables; Phase 1 scope is well-bounded |
| Terminology Consistency | 5 | Consistent terminology throughout; matches codebase naming conventions |
| **Overall** | **4.1** | Weighted: Actionability 2x, Feasibility 2x |

## Verdict

**APPROVE_WITH_NOTES**

The proposal is well-designed and codebase-aligned. The core model (normalized evidence signals, safety-first classification, scanner health tracking) is sound and follows established OpenDog patterns. Four medium-severity gaps (module layout convention, missing error contracts, `generated` classification, undefined `required_scanners`) should be addressed before implementation begins, but none are architectural blockers. The spec is ready to move to implementation with targeted amendments.
