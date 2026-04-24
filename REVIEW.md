# OPENDOG — Code Review Guide for AI Auditors

> **Version:** v1.0 | **Date:** 2026-04-24 | **Status:** 43/43 requirements complete

## What This Project Does

OPENDOG is a Rust-based multi-project file monitoring system for AI development workflows on WSL2/Linux. It tracks which project files AI tools (Claude Code, Codex, GPT, GLM) actually access, and identifies unused/stale files vs. actively-used core files. It provides an MCP stdio server for AI tool integration plus separate CLI and daemon modes for manual/background management.

## Architecture Overview

```
Layer:  CLI (clap) ─────┐
       MCP stdio (rmcp) ┼─► shared handlers / core ─► storage (SQLite)
       Daemon supervisor┘
```

### Key Design Decisions

1. **No PID from inotify** — inotify(7) explicitly provides no process information. The system uses **hybrid /proc scanning + inotify**: periodic `/proc/<pid>/fd` enumeration (primary) for process attribution, and inotify (secondary) for change detection only.

2. **Per-project isolation** — Each project gets its own SQLite `.db` file, own config, own monitoring threads. Projects start/stop/delete independently.

3. **Approximate attribution** — The system is honest about limitations. `/proc` scanning is statistical sampling (2-5s intervals). Duration is estimated from consecutive scan sightings.

4. **Single-writer SQLite** — All writes through one task. WAL mode for concurrent reads.

## Module Map

```
src/
  main.rs              # Entry point — delegates to cli::run()
  lib.rs               # Module declarations
  config.rs            # ProjectConfig (ignore patterns, process whitelist), ProjectInfo
  error.rs             # OpenDogError (thiserror), Result type alias
  daemon.rs            # Daemon mode: sd_notify, SIGTERM handler, WSL detection, starts background monitors
  core/
    project.rs         # ProjectManager — CRUD, registry DB, per-project DB access
    snapshot.rs        # take_snapshot() — walkdir scan, ignore filtering, incremental update
    monitor.rs         # start_monitor() — spawns /proc scanner thread + inotify watcher thread
    scanner.rs         # ProcScanner — /proc/<pid>/fd enumeration, whitelist matching
    stats.rs           # get_stats, get_unused_files, get_core_files, get_summary
  storage/
    database.rs        # Database wrapper — open, execute, query helpers
    schema.rs          # CREATE TABLE + indexes for registry and project databases
    queries.rs         # All SQL operations — project CRUD, snapshot, stats queries
  mcp/
    mod.rs             # OpenDogServer — 8 MCP tools via rmcp #[tool_router(server_handler)]
  cli/
    mod.rs             # clap Parser with 8 subcommands + daemon subcommand
    output.rs          # Terminal table formatting (aligned columns, truncation)
```

## Database Schema

**Registry DB** (`~/.opendog/registry.db`):
- `projects` — id, root_path, db_path, config (JSON), created_at, status

**Per-project DB** (`~/.opendog/data/projects/<id>.db`):
- `snapshot` — path (PK), size, mtime, file_type, scan_timestamp
- `file_stats` — file_path (PK), access_count, estimated_duration_ms, modification_count, last_access_time, first_seen_time, last_updated
- `file_sightings` — id, file_path, process_name, pid, seen_at
- `file_events` — id, file_path, event_type (create/modify/remove), event_time

## Critical Code Paths to Review

### 1. /proc Scanning (`core/scanner.rs`)
- Enumerates all `/proc` entries via `procfs::process::all_processes()`
- Filters by process name whitelist (case-insensitive substring match)
- Resolves `/proc/<pid>/fd/` symlinks → canonical paths
- Matches against snapshot paths (relative to project root)
- **Review for:** race conditions on process exit, permission handling, symlink resolution safety

### 2. Monitor Lifecycle (`core/monitor.rs`)
- Spawns two threads: scanner thread (periodic /proc scan) + watcher thread (inotify)
- Maintains `open_state: HashMap<(String, i32), u64>` for tracking file open/close transitions
- Accumulates duration on close (consecutive sightings → estimated open duration)
- Flushes remaining open durations on stop
- **Review for:** thread safety, Duration accumulation correctness, edge cases (process crash without close)

