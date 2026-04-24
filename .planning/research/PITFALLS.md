# Pitfalls Research: OPENDOG

## Critical Pitfalls (must address)

### PIT-00: inotify Does NOT Provide Process Attribution ⛔ FUNDAMENTAL
**Description:** Linux inotify(7) explicitly states: *"The inotify API provides no information about the user or process that triggered the inotify event."* There is no PID field in inotify events. **You cannot filter inotify events by process.** The original README's goal of "仅捕捉AI工具相关进程访问的文件" cannot be achieved by reading PID from inotify events — the data does not exist.
**Warning signs:** Any design that says "from inotify event, get PID, check /proc" — this is impossible.
**Prevention:** Use a completely different approach: **periodic /proc/<pid>/fd scanning** as the primary method. See ARCHITECTURE.md "Derived Design Decision: Process Attribution Strategy" for the full solution. inotify is used only for change detection (what changed), not for process attribution (who changed it).
**Phase:** Phase 2 — this determines the entire monitoring architecture.

### PIT-01: inotify Watch Limit Exhaustion
**Description:** Linux caps inotify watches per user (default: 8192 on WSL, 524288 on some distros). Large projects with many directories will silently fail to create watches.
**Warning signs:** Files in deeply nested directories not appearing in monitoring data; `ENOENT` or "no space left on device" errors from inotify.
**Prevention:** On startup, check `/proc/sys/fs/inotify/max_user_watches` and warn if low. Auto-increase via sysctl in installer. Track watch count vs limit at runtime. Fall back to polling for unwatchable directories.
**Phase:** Phase 1 (storage + snapshot) — plan watch budget during snapshot.

### PIT-02: inotify Event Overflow
**Description:** When events arrive faster than they can be processed, inotify drops events and sends `IN_Q_OVERFLOW`. The kernel buffer is finite (`max_queued_events`, default 16384).
**Warning signs:** Monitoring gaps — files that were clearly accessed show zero events.
**Prevention:** Read events in a tight loop with minimal processing. Decouple event reading from database writes via an async channel (tokio mpsc). Log overflow events. On overflow, trigger a snapshot diff to catch missed changes.
**Phase:** Phase 2 (monitoring engine) — core event loop design.

### PIT-03: /proc Scan Limitations (sampling gaps, not event-level attribution)
**Description:** The /proc scanning approach is inherently sampling-based. A process may open and close a file between scan intervals (2-5s). PID recycling means a PID that was claude may become an unrelated process by the next scan. Reading /proc/<pid>/fd is racy — fd symlinks may vanish mid-enumeration.
**Warning signs:** Brief file accesses (< scan interval) not recorded; impossible file access patterns.
**Prevention:** Accept this as statistical sampling, not precise auditing. Use procfs crate for robust /proc parsing. Snapshot process name alongside PID on each scan. Keep scan interval configurable. Document that this is approximate attribution.
**Phase:** Phase 2 (monitoring engine).

### PIT-04: SQLite Write Contention
**Description:** SQLite uses file-level locking. Multiple monitoring threads writing to the same database will contend, causing `SQLITE_BUSY` errors and potential data loss.
**Warning signs:** `database is locked` errors under load; stats not recording during high event volume.
**Prevention:** Use WAL mode (`PRAGMA journal_mode=WAL`). Route all writes through a single writer task using tokio channels. Batch writes (accumulate events in memory, flush every N seconds). Set busy timeout (`PRAGMA busy_timeout=5000`).
**Phase:** Phase 1 (storage layer) — this is foundational.

### PIT-05: MCP stdio Buffering
**Description:** JSON-RPC over stdio requires strict message framing. If the process or runtime buffers stdout, messages won't arrive as expected. Mixing debug prints with MCP protocol output on stdout will corrupt the stream.
**Warning signs:** MCP client hangs waiting for responses; parse errors on JSON-RPC messages.
**Prevention:** Use a dedicated serde serialization for MCP output. Never print to stdout — all logging goes to stderr or journal. Flush stdout after each message. Consider using `BufWriter` with explicit flush.
**Phase:** Phase 3 (MCP server).

## Moderate Pitfalls (should address)

