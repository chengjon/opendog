# OPENDOG — Quick Start

## Prerequisites

- **Rust** 1.75+ (`rustup install stable`)
- **Linux** with WSL2 or native (inotify requires Linux kernel)
- **systemd** enabled (for daemon mode, optional)

## Build

```bash
git clone https://github.com/chengjon/opendog.git
cd opendog
cargo build --release
```

The binary is at `target/release/opendog` (3.2MB, stripped).

## Install

```bash
sudo cp target/release/opendog /usr/local/bin/

# Optional: install systemd service
sudo cp deploy/opendog.service ~/.config/systemd/user/
systemctl --user daemon-reload
```

## Usage

### CLI (Terminal)

```bash
# Register a project
opendog create --id myapp --path /home/user/projects/myapp

# Scan all files in the project
opendog snapshot --id myapp

# View file usage statistics
opendog stats --id myapp

# List never-accessed files (candidates for cleanup)
opendog unused --id myapp

# List all registered projects
opendog list

# Start live monitoring (blocks until Ctrl+C)
opendog start --id myapp

# Delete a project and all its data
opendog delete --id myapp
```

### MCP Server (for AI Tools)

OPENDOG exposes an MCP server over stdio transport. Configure it in your AI tool's settings:

**Claude Code** (`~/.claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "opendog": {
      "command": "/opt/claude/opendog/target/release/opendog",
      "args": ["mcp"],
      "env": {
        "OPENDOG_HOME": "/home/user/.opendog"
      }
    }
  }
}
```

**OpenAI Codex CLI** (`~/.codex/config.json`):
```json
{
  "mcpServers": {
    "opendog": {
      "command": "/opt/claude/opendog/target/release/opendog",
      "args": ["mcp"],
      "env": {
        "OPENDOG_HOME": "/home/user/.opendog"
      }
    }
  }
}
```

**Any other MCP host**:
```json
{
  "mcpServers": {
    "opendog": {
      "command": "/absolute/path/to/opendog",
      "args": ["mcp"],
      "env": {
        "OPENDOG_HOME": "/home/user/.opendog"
      }
    }
  }
}
```

Or use the `opendog mcp` subcommand directly as an MCP stdio server:
```bash
opendog mcp      # Starts MCP server on stdio and auto-ensures daemon-backed monitoring reuse
opendog daemon   # Starts the background daemon explicitly (useful for systemd/user service)
```

`opendog mcp` is the recommended integration point for other MCP hosts. It now auto-starts and reuses the OPENDOG daemon when needed, so monitoring state survives MCP reconnects without requiring a separate manual `opendog daemon` step.

For stable cross-session reuse, prefer setting `OPENDOG_HOME` to a fixed absolute directory. If you omit it, OPENDOG falls back to `HOME/.opendog`, which is fine only when the host always launches `opendog mcp` with the same `HOME`.

**Current MCP Tool Surface:**

OPENDOG currently exposes 19 MCP tools:

- `create_project`, `take_snapshot`, `start_monitor`, `stop_monitor`, `get_stats`, `get_unused_files`, `list_projects`, `delete_project`
- `get_time_window_report`, `compare_snapshots`, `get_usage_trends`
- `get_global_config`, `get_project_config`
- `get_guidance`, `get_verification_status`, `record_verification_result`, `run_verification_command`
- `get_data_risk_candidates`, `get_workspace_data_risk_overview`

Detailed request and response shapes live in [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md).

### Daemon Mode (systemd)

```bash
# Start as user service
systemctl --user start opendog

# Enable on boot
systemctl --user enable opendog

# View logs
journalctl --user -u opendog -f
```

Resource limits: <1% CPU idle, <10MB RAM.

## How It Works

```
┌─────────────────────────────────────────────────┐
│  AI Tool (Claude/Codex/GPT)                     │
│    ├── MCP stdio ←→ opendog mcp                 │
│    └── systemd/background → opendog daemon      │
└─────────────────────────────────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
/proc scan     inotify        SQLite DB
(every 3s)     (file changes) (per-project)
    │               │               │
    └───────┬───────┘               │
            ▼                       ▼
     file_stats table      snapshot table
     (access count,        (all known files,
      duration, mods)       size, type)
            │                       │
            └─────── LEFT JOIN ─────┘
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
          get_stats  unused    core_files
```

**Process attribution** is approximate (sampling-based):
1. Every 3 seconds, scan `/proc/<pid>/fd/` for whitelisted AI processes
2. Match open file descriptors against project snapshot
3. Estimate duration from consecutive scan sightings
4. inotify detects file changes independently (not process-attributed)

## Configuration

Data stored in `~/.opendog/`:
```
~/.opendog/
  config.json              # Global defaults (ignore patterns, process whitelist)
  data/
    registry.db            # Project registry
    daemon.sock            # Local control socket
    daemon.pid             # Daemon PID file
    projects/
      myapp.db             # Per-project database
```

You can override that root with `OPENDOG_HOME=/absolute/path/to/opendog-state`. In that case the same layout is created under the directory you provide.

**Default ignore patterns:** `node_modules`, `.git`, `dist`, `target`, `__pycache__`, `.cache`, `build`, `.next`, `.nuxt`, `vendor`, `.venv`, `venv`, `.tox`, `.mypy_cache`, `.pytest_cache`, `.gradle`, `.idea`, `.vscode`, `*.pyc`, `.DS_Store`

**Default process whitelist:** `claude`, `codex`, `node`, `python`, `python3`, `gpt`, `glm`

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `inotify max_user_watches is low` | `sudo sysctl fs.inotify.max_user_watches=524288` |
| `Detected WSL1` | Upgrade to WSL2 for reliable inotify |
| `Invalid path` | Ensure directory exists before `create` |
| `Project not found` | Run `opendog list` to see registered projects |
| No stats showing | Run `opendog snapshot` first, then `opendog start` to begin monitoring |
| MCP state disappears between reconnects | Ensure every MCP host launch uses the same `OPENDOG_HOME` value, or at least the same `HOME` |

## Run Tests

```bash
cargo test                    # Full unit + integration suite
cargo test test_snapshot      # Snapshot-specific tests
cargo test -- --nocapture     # Show test output
RUST_LOG=debug cargo test     # Debug logging
```

---
*OPENDOG v1.0 — 2026-04-24*
