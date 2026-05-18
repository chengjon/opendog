# OpenDog Technical Debt Review

Date: 2026-05-18
Branch reviewed: `master`

## Baseline

Current quality gates are green:

- `cargo fmt --check`: pass
- `cargo clippy --all-targets --all-features -- -D warnings`: pass
- `cargo test`: pass, 252 lib tests and 28 integration tests
- `python3 scripts/validate_planning_governance.py`: pass, 114 requirements phase-mapped, 20 completed task cards, 0 structural hygiene violations

Static scan summary:

- Scan captured at commit: `63b5073`
- LOC values are non-empty Rust lines unless a total-line count is explicitly stated.
- Rust files scanned: 202
- Rust non-empty LOC: 30,135
- Rust functions: 1,094
- Rust test functions: 280
- Largest LOC bucket: `src/mcp` at 17,216 non-empty LOC across 122 Rust files, 18,415 total lines
- `src/mcp/mod.rs` is 424 non-empty LOC, 457 total lines
- Next largest buckets: `src/core` 4,028 LOC, `src/cli` 2,967 LOC, `src/control` 1,780 LOC, `tests/integration_test` 1,778 non-empty LOC across 10 Rust files, 1,991 total lines
- Source TODO/FIXME/HACK markers: none found in `src`

## Findings

### 1. Storage schema version is declared but not enforced

Severity: High before the next persisted feature.

Evidence:

- `src/storage/schema.rs` declares `SCHEMA_VERSION: u32 = 4`.
- `src/storage/database.rs` executes schema strings with `CREATE TABLE IF NOT EXISTS`.
- No storage code references `SCHEMA_VERSION`, `PRAGMA user_version`, or an explicit migration runner.

Why this matters:

The current schema path is safe for initial database creation and additive `CREATE TABLE IF NOT EXISTS` changes, but it does not provide a reliable path for altering existing tables, backfilling data, or enforcing compatibility. This becomes important before orphan detection gains persisted scan runs, classification history, or deletion-plan records.

Recommended deepening:

- Add a storage migration module that owns `PRAGMA user_version`, current schema version, and stepwise migrations.
- Make `Database::open_registry` and `Database::open_project` run migrations before returning a handle.
- Add regression tests that open older fixture databases and verify they migrate forward.

Suggested owner: storage.

Acceptance criteria:

- New registry and project databases have `PRAGMA user_version = SCHEMA_VERSION` immediately after open.
- A fixture database at the previous schema version migrates to the current version while preserving representative `snapshot`, `file_stats`, and `verification_runs` data.
- `Database::open_registry` and `Database::open_project` are the only code paths that initialize schema state, and both run pending migrations.

Rollback note:

Code rollback is not enough after a released forward migration has touched user databases. Each migration PR should include a fixture/backup strategy and a documented restore path; do not attempt automatic schema downgrades unless a specific down migration has been designed and tested.

### 2. Runtime panic paths remain in MCP/control startup and mutex handling

Severity: Medium.

Evidence:

- `Cargo.toml` sets `panic = "abort"` for release builds.
- `src/mcp/server_core.rs` uses `expect` in MCP startup paths and `unwrap` on the server mutex.
- `src/control/fallback.rs` uses repeated `controller.lock().unwrap()` calls.
- `src/core/verification.rs` uses an `expect` after writing a verification row.
- MCP payload and decision helpers use additional production `expect` calls around serialization invariants, including `src/mcp/guidance_payload.rs`, `src/mcp/workspace_decision.rs`, `src/mcp/constraints.rs`, `src/mcp/attention.rs`, and `src/mcp/project_recommendation.rs`.

Why this matters:

In release mode, a panic aborts the process. For an MCP stdio server and daemon-adjacent code, abrupt aborts are hard to diagnose from the client side and can look like protocol hangs. Mutex poisoning is uncommon, but if it happens the current behavior is process termination rather than a structured `OpenDogError`.

Recommended deepening:

- Introduce a small lock helper that converts `PoisonError` into `OpenDogError`.
- Make `run_stdio` delegate to a fallible `try_run_stdio() -> Result<()>` and keep the final CLI boundary responsible for printing/logging fatal startup errors.
- Replace the verification post-insert `expect` with a domain error such as `VerificationRunNotFoundAfterInsert`.
- Replace payload serialization `expect` calls with fallible helper functions or documented conversion points that return structured MCP errors.

Suggested owner: control/MCP.

Acceptance criteria:

- Production `unwrap`/`expect` calls are removed from MCP startup, server mutex locking, control fallback mutex locking, verification post-insert lookup, and MCP payload serialization paths, except for explicitly documented process-boundary fatal errors.
- A poisoned mutex test or equivalent unit-level regression demonstrates that lock failure returns `OpenDogError` rather than panicking.
- MCP startup failures produce a clear stderr/journal error and non-zero process exit instead of an unexplained abort.

### 3. Duplicate `schemars` versions in the dependency graph

Severity: Medium-low, but cheap to fix.

Evidence:

- `Cargo.toml` declares `schemars = "0.8"`.
- `cargo tree -d` reports both `schemars v0.8.22` and `schemars v1.2.1`.
- Current code imports `rmcp::schemars` for MCP parameter and orphan DTO schema derives.

Why this matters:

The duplicate version is not currently breaking builds, but it increases compile surface and creates avoidable confusion around which `JsonSchema` derive is compatible with `rmcp`. The recent orphan work already exposed this as a sharp edge.

Recommended deepening:

- Remove the direct `schemars = "0.8"` dependency if all derives can use `rmcp::schemars`.
- Add a lightweight dependency hygiene check, for example `cargo tree -d` review in release preparation.

