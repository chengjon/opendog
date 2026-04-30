# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**OPENDOG** — A multi-project observation and AI decision-support system for WSL. It tracks which files AI tools (Claude Code, Codex, GPT, GLM) access, identifies unused/stale files vs actively-used core files, and exposes reusable operator/AI entry surfaces through daemon, CLI, and MCP for repo risk, verification evidence, retained-evidence lifecycle, and suspicious mock or hardcoded data review.

**Current state**: v1 baseline is implemented, and Phase 6 is in progress. The observation core, local control plane, CLI operator surface, MCP AI surface, and retained-evidence cleanup/storage-maintenance signals are all live. The MCP surface now extends beyond the original CRUD-style tools and includes guidance, verification, cleanup, and data-risk tools.

**Authors**: JohnC (ninjas@sina.com) + Claude (GLM-5.1) + CodeX (GPT-5.4)

## Build & Run

```bash
cargo build --release            # Compile optimized binary
cargo test                       # Run all tests
cargo test test_snapshot         # Run single test by name
cargo test --test integration    # Run integration tests only
RUST_LOG=debug cargo run -- start --id myproject  # foreground monitor
cargo run -- daemon                               # daemon mode
cargo run -- mcp                                  # stdio MCP server
cargo run -- agent-guidance                       # workspace guidance view
cargo run -- data-risk --id myproject             # project data-risk view
cargo run -- workspace-data-risk                  # workspace data-risk view
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
MCP AI surface:           MCP stdio server (rmcp)
CLI operator surface:     CLI (clap)
Runtime coordination:     daemon supervisor + local control plane
Observation core:         Project Manager + Snapshot + Monitor + Stats + Reports + Verification + Retention
Storage:                  Per-project SQLite (.db files, WAL mode, single-writer pattern)
Base:                     WSL (Linux kernel) + systemd
```

### Module Structure

```
src/
  main.rs              # Entry: detect mode (cli/daemon/mcp), dispatch
  config.rs            # Project config loading (serde)
  error.rs             # Error types (thiserror)
  contracts.rs         # Versioned CLI/MCP JSON contract identifiers
  core/
    project.rs         # Project CRUD, namespace management
    snapshot.rs        # Recursive file scan, ignore patterns (notify)
    monitor.rs         # /proc scanner + inotify change detection
    scanner.rs         # /proc/<pid>/fd enumeration, AI process detection
    stats.rs           # Usage stats queries, unused file detection
    report.rs          # Time-window summaries, snapshot comparison, usage trends
    export.rs          # JSON/CSV export of project evidence
    verification.rs    # Record and execute test/lint/build evidence
    retention.rs       # Retained-evidence cleanup and storage metrics
  storage/
    database.rs        # SQLite connection management, WAL mode
    schema.rs          # CREATE TABLE statements, migrations
    queries.rs         # All read/write operations
  mcp/
    mod.rs             # MCP server (rmcp stdio), request dispatch
  cli/
    mod.rs             # clap subcommand definitions
    output.rs          # Terminal formatting (tables, colors)
  daemon.rs            # systemd: sd_notify, signal handling, pid file
  control.rs           # local control plane for daemon-owned project operations
  guidance.rs          # Shared agent-guidance and decision-brief assembly
```

### Per-Project Isolation

Each project: own SQLite `.db`, own config, own monitoring state. Projects can start/stop/delete independently. No cross-project data leakage.

## MCP Tools

Current MCP surface: 25 tools total.

Baseline control tools:

- `create_project`
- `take_snapshot`
- `start_monitor`
- `stop_monitor`
- `get_stats`
- `get_unused_files`
- `list_projects`
- `delete_project`

Comparative reporting:

- `get_time_window_report`
- `compare_snapshots`
- `get_usage_trends`

Retention / storage hygiene:

- `cleanup_project_data`

Configuration and export:

- `get_global_config`
- `get_project_config`
- `update_global_config`
- `update_project_config`
- `reload_project_config`
- `export_project_evidence`

AI-facing guidance, verification, and data-risk tools:

- `get_agent_guidance`
- `get_decision_brief`
- `get_verification_status`
- `record_verification_result`
- `run_verification_command`
- `get_data_risk_candidates`
- `get_workspace_data_risk_overview`

MCP parameter hints:

- `get_agent_guidance` accepts optional `project_id` and `top`
- `get_decision_brief` accepts optional `project_id` and `top`
- both tools prefer daemon-backed state through the local control plane when the daemon is live

## CLI Commands

Current CLI surface: 21 top-level entry commands.

Core control and runtime:

- `opendog create --id <ID> --path <DIR>`
- `opendog snapshot --id <ID>`
- `opendog start --id <ID>`
- `opendog stop --id <ID>`
- `opendog stats --id <ID>`
- `opendog unused --id <ID>`
- `opendog list`
- `opendog delete --id <ID>`
- `opendog daemon`
- `opendog mcp`

Configuration and export:

- `opendog config show [--id <ID>] [--json]`
- `opendog config set-project --id <ID> [--ignore-pattern <PATTERN>]... [--process <PROC>]... [--inherit-ignore-patterns] [--inherit-process-whitelist] [--json]`
- `opendog config set-global [--ignore-pattern <PATTERN>]... [--process <PROC>]... [--json]`
- `opendog config reload --id <ID> [--json]`
- `opendog export --id <ID> --format <json|csv> --view <stats|unused|core> --output <PATH> [--min-access-count N]`

Comparative reporting and cleanup:

