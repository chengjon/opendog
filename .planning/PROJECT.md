# OPENDOG

## What This Is

A multi-project file monitoring system for AI development workflows. Running as a background daemon on WSL, it uses Linux inotify to non-intrusively track which files AI tools (Claude Code, Codex, GPT, GLM) access across multiple projects — recording access frequency, duration, and modifications. Per-project SQLite databases store snapshots and usage stats, enabling identification of unused/stale files vs core files. Exposes both an MCP server (stdio) for AI tool integration and a CLI for manual management.

## Core Value

Accurately identify which project files AI tools actually use and which are dead weight — enabling data-driven file cleanup decisions across multiple concurrent projects.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Multi-project isolation — each project gets independent database, config, monitoring thread, and namespace
- [ ] File snapshot engine — recursive scan with smart filtering (node_modules, .git, dist, etc.), recording path/size/mtime/type
- [ ] AI process file tracking via /proc scanning — periodic /proc/<pid>/fd enumeration of whitelisted AI processes, cross-referenced with inotify change detection for approximate attribution (sampling-based, not precise auditing)
- [ ] Usage statistics — approximate access count (/proc scan sightings), estimated duration (consecutive scan intervals), modification count (inotify), last access timestamp per file
- [ ] Unused file detection — files in snapshot with zero AI accesses marked as cleanup candidates
- [ ] Core file identification — high-frequency, long-duration files flagged as important
- [ ] MCP server (stdio transport) — 8 tools: create_project, take_snapshot, start_monitor, stop_monitor, get_stats, get_unused_files, list_projects, delete_project
- [ ] CLI tool — matching 8 commands for manual project/monitoring management
- [ ] Systemd daemon — auto-start, auto-restart, low resource footprint (<1% CPU, <10MB RAM)

### Out of Scope

- Visual dashboard / web UI — terminal-based for v1
- Auto-cleanup of unused files — only identify, never delete
- Windows native support — WSL only
- Remote/network monitoring — local filesystem only
- Real-time streaming to external services — local SQLite storage only

## Context

- Built for WSL2 environment — requires real Linux kernel for inotify and /proc filesystem
- Target users are developers running multiple AI-assisted projects concurrently (Codex + GPT-5.4, Claude Code + GLM-5.1)
- ⚠ **Derived design decision**: inotify does NOT provide process attribution (per inotify(7)). Actual approach: periodic /proc/<pid>/fd scanning (primary — what files AI processes have open) + inotify change detection via notify crate (secondary — what files changed). Cross-referenced by timestamp for approximate attribution. This is statistical sampling (2-5s intervals), not precise per-event auditing.
- Each project maps to one SQLite .db file with two tables: snapshot (file baseline) and stats (usage data)
- MCP stdio transport via rmcp crate — AI tools call OPENDOG directly
- Rust chosen for memory safety, low resource overhead, and 7x24 daemon stability

## Constraints

- **Platform**: WSL (Windows Subsystem for Linux) — relies on Linux inotify API
- **Language**: Rust (release-optimized builds)
- **Resource Budget**: CPU < 1% at idle, memory < 10MB
- **Storage**: SQLite per project — no external database dependencies
- **Transport**: MCP over stdio (no HTTP/SSE in v1)
- **Deployment**: Systemd service for daemon management

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as implementation language | Memory safety + low overhead for long-running daemon | — Pending |
| Per-project SQLite isolation | Zero cross-project data leakage, simple backup/deletion | — Pending |
| PID whitelist + parent chain for process filtering | Accurate AI process identification without kernel modules | — Pending |
| MCP stdio transport | Standard for CLI-integrated MCP servers, matches Claude Code integration | — Pending |
| inotify for file watching | Kernel-level, non-intrusive, no filesystem modifications needed | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-24 after initialization*
