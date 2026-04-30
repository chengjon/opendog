# Requirements: OPENDOG

**Defined:** 2026-04-24
**Core Value:** Accurately identify which project files AI tools actually use and which are dead weight

## Function Tree Anchor

Canonical capability hierarchy now lives in `.planning/FUNCTION_TREE.md`.

Interpretation rule:

- `PROJECT.md` defines intent
- `FUNCTION_TREE.md` defines capability ownership
- `REQUIREMENTS.md` defines detailed requirement statements
- `ROADMAP.md` and future task cards define execution against those capabilities

Adoption rule:

- new requirements should map to at least one `FT-*` leaf node
- current requirement families now also carry inline `Maps to FT:` ownership
- `FUNCTION_TREE.md` remains the canonical cross-section summary even after inline mapping coverage was added
- task cards should adopt `FT-*` mapping immediately via `.planning/TASK_CARD_TEMPLATE.md`
- future requirement edits should preserve or refine their inline `Maps to FT:` ownership instead of letting drift reappear

## v1 Requirements

### Project Management (PROJ)

Maps to FT: `FT-01.01.01`, `FT-01.01.02`

- [x] **PROJ-01**: User can create a project with unique ID and root directory path
- [x] **PROJ-02**: User can list all registered projects with status, root path, and database location
- [x] **PROJ-03**: User can delete a project and all its associated data (database, config)
- [x] **PROJ-04**: Each project has isolated storage — independent SQLite database file
- [x] **PROJ-05**: Each project has independent configuration (root dir, ignore patterns, process whitelist)

### File Snapshot (SNAP)

Maps to FT: `FT-01.02.01`, `FT-01.02.02`

- [x] **SNAP-01**: User can trigger a full recursive file scan of a project's root directory
- [x] **SNAP-02**: Snapshot automatically filters known noise directories (node_modules, .git, dist, target, __pycache__, .cache, build)
- [x] **SNAP-03**: Snapshot records per file: path, size, modification time, file type/extension, scan timestamp
- [x] **SNAP-04**: Snapshot handles permission errors gracefully — skip inaccessible files without aborting
- [x] **SNAP-05**: Snapshot supports incremental update — add new files, remove deleted files, update changed metadata

### File Monitoring (MON)

Maps to FT: `FT-01.03.01`, `FT-01.03.02`

- [x] **MON-01**: User can start monitoring for a specific project independently of other projects
- [x] **MON-02**: User can stop monitoring for a specific project without affecting others
- [x] **MON-03**: Monitor uses /proc/<pid>/fd scanning (primary) to detect which files AI processes have open, with configurable scan interval (default 2-5s)
- [x] **MON-04**: Monitor uses inotify via notify crate (secondary) for file change detection — modifications, creates, deletes in project directories
- [x] **MON-05**: Monitor cross-references /proc scan data with inotify change events by timestamp for approximate attribution
- [x] **MON-06**: Monitor handles inotify watch limit gracefully (check max_user_watches, warn if insufficient)

### AI Process Detection (PROC)

Maps to FT: `FT-01.03.01`, `FT-01.03.02`

- [x] **PROC-01**: System periodically enumerates /proc entries and filters by configurable process name whitelist (claude, codex, node, python, etc.)
- [x] **PROC-02**: For each whitelisted process, system reads /proc/<pid>/fd/ directory and resolves fd symlinks to real file paths
- [x] **PROC-03**: System matches resolved file paths against project snapshot to identify which project files AI processes have open
- [x] **PROC-04**: Process whitelist is configurable per project
- [x] **PROC-05**: System records process name alongside each file sighting for auditability
- [x] **PROC-06**: Attribution is explicitly approximate — sampling-based, may miss brief accesses (< scan interval), duration is estimated from consecutive scan sightings

### Usage Statistics (STAT)

Maps to FT: `FT-01.04.01`, `FT-01.04.02`

