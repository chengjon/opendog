# Manual Self-Update Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task after approval. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI-only manual update commands that check and rebuild the OpenDog release binary from an explicit OpenDog source tree.

**Architecture:** Keep update behavior outside MCP. Add a small `core::self_update` module for source validation, status calculation, and build command execution; add `cli::self_update_commands` for command parsing/output. The command never kills processes, edits MCP host config, or updates business projects.

**Tech Stack:** Rust 2021, `clap`, `serde`, `serde_json`, `walkdir`, `std::process::Command`.

---

## Scope

Implement only:

```bash
opendog self-update status --source /opt/claude/opendog
opendog self-update status --source /opt/claude/opendog --json
opendog self-update build --source /opt/claude/opendog
opendog self-update build --source /opt/claude/opendog --json
```

Do not add MCP tools, daemon requests, host restarts, config edits, network downloads, `git pull`, or binary replacement outside `cargo build --release`.

## File Structure

- Create `src/core/self_update.rs`: validation, status structs, mtime scan, cargo build runner.
- Modify `src/core/mod.rs`: export `self_update`.
- Create `src/cli/self_update_commands.rs`: clap subcommands and text/JSON output.
- Modify `src/cli/mod.rs`: add `SelfUpdate { command: SelfUpdateCommand }`.
- Modify `QUICKSTART.md`, `docs/capability-index.md`, `CHANGELOG.md`, `FUNCTION_TREE.md`.
- Test in `src/core/self_update.rs` and `src/cli/self_update_commands.rs`.

## Contract

Status JSON shape:

```json
{
  "schema_version": "opendog.cli.self-update-status.v1",
  "source_path": "/opt/claude/opendog",
  "current_exe": "/opt/claude/opendog/target/release/opendog",
  "release_binary": "/opt/claude/opendog/target/release/opendog",
  "release_binary_exists": true,
  "release_binary_mtime": "2026-05-11T16:35:05+08:00",
  "source_latest_mtime": "2026-05-11T16:30:00+08:00",
  "needs_rebuild": false,
  "restart_required_for_mcp": false,
  "next_steps": []
}
```

Build JSON shape:

```json
{
  "schema_version": "opendog.cli.self-update-build.v1",
  "source_path": "/opt/claude/opendog",
  "command": "cargo build --release",
  "status": "built",
  "exit_code": 0,
  "release_binary": "/opt/claude/opendog/target/release/opendog",
  "restart_required_for_mcp": true,
  "next_steps": ["Restart or reconnect MCP hosts that use this binary."]
}
```

Use Unix seconds for internal tests if timezone formatting is inconvenient; user-facing text may print raw seconds or filesystem debug time if consistent.

## Task 1: Core Status Model

**Files:**

- Create: `src/core/self_update.rs`
- Modify: `src/core/mod.rs`

- [ ] **Step 1: Write failing validation/status tests**

Create tests:

```rust
#[test]
fn rejects_non_opendog_source_path() {
    let dir = tempfile::tempdir().unwrap();
    let err = validate_source_path(dir.path()).unwrap_err();
    assert!(err.to_string().contains("OpenDog source"));
}

#[test]
fn status_marks_rebuild_needed_when_source_is_newer_than_release() {
    let dir = fixture_source_tree();
    let status = self_update_status(dir.path(), fake_current_exe()).unwrap();
    assert!(status.needs_rebuild);
    assert!(status.restart_required_for_mcp);
}
```

Run:

```bash
cargo test core::self_update
```

Expected: FAIL because module/functions do not exist.

- [ ] **Step 2: Implement structs and validation**

