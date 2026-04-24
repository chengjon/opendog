# Stack Research: OPENDOG

## Recommended Stack

### File Watching: `notify` v7.x (high confidence)
- Cross-platform file watching abstraction built on inotify (Linux), FSEvents (macOS), ReadDirectoryChangesW (Windows)
- Why: Handles recursive watches, event debouncing, and inotify watch limit management internally. Much simpler than raw `inotify` crate.
- Alternative rejected: `inotify` crate — too low-level, requires manual watch management, no recursive support.

### Async Runtime: `tokio` v1.x (high confidence)
- Industry standard async runtime for Rust. Full-featured: tasks, channels, timers, fs, signals.
- Why: Needed for concurrent monitoring threads, MCP server I/O, and async channel-based event processing.
- Alternative rejected: `async-std` — smaller ecosystem, less mature for complex async patterns.

### SQLite: `rusqlite` v0.32+ with bundled SQLite (high confidence)
- Direct SQLite bindings. Synchronous API (no async needed — SQLite is fast enough locally).
- Why: Simple, no external service, `bundled` feature avoids system SQLite dependency issues. WAL mode supported.
- Alternative rejected: `sqlx` — async SQLite adds complexity without benefit for local single-writer access. `diesel` — ORM overhead unnecessary for simple schema.

### CLI: `clap` v4.x with `derive` feature (high confidence)
- Standard Rust CLI framework. Derive macro for type-safe argument parsing.
- Why: Widely used, excellent docs, handles subcommands (create/snapshot/start/stop/etc.).
- Alternative rejected: `lexopt` — too minimal for 8 subcommands with flags.

### Serialization: `serde` + `serde_json` v1.x (high confidence)
- JSON serialization/deserialization. Required for MCP JSON-RPC messages and config files.
- Why: Universal Rust standard. `serde_json` for MCP protocol, `serde` derives for all data types.

### Logging: `tracing` v0.1.x with `tracing-subscriber` (high confidence)
- Structured logging and tracing. Integrates with systemd journal via `tracing-journald`.
- Why: Superset of `log` crate. Spans for tracking operation durations. Journal integration for systemd.
- Alternative rejected: `log` + `env_logger` — insufficient for daemon observability.

### Process Inspection: Custom `/proc` parsing (medium confidence)
- No mature Rust crate for PID→name + parent chain traversal. Parse `/proc/<pid>/stat` and `/proc/<pid>/cmdline` directly.
- Why: Simple enough to implement directly. Only need: process name, parent PID, command line. `/proc/<pid>/stat` field 1 = name, field 4 = ppid.
- Alternative: `sysinfo` crate — too heavy (caches full system info), overkill for our needs.

### MCP Protocol: Custom JSON-RPC over stdio (medium confidence)
- No mature Rust MCP SDK exists as of 2025. Implement JSON-RPC 2.0 over stdio manually.
- Why: MCP protocol is essentially JSON-RPC with specific method names. A thin serde-based implementation is straightforward.
- Implementation: Read line-delimited JSON from stdin, parse as JSON-RPC request, dispatch to handler, write JSON-RPC response to stdout. Flush after each message.

### Systemd Integration: `sd-notify` crate (high confidence)
- Rust bindings for sd_notify (READY=1, STATUS=..., WATCHDOG=...).
- Why: Required for proper systemd service notification. Tiny dependency.
- Alternative: raw libc call — unnecessary when crate exists.

## Build Configuration

### Cargo.toml Key Dependencies
```toml
[dependencies]
notify = { version = "7", features = ["macos_kqueue"] }
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-journald = "0.3"
walkdir = "2"
sd-notify = "0.4"
chrono = { version = "0.4", features = ["serde"] }
```

### Release Profile
```toml
[profile.release]
opt-level = "z"     # Optimize for size (daemon stays resident)
lto = true          # Link-time optimization
strip = true        # Strip debug symbols
codegen-units = 1   # Maximum optimization
panic = "abort"     # Smaller binary, no unwinding
```

## Alternatives Considered

| Component | Rejected | Reason |
|-----------|----------|--------|
| File watching | `inotify` crate | Too low-level, no recursive support |
| Async | `async-std` | Smaller ecosystem |
| Database | `sqlx` | Async adds no value for local SQLite |
| Database | `diesel` | ORM overhead for simple schema |
| CLI | `structopt` | Merged into clap v4 derive |
| Logging | `log` + `env_logger` | Insufficient for daemon observability |
| Process | `sysinfo` | Heavy, caches full system state |
| MCP | Wait for Rust SDK | Protocol is simple enough to implement |

---
*Research completed: 2026-04-24*