- [x] **STAT-01**: System records per-file access count (number of /proc scans where file appeared as open fd — approximate)
- [x] **STAT-02**: System records per-file estimated usage duration (sum of consecutive scan intervals where file was seen as open — approximate)
- [x] **STAT-03**: System records per-file modification count (from inotify change events, not process-attributed)
- [x] **STAT-04**: System records per-file last access timestamp (from most recent /proc scan sighting)
- [x] **STAT-05**: System marks files as "accessed" or "never accessed" relative to snapshot baseline
- [x] **STAT-06**: User can query statistics for a project — per-file access count, estimated duration, modifications, last access
- [x] **STAT-07**: User can query list of never-accessed files (unused file candidates)
- [x] **STAT-08**: User can query list of high-frequency files (core file candidates)

### MCP Server (MCP)

Maps to FT: `FT-02.02.01`

Note: `MCP-01..09` describe the original baseline control surface. Later tools for reporting, config, export, cleanup, verification, guidance, and data-risk are intentionally tracked under their own requirement families instead of renumbering this baseline block.

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

Maps to FT: `FT-02.01.01`

Note: `CLI-01..09` describe the original baseline command surface that mirrored the initial MCP control tools. The current CLI is larger; later commands are tracked under their own requirement families instead of mutating this baseline block.

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

Maps to FT: `FT-02.03.01`

- [x] **DAEM-01**: System runs as background daemon with <1% CPU at idle, <10MB RAM
- [x] **DAEM-02**: Systemd service unit file for auto-start and auto-restart
- [x] **DAEM-03**: Graceful shutdown on SIGTERM — flush buffered events before exit
- [x] **DAEM-04**: Structured logging via journald (tracing + tracing-journald)
- [x] **DAEM-05**: WSL environment detection — warn if WSL1 or if /mnt/ paths are configured

## v2 Requirements

### Phase 6 Implementation Snapshot

- Guidance schema skeleton is implemented and returned through MCP with eight reusable layer slots
- Repository risk, verification evidence, and AI execution strategy are no longer hypothetical; they already have live MCP output paths
- Project-level and workspace-level mock/hardcoded data-risk detection are implemented in both MCP and CLI surfaces
- Local daemon control-plane coordination is implemented so CLI and MCP can reuse daemon-owned monitor state instead of silently diverging
- Retained-evidence cleanup and storage-maintenance signaling are implemented as shipped operational capabilities
- These requirements remain listed as v2 because coverage is still being expanded and refined; this section should not be read as “not started”

### Workspace Observation Layer (OBS)

Maps to FT: `FT-03.01.01`

- **OBS-01**: MCP can summarize per-project observation state including snapshot availability, snapshot freshness, monitor state, and whether activity data exists
- **OBS-02**: MCP can report whether OPENDOG currently has enough evidence to support cleanup, review, or hotspot conclusions for a project
- **OBS-03**: MCP can expose observation gaps such as no snapshot, no monitor, stale snapshot, or insufficient activity window
- **OBS-04**: Observation output is structured for machine consumption and reusable across different target projects

### Repository Status & Risk Summary (RISK)

Maps to FT: `FT-03.02.01`

- **RISK-01**: MCP can produce a repository status summary that combines OPENDOG activity signals with repository state signals available to the agent
- **RISK-02**: MCP can highlight likely risk areas such as mixed hot-and-unused patterns, unverified cleanup candidates, stale observation baselines, or low-confidence conclusions
- **RISK-03**: MCP can label findings by severity, priority, or confidence so an AI can triage what needs attention first
- **RISK-04**: Risk summaries remain advisory and evidence-backed; they do not auto-modify the repository

### AI Execution Strategy Suggestions (STRAT)

Maps to FT: `FT-03.02.02`

- **STRAT-01**: MCP can tell an AI when OPENDOG tools are the right choice versus shell commands such as `git status`, `git diff`, `cargo test`, `npm test`, or `pytest`
- **STRAT-02**: MCP can return project-level recommended next actions based on snapshot state, monitor state, recent activity, and candidate findings
- **STRAT-03**: MCP can suggest repository-appropriate follow-up commands based on detected stack, workspace conventions, or current evidence gaps
- **STRAT-04**: Strategy suggestions are explicit about why an action is recommended and what evidence supports it

### Verification & Evidence Layer (EVID)

Maps to FT: `FT-03.03.01`

