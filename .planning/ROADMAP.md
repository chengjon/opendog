# Roadmap: OPENDOG

**Created:** 2026-04-24
**Phases:** 5
**Requirements:** 42 (all mapped)
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
1. Access count, duration, modification count, and last access are correctly recorded per file
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

**Success Criteria:**
1. Claude Code can discover and call all 8 MCP tools successfully
2. CLI commands produce formatted, readable terminal output
3. MCP server handles malformed requests gracefully (JSON-RPC error responses)
4. CLI provides helpful help text and error messages for invalid usage
5. Both interfaces produce identical results for the same operations

**Plans:**
1. MCP protocol layer (JSON-RPC 2.0 types, stdin/stdout framing, message parsing)
2. MCP tool definitions and handlers (8 tools, parameter validation, response formatting)
3. CLI framework (clap derive, 8 subcommands, shared handler functions)
4. CLI output formatting (tables, colors, summary views)
5. MCP server lifecycle (initialize, tool listing, request dispatch)
6. End-to-end tests for both interfaces

**Dependencies:** Phases 1-3 (all core functionality)

**Parallelization:** MCP server (plans 1,2,5) and CLI (plans 3,4) can be built in parallel

---

## Phase 5: Daemon & Deployment

**Goal:** Production-ready daemon with systemd integration. By end of this phase, OPENDOG runs as a reliable background service that starts on boot.

**Requirements:** DAEM-01, DAEM-02, DAEM-03, DAEM-04, DAEM-05

**Success Criteria:**
1. Daemon runs with <1% CPU at idle and <10MB RAM as measured by /proc
2. systemd starts OPENDOG on boot and restarts it on crash
3. SIGTERM triggers clean shutdown with all buffered events flushed
4. Logs appear in journald with structured fields (project_id, event_type, etc.)
5. WSL detection warns if running on WSL1 or if /mnt/ paths are configured

**Plans:**
1. Daemon mode (background, pid file, signal handling)
2. Systemd integration (service unit file, sd_notify, watchdog)
3. Graceful shutdown (SIGTERM handler, buffer flush, resource cleanup)
4. Structured logging (tracing + journald, log levels, filtered output)
5. WSL detection (version check, /mnt/ warning, systemd availability check)
6. Performance validation (resource profiling under load)

**Dependencies:** Phases 1-4 (all functionality)

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
```

## Parallelization Notes

- Within Phase 4: MCP server and CLI can be developed in parallel (shared core, different interfaces)
- All other phases are sequential due to tight dependencies
- Within each phase, individual plans can run in parallel where noted

---
*Roadmap created: 2026-04-24*
*Last updated: 2026-04-24 after initial creation*
