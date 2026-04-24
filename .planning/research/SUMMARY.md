# Research Summary: OPENDOG

## Stack Recommendations

| Component | Choice | Confidence |
|-----------|--------|------------|
| File watching | `notify` v7 (wraps inotify) | High |
| Async runtime | `tokio` v1 (full) | High |
| Database | `rusqlite` v0.32 (bundled, WAL mode) | High |
| CLI | `clap` v4 (derive) | High |
| Serialization | `serde` + `serde_json` | High |
| Logging | `tracing` + `tracing-journald` | High |
| Process inspection | Custom `/proc` parsing | Medium |
| MCP protocol | Custom JSON-RPC over stdio | Medium |
| Systemd | `sd-notify` crate | High |

**Key insight:** No mature Rust MCP SDK exists yet. Implement JSON-RPC 2.0 manually — the protocol is simple (line-delimited JSON, serde handles the rest).

## Table Stakes (must ship)

1. **Recursive file watching** with smart filtering (node_modules, .git, target, dist)
2. **Multi-file event types** (create, modify, delete, access via inotify)
3. **Non-intrusive operation** — passive observer, no interception
4. **Persistent storage** — SQLite, survives restarts
5. **Start/stop control** — per-project, independent
6. **CLI interface** — 8 commands for manual management
7. **Resource budget** — <1% CPU, <10MB RAM at idle

## Differentiators (competitive advantage)

1. **AI process file tracking** — /proc/<pid>/fd scanning + inotify change detection (no other tool does this). **⚠ Derived design**: inotify cannot provide PID attribution; /proc scanning is the viable alternative.
2. **MCP server integration** — AI tools call OPENDOG directly
3. **Access duration tracking** — open→close timing, not just access count
4. **Unused file detection** — zero-access files since snapshot
5. **Multi-project isolation** — independent databases, configs, threads
6. **Dual interface** — MCP + CLI share same core

## Architecture Highlights

- **7-phase build order**: storage → project+snapshot → monitor+process → stats → MCP → CLI → daemon
- **Single writer pattern** for SQLite via tokio mpsc channel — avoids write contention
- **Actor pattern** for per-project monitoring tasks
- **Phases 5+6 (MCP + CLI)** can be built in parallel since they share core layer

## Critical Pitfalls to Address

1. **⛔ FUNDAMENTAL: inotify provides NO PID info** — must use /proc/<pid>/fd scanning instead. See PIT-00.
2. **inotify watch limit** — must check/increase `max_user_watches`, fall back for overflow
3. **Event overflow** — tight read loop + async channel decoupling + snapshot diff on overflow
4. **/proc scan limitations** — sampling-based (2-5s gaps), PID recycling, racy fd enumeration
4. **SQLite write contention** — WAL mode + single writer + batch flushes + busy_timeout
5. **MCP stdio corruption** — never print to stdout, flush after each message
6. **WSL inotify on /mnt/** — only monitor Linux-native paths, detect and warn
7. **Graceful shutdown** — SIGTERM handler to flush buffers, sd_notify integration

## WSL-Specific Notes

- WSL2 required (real Linux kernel for reliable inotify)
- systemd must be enabled via `/etc/wsl.conf`
- No inotify support on /mnt/ Windows mounts
- Some /proc fields differ — test process detection on actual WSL

---
*Synthesized: 2026-04-24*
