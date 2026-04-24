# Features Research: OPENDOG

Research Date: 2026-04-24
Sources: Watchman docs, inotify-tools GitHub, entr, fswatch, MCP specification, MCP reference servers, file monitoring ecosystem analysis

---

## Table Stakes Features

These are features users expect from any file monitoring tool. Without them, users will look elsewhere.

### TS-01: Recursive Directory Watching

**What**: Monitor entire directory trees, not just single directories.
**Evidence**: Every major tool (Watchman, inotify-tools, entr, fswatch, notify crate) supports recursive watching as the primary use case. inotify requires manual per-directory watch addition; higher-level tools wrap this automatically.
**OPENDOG implication**: The snapshot engine must walk the full tree. The monitor engine must add watches recursively for every subdirectory. New subdirectories created during monitoring must be automatically watched.

### TS-02: Smart Filtering / Ignore Patterns

**What**: Skip directories and files that should not be monitored (node_modules, .git, dist, __pycache__, target, .cache, build artifacts).
**Evidence**: Watchman uses `.watchmanconfig` with ignore patterns. entr takes an explicit file list via stdin (opt-in model). inotify-tools has `--exclude` patterns. Every production watcher must filter or it drowns in noise from dependency directories.
**OPENDOG implication**: Configurable ignore patterns per project. Sensible defaults covering common dependency/cache/build directories across JavaScript, Python, Rust, and Go projects.

### TS-03: Observation Types (Create, Modify, Delete, Move, Open-File Sighting)

**What**: Detect and distinguish between file creation, modification, deletion, rename/move, and whether an AI process currently has a file open.
**Evidence**: inotify provides create/modify/delete/move signals but no process identity. `/proc/<pid>/fd` enumeration provides open-file visibility for specific processes. Together they cover the required observation space.
**OPENDOG implication**: Use inotify/notify for create/modify/delete/move. Use periodic `/proc/<pid>/fd` scanning for open-file sightings. Treat access/duration as approximate metrics derived from repeated sightings, not exact event pairs.

### TS-04: Resource Budget Compliance

**What**: The monitoring system itself must not degrade system performance. Target: low CPU when idle, bounded memory.
**Evidence**: entr is designed for "zero overhead when idle." Watchman consolidates overlapping watches and reaps idle ones after 5 days. The Rust notify crate uses a single thread per backend. Any monitoring tool that pegs CPU or leaks memory gets uninstalled immediately.
**OPENDOG implication**: Hard constraints of <1% CPU idle, <10MB RAM. Event batching and debouncing to avoid CPU spikes. Memory-bounded event queues. Per-project isolation prevents one project from starving others.

### TS-05: Persistent Storage

**What**: Monitoring data must survive restarts. No data loss on daemon crash or system reboot.
**Evidence**: Watchman persists state files per watch. inotifywatch accumulates statistics for a session but loses them on exit (a limitation users work around). OPENDOG's SQLite approach directly addresses this gap.
**OPENDOG implication**: SQLite with WAL mode for crash resilience. Automatic schema migration on version upgrade. Database integrity checks on startup.

### TS-06: Start/Stop Control

**What**: Users must be able to start, stop, and query monitoring status per project.
**Evidence**: Watchman has watch/watch-project, clock, since queries. inotifywait has --timeout. entr stops when the child process exits. All tools provide lifecycle control.
**OPENDOG implication**: MCP tools start_monitor/stop_monitor + CLI equivalents. Status tracking per project (active/stopped/error). Clean resource release on stop (close inotify fd, release watches).

### TS-07: CLI Interface

**What**: Command-line tool for manual operation, scripting, and automation.
**Evidence**: Every file monitoring tool is CLI-first. inotifywait/inotifywatch are pure CLI. entr is CLI-only. Watchman has `watchman` CLI. fswatch provides `fswatch` CLI.
**OPENDOG implication**: CLI binary with 8 commands matching MCP tools. Must work standalone without daemon running (for some operations like list/snapshot). Help text, version flag, config file paths.

### TS-08: Basic Reporting / Statistics

**What**: Show what files were accessed, how often, and when.
**Evidence**: inotifywatch generates aggregate filesystem statistics. Watchman supports since/clock queries for change sets. entr reports exit status of the child command.
**OPENDOG implication**: get_stats MCP tool returning per-file access counts, estimated duration, modification counts, last access time. get_unused_files returning zero-access files. CLI stats command with formatted output.

