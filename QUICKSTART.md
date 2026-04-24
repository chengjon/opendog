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
      "command": "opendog",
      "args": ["mcp"]
    }
  }
}
```

**OpenAI Codex CLI** (`~/.codex/config.json`):
```json
{
  "mcpServers": {
    "opendog": {
      "command": "opendog",
      "args": ["mcp"]
    }
  }
}
```

Or use the `opendog mcp` subcommand directly as an MCP stdio server:
```bash
opendog mcp    # Starts MCP server on stdio (for any MCP client)
opendog daemon  # Same as mcp, plus sd_notify + journald (for systemd)
```

**8 MCP Tools Available:**

| Tool | Description |
|------|-------------|
| `create_project` | Register project with ID and root path |
| `take_snapshot` | Trigger recursive file scan |
| `start_monitor` | Begin /proc scanning + inotify monitoring |
| `stop_monitor` | Stop monitoring |
| `get_stats` | Per-file access count, duration, modifications |
| `get_unused_files` | Never-accessed files (cleanup candidates) |
| `list_projects` | All registered projects and status |
| `delete_project` | Remove project and all data |

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
│    └── MCP stdio ←→ opendog daemon              │
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
  registry.db              # Project registry
  data/
    projects/
      myapp.db             # Per-project database
  daemon.pid               # PID file (daemon mode)
```

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

## Run Tests

```bash
cargo test                    # All 25 tests
cargo test test_snapshot      # Snapshot-specific tests
cargo test -- --nocapture     # Show test output
RUST_LOG=debug cargo test     # Debug logging
```

---
*OPENDOG v1.0 — 2026-04-24*
