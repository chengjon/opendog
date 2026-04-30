# Roadmap: OPENDOG

**Created:** 2026-04-24
**Phases:** 6
**Requirements:** 114 total | 114 phase-mapped | 0 backlog
**Capability anchor:** `.planning/FUNCTION_TREE.md`
**Mode:** YOLO | Standard granularity | Parallel execution

---

## Phase 1: Foundation — Storage, Project & Snapshot

**Goal:** Establish the storage layer, project management, and file snapshot engine. By end of this phase, users can create projects, scan files, and query the snapshot.

**Requirements:** PROJ-01, PROJ-02, PROJ-03, PROJ-04, PROJ-05, SNAP-01, SNAP-02, SNAP-03, SNAP-04, SNAP-05

**Success Criteria:**
1. User can create a project and it persists in its own SQLite database
2. User can scan a 10K-file project in under 5 seconds with correct filtering
3. User can list all projects and see their status
4. User can delete a project and verify all files are removed
5. Snapshot correctly filters node_modules, .git, target, dist, __pycache__

**Plans:**
1. Rust project scaffolding (Cargo.toml, module structure, error types)
2. SQLite storage layer (schema, connection management, WAL mode, all queries)
3. Project manager (CRUD operations, config loading, namespace isolation)
4. Snapshot engine (walkdir recursive scan, ignore patterns, metadata extraction)
5. Integration tests for storage + snapshot pipeline

**Dependencies:** None (foundational phase)

---

## Phase 2: Monitoring Engine — /proc Scanner + inotify Change Detection

**Goal:** Build the file monitoring engine with approximate AI process attribution. By end of this phase, the system can identify which files AI processes have open and detect file changes in project directories.

**Requirements:** MON-01, MON-02, MON-03, MON-04, MON-05, MON-06, PROC-01, PROC-02, PROC-03, PROC-04, PROC-05, PROC-06

**Success Criteria:**
1. Monitor starts/stops per project independently without affecting others
2. /proc scanner correctly identifies files open by whitelisted AI processes (≥90% recall for files open >5s)
3. Non-AI process file access is excluded from stats (whitelist filtering verified)
4. inotify change detection captures file modifications in watched directories
5. Approximate attribution: files modified while an AI process had them open are attributed correctly (within scan interval tolerance)
6. Monitor handles inotify watch limit gracefully (warn + fallback)

**Plans:**
1. /proc scanner (periodic /proc/<pid> enumeration, process name whitelist, /proc/<pid>/fd/ symlink resolution via procfs crate)
2. inotify change detection (notify crate, recursive watches, file modification/create/delete events)
3. Approximate attribution engine (timestamp cross-reference between /proc sightings and inotify events)
4. Event-to-storage pipeline (tokio channels, single writer, batched SQLite writes)
5. Scan interval configuration and tuning
6. Integration tests with synthetic process/file scenarios

**Dependencies:** Phase 1 (storage layer for recording events)

---

## Phase 3: Statistics Engine — Usage Analytics

**Goal:** Build the statistics and analytics layer. By end of this phase, users can query file usage data, identify unused files, and find core files.

**Requirements:** STAT-01, STAT-02, STAT-03, STAT-04, STAT-05, STAT-06, STAT-07, STAT-08

**Success Criteria:**
1. Access count, estimated duration, modification count, and last access are correctly recorded per file
2. Unused file query returns only files in snapshot with zero AI access records
3. Core file query returns files ranked by access frequency/duration
4. Statistics survive daemon restart (persistent in SQLite)
5. Query response time <100ms for a 10K-file project

**Plans:**
1. Stats recording (access count increment, duration calculation, modification tracking)
2. Snapshot-vs-stats comparison (identify never-accessed files)
3. Query functions (per-file stats, unused list, core file ranking)
4. Data aggregation (project-level summaries, file type breakdowns)
5. Tests with realistic event sequences

**Dependencies:** Phase 1 (snapshot data), Phase 2 (event recording)

---

## Phase 4: Service Interfaces — MCP Server & CLI

**Goal:** Build the user-facing interfaces. By end of this phase, AI tools can interact with OPENDOG via MCP and users can manage it via CLI. Both interfaces share the same core logic.

**Requirements:** MCP-01 through MCP-09, CLI-01 through CLI-09

**Historical baseline note:** This phase defined the original 8-tool / 8-command control surface. The current shipped product is larger because later hardening added reporting, config, export, cleanup, verification, guidance, and data-risk entrypoints under later requirement families.

**Success Criteria:**
1. Claude Code can discover and call all 8 baseline MCP tools successfully
2. CLI commands produce formatted, readable terminal output
3. MCP server handles malformed requests gracefully (JSON-RPC error responses)
4. CLI provides helpful help text and error messages for invalid usage
5. Both interfaces produce identical results for the same operations

**Plans:**
1. MCP integration layer (rmcp stdio transport, server setup, capability registration)
2. MCP tool definitions and handlers (8 baseline tools, parameter validation, response formatting via rmcp)
3. CLI framework (clap derive, 8 baseline subcommands, shared handler functions)
4. CLI output formatting (tables, colors, summary views)
5. MCP server lifecycle (initialize, tool listing, request dispatch)
6. End-to-end tests for both interfaces

**Dependencies:** Phases 1-3 (all core functionality)

**Parallelization:** MCP server (plans 1,2,5) and CLI (plans 3,4) can be built in parallel

---

## Phase 5: Daemon & Deployment

**Goal:** Production-ready daemon with systemd integration. By end of this phase, OPENDOG runs as a reliable background service that starts on boot.

**Requirements:** DAEM-01, DAEM-02, DAEM-03, DAEM-04, DAEM-05, CTRL-01, CTRL-02, CTRL-03, CTRL-04, CTRL-05

