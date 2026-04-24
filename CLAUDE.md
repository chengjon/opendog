# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**OPENDOG** — A multi-project file monitoring system for AI development workflows on WSL. Tracks which files AI tools (Claude Code, Codex, GPT, GLM) access, identifying unused/stale files vs actively-used core files. Dual interface: MCP server (stdio) for AI tool integration + CLI for manual management.

**Current state**: Planning complete. 43 requirements across 5 phases defined in `.planning/`. No source code yet — ready for Phase 1 implementation.

**Authors**: JohnC (ninjas@sina.com) + Claude (GLM-5.1) + CodeX (GPT-5.4)

## Build & Run (planned — Rust project)

```bash
cargo build --release            # Compile optimized binary
cargo test                       # Run all tests
cargo test test_snapshot         # Run single test by name
cargo test --test integration    # Run integration tests only
RUST_LOG=debug cargo run -- start --id myproject  # Run with debug logging
```

## Tech Stack

| Component | Crate | Notes |
|-----------|-------|-------|
| File watching | `notify` 8.2 | Cross-platform, wraps inotify on Linux |
| MCP server | `rmcp` 1.5 | Official Rust MCP SDK, stdio transport |
| SQLite | `rusqlite` 0.39 | Bundled C source, WAL mode for concurrency |
| CLI | `clap` 4.6 | Derive macros for subcommands |
| Async | `tokio` 1.52 | rt-multi-thread for concurrent monitoring |
| Process inspection | `procfs` 0.18 | /proc parsing for PID→name, fd enumeration |
| Logging | `tracing` 0.1 | + tracing-subscriber, tracing-appender |
| Systemd | `sd-notify` 0.5 | READY/STATUS/WATCHDOG notifications |

Full dependency list in `.planning/research/STACK.md`.

## Architecture (Critical — Read Before Coding)

### ⚠ Key Design Decision: Process Attribution

**inotify does NOT provide process/PID information.** This is documented in inotify(7): *"The inotify API provides no information about the user or process that triggered the inotify event."* Never design around "get PID from inotify event" — it's impossible.

**Actual approach (hybrid /proc scanning + inotify):**

1. **Primary: Periodic /proc/<pid>/fd scanning** (every 2-5s)
   - Enumerate /proc entries, filter by process name whitelist (claude, codex, node, python)
   - For matched processes, read /proc/<pid>/fd/ symlinks → real file paths
   - Cross-reference with project snapshots → record which AI processes have which files open
   - This is **statistical sampling**, not precise auditing. Honest about limitations.

2. **Secondary: inotify via notify crate** for change detection
   - Detect file modifications, creates, deletes in project directories
   - Tells WHAT changed, not WHO changed it
   - Timestamp-based approximate attribution against /proc scan data

### Layered Architecture

```
Service:  MCP server (rmcp/stdio) + CLI (clap) → shared handlers → core
Core:     Project Manager + Monitor (/proc scanner + inotify) + Snapshot + Stats
Storage:  Per-project SQLite (.db files, WAL mode, single-writer pattern)
Base:     WSL (Linux kernel) + systemd
```

### Module Structure (target)

```
src/
  main.rs              # Entry: detect mode (cli/daemon/mcp), dispatch
  config.rs            # Project config loading (serde)
  error.rs             # Error types (thiserror)
  core/
    project.rs         # Project CRUD, namespace management
    snapshot.rs        # Recursive file scan, ignore patterns (notify)
    monitor.rs         # /proc scanner + inotify change detection
    scanner.rs         # /proc/<pid>/fd enumeration, AI process detection
    stats.rs           # Usage stats queries, unused file detection
  storage/
    database.rs        # SQLite connection management, WAL mode
    schema.rs          # CREATE TABLE statements, migrations
    queries.rs         # All read/write operations
  mcp/
    mod.rs             # MCP server (rmcp stdio), request dispatch
    handlers.rs        # 8 tool handlers, delegate to core::
    tools.rs           # Tool definitions (name, description, schema)
  cli/
    mod.rs             # clap subcommand definitions
    output.rs          # Terminal formatting (tables, colors)
  daemon.rs            # systemd: sd_notify, signal handling, pid file
```

### Per-Project Isolation

Each project: own SQLite `.db`, own config, own monitoring state. Projects can start/stop/delete independently. No cross-project data leakage.

## MCP Tools (8)

`create_project`, `take_snapshot`, `start_monitor`, `stop_monitor`, `get_stats`, `get_unused_files`, `list_projects`, `delete_project`

## CLI Commands (8)

`opendog create --id <ID> --path <DIR>`, `opendog snapshot --id <ID>`, `opendog start --id <ID>`, `opendog stop --id <ID>`, `opendog stats --id <ID>`, `opendog unused --id <ID>`, `opendog list`, `opendog delete --id <ID>`

## Planning Artifacts

All in `.planning/`:
- `PROJECT.md` — project context, requirements, constraints, key decisions
- `REQUIREMENTS.md` — 43 requirements with REQ-IDs and traceability
- `ROADMAP.md` — 5 phases with success criteria and plan breakdown
- `STATE.md` — current phase status
- `config.json` — GSD workflow settings (YOLO mode, standard granularity, parallel)
- `research/` — STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md, SUMMARY.md

## Implementation Roadmap

| Phase | Goal | Requirements |
|-------|------|-------------|
| 1 | Storage + Project CRUD + Snapshot | PROJ-01..05, SNAP-01..05 |
| 2 | /proc Scanner + inotify Monitor | MON-01..06, PROC-01..05 |
| 3 | Statistics & Analytics | STAT-01..08 |
| 4 | MCP Server + CLI | MCP-01..09, CLI-01..09 |
| 5 | Daemon & Systemd Deployment | DAEM-01..05 |

Next step: `/gsd-plan-phase 1`

## Known Constraints

- **WSL2 required** — WSL1 has poor inotify; /mnt/ paths don't support inotify
- **Resource budget**: <1% CPU idle, <10MB RAM
- **Approximate attribution only** — /proc scanning is sampling-based (2-5s intervals)
- **Release profile required** — `opt-level = "z"`, LTO, strip, `panic = "abort"`
- **Single SQLite writer** — all writes through one tokio task via mpsc channel
- **systemd must be enabled** in WSL (`/etc/wsl.conf` `[boot] systemd=true`)
