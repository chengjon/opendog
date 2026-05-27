# OPENDOG Context

This file records the domain language and load-bearing architecture decisions that should stay stable across implementation work, reviews, and agent handoffs.

## Domain Terms

- **Project**: One observed workspace registered in OPENDOG. Each project owns its own SQLite database, config, monitoring state, and retained evidence.
- **Observation evidence**: Data OPENDOG collects about file presence, file usage, modifications, snapshots, verification, and risk signals.
- **Snapshot**: A point-in-time recursive inventory of project files after ignore rules are applied.
- **File sighting**: A sampled `/proc/<pid>/fd` observation that a whitelisted process has a file open.
- **File event**: A filesystem change event observed through inotify/`notify`. It records change evidence, not process attribution.
- **File stats**: Aggregated per-file counters derived from snapshots, file sightings, and file events.
- **Retained evidence**: OPENDOG-owned operational data stored for later guidance, cleanup review, and audit. Retained-evidence cleanup must never delete source project files.
- **Activity rollup**: A daily aggregate preserved before old raw activity rows are pruned.
- **Verification evidence**: Recorded or executed validation results that guidance can use to judge readiness and freshness.
- **Guidance**: AI-facing recommendations assembled from observation evidence, verification evidence, risk signals, and toolchain detection.
- **Data risk**: Suspicious mock, placeholder, or hardcoded pseudo-business data signals that need review before source cleanup or release claims.
- **Governance lane**: A tracked stream of project governance work, observed by OPENDOG but not enforced by OPENDOG.
- **Governance node**: A tracked item inside a governance lane.
- **Control plane**: The daemon-owned local IPC interface that CLI and MCP use when the daemon is available.
- **CLI operator surface**: Human/operator commands such as config mutation, export, retained-data cleanup, daemon maintenance, and reports.
- **MCP AI surface**: MCP tools and resources exposed for AI hosts. MCP tools are versioned through contract identifiers and documented in the MCP tool reference.

## Stable Decisions

- Inotify does not provide process/PID attribution. OPENDOG attributes AI file usage through `/proc` sampling and uses inotify only as change evidence.
- Projects are isolated by database, config, monitor state, and cleanup operation.
- When the daemon is live, CLI and MCP should prefer the daemon-backed control plane instead of starting independent monitor or write paths.
- SQLite writes should remain serialized through OPENDOG-owned paths; WAL mode supports concurrent reads, not uncontrolled multi-writer behavior.
- Retained-evidence cleanup must preserve aggregate evidence first when raw activity rows are deleted.
- MCP/CLI JSON payloads are external contracts. Contract identifiers, tool inventory, docs, and tests must be updated together.
- OPENDOG observes governance state and recommends next actions; it does not enforce project governance rules or delete source files.

## Current Architecture Debt To Deepen

- MCP evidence payload modules should move toward typed internal models with JSON builders as adapters. Start with storage-maintenance and verification-evidence payloads.
- CLI, MCP, and control-plane operation mapping should continue moving toward a single descriptor-style source of truth.
- Historical planning research may describe the original 8-tool Phase 4 baseline. Current tool counts should be read from `src/mcp/tool_inventory.rs`, `docs/mcp-tool-reference.md`, `FUNCTION_TREE.md`, and `CLAUDE.md`.
