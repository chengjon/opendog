# Review: 2026-05-18-technical-debt-review.md

**Type**: .md / proposal (audit/review) | **Perspective**: completeness, consistency, feasibility | **Date**: 2026-05-18 | **Reviewer**: Claude

---

## Executive Summary

This technical debt review is well-structured and its six findings are accurately grounded in the live codebase. Every referenced file and function exists, and the behavioral claims (unused schema version constant, panic paths, duplicate schemars, orphan module size) are confirmed. Three numeric claims in the Baseline static scan section diverge from the current HEAD counts, likely because orphan-detection MCP code was merged after the scan was captured. The document would benefit from measurable acceptance criteria for its recommendations and a brief rollback note for the module-split and migration-runner changes.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | docs/superpowers/reviews/2026-05-18-technical-debt-review.md |
| File Type | .md |
| Doc Type | proposal (audit/review) |
| Sections | 4 (Baseline, Findings, What is healthy, Recommended order) |
| Referenced Files | 12 found / 0 missing |
| Referenced Symbols | 6 found / 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| src/storage/schema.rs | yes | src/storage/schema.rs |
| src/storage/database.rs | yes | src/storage/database.rs |
| src/mcp/server_core.rs | yes | src/mcp/server_core.rs |
| src/control/fallback.rs | yes | src/control/fallback.rs |
| src/core/verification.rs | yes | src/core/verification.rs |
| src/mcp/mod.rs | yes | src/mcp/mod.rs |
| src/contracts.rs | yes | src/contracts.rs |
| src/core/orphan.rs | yes | src/core/orphan.rs |
| Cargo.toml | yes | Cargo.toml |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| SCHEMA_VERSION | yes | src/storage/schema.rs:89 |
| PRAGMA user_version (any usage) | no | grep returned 0 matches in src/ — confirms finding #1 |
| expect in server_core.rs | yes | src/mcp/server_core.rs:13,18,23,28,32 |
| unwrap on mutex in server_core.rs | yes | src/mcp/server_core.rs:44,58 |
| controller.lock().unwrap() in fallback.rs | yes | src/control/fallback.rs:110,117,122,129,134,139,146,156 |
| expect in verification.rs | yes | src/core/verification.rs:109 |
| schemars = "0.8" in Cargo.toml | yes | Cargo.toml:20 |
| schemars v0.8.22 + v1.2.1 duplicates | yes | cargo tree -d confirms both versions |
| rmcp::schemars imports | yes | src/core/orphan.rs:4, src/mcp/params.rs:1 |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| SCHEMA_VERSION declared but not enforced | confirmed | Only reference is its declaration at schema.rs:89; no migration runner, no PRAGMA user_version in src/ |
| panic = "abort" in release profile | confirmed | Cargo.toml:30 |
| schemars = "0.8" direct dependency | confirmed | Cargo.toml:20; cargo tree -d shows both v0.8.22 and v1.2.1 |
| src/mcp is the largest LOC bucket | confirmed | 18,293 LOC in src/mcp vs 4,028 in src/core |
| src/core/orphan.rs is 980 non-empty LOC | confirmed | grep -c -v -E '^\s*$' returns 980 |
| 15 public data types in orphan.rs | confirmed | grep '^pub (struct|enum|type)' returns 15 |
| 27 tool-facing methods in mcp/mod.rs | confirmed | grep for method declarations returns 27 |
| 45 versioned contract constants in contracts.rs | confirmed | 48 pub const total minus 3 build metadata (CARGO_PKG_VERSION, GIT_HASH, BUILD_TIME) = 45 |
| No TODO/FIXME/HACK in src/ | confirmed | grep returns 0 matches |
| 252 lib tests | confirmed | find src/ -name '*.rs' + grep #[test] + #[tokio::test] = 252 |
| 28 integration tests | confirmed | find tests/ -name '*.rs' + grep #[test] + #[tokio::test] = 28 |
| 280 total Rust test functions | confirmed | 252 + 28 = 280 |
| 202 Rust files scanned | confirmed | find src/ + tests/ -name '*.rs' = 202 |
| 30,135 Rust non-empty LOC | confirmed | find src/ + tests/ -name '*.rs' + grep non-blank = 30,135 |
| src/mcp is 17,216 LOC | contradicted | Current count is 18,293 (scope: src/mcp, non-empty .rs lines) |
| src/mcp/mod.rs is 424 LOC | contradicted | Current count is 456 lines (wc -l) |
| tests/integration_test is 1,778 LOC | contradicted | Current count is 1,797 (scope: tests/integration_test/, non-empty .rs lines) |

## Checklist Results

9 items PASS. FAIL and N/A rows below:

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N4 | Cross-references | FAIL | Three LOC counts in Baseline differ from HEAD: src/mcp (17,216 vs 18,293), mcp/mod.rs (424 vs 456), integration_test (1,778 vs 1,797) |
| C3 | Implicit assumptions | FAIL | Finding #5 assumes orphan phase 2 is committed ("before phase 2") without stating this as conditional; finding #1 conditions on "before the next persisted feature" but others lack similar scoping |
| C4 | Acceptance criteria | FAIL | Recommendations are directional ("add a storage migration module") without measurable done-when criteria |
| F3 | Timeline realism | N/A | Review document does not propose timelines; recommended order is priority sequence only |
| F5 | Rollback plan | FAIL | No rollback note for the recommended orphan module split or migration runner introduction |
| C5 | Missing roles | FAIL | No owner or team assignment per finding |

