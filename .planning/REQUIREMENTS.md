# Requirements: OPENDOG

**Defined:** 2026-04-24
**Core Value:** Accurately identify which project files AI tools actually use and which are dead weight

## v1 Requirements

### Project Management (PROJ)

- [x] **PROJ-01**: User can create a project with unique ID and root directory path
- [x] **PROJ-02**: User can list all registered projects with status, root path, and database location
- [x] **PROJ-03**: User can delete a project and all its associated data (database, config)
- [x] **PROJ-04**: Each project has isolated storage — independent SQLite database file
- [x] **PROJ-05**: Each project has independent configuration (root dir, ignore patterns, process whitelist)

### File Snapshot (SNAP)

- [x] **SNAP-01**: User can trigger a full recursive file scan of a project's root directory
- [x] **SNAP-02**: Snapshot automatically filters known noise directories (node_modules, .git, dist, target, __pycache__, .cache, build)
- [x] **SNAP-03**: Snapshot records per file: path, size, modification time, file type/extension, scan timestamp
- [x] **SNAP-04**: Snapshot handles permission errors gracefully — skip inaccessible files without aborting
- [x] **SNAP-05**: Snapshot supports incremental update — add new files, remove deleted files, update changed metadata

### File Monitoring (MON)

- [x] **MON-01**: User can start monitoring for a specific project independently of other projects
- [x] **MON-02**: User can stop monitoring for a specific project without affecting others
- [x] **MON-03**: Monitor uses /proc/<pid>/fd scanning (primary) to detect which files AI processes have open, with configurable scan interval (default 2-5s)
- [x] **MON-04**: Monitor uses inotify via notify crate (secondary) for file change detection — modifications, creates, deletes in project directories
- [x] **MON-05**: Monitor cross-references /proc scan data with inotify change events by timestamp for approximate attribution
- [x] **MON-06**: Monitor handles inotify watch limit gracefully (check max_user_watches, warn if insufficient)

### AI Process Detection (PROC)

- [x] **PROC-01**: System periodically enumerates /proc entries and filters by configurable process name whitelist (claude, codex, node, python, etc.)
- [x] **PROC-02**: For each whitelisted process, system reads /proc/<pid>/fd/ directory and resolves fd symlinks to real file paths
- [x] **PROC-03**: System matches resolved file paths against project snapshot to identify which project files AI processes have open
- [x] **PROC-04**: Process whitelist is configurable per project
- [x] **PROC-05**: System records process name alongside each file sighting for auditability
- [x] **PROC-06**: Attribution is explicitly approximate — sampling-based, may miss brief accesses (< scan interval), duration is estimated from consecutive scan sightings

### Usage Statistics (STAT)

- [x] **STAT-01**: System records per-file access count (number of /proc scans where file appeared as open fd — approximate)
- [x] **STAT-02**: System records per-file estimated usage duration (sum of consecutive scan intervals where file was seen as open — approximate)
- [x] **STAT-03**: System records per-file modification count (from inotify change events, not process-attributed)
- [x] **STAT-04**: System records per-file last access timestamp (from most recent /proc scan sighting)
- [x] **STAT-05**: System marks files as "accessed" or "never accessed" relative to snapshot baseline
- [x] **STAT-06**: User can query statistics for a project — per-file access count, estimated duration, modifications, last access
- [x] **STAT-07**: User can query list of never-accessed files (unused file candidates)
- [x] **STAT-08**: User can query list of high-frequency files (core file candidates)

### MCP Server (MCP)

- [x] **MCP-01**: System exposes MCP server via rmcp crate over stdio transport
- [x] **MCP-02**: MCP tool `create_project` — register project with ID and root path
- [x] **MCP-03**: MCP tool `take_snapshot` — trigger file scan for a project
- [x] **MCP-04**: MCP tool `start_monitor` — begin file monitoring for a project
- [x] **MCP-05**: MCP tool `stop_monitor` — stop file monitoring for a project
- [x] **MCP-06**: MCP tool `get_stats` — query usage statistics for a project
- [x] **MCP-07**: MCP tool `get_unused_files` — list never-accessed files for a project
- [x] **MCP-08**: MCP tool `list_projects` — list all registered projects and status
- [x] **MCP-09**: MCP tool `delete_project` — remove a project and its data

### CLI Tool (CLI)

- [x] **CLI-01**: Binary `opendog` with 8 subcommands matching MCP tools
- [x] **CLI-02**: `opendog create --id <ID> --path <DIR>` — create project
- [x] **CLI-03**: `opendog snapshot --id <ID>` — trigger snapshot
- [x] **CLI-04**: `opendog start --id <ID>` — start monitoring
- [x] **CLI-05**: `opendog stop --id <ID>` — stop monitoring
- [x] **CLI-06**: `opendog stats --id <ID>` — show statistics
- [x] **CLI-07**: `opendog unused --id <ID>` — list unused files
- [x] **CLI-08**: `opendog list` — list projects
- [x] **CLI-09**: `opendog delete --id <ID>` — delete project

### Daemon & Deployment (DAEM)

- [x] **DAEM-01**: System runs as background daemon with <1% CPU at idle, <10MB RAM
- [x] **DAEM-02**: Systemd service unit file for auto-start and auto-restart
- [x] **DAEM-03**: Graceful shutdown on SIGTERM — flush buffered events before exit
- [x] **DAEM-04**: Structured logging via journald (tracing + tracing-journald)
- [x] **DAEM-05**: WSL environment detection — warn if WSL1 or if /mnt/ paths are configured

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
| PROJ-01 through PROJ-05 | Phase 1 | Complete |
| SNAP-01 through SNAP-05 | Phase 1 | Complete |
| MON-01 through MON-06 | Phase 2 | Complete |
| PROC-01 through PROC-06 | Phase 2 | Complete |
| STAT-01 through STAT-08 | Phase 3 | Complete |
| MCP-01 through MCP-09 | Phase 4 | Complete |
| CLI-01 through CLI-09 | Phase 4 | Complete |
| DAEM-01 through DAEM-05 | Phase 5 | Complete |

**Coverage:**
- v1 requirements: 43 total
- Mapped to phases: 43
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-24 after initial definition*