Add:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SelfUpdateStatus {
    pub schema_version: &'static str,
    pub source_path: String,
    pub current_exe: String,
    pub release_binary: String,
    pub release_binary_exists: bool,
    pub release_binary_mtime: Option<u64>,
    pub source_latest_mtime: Option<u64>,
    pub needs_rebuild: bool,
    pub restart_required_for_mcp: bool,
    pub next_steps: Vec<String>,
}
```

Validation rule: `source/Cargo.toml` must exist and contain `name = "opendog"`.

- [ ] **Step 3: Implement mtime scan**

Scan `src/`, `Cargo.toml`, `Cargo.lock`, `build.rs` if present. Ignore `target/`, `.git/`, `docs/`, `.planning/`, and `docs/project-exchange/` for rebuild decisions.

- [ ] **Step 4: Run targeted tests**

Run:

```bash
cargo test core::self_update
```

Expected: PASS.

## Task 2: Core Build Runner

**Files:**

- Modify: `src/core/self_update.rs`

- [ ] **Step 1: Write failing command construction tests**

Test that `build_command_for(source)` uses:

```text
program: cargo
args: build --release
current_dir: <source>
```

Run:

```bash
cargo test core::self_update
```

Expected: FAIL because build command helper does not exist.

- [ ] **Step 2: Implement build result and runner**

Add:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SelfUpdateBuildResult {
    pub schema_version: &'static str,
    pub source_path: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub release_binary: String,
    pub restart_required_for_mcp: bool,
    pub next_steps: Vec<String>,
}
```

`run_self_update_build(source)` validates source then runs `cargo build --release` in that directory with inherited stderr/stdout or captured output. Return `status="built"` on zero exit and an `OpenDogError::InvalidInput` or `OpenDogError::Io` on failure.

- [ ] **Step 3: Avoid real builds in unit tests**

Unit tests cover command construction and result shaping only. The actual command is exercised by final manual verification with the current repo.

## Task 3: CLI Subcommands

**Files:**

- Create: `src/cli/self_update_commands.rs`
- Modify: `src/cli/mod.rs`

- [ ] **Step 1: Write CLI parser tests**

Extend existing CLI tests or add new tests asserting these parse:

```bash
opendog self-update status --source /opt/claude/opendog
opendog self-update build --source /opt/claude/opendog --json
```

Run:

```bash
cargo test cli
```

Expected: FAIL before parser exists.

- [ ] **Step 2: Add clap subcommands**

Add:

```rust
#[derive(clap::Subcommand)]
pub(super) enum SelfUpdateCommand {
    Status { #[arg(long)] source: String, #[arg(long)] json: bool },
    Build { #[arg(long)] source: String, #[arg(long)] json: bool },
}
```

Add `Cli::SelfUpdate { command: SelfUpdateCommand }`.

- [ ] **Step 3: Implement output**

Text status must say:

- this is a WSL/Linux shell maintenance command
- source path checked
- release binary path
- whether rebuild is needed
- if rebuilt, restart/reconnect MCP hosts manually
- OpenDog did not kill processes or edit host config

JSON prints the structs from core.

- [ ] **Step 4: Run targeted CLI tests**

Run:

```bash
cargo test cli
```

Expected: PASS.

## Task 4: Docs and Governance

**Files:**

- Modify: `QUICKSTART.md`
- Modify: `docs/capability-index.md`
- Modify: `FUNCTION_TREE.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update QUICKSTART**

Replace future-tense wrapper examples with real commands:

```bash
opendog self-update status --source /opt/claude/opendog
opendog self-update build --source /opt/claude/opendog
```

Keep explicit boundary language: WSL/Linux shell, maintainer-run, not MCP automation.

- [ ] **Step 2: Update capability docs**

Map the feature to `FT-02.01.01` and CLI operator workflows.

- [ ] **Step 3: Update CHANGELOG**

Record new CLI-only manual self-update workflow.

## Final Verification

Run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
python3 scripts/validate_task_cards.py
python3 scripts/validate_planning_governance.py
git diff --check
```

Manual smoke after tests:

```bash
/opt/claude/opendog/target/release/opendog self-update status --source /opt/claude/opendog
```

Do not run `self-update build` in automated tests. If manually run, it must only execute `cargo build --release` in `/opt/claude/opendog` and then tell the operator to reconnect MCP hosts.

## Review Gate

Stop after this plan is reviewed. Do not implement until the user approves.
