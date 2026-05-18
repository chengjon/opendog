## Context

`docs/superpowers/reviews/2026-05-18-technical-debt-review.md` records six technical debt findings on a currently green `master`: storage schema versioning is not enforced, runtime panic paths remain in MCP/control code, the dependency graph carries duplicate `schemars` versions, MCP tool registration is manually synchronized, `src/core/orphan.rs` is at the next split point, and large MCP tests need better fixture locality.

The work is cross-cutting. It touches storage initialization, MCP server/runtime behavior, dependency declarations, test architecture, and the orphan detection core. The implementation should therefore proceed in small independently verifiable slices while preserving existing MCP/CLI wire contracts.

## Goals / Non-Goals

**Goals:**

- Make project and registry databases explicitly versioned and migration-aware.
- Convert production panic-prone paths to structured errors or documented process-boundary exits.
- Align schema generation dependencies around the `rmcp::schemars` path.
- Add a manifest/inventory that makes MCP tool metadata auditable from one place.
- Split orphan detection internals without changing the public API used by MCP handlers.
- Improve MCP test fixture reuse while preserving existing payload contract assertions.

**Non-Goals:**

- Changing existing MCP tool names, contract IDs, or JSON response schemas.
- Adding persisted orphan scan runs or external scanner execution in this change.
- Replacing SQLite, `rmcp`, or the current CLI/MCP split.
- Rewriting all MCP tests; only the largest repeated setup clusters need fixture extraction.

## Decisions

1. **Use additive migration scaffolding first.**
   Add a migration runner that is called from `Database::open_registry` and `Database::open_project`, owns `PRAGMA user_version`, and can migrate old fixture databases forward. Avoid automatic down migrations.

2. **Keep fatal startup errors at the process boundary only.**
   `run_stdio` should delegate to a fallible `try_run_stdio() -> Result<()>`. Lower-level mutex, serialization, and post-insert lookup failures should become structured errors, not panics.

3. **Prefer `rmcp::schemars` as the schema derive source.**
   Existing MCP params and orphan DTOs already import `rmcp::schemars`. Remove the direct `schemars = "0.8"` dependency only after verifying all derives still compile and `cargo tree -d` no longer reports duplicate schemars versions.

4. **Introduce MCP inventory as validation before generation.**
   The first version should enumerate tool name, contract ID, params type, handler module, payload builder, and test owner, then validate registered tools against it. Generated registration can be considered later if the `rmcp` macro model supports it cleanly.

5. **Split orphan internals behind a compatibility facade.**
   Move implementation into `src/core/orphan/` modules while preserving existing public item names through re-exports. `scanner_contract.rs` owns scanner health validation; `builtin_scanners.rs` owns candidate collection and built-in text evidence scanning.

6. **Extract test helpers around domain facts, not JSON shape.**
   Fixture builders should encode behaviors such as stale verification evidence, hardcoded data risk, or missing project verification commands. Contract tests should still assert schema fields and representative command strings.

## Risks / Trade-offs

- [Risk] Migration runner bugs can corrupt user databases. -> Mitigation: run migrations only on open, add fixture-based migration tests, document backup/restore expectations, and avoid down migrations.
- [Risk] Replacing panics may obscure truly fatal startup failures. -> Mitigation: keep one process-boundary error path that logs to stderr/journal and exits non-zero.
- [Risk] MCP inventory duplicates metadata before it removes duplication. -> Mitigation: make it validation-only initially and require tests to compare registration against inventory.
- [Risk] Orphan module splitting can create churn without behavior change. -> Mitigation: keep public re-exports stable and require existing orphan/MCP tests to pass unchanged.
- [Risk] Fixture builders can hide contract details. -> Mitigation: use helpers only for setup and keep explicit assertions in contract tests.

## Migration Plan

1. Implement storage migration scaffolding and fixture tests before any new persisted orphan data.
2. Replace runtime panic paths and keep MCP startup error reporting observable.
3. Align the `schemars` dependency and verify the dependency tree.
4. Add MCP inventory validation without changing registered tool behavior.
5. Split orphan internals behind public re-exports.
6. Extract fixture builders in the largest MCP payload/guidance test clusters.
7. Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `python3 scripts/validate_planning_governance.py`.

Rollback strategy:

- Migration changes require database backup/restore guidance; do not rely on automatic downgrades.
- MCP inventory, orphan module split, dependency hygiene, and fixture extraction are reversible with code rollback because they should not change wire contracts or schema data.

## Open Questions

- Should migration fixtures live under `tests/fixtures/` or a storage-local fixture directory?
- Should the MCP inventory be a Rust static structure, a generated Rust module, or a checked-in data file consumed by tests?
- Which MCP test cluster should be the first fixture-builder pilot: guidance basics, data-risk cases, or payload contracts?