- **EVID-01**: MCP can attach concise evidence to major recommendations, including source signal, file path, and detection basis
- **EVID-02**: MCP can distinguish direct observations from inferred conclusions so AI does not overstate certainty
- **EVID-03**: MCP can expose confidence, freshness, or coverage metadata for major summaries and findings
- **EVID-04**: Evidence output is designed for later user review, AI verification loops, and auditability

### Multi-Project Portfolio View (PORT)

Maps to FT: `FT-03.04.01`

- **PORT-01**: MCP can summarize cross-project state for AI use, including registered projects, active monitors, snapshot availability, and whether activity data exists
- **PORT-02**: MCP can compare projects by monitoring health, evidence quality, cleanup readiness, or review priority
- **PORT-03**: MCP can surface which project currently deserves attention first and why
- **PORT-04**: Portfolio views avoid leaking project-internal data between projects except through intended top-level summary fields

### Cleanup & Refactor Candidate Layer (CLEAN)

Maps to FT: `FT-03.05.01`

- **CLEAN-01**: MCP can return file-level recommendations for hotspots, unused candidates, suspicious mixed files, and review targets with concise rationale
- **CLEAN-02**: MCP can expose reusable cleanup signals that are not worth each target project implementing itself, such as snapshot-derived unused candidates and activity-derived review focus
- **CLEAN-03**: Cleanup and refactor guidance remains read-only by default; it recommends actions but does not auto-modify or auto-delete project files
- **CLEAN-04**: Candidate outputs can be consumed by users or AI agents for later confirmation, cleanup planning, or refactoring workflows

### Project Type & Toolchain Identification (STACKX)

Maps to FT: `FT-03.06.01`

- **STACKX-01**: MCP can infer likely project type or toolchain from repository markers and workspace conventions
- **STACKX-02**: MCP can recommend verification commands aligned with detected stack, such as Rust, Node, Python, or mixed repositories
- **STACKX-03**: Toolchain detection includes confidence or fallback behavior when repository signals are ambiguous
- **STACKX-04**: Toolchain guidance is reusable across multiple projects and not hardcoded to a single target repository

### Constraints & Boundary Layer (BOUND)

Maps to FT: `FT-03.07.01`

- **BOUND-01**: MCP can explicitly state what OPENDOG directly observed versus what it only inferred
- **BOUND-02**: MCP can explicitly state major blind spots such as brief accesses below the scan interval, no monitor running, or lack of repository metadata
- **BOUND-03**: MCP can tell an AI when OPENDOG should not be treated as authoritative for a decision and when additional shell or human verification is needed
- **BOUND-04**: Boundary information is returned alongside guidance so downstream AI behavior stays constrained by evidence quality

### MOCK / Hardcoded Data Detection (MOCK)

Maps to FT: `FT-03.08.01`, `FT-03.08.02`

- **MOCK-01**: System can identify mock, stub, fake, demo, fixture, sample, and test data artifacts or references in project files
- **MOCK-02**: System can detect suspected hardcoded pseudo-business data embedded directly in source files or configuration files
- **MOCK-03**: System outputs files containing explicit MOCK data as a separate result set for user or AI review
- **MOCK-04**: System outputs files containing suspected hardcoded business-like pseudo-data as a separately marked result set
- **MOCK-05**: Each finding includes file path, hit category, detection basis, and enough local context for follow-up inspection
- **MOCK-06**: Findings are designed to support later confirmation, cleanup planning, and risk analysis by a user or AI agent
- **MOCK-07**: Detection is read-only by default; the system records and marks findings but does not delete or rewrite files automatically
- **MOCK-08**: System distinguishes test-only mock data from more risky pseudo-data present in production or shared runtime paths
- **MOCK-09**: System flags mixed files that contain both business logic and mock or hardcoded pseudo-data, because these files need extra review
- **MOCK-10**: Detection output is structured and reusable across projects for governance, auditing, and AI-assisted refactoring workflows

### Data Export

Maps to FT: `FT-01.04.03`

- **EXPORT-01**: User can export project statistics to CSV format
- **EXPORT-02**: User can export project statistics to JSON format

### Enhanced Reporting

