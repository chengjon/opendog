# Architecture Research: OPENDOG

## Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    SERVICE LAYER                         │
│  ┌──────────────┐          ┌──────────────────────────┐ │
│  │   CLI (clap)  │          │  MCP Server (rmcp/stdio)  │ │
│  │  8 commands   │          │  8 tools                  │ │
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
│  │  CRUD      │ │ /proc+inotify│ │  filtering  │            │
│  └─────┬─────┘ │ AI scanner │ └──────┬─────┘            │
│        │       └─────┬──────┘        │                    │
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

## ⚠ Derived Design Decision: Process Attribution Strategy

> **This is NOT in the original README. It is a derived design decision required because inotify does not provide process attribution.**

**The problem:** Linux inotify(7) explicitly states: "The inotify API provides no information about the user or process that triggered the inotify event." There is no PID field in inotify events. The original README's "仅捕捉AI工具相关进程访问的文件" cannot be implemented by filtering inotify events by PID — the data simply doesn't exist.

**The solution: Hybrid /proc scanning + inotify change detection**

1. **Primary: Periodic /proc/<pid>/fd scanning** (every 2-5 seconds)
   - Enumerate all processes via /proc, filter by whitelist (claude, codex, node, python, etc.)
   - For each matched process, read /proc/<pid>/fd/ directory (symlinks to open files)
   - Resolve each fd symlink → real file path
   - Cross-reference with project snapshots → identify which project files are open
   - Record: file_path, process_name, pid, timestamp, open state

2. **Secondary: inotify via notify crate** for change detection
   - Watch project directories for file modifications (IN_MODIFY, IN_CREATE, IN_DELETE)
   - This tells us WHAT changed, but NOT who changed it
   - Use timestamps to approximately attribute changes to AI processes seen in /proc scans

3. **Approximate attribution logic:**
   - If file F was modified at time T, AND /proc scan at time T±scan_interval showed an AI process had F open → attribute modification to that AI process
   - Access count = number of /proc scans where file appeared as open fd
   - Duration = sum of (close_time - open_time) estimated from consecutive scans
   - **This is statistical sampling, not precise auditing.** Honest about limitations.

**Tradeoffs:**
- ✅ Actually works (unlike inotify+PID filtering which is impossible)
- ✅ Non-intrusive (reads /proc, no ptrace or fanotify)
- ⚠ Sampling-based (2-5s interval, may miss very brief file accesses)
- ⚠ Approximate duration (based on scan intervals, not exact open/close)
- ⚠ /proc access requires same-user or root permissions for other users' processes

## Data Flow

### Flow 1: AI Process File Tracking (primary — /proc scanning)
```
Timer (every 2-5s) → Scanner (tokio task)
  → enumerate /proc/ entries
  → for each pid: read /proc/<pid>/comm → process name
  → whitelist match? (claude, codex, node, python, etc.)
    → No: skip this process
    → Yes: read /proc/<pid>/fd/ directory
      → for each fd symlink: resolve to real path
      → cross-reference with project snapshot paths
      → if path belongs to a monitored project:
        → record/update: file_path, process_name, pid, timestamp
        → if newly appeared: mark as "opened" (access_count++)
        → if disappeared since last scan: mark as "closed", compute duration
```

### Flow 1b: File Change Detection (secondary — inotify via notify)
```
notify event → EventReader (tokio task)
  → extract (kind, paths)
  → resolve to project path + file path
  → record modification event with timestamp
  → NO process attribution from inotify (impossible)
  → approximate attribution: match timestamp against /proc scan records
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

### Flow 3: MCP Request (via rmcp)
```
rmcp receives JSON-RPC request over stdin
  → match tool name:
    create_project  → ProjectManager::create()
    take_snapshot   → SnapshotEngine::run()
    start_monitor   → MonitorEngine::start()
    stop_monitor    → MonitorEngine::stop()
    get_stats       → StatsEngine::query()
    get_unused      → StatsEngine::unused()
    list_projects   → ProjectManager::list()
    delete_project  → ProjectManager::delete()
  → rmcp serializes response → stdout
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
│   │   ├── monitor.rs       # /proc scanner + inotify change detection
│   │   ├── scanner.rs       # /proc/<pid>/fd enumeration, AI process detection
│   │   └── stats.rs         # Usage stats queries, unused file detection
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs      # Connection pool, WAL mode, per-project open
│   │   ├── schema.rs        # CREATE TABLE statements, migrations
│   │   └── queries.rs       # All read/write operations
│   │
│   ├── mcp/
│   │   ├── mod.rs           # MCP server entry point (rmcp stdio transport)
│   │   ├── handlers.rs      # 8 tool handlers, delegate to core::
│   │   └── tools.rs         # Tool definitions (name, description, schema via rmcp)
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
| 3 | Core: monitor + /proc scanner | storage | /proc fd scanning, inotify change detection, approximate attribution |
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