### 3. Stats Queries (`storage/queries.rs`)
- `get_unused_files()` — LEFT JOIN snapshot with file_stats WHERE access_count IS NULL or 0
- `get_core_files(min_access_count)` — INNER JOIN WHERE access_count >= threshold
- `get_stats_with_snapshot()` — LEFT JOIN with COALESCE for null-safe enrichment
- **Review for:** SQL correctness, join semantics, NULL handling

### 4. MCP Server (`mcp/mod.rs`)
- `OpenDogServer` wraps `Mutex<ServerInner>` containing `ProjectManager` and `HashMap<String, MonitorHandle>`
- Uses `#[tool_router(server_handler)]` macro from rmcp 1.5
- Each tool handler locks mutex, delegates to core, formats JSON response
- `get_project()` helper opens project DB + fetches ProjectInfo, drops mutex lock before long operations
- **Review for:** mutex contention, error propagation, JSON response consistency

### 5. Snapshot Engine (`core/snapshot.rs`)
- Uses `walkdir` for recursive traversal, skips non-files and ignored paths
- Incremental: inserts new/changed files, deletes stale entries (path not found on disk)
- `should_ignore()` checks each path component against ignore patterns (supports glob suffix like `*.pyc`)
- **Review for:** symlink handling, ignore pattern correctness, large directory performance

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| rusqlite | 0.39 | SQLite with bundled C, WAL mode |
| notify | 8.2 | Cross-platform inotify wrapper |
| procfs | 0.18 | /proc parsing for PID/fd enumeration |
| rmcp | 1.5 | MCP server SDK, stdio transport |
| clap | 4 | CLI with derive macros |
| tokio | 1.52 | Async runtime for MCP server |
| walkdir | 2 | Recursive directory traversal |
| sd-notify | 0.5 | systemd readiness notifications |
| tracing-journald | 0.3 | Structured journald logging |

## Test Coverage

- **25 integration tests** across 4 test suites
- Phase 1: project CRUD, snapshot scanning, ignore patterns, incremental updates
- Phase 3: stats queries (empty DB, with data, unused files, core files, detail, summary)
- Tests use `tempfile` for isolated database instances

**Not tested (requires live /proc):** Monitor threads, /proc scanning, inotify events, MCP server lifecycle.

## Known Limitations

1. **WSL2 required** — WSL1 has poor inotify; `/mnt/` paths don't support inotify
2. **Approximate attribution only** — 2-5s sampling interval may miss brief file accesses
3. **No cross-process state** — CLI commands are one-shot; monitor handles are per-process (MCP server tracks them in-memory)
4. **No config persistence** — ProjectConfig is stored as JSON in registry but never modified after creation (v2: CONF-01..03)
5. **CLI `start` blocks** — The `start` command runs until Ctrl+C by design; background supervision lives in `opendog daemon`
6. **Single-process MCP monitor state** — `start_monitor` / `stop_monitor` only manage monitors created inside the current `opendog mcp` process

## Requirements Traceability

All 43 v1 requirements mapped and verified:

| Phase | Requirements | Lines of Code |
|-------|-------------|---------------|
| 1: Foundation | PROJ-01..05, SNAP-01..05 | project.rs, snapshot.rs, storage/* |
| 2: Monitoring | MON-01..06, PROC-01..06 | monitor.rs, scanner.rs |
| 3: Statistics | STAT-01..08 | stats.rs, queries.rs (stats section) |
| 4: Interfaces | MCP-01..09, CLI-01..09 | mcp/mod.rs, cli/mod.rs, cli/output.rs |
| 5: Daemon | DAEM-01..05 | daemon.rs, deploy/opendog.service |

## Build & Verify

```bash
cargo test          # 25 tests, 0 failures
cargo build --release  # 3.2MB stripped binary
```

---
*Generated for AI code review — 2026-04-24*