Maps to FT: `FT-01.04.04`

- **RPT-01**: User can view time-windowed statistics (last 24h, 7d, 30d)
- **RPT-02**: User can compare two snapshots to identify file changes
- **RPT-03**: Trend analysis — file usage over time

### Configuration Management

Maps to FT: `FT-01.01.03`

- **CONF-01**: Per-project ignore pattern management via CLI/MCP
- **CONF-02**: Global default configuration file
- **CONF-03**: Hot-reload configuration without restart

### Local Control Plane & Runtime Coordination (CTRL)

Maps to FT: `FT-02.03.02`

- **CTRL-01**: CLI and MCP prefer daemon-owned monitor and project state through the local control plane when the daemon is available
- **CTRL-02**: The local control plane supports project operations beyond start/stop, including snapshot, reports, guidance, verification, cleanup, and configuration reload
- **CTRL-03**: When daemon coordination is unavailable, the system returns explicit remediation guidance and only falls back to local execution where behavior remains safe and consistent
- **CTRL-04**: The control plane prevents duplicate monitor ownership and reduces state drift between daemon, CLI, and MCP entrypoints
- **CTRL-05**: Control-plane project operations preserve per-project isolation and return result shapes consistent with local in-process execution

### Retained Evidence Lifecycle & Storage Hygiene (RET)

Maps to FT: `FT-01.04.05`

- **RET-01**: User or AI can selectively clean retained OPENDOG evidence by scope: `activity`, `snapshots`, `verification`, or `all`
- **RET-02**: Cleanup supports dry-run preview so users can inspect deletion counts and storage impact before mutating retained evidence
- **RET-03**: Activity cleanup prunes raw sightings and file events without deleting aggregate `file_stats` evidence
- **RET-04**: Snapshot cleanup prunes historical snapshot runs and snapshot history without deleting the current snapshot baseline
- **RET-05**: Cleanup returns storage metrics before and after mutation, including approximate database size and reclaimable space
- **RET-06**: Storage maintenance actions such as `PRAGMA optimize` and `VACUUM` remain explicit, auditable, and never touch source files

## Out of Scope

| Feature | Reason |
|---------|--------|
| Auto-cleanup of unused files | Safety — only identify, never delete. User makes the decision. |
| Web dashboard / visual UI | Terminal-first for v1. MCP + CLI sufficient. |
| Real-time streaming to external services | Local SQLite only. No network dependencies. |
| Network/remote filesystem monitoring | WSL local filesystem only. inotify doesn't work over network. |
| Windows native support | WSL-only. Linux inotify required. |
| Cross-platform support (macOS, Windows) | WSL+Linux scope only. |
| Broad file content analysis | v1 tracks access patterns, not deep semantics. Later milestones may add targeted heuristics such as MOCK-data detection. |
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
| OBS-01 through OBS-04 | Phase 6 | Planned |
| RISK-01 through RISK-04 | Phase 6 | Planned |
| STRAT-01 through STRAT-04 | Phase 6 | Planned |
| EVID-01 through EVID-04 | Phase 6 | Planned |
| PORT-01 through PORT-04 | Phase 6 | Planned |
| CLEAN-01 through CLEAN-04 | Phase 6 | Planned |
| STACKX-01 through STACKX-04 | Phase 6 | Planned |
| BOUND-01 through BOUND-04 | Phase 6 | Planned |
| MOCK-01 through MOCK-10 | Phase 6 | Planned |
| EXPORT-01 through EXPORT-02 | Phase 3/4 Hardening | Complete |
| RPT-01 through RPT-03 | Phase 3/4/5 Hardening | Complete |
| CONF-01 through CONF-03 | Phase 4/5 Hardening | Complete |
| CTRL-01 through CTRL-05 | Phase 5 Hardening | Complete |
| RET-01 through RET-06 | Phase 6 Hardening | Complete |

**Coverage:**
- v1 requirements: 53 total
- v2+ requirements drafted: 61
- Mapped to FT leaves: 114
- Mapped to phases: 114
- Backlog / unscheduled: 0
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-27 after formalizing control-plane coordination and retained-evidence cleanup as first-class requirement families*