### PIT-06: WSL inotify Performance Degradation
**Description:** WSL1 translates Linux syscalls to Windows NT APIs. inotify on WSL1 is significantly slower and less reliable than native Linux. WSL2 uses a real Linux kernel and is much better, but Windows filesystem access (via /mnt/c) has poor inotify support.
**Warning signs:** Events delayed by seconds; missed events on /mnt/ paths.
**Prevention:** Detect WSL version at startup and warn on WSL1. Only monitor paths on the Linux filesystem (/home, /root, etc.), not Windows mounts. Document that WSL2 is required.
**Phase:** Phase 1 (setup).

### PIT-07: Graceful Shutdown Data Loss
**Description:** Killing the daemon (SIGTERM, SIGKILL) while events are buffered in memory means unsaved stats are lost. Systemd may kill the process after TimeoutStopSec.
**Warning signs:** After restart, stats don't match actual usage; gaps in access records.
**Prevention:** Handle SIGTERM to flush buffers before exit. Use sd_notify("READY=1") and sd_notify("STATUS=...") with systemd. Set reasonable TimeoutStopSec. Consider write-ahead logging for in-flight events.
**Phase:** Phase 5 (daemon).

### PIT-08: Recursive Directory Scan Scalability
**Description:** Scanning large projects (100K+ files) takes time and memory. Node_modules alone can have 50K+ files.
**Warning signs:** Snapshot takes >30 seconds; memory spike during scan; OOM on small systems.
**Prevention:** Use walkdir with parallel traversal. Enforce ignore patterns early (skip node_modules, .git, target, dist at directory level, not file level). Set a reasonable file count limit with warning. Stream results to SQLite instead of collecting in memory.
**Phase:** Phase 1 (snapshot).

### PIT-09: MCP Protocol Version Compatibility
**Description:** The MCP spec is evolving. Tools, prompts, and resource APIs may change between versions. Hardcoding a specific version may cause incompatibility with newer clients.
**Warning signs:** MCP clients reject the server; missing capabilities.
**Prevention:** Implement the MCP spec version that Claude Code and Codex currently use. Use capability negotiation on connect. Keep the protocol layer isolated so it can be updated independently.
**Phase:** Phase 3 (MCP server).

### PIT-10: Clock Skew Between Event Time and Recording Time
**Description:** There's inherent delay between when a file is accessed and when the event is recorded. For duration tracking (open→close), this matters.
**Warning signs:** Negative or implausible durations in stats.
**Prevention:** Use monotonic clock (Instant) for duration measurement, not wall clock. Record both event timestamp (from inotify) and recording timestamp. Accept microsecond-level imprecision as inherent to the design.
**Phase:** Phase 2 (monitoring).

## Minor Pitfalls (nice to address)

### PIT-11: Symlink Handling
**Description:** Symlinks can cause infinite loops in recursive scans, or the same file being tracked under multiple paths.
**Prevention:** Resolve symlinks to real paths. Detect symlink loops. Consider canonicalizing all paths.

### PIT-12: Special Filesystems
**Description:** /proc, /dev, /sys, and FUSE filesystems should never be monitored.
**Prevention:** Skip non-regular files during snapshot. Filter by filesystem type if possible.

### PIT-13: Database Migration
**Description:** As the schema evolves across versions, existing databases need migration.
**Prevention:** Include a schema version table. Write migration functions for each version bump. Test migration with real databases.

### PIT-14: Binary File Detection
**Description:** Monitoring binary files (images, compiled objects) generates noise. The spec says to track by file type but doesn't define how.
**Prevention:** Use file extension whitelist/blacklist. Skip files without recognized extensions unless explicitly configured.

## WSL-Specific Pitfalls

### WSL-01: inotify on /mnt/ Paths
WSL2 does not reliably support inotify on Windows filesystem mounts (/mnt/c, /mnt/d). Only monitor Linux-native paths. Detect and warn if user configures a /mnt/ path.

### WSL-02: WSL Instance Lifecycle
WSL instances can be terminated by Windows (reboot, wsl --shutdown). Systemd services restart automatically, but unsaved data is lost. Handle gracefully.

### WSL-03: /proc Quirks on WSL
Some /proc fields differ on WSL (e.g., /proc/<pid>/exe may show different paths). Test process name detection on actual WSL environment, not just Linux VMs.

### WSL-04: systemd Not Enabled by Default
WSL2 doesn't enable systemd by default. Users need `/etc/wsl.conf` with `[boot] systemd=true`. The installer should detect and guide.

---
*Research completed: 2026-04-24*
