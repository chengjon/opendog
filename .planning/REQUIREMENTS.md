# Requirements: OPENDOG

**Defined:** 2026-04-24
**Core Value:** Accurately identify which project files AI tools actually use and which are dead weight

## v1 Requirements

### Project Management (PROJ)

- [ ] **PROJ-01**: User can create a project with unique ID and root directory path
- [ ] **PROJ-02**: User can list all registered projects with status, root path, and database location
- [ ] **PROJ-03**: User can delete a project and all its associated data (database, config)
- [ ] **PROJ-04**: Each project has isolated storage — independent SQLite database file
- [ ] **PROJ-05**: Each project has independent configuration (root dir, ignore patterns, process whitelist)

### File Snapshot (SNAP)

- [ ] **SNAP-01**: User can trigger a full recursive file scan of a project's root directory
- [ ] **SNAP-02**: Snapshot automatically filters known noise directories (node_modules, .git, dist, target, __pycache__, .cache, build)
- [ ] **SNAP-03**: Snapshot records per file: path, size, modification time, file type/extension, scan timestamp
- [ ] **SNAP-04**: Snapshot handles permission errors gracefully — skip inaccessible files without aborting
- [ ] **SNAP-05**: Snapshot supports incremental update — add new files, remove deleted files, update changed metadata

### File Monitoring (MON)

- [ ] **MON-01**: User can start monitoring for a specific project independently of other projects
- [ ] **MON-02**: User can stop monitoring for a specific project without affecting others
- [ ] **MON-03**: Monitor uses Linux inotify for kernel-level, non-intrusive file event detection
- [ ] **MON-04**: Monitor recursively watches all subdirectories within the project root
- [ ] **MON-05**: Monitor automatically adds watches for newly created subdirectories
- [ ] **MON-06**: Monitor detects file events: open, close, read, modify, create, delete, move

### Process Filtering (PROC)

- [ ] **PROC-01**: Monitor filters file events by AI tool process names using a configurable whitelist
- [ ] **PROC-02**: Primary filter: check if process name matches known AI tools (claude, codex, python, node, etc.)
- [ ] **PROC-03**: Supplementary filter: check parent process chain up to 3 levels for AI-related parent processes
- [ ] **PROC-04**: Process whitelist is configurable per project
- [ ] **PROC-05**: Monitor records the process name alongside each file event for auditability

### Usage Statistics (STAT)

- [ ] **STAT-01**: System records per-file access count (each open/read event increments)
- [ ] **STAT-02**: System records per-file usage duration (time from open to close, cumulative)
- [ ] **STAT-03**: System records per-file modification count
- [ ] **STAT-04**: System records per-file last access timestamp
- [ ] **STAT-05**: System marks files as "accessed" or "never accessed" relative to snapshot baseline
- [ ] **STAT-06**: User can query statistics for a project — per-file access count, duration, modifications, last access
- [ ] **STAT-07**: User can query list of never-accessed files (unused file candidates)
- [ ] **STAT-08**: User can query list of high-frequency files (core file candidates)

### MCP Server (MCP)

- [ ] **MCP-01**: System exposes MCP server over stdio transport (JSON-RPC 2.0)
- [ ] **MCP-02**: MCP tool `create_project` — register project with ID and root path
- [ ] **MCP-03**: MCP tool `take_snapshot` — trigger file scan for a project
- [ ] **MCP-04**: MCP tool `start_monitor` — begin file monitoring for a project
- [ ] **MCP-05**: MCP tool `stop_monitor` — stop file monitoring for a project
- [ ] **MCP-06**: MCP tool `get_stats` — query usage statistics for a project
- [ ] **MCP-07**: MCP tool `get_unused_files` — list never-accessed files for a project
- [ ] **MCP-08**: MCP tool `list_projects` — list all registered projects and status
- [ ] **MCP-09**: MCP tool `delete_project` — remove a project and its data

### CLI Tool (CLI)

- [ ] **CLI-01**: Binary `opendog` with 8 subcommands matching MCP tools
- [ ] **CLI-02**: `opendog create --id <ID> --path <DIR>` — create project
- [ ] **CLI-03**: `opendog snapshot --id <ID>` — trigger snapshot
- [ ] **CLI-04**: `opendog start --id <ID>` — start monitoring
- [ ] **CLI-05**: `opendog stop --id <ID>` — stop monitoring
- [ ] **CLI-06**: `opendog stats --id <ID>` — show statistics
- [ ] **CLI-07**: `opendog unused --id <ID>` — list unused files
- [ ] **CLI-08**: `opendog list` — list projects
- [ ] **CLI-09**: `opendog delete --id <ID>` — delete project

### Daemon & Deployment (DAEM)

- [ ] **DAEM-01**: System runs as background daemon with <1% CPU at idle, <10MB RAM
- [ ] **DAEM-02**: Systemd service unit file for auto-start and auto-restart
- [ ] **DAEM-03**: Graceful shutdown on SIGTERM — flush buffered events before exit
- [ ] **DAEM-04**: Structured logging via journald (tracing + tracing-journald)
- [ ] **DAEM-05**: WSL environment detection — warn if WSL1 or if /mnt/ paths are configured

## v2 Requirements

### Data Export

- **EXPORT-01**: User can export project statistics to CSV format
- **EXPORT-02**: User can export project statistics to JSON format

### Enhanced Reporting

- **RPT-01**: User can view time-windowed statistics (last 24h, 7d, 30d)
- **RPT-02**: User can compare two snapshots to identify file changes
- **RPT-03**: Trend analysis — file usage over time

### Configuration Management

- **CONF-01**: Per-project ignore pattern management via CLI/MCP
- **CONF-02**: Global default configuration file
- **CONF-03**: Hot-reload configuration without restart

## Out of Scope

| Feature | Reason |
|---------|--------|
| Auto-cleanup of unused files | Safety — only identify, never delete. User makes the decision. |
| Web dashboard / visual UI | Terminal-first for v1. MCP + CLI sufficient. |
| Real-time streaming to external services | Local SQLite only. No network dependencies. |
| Network/remote filesystem monitoring | WSL local filesystem only. inotify doesn't work over network. |
| Windows native support | WSL-only. Linux inotify required. |
| Cross-platform support (macOS, Windows) | WSL+Linux scope only. |
| File content analysis | Track access patterns, not content. Privacy-preserving. |
| AI tool orchestration | Observe only, never control AI tools. |
| Plugin system | Premature for v1. YAGNI. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROJ-01 through PROJ-05 | Phase 1 | Pending |
| SNAP-01 through SNAP-05 | Phase 1 | Pending |
| MON-01 through MON-06 | Phase 2 | Pending |
| PROC-01 through PROC-05 | Phase 2 | Pending |
| STAT-01 through STAT-08 | Phase 3 | Pending |
| MCP-01 through MCP-09 | Phase 4 | Pending |
| CLI-01 through CLI-09 | Phase 4 | Pending |
| DAEM-01 through DAEM-05 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 42 total
- Mapped to phases: 42
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-24 after initial definition*