## Findings

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Baseline (Static scan summary) | Three LOC counts are stale relative to HEAD: src/mcp 17,216 vs 18,293, mcp/mod.rs 424 vs 456, integration_test 1,778 vs 1,797 | Readers may underestimate current MCP surface size when planning refactor scope | Verified by counting non-empty .rs lines in src/mcp (18,293), wc -l src/mcp/mod.rs (456), and non-empty lines in tests/integration_test/ (1,797) | Re-run the static scan at current HEAD and update the three counts. Add a footnote stating the commit hash the scan was captured at |
| 2 | Findings (all six) | Recommendations lack measurable acceptance criteria | Cannot objectively verify when a recommendation is complete | Checked each finding's "Recommended deepening" section: all use directional language ("add a module", "replace paths", "introduce a manifest") without done-when tests | For each recommendation, add 1-2 concrete acceptance criteria, e.g. "DONE WHEN: PRAGMA user_version is set on every new database and a regression test opens a v3 fixture and asserts it migrates to v4" |
| 3 | Findings #1, #4, #5 | No rollback consideration for recommended structural changes | Module split and migration runner are one-way refactorings; if they introduce regressions, the rollback path is unclear | Document does not mention rollback or reversion strategy anywhere | Add a brief "Rollback note" to finding #1 (migration runner can be feature-gated behind a config flag) and finding #5 (module split preserves public re-exports, so rollback is a directory flatten) |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | Findings #2 | Claim says "src/mcp/server_core.rs uses expect in MCP startup paths and unwrap on the server mutex" but does not mention the 7 additional expect calls in other MCP modules (guidance_payload, workspace_decision, constraints, attention, project_recommendation) | grep for .expect( in src/mcp/*.rs returns 13 results; server_core.rs accounts for 5, leaving 8 in other modules | Broaden finding #2 scope to "MCP layer uses expect for serialization invariants" or add a separate note about non-startup expect usage |
| 2 | Findings #5 | Module split proposal lists 5 sub-modules but does not mention where scanner-health validation or candidate-collection logic would land | orphan.rs defines classification, scanner-health, built-in scanners, candidate collection, and deletion-plan; the proposed split covers 5 of these but scanner-health and candidate-collection have no named module | Add scanner_health.rs and candidate.rs (or fold into types.rs/classification.rs) to the split proposal |

## Strengths

- Every referenced file and symbol exists in the codebase; no fabricated references
- Finding #1 (schema version not enforced) is precisely evidenced: SCHEMA_VERSION is declared once and never read, PRAGMA user_version is absent from the entire codebase
- Finding #3 (duplicate schemars) correctly identifies the practical resolution path (use rmcp::schemars exclusively)
- Finding #5 proposes a module split that preserves the existing public API via re-exports, which is the right architectural choice
- The "What is healthy" section provides valuable counter-evidence that prevents the review from being purely negative
- The recommended order is pragmatic: migration runner before persisted features, panic cleanup before new error paths

## Recommendations

1. **Refresh the Baseline LOC counts** at current HEAD and add the commit hash to the static scan summary. The src/mcp count in particular has drifted by ~1,000 LOC since the scan was captured, which affects the perceived severity of finding #4.

2. **Add acceptance criteria** to each finding's recommended deepening. For example, finding #1 should state: "DONE WHEN: Database::open_project runs pending migrations, PRAGMA user_version matches SCHEMA_VERSION after open, and a test fixture with schema v3 migrates to v4."

3. **Add rollback notes** for structural recommendations (#1 migration runner, #5 orphan split). The orphan split is naturally reversible (flatten directory, update mod.rs), but the migration runner is harder to undo once databases have been migrated forward.

4. **Broaden finding #2** to cover non-startup expect usage in the MCP payload layer (guidance_payload, workspace_decision, constraints, attention, project_recommendation). These 8 additional expect calls on serialization invariants have the same abort-on-failure risk in release mode.

5. **Complete the orphan split proposal** by assigning scanner-health validation and candidate-collection logic to named sub-modules, so the proposal is actionable without further design work.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 4 | All behavioral claims confirmed; three numeric counts stale at HEAD |
| Completeness | 3 | Six findings are well-chosen but lack acceptance criteria and owner assignment |
| Codebase Alignment | 5 | Every referenced file/symbol verified present; code patterns accurately described |
| Actionability | 3 | Directional recommendations without measurable done-when criteria |
| Terminology Consistency | 5 | File paths, module names, and function names match codebase exactly |
| **Overall** | **4.0** | |

## Verdict

APPROVE_WITH_NOTES — The review is well-evidenced and all six findings are valid and accurately grounded in the codebase. Three minor numeric stale-count issues, plus the absence of measurable acceptance criteria, prevent a clean APPROVE. No findings require the review to be rewritten; the notes above are additive.
