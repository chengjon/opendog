---
title: "Add manual self-update workflow"
id: "TASK-20260511-manual-self-update-workflow"
status: completed
owner: "unassigned"
priority: medium
phase_hint: "Operator workflow hardening"
ft_ids_touched:
  - FT-02.01.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-02.01.01 owns the operator CLI surface; manual update status/build commands belong to local operations, not MCP automation."
  - "FT-03.07.01 owns authority boundaries; update output must state that MCP hosts need reconnect and OpenDog will not kill processes or edit host configs."
requirement_ids:
  - CLI-01
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - cli
non_goals:
  - "Do not add MCP update tools."
  - "Do not kill or restart `opendog mcp`, Claude Code, Codex CLI, or any MCP host."
  - "Do not edit `.claude.json`, Codex config, or any host MCP configuration."
  - "Do not modify business-project source trees."
  - "Do not fetch remote code or perform network update checks."
verification_plan:
  - "Add tests for source path validation, status payload shape, rebuild detection, and cargo command construction."
  - "Run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`."
  - "Run `python3 scripts/validate_task_cards.py`, `python3 scripts/validate_planning_governance.py`, and `git diff --check`."
evidence_outputs:
  - "Implementation plan: `.planning/implementation-plans/manual-self-update-workflow-2026-05-11.md`."
  - "Updated QUICKSTART and CLI docs showing where and by whom update commands are run."
  - "CHANGELOG entry before closure."
  - "Manual smoke: `cargo run -- self-update build --source /opt/claude/opendog --json` completed release build."
  - "Manual smoke: `target/release/opendog self-update status --source /opt/claude/opendog --json` returned `needs_rebuild=false`."
---

## Goal

Provide explicit CLI-only update assistance so maintainers can check whether OpenDog needs a rebuild and run a local release build safely.

## Operator Boundary

This is a WSL/Linux shell maintenance workflow for the OpenDog repository. It may be run from any current directory, but it must receive `--source /opt/claude/opendog` or another explicit OpenDog source path.

## Change Plan

1. Add `opendog self-update status --source <opendog-source> [--json]`. Completed.
2. Add `opendog self-update build --source <opendog-source> [--json]`. Completed.
3. Validate that `--source` points to an OpenDog source tree. Completed.
4. Report current executable path, release binary path, mtimes, `needs_rebuild`, and `restart_required_for_mcp`. Completed.
5. Run `cargo build --release` only for the explicit source path. Completed.
6. Document that MCP hosts must be restarted/reconnected after a successful build. Completed.

## Verification Evidence

- `cargo test core::self_update`
- `cargo test cli`
- `cargo run -- self-update status --source /opt/claude/opendog --json`
- `cargo run -- self-update build --source /opt/claude/opendog --json`
- `target/release/opendog self-update status --source /opt/claude/opendog --json`

## Guardrails

- Keep this CLI-only.
- Keep `--source` explicit; do not infer business project roots as OpenDog source.
- Build locally only; do not pull, fetch, download, or self-replace from the network.
- Never restart or edit MCP hosts.

## Completion Criteria

- Maintainer can run status/build from OpenDog or another project directory without ambiguity.
- Output clearly states next steps and MCP reconnect requirement.
- Tests cover status/build behavior without requiring a real release build in normal unit tests.
