# OPENDOG Stack Recommendations

> Researched 2026-04-24 via crates.io API. All versions are latest stable.

## 1. File Watching: `notify` 8.2.0

**Confidence: HIGH**

Cross-platform file watching that uses inotify on Linux. Declarative API with `RecommendedWatcher`, handles recursive watches, debouncing, and event coalescing.

- **Rejected `inotify` 0.11.1**: Raw inotify bindings. Too low-level -- manual fd lifecycle, watch descriptors, event parsing. Only justified for IN_EXCL_UNLINK or other edge-case flags.
- **Rejected `notify` 9.0.0-rc.3**: Release candidate. Use 8.2.0 stable for production.

```toml
notify = "8.2"
```

## 2. MCP Server: `rmcp` 1.5.0

**Confidence: HIGH**

Official Rust SDK for Model Context Protocol. Supports stdio transport, JSON-RPC 2.0, tools, resources, and prompts. Actively maintained by the MCP community. Eliminates hand-rolling JSON-RPC framing.

- **Rejected hand-rolled JSON-RPC over stdio**: Error-prone line-delimited framing, request/response correlation, batch handling. `rmcp` handles all of this.
- **Rejected `jsonrpc-core` 18.0.0**: Generic JSON-RPC without MCP schema types. Would need to implement the entire MCP type layer manually.
- **Fallback**: If `rmcp` proves insufficient, fall back to `serde_json` + manual line-delimited JSON-RPC on stdin/stdout. The MCP protocol is simple enough for this to be viable.

```toml
rmcp = { version = "1.5", features = ["server", "transport-io"] }
```

## 3. SQLite: `rusqlite` 0.39.0

**Confidence: HIGH**

Synchronous SQLite bindings with bundled C source. Per-project databases mean simple file paths, no network layer. `bundled` feature avoids system SQLite version issues on WSL.

- **Rejected `sqlx` 0.9.0-alpha.1**: Async SQLite adds complexity with no benefit -- single-threaded per-project writes, not concurrent query pools. Also currently alpha.
- **Rejected `diesel`**: ORM overhead unnecessary for simple schema.

```toml
rusqlite = { version = "0.39", features = ["bundled"] }
```

## 4. CLI: `clap` 4.6.1

**Confidence: HIGH**

De facto standard Rust CLI parser. Derive macro for type-safe argument definitions. Built-in help, shell completions, and version flags.

- **Rejected `lexopt`**: Too minimal -- requires manual help text generation, no derive macros.
- **Rejected `structopt`**: Merged into clap v4 derive. No longer separate.

```toml
clap = { version = "4.6", features = ["derive"] }
```

## 5. Async Runtime: `tokio` 1.52.1

**Confidence: HIGH**

Industry-standard async runtime. Required for `rmcp` (uses tokio for async I/O). `rt-multi-thread` for file watching + MCP server concurrency.

- **Rejected `async-std`**: Smaller ecosystem. `rmcp` and most async crates target tokio.
- **Rejected `smol`**: Minimalist. Would require manual glue for tokio-based dependencies.

```toml
tokio = { version = "1.52", features = ["rt-multi-thread", "macros", "sync", "io-util", "process"] }
```

## 6. Serialization: `serde` 1.0.228 + `serde_json` 1.0.149

**Confidence: HIGH**

Universal Rust serialization. Required by `rmcp`, `clap` (for config), and all JSON output. No alternative comes close in ecosystem support.

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 7. Systemd Integration: `sd-notify` 0.5.0

**Confidence: MEDIUM**

Minimal crate wrapping `sd_notify()`. Sends READY, STATUS, WATCHDOG notifications to systemd. Zero dependencies.

- **Rejected hand-rolling**: Just `unsafe { libc::sendto(...) }` but error-prone with fd passing. Not worth skipping the 1 dependency.
- **Medium confidence**: Crate is stable but low-activity (feature-complete). Normal for thin systemd wrappers.

```toml
sd-notify = "0.5"
```

## 8. Process Inspection: `procfs` 0.18.0

**Confidence: HIGH**

Rust interface to `/proc` filesystem. Parses `/proc/[pid]/cmdline`, `/proc/[pid]/status`, `/proc/[pid]/fd` with proper types. Used by `procs`, `bottom`, and other system tools.

- **Rejected hand-parsing /proc**: File format has edge cases (null-delimited cmdline, zombie processes, permission errors on foreign PIDs). `procfs` handles these.
- **Rejected `sysinfo`**: Too heavy -- caches full system info, overkill for PID filtering.
- **Usage**: Iterate `/proc` entries, filter by cmdline pattern, resolve cwd via `/proc/[pid]/cwd` symlink.

```toml
procfs = "0.18"
```

## 9. Logging: `tracing` 0.1.44 + `tracing-subscriber` 0.3.23 + `tracing-appender` 0.2.5

**Confidence: HIGH**

Structured logging and async-aware diagnostics. `tracing-subscriber` for formatting and filtering. `tracing-appender` for log file rotation when running as a daemon.

- **Rejected `log` + `env_logger`**: Unstructured. No spans, no async context, no structured fields. `tracing` is a superset.
- **Rejected `slog`**: Mature but less active. `tracing` has better async integration and is the ecosystem standard.

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
```

## Dependency Summary

```toml
[dependencies]
notify = "8.2"
rmcp = { version = "1.5", features = ["server", "transport-io"] }
rusqlite = { version = "0.39", features = ["bundled"] }
clap = { version = "4.6", features = ["derive"] }
tokio = { version = "1.52", features = ["rt-multi-thread", "macros", "sync", "io-util", "process"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sd-notify = "0.5"
procfs = "0.18"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
panic = "abort"
```

Total: 12 crates (11 direct + serde_json). All stable, all well-maintained, all compatible with Linux/WSL2 target.

## Key Changes from Previous Research

| Component | Before | After | Reason |
|-----------|--------|-------|--------|
| File watching | notify 7.x | notify 8.2.0 | Major version bump with API improvements |
| MCP server | Hand-rolled JSON-RPC | rmcp 1.5.0 | Official Rust MCP SDK now exists |
| SQLite | rusqlite 0.32+ | rusqlite 0.39.0 | Latest stable |
| Process inspection | Custom /proc parsing | procfs 0.18.0 | Mature crate available, handles edge cases |
| sd-notify | 0.4 | 0.5.0 | Latest stable |
| Removed | walkdir, chrono, tracing-journald | -- | Not needed; notify handles recursion, timestamps from std |

---
*Research completed: 2026-04-24*