**Success Criteria:**
1. Daemon runs with <1% CPU at idle and <10MB RAM as measured by /proc
2. systemd starts OPENDOG on boot and restarts it on crash
3. SIGTERM triggers clean shutdown with all buffered events flushed
4. Logs appear in journald with structured fields (project_id, event_type, etc.)
5. WSL detection warns if running on WSL1 or if /mnt/ paths are configured
6. CLI and MCP reuse daemon-owned monitor state and project operations through a stable local control plane when the daemon is live

**Plans:**
1. Daemon mode (background, pid file, signal handling)
2. Systemd integration (service unit file, sd_notify, watchdog)
3. Graceful shutdown (SIGTERM handler, buffer flush, resource cleanup)
4. Structured logging (tracing + journald, log levels, filtered output)
5. WSL detection (version check, /mnt/ warning, systemd availability check)
6. Performance validation (resource profiling under load)
7. Local control plane coordination (daemon socket protocol, remote project operations, fallback and remediation behavior)

**Dependencies:** Phases 1-4 (all functionality)

---

## Phase 6: AI Guidance & Reusable Intelligence

**Goal:** Turn OPENDOG from a monitoring backend into a reusable information and decision-support layer for AI workflows. By end of this phase, MCP exposes eight high-value reusable layers: workspace observation, repository status and risk summaries, AI execution strategy suggestions, verification evidence, multi-project portfolio views, cleanup and refactor candidates, project type and toolchain identification, and explicit constraints and boundaries. It also identifies MOCK or hardcoded pseudo-data without requiring every target project to build these capabilities itself.

**Current status:** In progress. Guidance schema, repository-risk summaries, verification evidence, daemon IPC coordination, retained-evidence cleanup/storage-maintenance signaling, and project/workspace data-risk views are implemented. Remaining work is focused on refinement, broader coverage, and documentation/usage hardening rather than greenfield introduction.

**Requirements:** OBS-01 through OBS-04, RISK-01 through RISK-04, STRAT-01 through STRAT-04, EVID-01 through EVID-04, PORT-01 through PORT-04, CLEAN-01 through CLEAN-04, STACKX-01 through STACKX-04, BOUND-01 through BOUND-04, MOCK-01 through MOCK-10, RET-01 through RET-06

**Success Criteria:**
1. MCP returns structured workspace observation summaries with explicit evidence gaps and freshness state
2. MCP returns repository risk summaries and AI execution suggestions that are explicit about evidence and confidence
3. MCP returns multi-project prioritization views and per-project review focus without requiring target repos to implement their own meta-layer
4. MCP returns file-level cleanup/refactor candidates and project-type-aware validation suggestions
5. MOCK detection distinguishes explicit test/mock artifacts from riskier hardcoded business-like pseudo-data
6. Users and AI can preview and selectively prune retained OPENDOG evidence with explicit storage metrics and without mutating source files
7. All new outputs remain read-only, structured, and auditable for later user or AI review

**Plans:**
1. Workspace observation schema and evidence-gap reporting
2. Repository risk summary and AI execution strategy model
3. Verification/evidence payload design with confidence and boundary metadata
4. Multi-project portfolio and prioritization views
5. Cleanup/refactor candidate outputs and project type/toolchain detection
6. MOCK detection heuristics (path names, token patterns, suspicious hardcoded data signatures)
7. Retained-evidence cleanup and storage-maintenance outputs
8. Structured finding storage or export format for user/AI review workflows
9. Integration tests for guidance correctness, evidence boundaries, retention-safety rules, and MOCK detection false-positive boundaries

**Dependencies:** Phases 3-5 (stats, interfaces, daemon control plane)

---

## Dependency Graph

```
Phase 1 (Foundation)
    ↓
Phase 2 (Monitoring) ← depends on Phase 1
    ↓
Phase 3 (Statistics) ← depends on Phase 1 + 2
    ↓
Phase 4 (Interfaces) ← depends on Phase 1 + 2 + 3
    ↓
Phase 5 (Daemon)     ← depends on Phase 1 + 2 + 3 + 4
    ↓
Phase 6 (AI Intel)   ← depends on Phase 3 + 4 + 5
```

## Backlog Queue

There are currently no backlog-only requirement families.

Most recent promotion:

- `RPT-01..03` via [`TASK-20260427-comparative-time-window-analytics`](./task-cards/TASK-20260427-comparative-time-window-analytics.md) is now shipped and no longer backlog-only

Current implication:

- all 114 mapped requirements are now phase-assigned
- future backlog cards should only appear here when a requirement family is deliberately left unscheduled again

Promotion rule remains:

- a backlog family should normally gain a task card before it gains a numbered phase
- adding a task card does not change the phase-mapped count until the work is actually scheduled

## Parallelization Notes

- Within Phase 4: MCP server and CLI can be developed in parallel (shared core, different interfaces)
- Within Phase 6: guidance logic and MOCK detection can be developed in parallel if they share a stable result schema
- All other phases are sequential due to tight dependencies
- Within each phase, individual plans can run in parallel where noted

## Governance Note

Future roadmap edits and task cards should declare which `FT-*` leaf nodes from `.planning/FUNCTION_TREE.md` they change.

Default task-card skeleton:

- `.planning/TASK_CARD_TEMPLATE.md`
- `.planning/task-cards/` for concrete cards
- `.planning/GOVERNANCE.md` for the end-to-end governance workflow
- `python3 scripts/validate_planning_governance.py` as the preferred single check
- `python3 scripts/validate_task_cards.py` for lightweight gate checks
- `python3 scripts/validate_requirement_mappings.py` for requirement-side mapping checks

---
*Roadmap created: 2026-04-24*
*Last updated: 2026-04-27 after formalizing control-plane coordination and retained-evidence cleanup in the active phase map*
