## Why

OpenDog's current quality gates are green, but the technical debt review identifies several structural risks that will become expensive as orphan detection, MCP tooling, and persisted evidence grow. This change turns those review findings into governed implementation work before new persistence and tool-surface expansion compound the debt.

## What Changes

- Add a real storage migration path around `SCHEMA_VERSION` and `PRAGMA user_version`.
- Replace production panic paths in MCP/control/runtime payload code with structured errors where possible.
- Remove or align the duplicate `schemars` dependency path.
- Add an MCP tool inventory so registration, contracts, params, handlers, payload tests, and tool-surface tests stay synchronized.
- Split the orphan detection core into focused submodules before phase 2 persistence or external scanner execution.
- Extract MCP payload/guidance fixture builders so large contract tests remain readable and behavior-focused.

## Capabilities

### New Capabilities

- `storage-migrations`: Versioned registry/project database migration management.
- `runtime-error-boundaries`: Structured runtime error handling for MCP/control panic-prone paths.
- `dependency-hygiene`: Dependency graph hygiene for schema-generation dependencies and duplicate-version checks.
- `mcp-tool-inventory`: Single source of truth for MCP tool registration and contract metadata.
- `orphan-module-boundary`: Stable orphan detection module boundary split with public API compatibility.
- `mcp-test-fixtures`: Reusable domain fixture builders for large MCP payload and guidance tests.

### Modified Capabilities

- None.

## Impact

- `src/storage/schema.rs`, `src/storage/database.rs`, and storage tests.
- `src/mcp/server_core.rs`, `src/control/fallback.rs`, `src/core/verification.rs`, and MCP payload/decision helpers that currently use production `expect`.
- `Cargo.toml` and dependency review workflow.
- `src/mcp/mod.rs`, `src/contracts.rs`, `src/mcp/params.rs`, payload builders, and MCP tests.
- `src/core/orphan.rs` and future `src/core/orphan/` submodules.
- MCP payload/guidance test modules under `src/mcp/tests/**`.