### TS-09: Non-Intrusive Operation

**What**: Monitoring must not interfere with the watched processes or filesystem operations.
**Evidence**: inotify is kernel-level and read-only (no filesystem modification). entr uses kqueue/inotify without interception. Watchman reads filesystem metadata. This is table stakes for any monitoring tool.
**OPENDOG implication**: inotify provides this natively. OPENDOG must never lock files, inject metadata, or require filesystem modifications. Read-only access to watched directories.

---

## Differentiating Features

These features set OPENDOG apart from existing file monitoring tools. No current tool combines all of these.

### DIFF-01: AI Process File Tracking via /proc Scanning

**What**: Identify which files AI tools (Claude Code, Codex, GPT, GLM) have open by periodically scanning /proc/<pid>/fd for whitelisted processes. Cross-reference with inotify change detection for approximate modification attribution.
**Why differentiating**: No existing file monitor tracks per-process file usage. The approach is honest about being statistical sampling (not precise auditing) — 2-5s scan intervals capture sustained file usage but may miss very brief accesses.
**Evidence**: inotify(7) explicitly states it provides NO process attribution data. Tools like `lsof` and `fuser` use /proc scanning for exactly this purpose — finding which processes have which files open. OPENDOG automates this at scale for AI development workflows.
**OPENDOG implication**: Primary tracking via periodic /proc scanning (what files AI processes have open). Secondary change detection via inotify/notify (what files changed). Timestamp-based approximate attribution. Configurable whitelist: ["claude", "codex", "python", "node"]. This is a **derived design decision** — the original README assumed inotify could provide PID info, but it cannot.

### DIFF-02: MCP Server Integration

**What**: Expose all monitoring operations as MCP tools over stdio transport, enabling AI tools to directly query and control monitoring.
**Why differentiating**: No existing file monitor provides an MCP interface. MCP is the emerging standard for AI tool integration (used by Claude Code, Codex, and other AI assistants). This makes OPENDOG the only file monitor that AI tools can self-manage.
**Evidence**: MCP reference servers (filesystem, git, memory) demonstrate the pattern. MCP SDKs exist for Rust. stdio transport is the standard for CLI-integrated servers.
**OPENDOG implication**: 8 MCP tools with JSON Schema input validation. Proper MCP error handling (isError field, not protocol errors). Tool annotations (readOnlyHint for get_stats/get_unused_files/list_projects, destructiveHint for delete_project). Capability advertisement on initialize.

### DIFF-03: Approximate Usage Duration Tracking

**What**: Estimate how long files remain open in AI processes, not just whether they were observed.
**Why differentiating**: Most file monitors report changes, not sustained attention. OPENDOG derives a useful "attention time" metric from consecutive `/proc` scan sightings, which is enough to rank core files even though it is approximate.
**Evidence**: Tools like `lsof`/`fuser` show point-in-time open files, but do not accumulate historical duration metrics for AI workflows. Repeated sampling makes that possible.
**OPENDOG implication**: Maintain per-file/per-process open-state across scans. When a file first appears, mark it opened; while it continues appearing, accumulate elapsed scan intervals; when it disappears, close the interval. Document that brief accesses may be missed.

### DIFF-04: Unused File Detection

**What**: Cross-reference file snapshot (all files that exist) against access statistics to identify files never touched by AI tools.
**Why differentiating**: This is OPENDOG's core value proposition. No file monitor does this. The closest analog is `git clean --dry-run` or `fd -e unused`, but those operate on git tracking status, not access patterns.
**Evidence**: The user's original requirement explicitly states this: "I don't know which files are garbage/outdated/useful." The snapshot-vs-stats comparison is unique.
**OPENDOG implication**: Snapshot engine records all files. Monitor engine records accessed files. Comparison = snapshot - accessed = unused candidates. Must handle files created after snapshot (not unused, just new). Configurable threshold: "unused for N days" vs "never accessed."

### DIFF-05: Core File Identification

