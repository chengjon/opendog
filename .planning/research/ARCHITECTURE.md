# Architecture Research: OPENDOG

## Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    SERVICE LAYER                         │
│  ┌──────────────┐          ┌──────────────────────────┐ │
│  │   CLI (clap)  │          │  MCP Server (stdio)      │ │
│  │  8 commands   │          │  JSON-RPC 2.0            │ │
│  └──────┬───────┘          └──────────┬───────────────┘ │
│         │                              │                 │
│         └──────────┬───────────────────┘                 │
│                    ▼                                     │
│            ┌──────────────┐                              │
│            │   Handlers   │  (shared business logic)     │
│            │  8 operations │                              │
│            └──────┬───────┘                              │
├───────────────────┼─────────────────────────────────────┤
│            CORE LAYER │                                  │
│  ┌───────────┐ ┌─────┴─────┐ ┌────────────┐            │
│  │  Project   │ │  Monitor   │ │  Snapshot   │            │
│  │  Manager   │ │  Engine    │ │  Engine     │            │
│  │  CRUD      │ │  inotify   │ │  walkdir    │            │
│  └─────┬─────┘ │  PID filter│ │  filtering  │            │
│        │       └─────┬──────┘ └──────┬─────┘            │
│        │             │               │                    │
│        ▼             ▼               ▼                    │
│  ┌──────────────────────────────────────┐               │
│  │         Stats Engine                  │               │
│  │  access count / duration / last used  │               │
│  └───────────────┬──────────────────────┘               │
├──────────────────┼──────────────────────────────────────┤
│          STORAGE LAYER │                                 │
│  ┌───────────────▼──────────────────────┐               │
│  │     SQLite (per project)              │               │
│  │  ┌──────────┐  ┌──────────────────┐  │               │
│  │  │ snapshot  │  │ file_stats       │  │               │
│  │  │ table     │  │ table            │  │               │
│  │  └──────────┘  └──────────────────┘  │               │
│  └──────────────────────────────────────┘               │
├─────────────────────────────────────────────────────────┤
│                    BASE LAYER                            │
│  WSL (Linux kernel) │ inotify │ /proc │ systemd          │
└─────────────────────────────────────────────────────────┘
```

## Data Flow

### Flow 1: File Monitoring (hot path)
```
inotify event → EventReader (tokio task)
  → extract (wd, mask, cookie, name)
  → resolve watch descriptor → project path + file path
  → PID lookup: read /proc/<pid>/stat → process name
  → Process filter: whitelist check (name match?)
    → No: check parent chain (/proc/<ppid>/stat up to 3 levels)
    → Still no: discard event
  → Yes (AI process): push to StatsChannel (tokio mpsc)
    → StatsWriter (single writer task): batch + write to SQLite
      → INSERT/UPDATE file_stats SET access_count++, last_access=now
```

### Flow 2: Snapshot
```
take_snapshot(project_id)
  → walkdir::WalkDir::new(project_root)
    → filter: skip node_modules, .git, dist, target, __pycache__, etc.
    → filter: skip non-regular files (symlinks, devices, pipes)
    → for each file: INSERT OR REPLACE INTO snapshot (path, size, mtime, type)
  → RETURN total file count
```

### Flow 3: MCP Request
```
stdin → read line → parse JSON-RPC request
  → match method:
    "tools/call" → extract tool name + params
    → dispatch to handler:
      create_project  → ProjectManager::create()
      take_snapshot   → SnapshotEngine::run()
      start_monitor   → MonitorEngine::start()
      stop_monitor    → MonitorEngine::stop()
      get_stats       → StatsEngine::query()
      get_unused      → StatsEngine::unused()
      list_projects   → ProjectManager::list()
      delete_project  → ProjectManager::delete()
    → serialize response as JSON-RPC
  → write to stdout + flush
```

### Flow 4: CLI Command
```
clap parse args → match subcommand
  → call same handler functions as MCP
  → format output for terminal (tables, lists)
  → write to stdout/stderr
```

## Module Structure

```
opendog/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry: detect mode (cli/daemon/mcp), dispatch
│   ├── lib.rs               # Re-exports
│   ├── config.rs            # Global config, project config loading (serde)
│   ├── error.rs             # Error types (thiserror)
│   │
│   ├── core/
│   │   ├── mod.rs
│   │   ├── project.rs       # Project CRUD, namespace management
│   │   ├── snapshot.rs      # Recursive file scan, ignore patterns
│   │   ├── monitor.rs       # inotify setup, per-project watch management
│   │   ├── process.rs       # PID→name lookup, parent chain, whitelist
│   │   └── stats.rs         # Usage stats queries, unused file detection
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs      # Connection pool, WAL mode, per-project open
│   │   ├── schema.rs        # CREATE TABLE statements, migrations
│   │   └── queries.rs       # All read/write operations
│   │
│   ├── mcp/
│   │   ├── mod.rs           # MCP server entry point (stdin/stdout loop)
│   │   ├── protocol.rs      # JSON-RPC types (Request, Response, Error)
│   │   ├── handlers.rs      # 8 tool handlers, delegate to core::
│   │   └── tools.rs         # Tool definitions (name, description, schema)
│   │
│   ├── cli/
│   │   ├── mod.rs           # clap app definition, subcommands
│   │   └── output.rs        # Terminal formatting (tables, colors)
│   │
│   └── daemon.rs            # systemd: sd_notify, signal handling, pid file
│
├── systemd/
│   └── opendog.service      # systemd unit file
│
└── tests/
    ├── integration/
    │   ├── snapshot_test.rs
    │   ├── monitor_test.rs
    │   └── mcp_test.rs
    └── fixtures/
        └── sample_project/
```

## Build Order

Based on dependency analysis:

| Phase | What | Depends On | Delivers |
|-------|------|-----------|----------|
| 1 | Storage layer | nothing | SQLite schema, connection mgmt, all queries |
| 2 | Core: project + snapshot | storage | Project CRUD, recursive file scanning |
| 3 | Core: monitor + process | storage | inotify watching, PID filtering, event recording |
| 4 | Core: stats | storage | Usage queries, unused detection |
| 5 | Service: MCP server | core (all) | 8 MCP tools over stdio JSON-RPC |
| 6 | Service: CLI | core (all) | 8 CLI commands with terminal output |
| 7 | Daemon + integration | everything | Systemd service, signal handling, e2e tests |

### Parallelization Opportunities
- Phases 2, 3, 4 can be partially parallelized (all depend only on storage)
- Phases 5 and 6 can be fully parallelized (both depend on core, not each other)

## Key Design Patterns

### Actor Pattern (Per-Project Monitoring)
Each project gets its own `tokio::task` that owns its inotify watches and events. Communication via channels.
```
ProjectActor {
    project_id: String,
    db: Database,
    watcher: RecommendedWatcher,
    stats_tx: mpsc::Sender<FileEvent>,
}
```

### Repository Pattern (Storage)
All database access goes through `storage::Database` struct. No raw SQL outside the storage module.
```
Database::open(path) → Database
Database::insert_snapshot(&self, entries) → Result<()>
Database::record_access(&self, file, pid, duration) → Result<()>
Database::get_stats(&self, project_id) → Result<Vec<FileStats>>
Database::get_unused(&self, project_id) → Result<Vec<FileEntry>>
```

### Handler Pattern (Dual Interface)
Core functions are interface-agnostic. MCP handlers and CLI commands both call the same core functions, differing only in input parsing and output formatting.

### Single Writer (SQLite)
One `tokio::task` receives events from all project monitors via a shared mpsc channel and writes to SQLite sequentially. This avoids write contention.

---
*Research completed: 2026-04-24*
