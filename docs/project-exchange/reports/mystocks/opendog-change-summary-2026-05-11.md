# OpenDog Change Summary - 2026-05-11

Purpose: summarize the OpenDog-side changes that matter before mystocks retests MCP Case H and Case I.

## Main Outcomes

- FD attribution credibility is governed by `openspec/specs/fd-attribution/spec.md`, originating from archived change `openspec/changes/archive/2026-05-28-fix-fd-attribution`.
- MCP `get_stats` and `get_unused_files` now expose bounded file rows with `result_window` metadata.
- MCP read-only Resources are implemented for project list and per-project verification state.
- Daemon IPC integrity errors now distinguish empty/truncated daemon responses from normal serialization errors.
- UTF-8 content preview handling is hardened for guidance and mock-data detection.
- Source, infrastructure, backup, and project file classification is exposed in stats/unused payloads and CLI output.
- Source-first `path_classification` filters are implemented for MCP and CLI stats/unused views.
- Project feedback now lives under `docs/project-exchange/` with shared issue routing.

## Retest-Relevant Files

- `docs/project-exchange/reports/mystocks/opendog-retest-handoff-2026-05-11.md`
- `docs/project-exchange/reports/mystocks/source-first-observation-filter-retest-handoff-2026-05-11.md`
- `docs/project-exchange/issues/INDEX.md`
- `QUICKSTART.md`
- `docs/mcp-tool-reference.md`
- `docs/json-contracts.md`
- `FIELD_NOTES.md`

## Verification Gates

Passed on OpenDog side:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/validate_planning_governance.py`
- `openspec validate fix-fd-attribution` at implementation time; after archive, validate the retained contract with `openspec validate --specs --strict`
- `cargo build --release`

Release binary checked for MCP retest:

- `/opt/claude/opendog/target/release/opendog`
- built `2026-05-11 16:35:05 +0800`

## mystocks Next Step

Return to mystocks, restart or reconnect Claude Code MCP, then follow:

- `docs/project-exchange/reports/mystocks/opendog-retest-handoff-2026-05-11.md`
- `docs/project-exchange/reports/mystocks/source-first-observation-filter-retest-handoff-2026-05-11.md`

If Case H or Case I still fails, capture the configured MCP command, binary path/process, `OPENDOG_HOME`, host version, and raw MCP response envelope.
