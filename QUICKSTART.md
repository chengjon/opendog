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
opendog register --id myapp --path /home/user/projects/myapp

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

#### MCP Binary Updates

MCP hosts execute the binary path configured in their MCP settings. Updating OpenDog source code does not update already configured MCP servers by itself.

If a host is configured like this:

```json
{
  "command": "/opt/claude/opendog/target/release/opendog",
  "args": ["mcp"]
}
```

then it will use the latest code only after both conditions are true:

- OpenDog has been rebuilt with `cargo build --release`.
- The MCP host has restarted or reconnected so the old `opendog mcp` process exits and a new process starts.

Recommended update flow:

```bash
cd /opt/claude/opendog
opendog self-update status --source /opt/claude/opendog
opendog self-update build --source /opt/claude/opendog
```

Execution boundary:

- Run this from a WSL/Linux shell, not as an automatic MCP tool action.
- Treat it as an OpenDog maintenance/operations command, not as a business-project command.
- Prefer running it from `/opt/claude/opendog`; if you run from another directory such as `/opt/claude/mystocks_spec`, explicitly target the OpenDog source path and do not treat the current project as the OpenDog source tree.
- The human maintainer or an explicitly authorized local operations agent should run it.
- The command updates the OpenDog release binary, not other projects' source code.

The build command runs `cargo build --release` against the explicit `--source` path.

Then restart or reconnect Claude Code, Codex CLI, or the MCP host that uses OpenDog. A currently running MCP server process does not hot-reload when the file on disk changes. OpenDog must not kill the MCP process, restart the host, or edit `.claude.json` / Codex MCP config automatically.

For multi-project use, prefer configuring every project to point at the same release binary:

```text
/opt/claude/opendog/target/release/opendog
```

If a project uses a copied binary such as `/opt/claude/<project>/bin/opendog`, it will not receive OpenDog updates automatically. Either copy the rebuilt binary to that project-specific path, or update the MCP config to point to the shared release binary above.

**Current MCP Tool Surface:**

OPENDOG currently exposes 19 MCP tools:

- `register_project`, `take_snapshot`, `start_monitor`, `stop_monitor`, `get_stats`, `get_unused_files`, `list_projects`, `delete_project`
- `get_time_window_report`, `compare_snapshots`, `get_usage_trends`
- `get_global_config`, `get_project_config`
- `get_guidance`, `get_verification_status`, `record_verification_result`, `run_verification_command`
- `get_data_risk_candidates`, `get_workspace_data_risk_overview`

Detailed request and response shapes live in [docs/mcp-tool-reference.md](/opt/claude/opendog/docs/mcp-tool-reference.md).

**Current Read-Only MCP Resources:**

Use resources when an MCP client only needs stable state and no operation should run:

- `opendog://projects` — registered project-list state as JSON
- `opendog://project/{id}/verification` — latest recorded verification status for one project

Tools remain the right surface for registration, snapshots, monitoring, verification execution, deletion, export, and cleanup.

### Using OPENDOG from External Projects

Treat OPENDOG as an external tool with two supported integration surfaces:

- `MCP stdio` for MCP-capable hosts, IDEs, agents, and orchestration runtimes
- `CLI` for scripts, CI jobs, cron tasks, and operator workflows

Do **not** treat these as stable external interfaces:

- files under `~/.opendog/` or `$OPENDOG_HOME`
- SQLite databases
- `daemon.sock`
- internal Rust modules or crate APIs

#### Recommended Integration Pattern

- If your external project supports MCP, use `opendog mcp`
- If your external project does not support MCP, invoke the `opendog` CLI directly
- For stable cross-session reuse, always set a fixed `OPENDOG_HOME`

#### Multi-Project MCP Isolation

OpenDog supports multiple projects and MCP hosts sharing one daemon when they use the same fixed `OPENDOG_HOME`: `registry.db` stores unique project ids, `daemon.sock` is shared by MCP sessions, the daemon keeps one monitor handle per project id, repeated `start_monitor` returns `already_running`, and each project writes its own SQLite file at `$OPENDOG_HOME/data/projects/<project_id>.db`.

Storage layout: `$OPENDOG_HOME/config.json`, `$OPENDOG_HOME/data/registry.db`, `$OPENDOG_HOME/data/daemon.sock`, `$OPENDOG_HOME/data/daemon.pid`, and `$OPENDOG_HOME/data/projects/<project_id>.db`. SQLite uses WAL and a busy timeout for normal concurrent CLI/MCP reads and daemon writes. If `OPENDOG_HOME` is not set, OpenDog falls back to `$HOME/.opendog/`; for multi-project MCP use, set the same absolute `OPENDOG_HOME` in every host config.

#### Integration Boundary Hint

- Prefer observation-first calls: `register_project`, `take_snapshot`, `start_monitor`, `get_guidance`, reports, stats, data-risk, and verification-status.
- For read-only state in MCP hosts, prefer `opendog://projects` and `opendog://project/{id}/verification` when available.
- OPENDOG does not normally modify project source. Extra attention is needed for `run_verification_command`, which runs your command in the project root, and `export`, which writes artifacts to your chosen output path.

Recommended generic MCP host config:

```json
{
  "mcpServers": {
    "opendog": {
      "command": "/absolute/path/to/opendog",
      "args": ["mcp"],
      "env": {
        "OPENDOG_HOME": "/absolute/path/to/opendog-state"
      }
    }
  }
}
```

#### Minimal MCP Call Sequence

`register_project` is a one-time registration step for a new external project, not a command you should call on every MCP session.

For a project that has not been registered into OPENDOG yet, the safe default sequence is:

1. `register_project`
2. `take_snapshot`
3. `start_monitor`
4. `get_guidance`

