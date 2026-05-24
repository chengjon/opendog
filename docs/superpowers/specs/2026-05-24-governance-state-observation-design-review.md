# Review: 2026-05-24-governance-state-observation-design.md

**Type**: .md / arch | **Perspective**: architecture, consistency | **Date**: 2026-05-24

## Summary

A well-structured design spec for governance state observation that correctly maps to FT-03 and aligns with OPENDOG's observe-don't-enforce philosophy. The data model and tool surface are cleanly scoped. One factual error on guidance layer count and a few naming inconsistencies need resolution before implementation.

## Verified

- **A1 Component boundaries**: Lane/node separation is clear; steward node scope (evidence, artifacts, git head, forbidden scope) is well-bounded.
- **A3 Coupling**: Governance module has no dependency on monitor, scanner, or snapshot internals — it only reads existing stats/verification/data-risk through well-known query surfaces.
- **A4 Interface contracts**: All 4 MCP tools have explicit param and response JSON examples.
- **A5 Scalability**: Per-project SQLite isolation, no cross-project data, JSON array fields for refs scale with project complexity.
- **A6 Terminology consistency**: "lane", "node", "steward" used consistently throughout; matches the Steward Tree pattern described.
- **A7 Backward compatibility**: New tables only, no schema changes to existing tables. `has_governance_state: false` for projects without governance preserves existing guidance consumers.
- **A9 Named entities — existing files**: All 8 files listed for modification exist (`schema.rs`, `migrations.rs`, `guidance_types.rs`, `guidance_payload.rs`, `params.rs`, `tool_inventory.rs`, `mod.rs`, `contracts.rs`).
- **N3 Formatting**: Consistent heading hierarchy, table usage, and code block formatting throughout.
- **N5 Style consistency**: Uniform formal technical writing style.
- **Schema version**: Doc claims 4→5. Confirmed `SCHEMA_VERSION = 4` in `src/storage/schema.rs:89`. Correct.
- **MCP tool count**: Doc claims 22 existing. Confirmed 22 `McpToolSpec` entries in `src/mcp/tool_inventory.rs`. Correct.
- **CLI command count**: Doc claims 22 existing. Confirmed 22 `Cli` enum variants in `src/cli/mod.rs`. Correct.
- **FT-03 parent**: Doc says FT-03 is "AI Decision Support and Governance". Confirmed in `FUNCTION_TREE.md:324`. Correct.
- **FT-03.09 sequencing**: Current FT-03 leaves end at FT-03.08. FT-03.09 is the next available slot. Correct.

## Issues

- [ ] **[HIGH]** Guidance layer count is wrong — doc says "5th layer" but codebase has 7 existing layers — Guidance Integration:line 199-200
      Evidence: Doc says "The existing `get_guidance` response gains a 5th layer." But `src/mcp/guidance_payload.rs` sets 7 existing layer keys: `workspace_observation`, `execution_strategy`, `multi_project_portfolio`, `storage_maintenance`, `verification_evidence`, `project_toolchain`, `constraints_boundaries` (lines 459, 491, 544, 554, 555, 557, 558). The doc's example JSON (lines 203-247) shows only 3 of these with `"..."` placeholders, implying 4 total. The governance layer would be the 8th, not the 5th.

- [ ] **[MED]** `delete_governance_lane` MCP tool name is misleading — MCP Tools:line 184
      Evidence: The tool supports three actions: `delete` (hard delete), `complete` (mark completed), `defer` (mark deferred). The tool name `delete_governance_lane` implies only hard deletion. The CLI equivalent is `close-lane`, which is more accurate. Consider renaming to `close_governance_lane` for MCP/CLI naming alignment, or document the naming rationale.

- [ ] **[MED]** Inconsistent response example coverage across MCP tool descriptions — MCP Tools:lines 95-195
      Evidence: `create_governance_lane` (line 109) and `get_governance_state` (line 150) include response JSON examples. `upsert_steward_node` (line 118) and `delete_governance_lane` (line 189) omit response examples. Checked the doc — no internal section addresses upsert/delete response shapes. Implementation specs should show all response contracts.

- [ ] **[MED]** Migration approach unspecified — Code Impact:line 359
      Evidence: Doc says `src/storage/migrations.rs` needs "v4→v5 migration" (+40 lines). Current migration system uses a single `migrate()` function (line 29) that calls `SCHEMA_VERSION` from `schema.rs` — there are no individual `migrate_v4_to_v5` step functions. The +40 line estimate implies a separate migration function, which departs from the current pattern. Doc should clarify whether it follows the existing `migrate()` pattern or introduces versioned migration steps.

- [ ] **[LOW]** Numeric claim "22→26 (+18%)" is 18.18% — MCP Tools:line 93
      Evidence: 4/22 = 18.18%. Rounded to 18% is fine but the precision should match other claims (e.g., "+8.5%" for source lines, "+4.5%" for CLI commands). Scope: `src/mcp/tool_inventory.rs` McpToolSpec count.

- [ ] **[LOW]** `tests/governance_test.rs` path doesn't match existing test structure — Code Impact:line 371
      Evidence: Existing tests live under `tests/integration_test/` with subdirectory modules (e.g., `tests/integration_test/cli_guidance/`). `tests/governance_test.rs` as a top-level file diverges from this pattern. Consider `tests/integration_test/governance.rs` or a `tests/integration_test/cli_governance/` subdirectory.

## Suggestions

- Fix the guidance layer count and expand the example JSON in the Guidance Integration section to show all 7 existing layer keys with `"..."` placeholders, then add governance as the 8th.
- Rename `delete_governance_lane` to `close_governance_lane` on the MCP surface, or add a brief design rationale for the naming choice.
- Add response JSON examples for `upsert_steward_node` and `delete_governance_lane` to complete the interface contract specification.
- Clarify the migration strategy: will the v4→v5 step be inlined into the existing `migrate()` function, or will a new versioned migration dispatch pattern be introduced?
- Align the test file path with the `tests/integration_test/` convention used by the rest of the codebase.

## Verdict
NEEDS_REVISION — The guidance layer count error is a factual inaccuracy that would propagate into implementation. The naming and migration approach ambiguities should be resolved before code is written. Otherwise the design is solid and well-aligned with OPENDOG's architecture.