- `opendog report window --id <ID> [--window 24h|7d|30d] [--limit N] [--json]`
- `opendog report compare --id <ID> [--base-run-id N --head-run-id N] [--limit N] [--json]`
- `opendog report trend --id <ID> [--window 24h|7d|30d] [--limit N] [--json]`
- `opendog cleanup-data --id <ID> --scope <activity|snapshots|verification|all> [--older-than-days N] [--keep-snapshot-runs N] [--dry-run] [--vacuum] [--json]`

Guidance, verification, and data-risk:

- `opendog agent-guidance [--project <ID>] [--top <N>] [--json]`
- `opendog decision-brief [--project <ID>] [--top <N>] [--json]`

- `opendog record-verification --id <ID> --kind <test|lint|build> --status <passed|failed> --command <CMD> [--json]`
- `opendog verification --id <ID> [--json]`
- `opendog run-verification --id <ID> --kind <test|lint|build> --command <CMD> [--json]`
- `opendog data-risk --id <ID> [--candidate-type all|mock|hardcoded] [--min-review-priority low|medium|high] [--json]`
- `opendog workspace-data-risk [--candidate-type all|mock|hardcoded] [--min-review-priority low|medium|high] [--project-limit N] [--json]`

## Quick Start for AI Agents

Fast navigation:

- capability-to-entrypoint map: [docs/capability-index.md](/opt/claude/opendog/docs/capability-index.md)
- AI workflow and safety order: [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)
- MCP tool shapes: [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)
- CLI/MCP JSON contracts: [docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)

Use this default sequence unless the task clearly requires something else:

1. Make sure the project is registered.
2. Run `take_snapshot` or `opendog snapshot --id <ID>` if no fresh baseline exists.
3. Run `start_monitor` or `opendog start --id <ID>` if no monitor is active.
4. Before editing, check `get_agent_guidance`.
   CLI equivalent: `opendog agent-guidance`
   Use `project_id` for single-project scope and `top` to shorten the recommendation queue.
   If you want one stable AI entry envelope first, check `get_decision_brief`.
   CLI equivalent: `opendog decision-brief`
5. If storage maintenance is flagged, inspect `cleanup_project_data` or `opendog cleanup-data --dry-run` before long cleanup/refactor sessions.
6. Before cleanup or refactor, check `get_verification_status` and then `get_data_risk_candidates`.
7. If choosing among multiple projects, start with `get_workspace_data_risk_overview`.

Practical tool-choice rules:

- Use `get_workspace_data_risk_overview` when the question is "which project deserves attention first?"
- Use `get_decision_brief` when the question is "give me one stable decision envelope first, then I will choose tools from it"
- Use `get_agent_guidance` or `opendog agent-guidance` when the question is "what should I do next overall or in this project?"
- Use `get_verification_status` before claiming a project is safe for cleanup or broad edits
- Use `get_data_risk_candidates` or `opendog data-risk` before touching suspicious mock/demo/seed/business-like literals
- Use `cleanup_project_data` or `opendog cleanup-data` when OPENDOG-retained evidence itself should be pruned; this never deletes source files
- If daemon is live, prefer daemon-backed state via the local control plane rather than starting a second independent monitor path

See also: [docs/ai-playbook.md](/opt/claude/opendog/docs/ai-playbook.md)
For machine-consumption rules of the structured CLI outputs, see [docs/json-contracts.md](/opt/claude/opendog/docs/json-contracts.md)
For direct MCP request-shape examples, see [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md)

## Planning Artifacts

All in `.planning/`:
- `PROJECT.md` — project context, requirements, constraints, key decisions
- `FUNCTION_TREE.md` — canonical capability hierarchy and FT ownership
- `REQUIREMENTS.md` — 114 mapped requirements across v1 baseline and Phase 6+ hardening
- `ROADMAP.md` — current phased roadmap with success criteria and plan breakdown
- `STATE.md` — current phase status
- `config.json` — GSD workflow settings (YOLO mode, standard granularity, parallel)
- `task-cards/` — concrete execution cards with `FT-*` leaf mappings
- `research/` — STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md, SUMMARY.md

Historical note:

- the original Phase 4 baseline was 8 MCP tools and 8 matching CLI subcommands
- the current shipped surface is larger because later hardening added reporting, config, export, cleanup, verification, guidance, and data-risk entrypoints without collapsing them back into the baseline requirement family

## Implementation Roadmap

| Phase | Goal | Requirements |
|-------|------|-------------|
| 1 | Storage + Project CRUD + Snapshot | PROJ-01..05, SNAP-01..05 |
| 2 | /proc Scanner + inotify Monitor | MON-01..06, PROC-01..06 |
| 3 | Statistics & Analytics | STAT-01..08 |
| 4 | MCP Server + CLI | MCP-01..09, CLI-01..09 |
| 5 | Daemon & Systemd Deployment | DAEM-01..05, CTRL-01..05 |
| 6 | AI Guidance & Reusable Intelligence | OBS / RISK / STRAT / EVID / PORT / CLEAN / STACKX / BOUND / MOCK / RET |

Current next step: continue Phase 6 refinement and documentation alignment

## Known Constraints

- **WSL2 required** — WSL1 has poor inotify; /mnt/ paths don't support inotify
- **Resource budget**: <1% CPU idle, <10MB RAM
- **Approximate attribution only** — /proc scanning is sampling-based (2-5s intervals)
- **Release profile required** — `opt-level = "z"`, LTO, strip, `panic = "abort"`
- **Single SQLite writer** — all writes through one tokio task via mpsc channel
- **systemd must be enabled** in WSL (`/etc/wsl.conf` `[boot] systemd=true`)
- **Daemon ownership matters** — if daemon is running, CLI/MCP should prefer the local control plane instead of silently diverging into separate monitor state
- **Retained evidence is not source code** — `cleanup_project_data` / `cleanup-data` only prune OPENDOG history and storage overhead
