# Source-First Observation Filter Retest Handoff - mystocks

Date: 2026-05-11

Purpose: provide the OpenDog-side release and exact mystocks MCP retest steps for source-first observation filters.

## OpenDog-Side Status

- Release binary: `/opt/claude/opendog/target/release/opendog`
- Release binary timestamp: `2026-05-11 16:35:05 +0800`
- Related issue: `ODX-20260511-source-signal-observation-calibration`
- Related task card: `.planning/task-cards/TASK-20260511-source-first-observation-views.md`
- Scope: MCP/CLI presentation filters and guidance boundaries only; scanner attribution semantics unchanged.

## What Changed

- MCP `get_stats` and `get_unused_files` accept `path_classification`.
- CLI `opendog stats` and `opendog unused` accept `--path-classification`.
- Accepted values: `all`, `source`, `infrastructure`, `backup`, `project`.
- `result_window.path_classification` reports the active filter.
- `result_window.total_count` and `returned_count` describe the filtered row set.
- `classification_summary` remains full-input, not filtered.
- `get_unused_files` adds `filtered_unused_count` when the active filter is not `all`.
- Guidance now warns that very brief AI/host reads may be missed and that `access_count=0` is not deletion proof.

## Required MCP Retest

Restart or reconnect Claude Code MCP first so it uses the rebuilt binary.

Confirm:

- MCP command points to `/opt/claude/opendog/target/release/opendog`
- Connected binary timestamp is at or after `2026-05-11 16:35:05 +0800`
- `OPENDOG_HOME` is the expected shared OpenDog state directory

Run these MCP calls against mystocks:

```json
get_stats {"id":"mystocks","path_classification":"source","limit":50}
get_unused_files {"id":"mystocks","path_classification":"source","limit":50}
get_stats {"id":"mystocks","path_classification":"infrastructure","limit":10}
get_unused_files {"id":"mystocks","path_classification":"infrastructure","limit":10}
get_stats {"id":"mystocks","path_classification":"backup","limit":10}
```

Expected:

- `files.length <= limit`
- every returned row has matching `files[*].path_classification`
- `result_window.path_classification` equals the requested value
- `result_window.total_count` is filtered count
- `classification_summary` still includes full source/infrastructure/backup/project counts
- source calls return source rows or a bounded empty result with clear metadata
- infrastructure calls still expose `.claude/` or other infrastructure evidence when present
- `get_unused_files` with non-`all` filter includes `filtered_unused_count`
- guidance contains the transient-read / open-descriptor boundary language

## CLI Smoke Retest

Optional but useful from `/opt/claude/mystocks_spec`:

```bash
/opt/claude/opendog/target/release/opendog stats --id mystocks --path-classification source
/opt/claude/opendog/target/release/opendog unused --id mystocks --path-classification source
/opt/claude/opendog/target/release/opendog stats --id mystocks --path-classification infrastructure
```

Expected:

- headers include `filter=source` or `filter=infrastructure`
- output is bounded by the existing CLI row limits
- infrastructure evidence remains visible when requested

## OpenDog Verification Gates

OpenDog side passed:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/validate_task_cards.py`
- `python3 scripts/validate_planning_governance.py`
- `git diff --check`
- `cargo build --release`

## Report Back

Write mystocks results to:

- `/opt/claude/mystocks_spec/docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`
- `/opt/claude/mystocks_spec/docs/project-exchange/issues/INDEX.md`

Then sync the result back to OpenDog:

- `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`
- `docs/project-exchange/issues/INDEX.md`