**What**: Identify files that receive disproportionate AI attention (high access frequency, long duration) as "core" files.
**Why differentiating**: Complements unused detection. Helps developers understand which files matter most to their AI workflow. No existing tool provides this.
**OPENDOG implication**: Scoring formula combining access count, total duration, modification count. Configurable thresholds or percentile-based ranking. Output distinguishes "core," "moderately used," and "unused" tiers.

### DIFF-06: Multi-Project Isolation

**What**: Each monitored project has its own database, configuration, monitoring thread, and namespace. Projects are fully independent.
**Why differentiating**: Watchman supports project-based watches but shares state globally. Most monitoring tools are single-project. OPENDOG's per-project SQLite isolation means zero cross-contamination and independent lifecycle management.
**Evidence**: Watchman uses `.watchmanconfig` for project roots but stores metadata centrally. inotify-tools is inherently single-watch.
**OPENDOG implication**: ProjectManager struct maintaining a HashMap<ProjectId, Project>. Each Project owns its SQLite connection pool, inotify watch set, config, and state. Add/remove projects without affecting others.

### DIFF-07: Dual Interface (MCP + CLI)

**What**: Both an MCP server for AI tool integration and a CLI for human operation, backed by the same core engine.
**Why differentiating**: Most tools are either CLI-only (entr, inotify-tools) or service-only (Watchman). Having both interfaces share the same backend means MCP-driven automation and CLI-driven manual inspection see identical data.
**OPENDOG implication**: Single binary with two modes: `opendog daemon` (starts MCP server) and `opendog <command>` (CLI that communicates with daemon via Unix socket or shared database).

### DIFF-08: Data Export Formats

**What**: Export monitoring data in structured formats for external analysis (JSON, CSV).
**Why differentiating**: Enables integration with other tools (spreadsheets, custom analysis scripts, visualization). inotifywatch has basic CSV output. Watchman outputs JSON. OPENDOG can provide richer structured data combining snapshots + stats.
**OPENDOG implication**: get_stats returns JSON via MCP. CLI stats command supports --format=json|csv|table. Snapshot data exportable as JSON. Unused file list exportable as CSV for batch processing.

---

## Anti-Features (Not in v1, per README scope)

Features excluded from v1 scope. README lists some of these as future expansion directions.

### ANTI-01: Auto-Cleanup / File Deletion

**Decision**: OPENDOG identifies unused files but never deletes them.
**Reason**: File deletion is destructive and irreversible. False positives in unused-file detection could delete important configuration, rarely-used but critical assets, or generated-but-necessary files. The cost of a wrong deletion far exceeds the cost of manual review. Users should review the list and decide.
**Alternative**: Export unused file list as CSV/JSON. User can pipe to custom cleanup scripts if desired.

### ANTI-02: Web Dashboard / Visual UI

**Decision**: Terminal-only for v1. No web UI, no TUI dashboard.
**Reason**: A web UI requires an HTTP server, frontend code, websocket/SSE transport, and significantly increases attack surface and resource usage. This conflicts with the resource budget (<10MB RAM, <1% CPU). Terminal output via CLI is sufficient for the target audience (developers comfortable with command-line tools).
**Alternative**: Rich CLI output with formatted tables. JSON export for users who want visualization (they can pipe to jq, csvkit, or custom scripts).

### ANTI-03: Real-Time Streaming / Push Notifications

**Decision**: Query-based access only. No websocket, no SSE, no push notifications.
**Reason**: Streaming requires persistent connections, a transport layer beyond stdio, and significantly more complex state management. MCP's request-response model is simpler and sufficient for periodic status checks. The AI tool can poll get_stats at whatever interval suits it.
**Alternative**: MCP tools return current state on demand. CLI provides ad-hoc queries. If streaming is needed in future, it can be added as an optional SSE transport without changing the core.

### ANTI-04: Network / Remote Monitoring

**Decision**: Local filesystem only. No remote monitoring, no SSH tunneling, no network protocols.
**Reason**: Network monitoring introduces latency, authentication complexity, security concerns, and deployment complexity. The target use case is a developer's local WSL environment. Adding network capability would triple the codebase size and introduce failure modes unrelated to the core monitoring function.
**Alternative**: Monitor only local WSL paths. If remote monitoring is needed, run OPENDOG on the remote machine.

### ANTI-05: Cross-Platform Support (Non-Linux)