Typical request examples:

`register_project`
```json
{
  "id": "demo",
  "path": "/absolute/path/to/project"
}
```

`take_snapshot`
```json
{
  "id": "demo"
}
```

`start_monitor`
```json
{
  "id": "demo"
}
```

After a project has already been registered, later MCP sessions usually skip `register_project` and go straight to one of these read or runtime surfaces:

- `opendog://projects` to read registered projects without invoking a tool
- `opendog://project/{id}/verification` to read latest verification evidence without invoking a tool
- `get_guidance` for the recommended next action
- `get_stats` for hot files
- `get_unused_files` for cleanup candidates
- `get_time_window_report`, `compare_snapshots`, `get_usage_trends` for report-style observation
- `get_verification_status` and `get_data_risk_candidates` for readiness and data-risk checks

Source-first observation examples for AI-assisted repositories:

```json
get_stats {"id":"mystocks","path_classification":"source","limit":50}
get_unused_files {"id":"mystocks","path_classification":"source","limit":50}
get_stats {"id":"mystocks","path_classification":"infrastructure","limit":10}
```

`path_classification` accepts `all`, `source`, `infrastructure`, `backup`, and `project`. Filtering changes the returned `files` window, not the full project counts or `classification_summary`; infrastructure evidence such as `.claude/` remains available when requested.

#### CLI Usage for Scripts and CI

If your external project cannot speak MCP, call the CLI instead:

```bash
opendog register --id demo --path /absolute/path/to/project
opendog snapshot --id demo
opendog stats --id demo
opendog list
```

Important behavior notes:

- `opendog start --id <ID>` is a blocking foreground command
- `opendog mcp` is the preferred long-lived integration surface for external hosts
- config mutation, evidence export, and retained-data cleanup are currently CLI-only operator flows

### Project Exchange Reports

OpenDog keeps cross-project usage feedback here so reports do not scatter across target projects.

Core files:

- Directory guide: [docs/project-exchange/README.md](/opt/claude/opendog/docs/project-exchange/README.md)
- Shared issue index: [docs/project-exchange/issues/INDEX.md](/opt/claude/opendog/docs/project-exchange/issues/INDEX.md)
- Feedback template: [docs/project-exchange/templates/OPENDOG_USAGE_FEEDBACK_TEMPLATE.md](/opt/claude/opendog/docs/project-exchange/templates/OPENDOG_USAGE_FEEDBACK_TEMPLATE.md)

Archived project reports:

- [docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md) - migrated `mystocks_spec` feedback.
- [docs/project-exchange/reports/mystocks/opendog-mcp-retest-results-2026-05-11.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/opendog-mcp-retest-results-2026-05-11.md) - mystocks Case H / Case I PASS retest results.
- [docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md) - mystocks source-signal observation calibration plan.
- [docs/project-exchange/reports/mystocks/source-first-observation-filter-retest-handoff-2026-05-11.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/source-first-observation-filter-retest-handoff-2026-05-11.md) - mystocks source-first filter retest handoff.
- [docs/project-exchange/reports/mystocks/opendog-retest-handoff-2026-05-11.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/opendog-retest-handoff-2026-05-11.md) - mystocks Case H / Case I retest handoff.
- [docs/project-exchange/reports/mystocks/opendog-change-summary-2026-05-11.md](/opt/claude/opendog/docs/project-exchange/reports/mystocks/opendog-change-summary-2026-05-11.md) - OpenDog-side change summary before mystocks retest.
- [docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md](/opt/claude/opendog/docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md) - `quantix-rust` MCP report.

For new reports, start from the template and save under `docs/project-exchange/reports/<project>/` so project A's report and OpenDog response stay one-to-one. If the same issue applies to A/B/C, assign an `ODX-YYYYMMDD-<slug>` id in `docs/project-exchange/issues/INDEX.md`, link each project report, and update both the shared index and origin report when fixed/deferred/rejected.

Report hygiene:

- Create or update `.planning/task-cards/` for product follow-up.
- Update root `CHANGELOG.md` for substantial OpenDog changes.
- Record exact MCP or CLI calls, binary path, `OPENDOG_HOME`, and relevant host version.
- Do not paste secrets, tokens, private business data, or full raw payloads unless they are required evidence and safe to store.
- Treat `unused` as evidence-window-limited; never interpret it as proof that a file is safe to delete.

Large MCP payload retest:

1. Call `get_stats {"id":"<project>"}`.
2. Call `get_stats {"id":"<project>","limit":50}`.
3. Call `get_unused_files {"id":"<project>"}`.
4. Call `get_unused_files {"id":"<project>","limit":50}`.
5. Call `get_stats {"id":"<project>","path_classification":"source","limit":50}` for source-first view validation.
6. Confirm `files.length <= limit`, `result_window.limit == limit`, and `result_window.truncated` reports whether filtered counts exceed returned rows.
7. Remember that `summary`, `unused_count`, and `classification_summary` may still report full project totals by design.
8. If a host still receives MB-scale output, capture the configured MCP command, connected binary path/process, `OPENDOG_HOME`, and raw response envelope in the project report.

MCP resource-discovery retest:

1. Restart or reconnect the MCP host after any OpenDog rebuild.
2. Confirm initialize capabilities include `resources`.
3. Confirm `resources/list` returns `opendog://projects`.
4. Confirm `resources/templates/list` returns `opendog://project/{id}/verification`.
5. Confirm `resources/read` works for `opendog://projects` and `opendog://project/<project>/verification`.
6. If the host still shows no resources, capture initialize capabilities, `resources/list` raw response, binary path/process, and host version.

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
| `Invalid path` | Ensure directory exists before `register` |
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
