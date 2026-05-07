# Review: QUICK_START.md

**Type**: .md / general (quick start guide) | **Perspective**: auto (completeness + consistency + codebase alignment) | **Date**: 2026-05-06

## Summary

Accurate, well-structured quick start guide. All CLI commands, MCP tool names, flag shapes, default values, behavioral claims, and file references verified against the live codebase with one factual omission (`delete_project` missing from MCP tool listing) and minor coverage gaps around `--json` availability.

## Verified

- All 6 referenced documentation files exist (`docs/positioning.md`, `docs/ai-playbook.md`, `docs/mcp-tool-reference.md`, `docs/json-contracts.md`, `docs/capability-index.md`, `README.md`)
- All CLI commands and subcommands (`create`, `snapshot`, `start`, `stop`, `delete`, `list`, `stats`, `unused`, `daemon`, `mcp`, `config show/set-global/set-project/reload`, `export`, `report window/compare/trend`, `cleanup-data`, `agent-guidance`, `decision-brief`, `data-risk`, `workspace-data-risk`, `verification`, `run-verification`, `record-verification`) match clap definitions in `src/cli/mod.rs` and `src/cli/report_commands.rs`
- 19 MCP tools confirmed via `#[tool(` attribute count in `src/mcp/mod.rs` (lines 138-336)
- Default ignore patterns: doc lists `.git`, `node_modules`, `dist`, `target`, `__pycache__` as representative subset; actual list is 22 patterns in `src/config.rs:26-47` -- subset is accurate
- Default process whitelist: `claude`, `codex`, `node`, `python`, `python3`, `gpt`, `glm` -- exact match with `src/config.rs:142-148`
- Cleanup scopes (`activity`, `snapshots`, `verification`, `all`) -- exact match with `CleanupScope` enum in `src/core/retention.rs:13-17`
- Project ID rules (alphanumeric + `-` + `_`, max 64 chars) -- exact match with `src/config/validation.rs:3-9`
- Path validation (absolute, must be directory) -- matches `validate_root_path` in `src/config/validation.rs:11-13`
- Report time windows (`24h`, `7d`, `30d`) -- confirmed in `src/cli/report_commands.rs:18`
- Verification kinds (`test`, `lint`, `build`) -- confirmed in CLI struct fields
- Two-snapshot minimum for `report compare` -- confirmed by error message in `src/core/report/snapshot_compare.rs:13-16`
- Daemon behavior (start prefers daemon, returns immediately; start without daemon blocks terminal) -- consistent with CLI architecture
- Data directory structure (`~/.opendog/` with `data/daemon.pid`, `data/daemon.sock`, `data/registry.db`, `data/projects/<id>.db`) -- consistent with `src/config/paths.rs` function names and test references in `src/cli/mod.rs:374-376`
- `--exit-code` and `--summary` optional flags on `record-verification` -- confirmed in `src/cli/mod.rs:160-163`
- `--min-access-count` on `export` (default 5) -- confirmed in `src/cli/mod.rs:67-68`

## Issues

- [ ] **[MED]** `delete_project` MCP tool missing from Section 7 tool grouping -- Section 7, line 400
      Evidence (codebase): `delete_project` is defined as an MCP tool at `src/mcp/mod.rs:336-345` and listed in CLAUDE.md under "Baseline control tools". The QUICK_START.md "基础控制" group lists 5 tools (`create_project`, `take_snapshot`, `start_monitor`, `stop_monitor`, `list_projects`) but omits `delete_project`, totaling 18 instead of the claimed 19.
      Evidence (document): Searched QUICK_START.md for "delete_project" -- not present. Section 6.1 line 176 shows the CLI `delete` command but the MCP surface in Section 7 does not include its MCP counterpart. The omission is not scoped out or addressed elsewhere.

- [ ] **[LOW]** `--json` flag not mentioned anywhere in the document -- all sections
      Evidence (codebase): `--json` exists on 15 CLI commands (`cleanup-data`, `agent-guidance`, `decision-brief`, `data-risk`, `workspace-data-risk`, `verification`, `run-verification`, `record-verification`, `report window/compare/trend`, `config show/set-global/set-project/reload`) confirmed via `json: bool` fields in `src/cli/mod.rs` and `src/cli/report_commands.rs`.
      Evidence (document): Searched QUICK_START.md for "--json" and "json" -- not found. The document focuses on terminal-readable output only. For a quick start guide this is acceptable, but users who want JSON output for scripting have no indication it exists. A single note in Section 3 or Section 9 would suffice.

- [ ] **[LOW]** Default ignore patterns listed as incomplete representative subset -- Section 6.6, line 353
      Evidence (codebase): `DEFAULT_IGNORE_PATTERNS` in `src/config.rs:26-47` contains 22 patterns including `.cache`, `build`, `.next`, `.nuxt`, `vendor`, `.venv`, `venv`, `.tox`, `.mypy_cache`, `.pytest_cache`, `.gradle`, `.idea`, `.vscode`, `*.pyc`, `.DS_Store`.
      Evidence (document): Line 353 says "默认忽略项已经包含 `.git`、`node_modules`、`dist`、`target`、`__pycache__` 等常见目录" with "等" meaning "etc." This is not wrong, but the full list is not trivially discoverable (it's in source code, not a config file). No pointer to where the full list lives.

## Suggestions

- Add `delete_project` to the "基础控制" group in Section 7 so the listed count matches the actual 19 tools
- Add a one-liner in Section 3 or 9 noting that most reporting and analysis commands support `--json` for machine-readable output
- Consider referencing where to find the complete default ignore pattern list (e.g., `src/config.rs` or `opendog config show`)

## Verdict

APPROVE_WITH_NOTES -- factually accurate with one MCP tool omission and minor discoverability gaps for `--json` and full default patterns. No blocking issues.