**Decision**: Linux/WSL only. No macOS (FSEvents/kqueue), no Windows (ReadDirectoryChangesW), no BSD.
**Reason**: Cross-platform file monitoring requires abstracting over fundamentally different kernel APIs (inotify vs kqueue vs FSEvents vs ReadDirectoryChangesW). Each has different event semantics, limitations, and edge cases. Supporting multiple platforms would dilute testing effort and hide platform-specific bugs. The target environment is WSL.
**Alternative**: The Rust notify crate provides cross-platform abstraction. If cross-platform is needed later, the inotify-specific code can be behind a trait and replaced with notify's cross-platform backend.

### ANTI-06: Content Analysis / File Diffing

**Decision**: Monitor file metadata and access patterns only. No content analysis, no diff computation, no content-based search.
**Reason**: Content analysis is a different product category (code search, static analysis). Reading file contents would violate the non-intrusive constraint and increase I/O load significantly. OPENDOG's value is in access pattern analysis, not content understanding.
**Alternative**: Track file size changes and modification timestamps. If users need content analysis, they can combine OPENDOG's file lists with dedicated tools (ripgrep, git diff, etc.).

### ANTI-07: AI Tool Orchestration / Task Management

**Decision**: OPENDOG observes AI tools but does not control them.
**Reason**: Orchestration is a separate concern. OPENDOG's role is passive observation. Controlling AI tools would require tool-specific APIs, create tight coupling, and add failure modes. Keeping observation and control separate maintains OPENDOG's simplicity and reliability.
**Alternative**: OPENDOG provides data. AI tools or users make decisions based on that data.

### ANTI-08: Embedded Database Server / Multi-Process Writes

**Decision**: Single daemon process owns all databases. No concurrent multi-process writes.
**Reason**: SQLite is designed for single-writer access. Supporting multi-process writes would require WAL mode tuning, lock management, or migrating to a client-server database. The single-daemon model is simpler and matches the systemd service architecture.
**Alternative**: CLI communicates with daemon via shared state (Unix socket or direct DB reads in read-only mode). All writes go through the single daemon process.

### ANTI-09: Plugin / Extension System

**Decision**: No plugin architecture, no Lua/Python scripting, no custom analysis pipelines.
**Reason**: Plugins add complexity, instability risk, and a maintenance burden for API compatibility. The MCP interface itself serves as the extensibility point -- external tools can query OPENDOG and run custom analysis on the data.
**Alternative**: Export data in structured formats. Users can build external tools that consume OPENDOG's output.

---

## Feature Dependencies

Which features depend on which others. This determines build order and integration points.

### Dependency Graph (text representation)

```
TS-09 (Non-Intrusive)
  |
  v
TS-01 (Recursive Watching) -----> TS-02 (Smart Filtering)
  |                                      |
  v                                      v
DIFF-06 (Multi-Project Isolation) --> DIFF-01 (AI Process ID)
  |                                      |
  v                                      v
TS-05 (Persistent Storage/SQLite) --> DIFF-03 (Duration Tracking)
  |                                      |
  v                                      v
TS-03 (File Event Types) ---------> DIFF-04 (Unused Detection)
  |                                      |
  v                                      v
TS-04 (Resource Budget) ---------> TS-08 (Basic Reporting)
  |                                      |
  v                                      v
TS-06 (Start/Stop Control) ------> DIFF-05 (Core File ID)
                                         |
                                         v
TS-07 (CLI) ---------> DIFF-07 (Dual Interface) <-- DIFF-02 (MCP Server)
                                |
                                v
                          DIFF-08 (Data Export)
```

### Detailed Dependencies

