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

- MCP evidence and guidance payload modules should move toward typed internal models with JSON builders as adapters. Storage-maintenance candidate assessment, workspace aggregation, and execution-template inputs now have typed models. Verification-evidence workspace aggregation, single-project status, and gate-assessment payloads now have typed models. Workspace-observation layer status now uses a typed model. Execution-strategy workspace profile selection now has a typed model for mode, tool preference, evidence priority, and recommended-flow text; guidance data-risk focus and repo-truth gap aggregation now have typed distribution models, repo-risk strategy coupling now has a typed internal model, typed source, typed repository-risk finding details, and typed recommended-action, strategy-mode, and preferred-primary-tool models, execution-strategy summary counts/lists now use concrete internal types, execution-strategy layer status and profile fields now use typed status, global strategy mode, preferred tool, and evidence-priority models, execution-strategy recommended flow now uses a concrete string list, execution-strategy review-focus projection now has a typed status/source model and concrete optional source-project string, and execution-strategy external-truth boundary now has a typed status/source/checks model, concrete optional source-project string, and typed mode enum. Decision-support action, risk, and entrypoint selection now has typed models for action class, phase, mutability scope, verification requirement, primary-goal text, risk tier, gate fallback, blockers, repo-risk findings, manual-review flags, next MCP tools, CLI commands, selection reasons, and tool-selection policy. Constraints readiness snapshots now have a typed model for cleanup/refactor blockers, verification gate fallback, repository-risk signals, and readiness reasons. Project-recommendation evidence-collection, review, review-focus, and forced-action recommendations now have typed models for baseline evidence collection, unused-file and hot-file review payloads, candidate family, candidate basis, repo-risk hints, failing-verification recovery, verification-before-high-risk guidance, and repository-stabilization recommendation payloads.
- CLI, MCP, and control-plane operation mapping should continue moving toward a single descriptor-style source of truth.
- Historical planning research may describe the original 8-tool Phase 4 baseline. Current tool counts should be read from `src/mcp/tool_inventory.rs`, `docs/mcp-tool-reference.md`, `FUNCTION_TREE.md`, and `CLAUDE.md`.