Suggested owner: dependency hygiene.

Acceptance criteria:

- `cargo tree -d` no longer reports both `schemars v0.8.x` and `schemars v1.x`.
- All MCP parameter and orphan DTO schema derives compile through the same schema crate path.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` remain green after dependency cleanup.

### 4. MCP surface is becoming the dominant change amplifier

Severity: Medium.

Evidence:

- `src/mcp` is 17,216 LOC, more than four times `src/core`.
- `src/mcp/mod.rs` is 424 LOC with 27 tool-facing methods.
- `src/contracts.rs` has 45 versioned contract constants.
- Adding a new MCP capability currently touches constants, params, handler, payload builder, tool registration, payload contract tests, and tool-surface tests.

Why this matters:

The project intentionally treats MCP contracts as stable operator-facing surfaces. That is correct, but the manual synchronization cost is rising. Each new tool increases the chance of missing one registration, one schema version constant, or one contract test.

Recommended deepening:

- Keep handlers thin, but introduce a single tool inventory/manifest that lists tool name, contract id, params type, payload builder, and handler module.
- Consider domain-specific MCP registration modules if the `rmcp` macro model allows it.
- Generate or validate the tool surface from the manifest so `src/mcp/mod.rs` remains a facade rather than a registry hotspot.

Suggested owner: MCP tool surface.

Acceptance criteria:

- A single inventory or manifest enumerates every MCP tool name, contract id, params type, handler module, and payload contract test owner.
- Tool-surface tests compare the registered tool list against the inventory so missing registrations fail fast.
- Adding a new tool requires changing the inventory and the domain handler/payload code, not manually updating unrelated test lists in several places.

Rollback note:

Introduce the inventory as validation-only first, with existing manual `rmcp` registration remaining authoritative. If the manifest design creates friction, it can be reverted without changing tool behavior or wire contracts.

### 5. Orphan detection core has good locality but is at the split point

Severity: Medium before phase 2.

Evidence:

- `src/core/orphan.rs` is 980 non-empty LOC.
- It defines 15 public data types plus classification, scanner-health validation, built-in text scanners, candidate collection, and deletion-plan verification.
- This is currently cohesive, but phase 2 will likely add external scanner protocol handling, persistence, and richer confidence logic.

Why this matters:

The module currently earns its keep: callers do not need to understand scanner health, evidence polarity, classification thresholds, and deletion-plan policy. However, if external scanner execution and DB persistence are added in the same file, the module will become harder to review and test.

Recommended deepening:

- Before persisted orphan scan runs, split to `src/core/orphan/` with focused modules:
  - `types.rs`
  - `classification.rs`
  - `scanner_contract.rs`
  - `builtin_scanners.rs`
  - `deletion_plan.rs`
- Assign scanner-health validation to `scanner_contract.rs` and candidate collection plus built-in text evidence scanning to `builtin_scanners.rs`.
- Keep the public API re-exported from `src/core/orphan.rs` or `src/core/orphan/mod.rs` so MCP callers do not absorb the split.

Suggested owner: core/orphan.

Acceptance criteria:

- `src/core/orphan.rs` is replaced by `src/core/orphan/` submodules while keeping existing public API imports working for MCP callers.
- Current orphan unit tests, MCP payload contract tests, and the MCP session integration test pass without behavior changes.
- Phase 2 persistence or external scanner execution code lands in named submodules instead of expanding a single top-level orphan file.

Rollback note:

This split is structurally reversible because it does not require schema or wire-contract changes. Rollback means flattening the submodules back into the facade and keeping the existing public type/function names intact.

### 6. Test coverage is broad, but helper locality is uneven

Severity: Low-medium.

Evidence:

- The suite has 280 Rust test functions.
- Several test files are large, including MCP guidance and data-risk tests above 600 LOC.
- Most high `unwrap` counts are in tests, not production paths.

Why this matters:

High unwrap use in tests is usually acceptable, but large test files with many inline fixtures make failures harder to interpret. As MCP payload contracts grow, copy-pasted JSON setup can obscure the behavior under test.

Recommended deepening:

- Extract domain fixture builders for recurring MCP payload scenarios.
- Prefer named helpers that encode the business fact being constructed, not only JSON shape.
- Keep payload contract tests, but make repeated setup data local to helpers.

Suggested owner: test architecture.

Acceptance criteria:

- The largest MCP payload contract or guidance test clusters use fixture builders for repeated project, risk, recommendation, and verification setup.
- Helper names describe domain facts, for example "project with stale verification evidence" rather than "json fixture 1".
- Existing payload contract assertions remain behavior-focused and still verify representative command strings, schema fields, and failure states.

## What is healthy

- Rust formatting, clippy, and tests are green.
- Planning governance is green.
- Structural hygiene rules report 0 violations.
- Source code has no TODO/FIXME/HACK markers.
- Storage access is mostly centralized through the storage layer.
- MCP handlers are mostly thin and delegate to payload/core helpers.
- The new orphan feature is placed in core with MCP as an adapter, which matches the existing architecture.

## Recommended order

1. Add a real storage migration runner before adding persisted orphan scan data.
2. Replace production `unwrap`/`expect` paths around MCP startup, mutex locks, and verification post-insert lookup with structured errors.
3. Remove or align the duplicate `schemars` dependency.
4. Split `src/core/orphan.rs` before implementing phase 2 persistence or external scanner execution.
5. Introduce a tool inventory/manifest to reduce MCP registration drift.
6. Add fixture builders for the largest MCP payload contract test clusters.