| Feature | Depends On | Reason |
|---------|-----------|--------|
| TS-01 Recursive Watching | TS-09 Non-Intrusive | Must not interfere while watching |
| TS-02 Smart Filtering | TS-01 Recursive Watching | Filtering applies to watched tree |
| DIFF-06 Multi-Project | TS-01, TS-02 | Each project needs its own watch set with its own filters |
| TS-05 Persistent Storage | DIFF-06 | Per-project SQLite databases |
| DIFF-01 AI Process ID | DIFF-06 | Process filtering is per-project (each project may watch different process sets) |
| TS-03 File Events | TS-05 | Events need to be persisted to database |
| DIFF-03 Duration Tracking | TS-03, TS-05 | Requires repeated `/proc` open-file sightings + persistence |
| DIFF-04 Unused Detection | TS-03, TS-05, DIFF-06 | Snapshot minus access stats comparison, stored per project |
| DIFF-05 Core File ID | DIFF-04 | Builds on the same stats data as unused detection |
| TS-04 Resource Budget | DIFF-06 | Resource limits must be enforced across all projects |
| TS-08 Basic Reporting | DIFF-04, DIFF-05 | Reports are generated from unused/core analysis |
| TS-06 Start/Stop Control | DIFF-06, TS-04 | Per-project lifecycle control within resource budget |
| TS-07 CLI | TS-05 | CLI reads from database |
| DIFF-02 MCP Server | TS-06, TS-08 | MCP tools wrap control and reporting functions |
| DIFF-07 Dual Interface | TS-07, DIFF-02 | Both CLI and MCP share the same core |
| DIFF-08 Data Export | TS-08, DIFF-07 | Export is an output format of reporting, available via both interfaces |

### Critical Path

The longest dependency chain determines the minimum build sequence:

1. **TS-09** Non-Intrusive (foundation -- inotify provides this natively)
2. **TS-01** Recursive Watching (inotify wrapper with automatic subdirectory tracking)
3. **TS-02** Smart Filtering (ignore patterns applied during watch setup)
4. **DIFF-06** Multi-Project Isolation (project manager with per-project watch sets)
5. **TS-05** Persistent Storage (SQLite schema, per-project database files)
6. **DIFF-01** AI Process Identification (`/proc` scanning + whitelist)
7. **TS-03** Observation Pipeline (inotify change events + `/proc` file sightings)
8. **DIFF-03** Duration Tracking (consecutive scan sightings)
9. **DIFF-04** Unused File Detection (snapshot vs stats comparison)
10. **TS-08** Basic Reporting (query and format stats)
11. **DIFF-05** Core File Identification (scoring from stats)
12. **TS-06** Start/Stop Control (lifecycle management)
13. **TS-07** CLI (command-line interface)
14. **DIFF-02** MCP Server (stdio transport + tool definitions)
15. **DIFF-07** Dual Interface (unified binary)
16. **DIFF-08** Data Export (structured output formats)

### Parallelization Opportunities

Several features can be built in parallel once their dependencies are met:

- **Batch A** (after DIFF-06): DIFF-01 (process ID), TS-05 (storage) -- independent
- **Batch B** (after DIFF-01 + TS-05): TS-03 (events), DIFF-03 (duration) -- DIFF-03 depends on TS-03, so sequential within batch
- **Batch C** (after DIFF-04): TS-08 (reporting), DIFF-05 (core files) -- independent
- **Batch D** (after TS-08 + TS-06): TS-07 (CLI), DIFF-02 (MCP) -- independent interfaces
- **Batch E** (after TS-07 + DIFF-02): DIFF-07 (dual interface), DIFF-08 (export)

---

## Competitive Positioning

### What exists today

| Tool | Monitoring | Process Filter | MCP | Duration | Multi-Project | Stats |
|------|-----------|---------------|-----|----------|---------------|-------|
| Watchman | Yes | No | No | No | Partial | No |
| inotify-tools | Yes | No | No | No | No | Basic |
| entr | Yes | By command | No | No | No | No |
| fswatch | Yes | No | No | No | No | No |
| Rust notify crate | Yes | No | No | No | No | No |
| **OPENDOG** | **Yes** | **Yes** | **Yes** | **Yes** | **Yes** | **Yes** |

### OPENDOG's unique intersection

No existing tool combines file monitoring + AI process filtering + MCP integration. This intersection is the product. The three pillars reinforce each other:

1. **File monitoring** (inotify) provides the raw data
2. **AI process filtering** (PID whitelist) makes the data relevant to the AI workflow use case
3. **MCP integration** makes the tool accessible to the very AI tools being monitored (self-referential loop)

Without any one pillar, the product degrades to "yet another file watcher." All three are needed for the product to have its intended value.

---

*Research completed: 2026-04-24. Informed by analysis of Watchman, inotify-tools, entr, fswatch, MCP specification, MCP reference servers, and the OPENDOG project specification.*
