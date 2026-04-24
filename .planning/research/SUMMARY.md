# Research Summary: OPENDOG

## Stack Recommendations (aligned with STACK.md)

| Component | Choice | Version | Confidence |
|-----------|--------|---------|------------|
| File watching | `notify` | 8.2.0 | High |
| MCP server | `rmcp` | 1.5.0 (official Rust MCP SDK) | High |
| Database | `rusqlite` (bundled, WAL mode) | 0.39.0 | High |
| CLI | `clap` (derive) | 4.6.1 | High |
| Async runtime | `tokio` | 1.52.1 | High |
| Serialization | `serde` + `serde_json` | 1.0.228 / 1.0.149 | High |
| Process inspection | `procfs` | 0.18.0 | High |
| Logging | `tracing` + `tracing-subscriber` + `tracing-appender` | 0.1.44 / 0.3.23 / 0.2.5 | High |
| Systemd | `sd-notify` | 0.5.0 | Medium |

**Key changes from initial research:** rmcp 1.5.0 replaces hand-rolled JSON-RPC. procfs 0.18.0 replaces custom /proc parsing. notify 8.2.0 is current stable (v9 is RC only).

## ⚠ Fundamental Design Correction: Process Attribution

**inotify does NOT provide PID/process information** (per inotify(7) man page). The original README's "仅捕捉AI工具相关进程访问的文件" cannot be implemented by filtering inotify events.

**Actual approach (derived design decision):**
- **Primary**: Periodic /proc/<pid>/fd scanning (every 2-5s) — enumerate AI processes, resolve their open file descriptors
- **Secondary**: inotify via notify crate — detect file changes (what changed, not who changed it)
- **Attribution**: Timestamp cross-reference — if file was modified AND an AI process had it open around that time → approximate attribution
- **This is statistical sampling, not precise auditing.** Honest about limitations: brief accesses (< scan interval) may be missed, duration is estimated.

## Table Stakes (must ship)

1. Recursive file watching with smart filtering (node_modules, .git, target, dist)
2. Multi-file event detection (create, modify, delete via inotify; open files via /proc scan)
3. Non-intrusive operation — read-only /proc access, passive inotify watching
4. Persistent storage — SQLite with WAL mode, survives restarts
5. Start/stop control — per-project, independent
6. CLI interface — 8 commands for manual management
7. Resource budget — <1% CPU, <10MB RAM at idle

## Differentiators

1. **AI process file tracking** — /proc/<pid>/fd scanning + inotify change detection (no other tool does this)
2. **MCP server integration** — via rmcp crate, AI tools call OPENDOG directly
3. **Approximate usage duration** — estimated from consecutive /proc scan sightings
4. **Unused file detection** — files in snapshot with zero AI process sightings
5. **Multi-project isolation** — independent databases, configs, monitoring state

## Critical Pitfalls

1. **⛔ FUNDAMENTAL: inotify provides NO PID info** — must use /proc/<pid>/fd scanning. See PIT-00.
2. **inotify watch limit** — check/increase max_user_watches, fall back for overflow
3. **Event overflow** — tight read loop + async channel decoupling
4. **/proc scan limitations** — sampling gaps (2-5s), PID recycling, racy fd enumeration
5. **SQLite write contention** — WAL mode + single writer via tokio mpsc + busy_timeout
6. **MCP stdio corruption** — never print to stdout, flush after each message
7. **WSL inotify on /mnt/** — only monitor Linux-native paths

## WSL-Specific Notes

- WSL2 required (real Linux kernel for reliable inotify + /proc)
- systemd must be enabled via /etc/wsl.conf
- No inotify support on /mnt/ Windows mounts
- /proc fields may differ on WSL — test process detection on actual environment

---
*Synthesized: 2026-04-24 (revised for consistency with STACK.md and process attribution correction)*
