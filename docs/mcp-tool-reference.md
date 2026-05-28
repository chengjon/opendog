# MCP Tool Reference

This document gives AI clients and MCP integrators direct request-shape guidance for OPENDOG's AI-facing tools.

Treat MCP as the AI-facing entry surface over OPENDOG's shared capabilities, not as a separate business-capability tree. The same core capabilities are also exposed through CLI and daemon-coordinated runtime operations.

Primary role of this page:

- request/response usage for MCP consumers
- not full product framing; that belongs in [Positioning](./positioning.md)
- `git`, tests, lint, and build are external truth sources; treat OPENDOG output as decision-support evidence and switch to shell or project-native validation when confirmation is required.

## Quick Navigation

- Need current product framing first: [Positioning](./positioning.md)
- Need the fastest capability-to-command map: [Capability Index](./capability-index.md)
- Need AI workflow order and shell handoff rules: [AI Playbook](./ai-playbook.md)
- You are here: `MCP Tool Reference` — request/response usage for OPENDOG's MCP surface
- Need machine-readable output fields: [JSON Contracts](./json-contracts.md)

Current inventory note:

- OPENDOG currently ships 27 MCP tools
- this file is the practical request/response registry for that current inventory
- if an AI only needs one first entrypoint, prefer `get_guidance`
- read-only MCP Resources, CLI-only operator mutations, and versioned contract guidance are documented in the sections and related docs below

## Recommended First Stops

Start from the tool that matches the decision you need now, then drill into lower-level setup or control tools only when necessary.

| If you need... | Read this section first | Why |
|---|---|---|
| One stable AI-facing decision envelope | [`get_guidance`](#get_guidance) with `detail = "decision"` | Best first stop when the AI should choose narrower tools from one structured summary |
| A broader "what should I do next?" recommendation | [`get_guidance`](#get_guidance) with `detail = "summary"` | Best first stop for workspace or project-level sequencing and evidence gaps |
| Cross-project prioritization | [`get_workspace_data_risk_overview`](#get_workspace_data_risk_overview) | Best first stop when the question is which project deserves attention first |
| Safety before cleanup or refactor | [`get_verification_status`](#get_verification_status) | Best first stop before broad edits or deletion candidates |
| Suspicious files to review | [`get_data_risk_candidates`](#get_data_risk_candidates) | Best first stop for mock / hardcoded / mixed-review candidate inspection |
| Recent activity shape | [`get_time_window_report`](#get_time_window_report) | Best first stop for short-term concentration and active-file review |
| Baseline or inventory change | [`compare_snapshots`](#compare_snapshots) | Best first stop for added / removed / modified file inventory changes |
| Heating / cooling trends | [`get_usage_trends`](#get_usage_trends) | Best first stop for activity momentum over time |
| Project setup and monitor lifecycle | [`register_project`](#register_project), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor) | Use these when OPENDOG state does not exist yet or observation is not running |

## Reading By Cluster

- Decision and prioritization: [`get_guidance`](#get_guidance), [`get_workspace_data_risk_overview`](#get_workspace_data_risk_overview)
- Review and safety: [`get_verification_status`](#get_verification_status), [`get_data_risk_candidates`](#get_data_risk_candidates), [`scan_orphans`](#scan_orphans), [`verify_deletion_plan`](#verify_deletion_plan)
- Observation and reporting: [`get_time_window_report`](#get_time_window_report), [`compare_snapshots`](#compare_snapshots), [`get_usage_trends`](#get_usage_trends), [`get_activity_rollups`](#get_activity_rollups), [`get_stats`](#get_stats), [`get_unused_files`](#get_unused_files)
- Governance state: [`create_governance_lane`](#create_governance_lane), [`upsert_governance_node`](#upsert_governance_node), [`get_governance_state`](#get_governance_state), [`close_governance_lane`](#close_governance_lane)
- Setup and lifecycle: [`register_project`](#register_project), [`list_projects`](#list_projects), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor), [`stop_monitor`](#stop_monitor), [`delete_project`](#delete_project)
- Configuration inspection: [`get_global_config`](#get_global_config), [`get_build_info`](#get_build_info), [`get_project_config`](#get_project_config)
- Verification recording: [`record_verification_result`](#record_verification_result), [`run_verification_command`](#run_verification_command)

## Read-Only Resources

Use resources when the client only needs stable state and no operation needs to run.

| URI | Kind | Notes |
|---|---|---|
| `opendog://projects` | static resource | Equivalent project-list state as JSON; tools remain preferred when the client needs normal tool-call flow. |
| `opendog://project/{id}/verification` | resource template | Latest verification status JSON for one project. |

Resources are read-only. Registration, monitoring, snapshots, verification execution, deletion, export, and cleanup remain tool or CLI operations.

If resources are missing, reconnect host; follow `QUICKSTART.md`.

## `register_project`

Purpose:

- register a new project root with OPENDOG
- establish the per-project storage boundary before snapshotting or monitoring

Request shape:

```json
{
  "id": "demo",
  "path": "/abs/path/to/demo"
}
```

Useful response fields:

- `schema_version`
- `id`
- `root_path`
- `status`
- `guidance`

## `list_projects`

Purpose:

- answer "which projects does OPENDOG currently know about?"
- expose per-project status before starting monitors or project-scoped review

Request shape:

```json
{}
```

Useful response fields:

- `schema_version`
- `count`
- `projects`
- `projects[*].id`
- `projects[*].root_path`
- `projects[*].status`
- `projects[*].created_at`
- `guidance`

## `take_snapshot`

Purpose:

- trigger a baseline or refresh scan for one project
- update the file inventory before stats, unused-file review, or reporting

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `total_files`
- `new_files`
- `removed_files`
- `guidance`

## `start_monitor`

Purpose:

- begin daemon-backed monitoring for one project
- ensure OPENDOG starts recording activity and change evidence

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `status`
- `already_running`
- `snapshot_taken`
- `guidance`

## `stop_monitor`

Purpose:

- stop monitoring for one project
- release daemon-owned or local monitor state explicitly

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `status`
- `error`

Expected statuses:

- `stopped`
- `not_running`

## `delete_project`

Purpose:

- remove one project record and its OPENDOG-managed storage
- stop any active monitoring for that project first when needed

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `status`
- `error`

Expected statuses:

- `deleted`
- `not_found`

## `get_stats`

Purpose:

- answer "what is the current accumulated usage picture for this project?"
- expose file-level access, duration, and modification evidence

Request shape:

```json
{
  "id": "demo",
  "limit": 50,
  "path_classification": "source"
}
```

`limit` is optional. MCP defaults to 50 returned file rows so large repositories do not produce unbounded JSON payloads. `path_classification` is optional and defaults to `all`; accepted values are `all`, `source`, `infrastructure`, `backup`, and `project`. Summary fields still describe the full project.

Useful response fields:

- `schema_version`
- `project_id`
- `summary.total_files`
- `summary.accessed`
- `summary.unused`
- `classification_summary.{source_files,infrastructure_files,backup_files,project_files}`
- `result_window.total_count`
- `result_window.returned_count`
- `result_window.limit`
- `result_window.truncated`
- `result_window.path_classification`
- `files[*].path_classification`
- `guidance`

`path_classification` separates source files from AI/tool infrastructure such as `.claude/`, `.amazonq/`, `.cursor/`, `.agents/`, `.zread/`, and backup-file patterns. This is soft classification: files remain visible unless project ignore patterns explicitly exclude them.

When a filter is active, `result_window.total_count` and `returned_count` describe the filtered row set. `classification_summary` still describes the full unfiltered stats input. If a filter matches no rows, `files` is empty and `truncated=false`; this means the selected view is empty, not that the project has no files.

Review-candidate aids: `guidance.file_recommendations[*].candidate_{basis,risk_hints,priority}` plus matching cleanup/refactor candidates. Use parent cleanup/refactor gate state for safety decisions.

## `get_unused_files`

Purpose:

- answer "which snapshot-tracked files have never been observed as accessed?"
- expose cleanup candidates without deleting anything

Request shape:

```json
{
  "id": "demo",
  "limit": 50,
  "path_classification": "source"
}
```

`limit` is optional. MCP defaults to 50 returned file rows. `path_classification` is optional and defaults to `all`; accepted values are `all`, `source`, `infrastructure`, `backup`, and `project`. `unused_count` still reports the full number of unused candidates even when `files` is truncated or filtered.

Useful response fields:

- `schema_version`
- `project_id`
- `unused_count`
- `filtered_unused_count`
- `classification_summary.{source_files,infrastructure_files,backup_files,project_files}`
- `result_window.total_count`
- `result_window.returned_count`
- `result_window.limit`
- `result_window.truncated`
- `result_window.path_classification`
- `files[*].path_classification`
- `guidance`

Unused recommendations prefer source-classified candidates before infrastructure noise, but infrastructure entries remain visible and counted.

When a non-`all` filter is active, `filtered_unused_count` reports the filtered candidate count while `unused_count` remains the full unfiltered unused count. Infrastructure entries remain available with `path_classification=infrastructure`; OpenDog does not hide them globally.

Review-candidate aids: `guidance.file_recommendations[*].candidate_{basis,risk_hints,priority}` plus matching cleanup/refactor candidates. Use parent cleanup/refactor gate state for safety decisions.

## `get_global_config`

Purpose: inspect OPENDOG global defaults such as ignore patterns and process whitelist.
Request shape: `{}`
Useful fields: `schema_version`, `global_defaults`, `guidance`.

## `get_build_info`

Purpose: inspect MCP binary/build metadata, `storage_schema_version`, daemon reachability, OPENDOG home, and rebuild guidance.
Request shape: `{}`
Useful fields: `schema_version`, `storage_schema_version`, `version`, `git_hash`, `build_time`, `binary_path`, `needs_rebuild`, `daemon_running`, `opendog_home`, `rebuild_hint`, `guidance`.
Boundary: `daemon_running` checks OPENDOG daemon reachability only; host-side tool visibility must be checked in the AI host.

## `get_project_config`

Purpose: inspect resolved config for one project and compare global defaults, overrides, and effective runtime values.
Request shape: `{ "id": "demo" }`
Useful fields: `schema_version`, `project_id`, `global_defaults`, `project_overrides`, `effective`, `inherits`, `guidance`.
Operator note: config mutation, config reload, evidence export, and retained-evidence cleanup are CLI-only flows (`opendog config ...`, `opendog export`, `opendog cleanup-data`).

## `get_guidance`

Purpose:

- provide the single MCP guidance entry surface for workspace or project scope
- support both the broader recommendation view and the stable decision-envelope view
- return recommendation ordering, execution strategy, risk signals, and storage-maintenance hints without forcing the AI to guess between multiple MCP guidance tools

Detailed request shapes, schema-version rules, response field maps, and mode-specific guidance live in [MCP `get_guidance` Reference](./mcp-tool-reference-get-guidance.md).

Use this root heading as the canonical MCP tool inventory entry; `get_guidance` remains part of the documented 27-tool MCP surface.

## `get_time_window_report`

Purpose:

- answer "what was recently active in this project?"
- compare recent activity concentration across `24h`, `7d`, and `30d`

Request shape:

```json
{
  "id": "demo"
}
```

Extended request shape:

```json
{
  "id": "demo",
  "window": "7d",
  "limit": 10
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `window`
- `range`
- `summary`
- `files`

## `compare_snapshots`

Purpose:

- answer "what changed between snapshot baselines?"
- expose added, removed, and modified files without requiring a shell diff first

Default request shape:

```json
{
  "id": "demo"
}
```

Explicit run request shape:

```json
{
  "id": "demo",
  "base_run_id": 3,
  "head_run_id": 4,
  "limit": 20
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `base_run`
- `head_run`
- `summary`
- `changes`

## `get_usage_trends`

Purpose:

- answer "which files are heating up or cooling down?"
- expose bucketed usage deltas over `24h`, `7d`, or `30d`

Request shape:

```json
{
  "id": "demo"
}
```

Extended request shape:

```json
{
  "id": "demo",
  "window": "30d",
  "limit": 10
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `window`
- `summary.bucket_size`
- `summary.bucket_count`
- `files`

## `get_activity_rollups`
Purpose: inspect daily aggregate access/modification/event counts preserved after retained raw activity rows are compacted.
Request shape: `{"id":"demo","window":"30d","limit":30}`; `window` and `limit` are optional and default to `30d` / `30`.
Useful response fields: `schema_version`, `project_id`, `window`, `range`, `summary.total_*`, `summary.truncated`, `days`. Daily aggregates cannot reconstruct deleted per-file or per-process rows; use `get_usage_trends` or `get_time_window_report` before cleanup when file-level detail is required.

## `get_verification_status`

Purpose:

- answer "do I already have test/lint/build evidence for this project?"
- expose the latest recorded verification runs before risky edits

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `verification.latest_runs`
- `verification.gate_assessment.cleanup`
- `verification.gate_assessment.refactor`
- `verification.missing_kinds`
- `verification.freshness.policy`
- `verification.gate_assessment.*.freshness_policy`
- `verification.safe_for_cleanup`
- `verification.safe_for_refactor`

Interpretation notes:

- Read `verification.gate_assessment.cleanup.level` and `verification.gate_assessment.refactor.level` first.
- `allow` means required evidence is present and fresh enough for that review mode.
- `caution` means required evidence passed, but advisory evidence is missing or stale.
- `blocked` means required evidence is missing, stale, or failing.
- Default TTL policy is machine-readable: fresh <= 24h, aging <= 7d, stale > 7d.
- `verification.safe_for_cleanup` and `verification.safe_for_refactor` keep the legacy pass/fail contract, so they can still be `true` while the matching gate level is `caution`.
- `verification.cleanup_blockers` and `verification.refactor_blockers` stay blocker-only; advisory-only guidance lives under `verification.gate_assessment.*.reasons` and `verification.gate_assessment.*.next_steps`.

## `record_verification_result`

Purpose:

- persist an externally executed verification result into OPENDOG
- let later guidance and decision layers treat that evidence as recorded

Minimum request shape:

```json
{
  "id": "demo",
  "kind": "test",
  "status": "passed",
  "command": "cargo test"
}
```

Extended request shape:

```json
{
  "id": "demo",
  "kind": "lint",
  "status": "failed",
  "command": "cargo clippy --all-targets --all-features -- -D warnings",
  "exit_code": 1,
  "summary": "1 lint failure",
  "source": "mcp",
  "started_at": "2026-04-26T12:00:00Z"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `recorded.kind`
- `recorded.status`
- `recorded.command`
- `recorded.finished_at`

## `run_verification_command`

Purpose:

- execute a project-native test/lint/build command
- record the result into OPENDOG in one step

Request shape:

```json
{
  "id": "demo",
  "kind": "test",
  "command": "cargo test"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `executed.run.status`
- `executed.run.exit_code`
- `executed.run.summary`
- `executed.stdout_tail`
- `executed.stderr_tail`

## `get_data_risk_candidates`

Purpose:

- answer "which files in this project look like mock/demo/fixture data?"
- answer "which files look like suspicious hardcoded business-like data?"

Minimum request shape:

```json
{
  "id": "demo"
}
```

Filtered request shape:

```json
{
  "id": "demo",
  "candidate_type": "hardcoded",
  "min_review_priority": "medium",
  "limit": 10
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `data_risk_focus`
- `mock_candidate_count`, `hardcoded_candidate_count`, `mixed_review_file_count`
- `mock_data_candidates`, `hardcoded_data_candidates`
- `guidance`
- `data_risk_focus` uses `primary_focus` (`none | mock | hardcoded | mixed`), `priority_order`, and stable `basis` keys such as `hardcoded_candidates_present` or `mixed_review_files_present`

Documentation and template-placeholder findings are down-ranked instead of hidden; inspect `path_classification`, `review_priority`, `confidence`, and `rule_hits` before treating a candidate as runtime risk.

## `get_workspace_data_risk_overview`

Purpose:

- answer "which project should I inspect first across the whole workspace?"
- aggregate mock/hardcoded-data risk across registered projects

Default request shape:

```json
{}
```

Filtered request shape:

```json
{
  "candidate_type": "all",
  "min_review_priority": "medium",
  "project_limit": 5
}
```

Useful response fields:

- `guidance.schema_version`
- `guidance.recommended_flow`
- `guidance.layers.workspace_observation.{data_risk_focus_distribution,projects_requiring_hardcoded_review,projects_requiring_mock_review,projects_requiring_mixed_file_review}`
- `guidance.layers.execution_strategy.{data_risk_focus_distribution,projects_requiring_hardcoded_review,projects_requiring_mock_review,projects_requiring_mixed_file_review}`
- `guidance.layers.multi_project_portfolio.priority_projects`
- `guidance.layers.multi_project_portfolio.priority_projects[*].data_risk_focus`
- `projects`

## `create_governance_lane`

Purpose:

- create a governance work lane for tracking an AI session's intentions and boundaries
- establish a named scope before upserting governance nodes

Request shape:

```json
{
  "id": "demo",
  "lane_id": "refactor-auth-2026w21",
  "title": "Refactor auth module",
  "description": "Track the auth refactor governance boundary for week 21"
}
```

`id` and `lane_id` and `title` are required. `description` is optional.

Useful response fields:

- `schema_version`
- `project_id`
- `lane_id`
- `title`
- `description`
- `status`
- `created_at`

## `upsert_governance_node`

Purpose:

- create or update a governance node within a lane
- persist session state, evidence references, safety boundaries, and suggested next steps

Request shape (create):

```json
{
  "id": "demo",
  "lane_id": "refactor-auth-2026w21",
  "node_id": "pre-edit-checkpoint",
  "state": "planned",
  "summary": "Verify tests pass before starting auth refactor",
  "evidence_refs": ["verification:cargo-test:passed"],
  "artifact_refs": ["src/auth/mod.rs"],
  "reported_git_head": "a1b2c3d",
  "suggested_next": "run cargo clippy before editing",
  "forbidden_scope": ["src/auth/legacy.rs"],
  "external_anchors": ["CI pipeline #4521"]
}
```

`id`, `lane_id`, and `node_id` are required. `state` is required on create. Optional fields: `summary`, `evidence_refs`, `artifact_refs`, `reported_git_head`, `suggested_next`, `forbidden_scope`, `external_anchors`.

Useful response fields:

- `schema_version`
- `project_id`
- `node_id`
- `lane_id`
- `state`
- `created`

## `get_governance_state`

Purpose:

- read governance lanes and nodes for one project
- support narrow queries by lane or node, or broad queries across all active lanes

Minimum request shape:

```json
{
  "id": "demo"
}
```

Filtered request shape:

```json
{
  "id": "demo",
  "lane_id": "refactor-auth-2026w21",
  "active_only": true
}
```

Single-node request shape:

```json
{
  "id": "demo",
  "lane_id": "refactor-auth-2026w21",
  "node_id": "pre-edit-checkpoint"
}
```

`id` is required. Optional: `lane_id`, `node_id`, `active_only`.

Useful response fields:

- `schema_version`
- `project_id`
- `lanes`
- `lanes[*].lane_id`
- `lanes[*].title`
- `lanes[*].status`
- `nodes`
- `nodes[*].node_id`
- `nodes[*].state`
- `nodes[*].summary`
- `nodes[*].suggested_next`
- `nodes[*].forbidden_scope`
- `observation_hints`

## `close_governance_lane`

Purpose:

- close, defer, or delete a governance lane
- cascade the action to all nodes within the lane

Request shape:

```json
{
  "id": "demo",
  "lane_id": "refactor-auth-2026w21",
  "action": "complete"
}
```

`id`, `lane_id`, and `action` are required. Accepted action values: `complete`, `defer`, `delete`.

Useful response fields:

- `schema_version`
- `project_id`
- `lane_id`
- `action_taken`
- `status`
- `nodes_affected`

## `scan_orphans`

Purpose:

- classify orphan cleanup candidates for one project
- combine Rust-internal scanners with optional normalized external scanner reports
- return classified candidates with remove/review/blocked verdicts

Request shape:

```json
{
  "id": "demo",
  "include_internal_scanners": true,
  "limit": 20
}
```

Response fields:

- `schema_version`
- `project_id`
- `status`
- `scanner_health`
- `summary` (total_candidates, remove_candidate_count, review_required_count, blocked_count)
- `candidates`
- `warnings`
- `recommended_next_actions`

## `verify_deletion_plan`

Purpose:

- verify whether proposed deletion targets have enough orphan-detection evidence
- return a safety verdict before human-reviewed deletion

Request shape:

```json
{
  "id": "demo",
  "targets": [
    {
      "subject_kind": "file",
      "subject": "src/deprecated.rs",
      "path": "src/deprecated.rs"
    }
  ]
}
```

Response fields:

- `schema_version`
- `project_id`
- `status`
- `safe_to_plan_deletion`
- `blocked_targets`
- `review_required_targets`
- `remove_candidates`
- `required_project_verification_commands`
- `evidence_gaps`

## Runtime behavior

- MCP tools should be understood as the AI-facing entry surface over shared OPENDOG capabilities, not as a separate ownership layer.
- `opendog mcp` now auto-ensures the OPENDOG daemon is available before serving requests, so normal MCP sessions reuse daemon-backed monitor state across reconnects.
- For stable reuse across different MCP hosts or launcher environments, set `OPENDOG_HOME` to a fixed absolute state directory. If unset, OPENDOG falls back to `HOME/.opendog`.
- `get_guidance` prefers daemon-backed state through the local control plane when the OPENDOG daemon is live, regardless of `detail`.
- `get_time_window_report`, `compare_snapshots`, `get_usage_trends`, and `get_activity_rollups` also prefer daemon-backed state through the local control plane when the daemon is live.
- CLI-only operator flows such as config mutation, evidence export, and retained-evidence cleanup still reuse the same daemon-first local control path where available.
- Other MCP tools use the same daemon-first pattern where remote control support already exists.
- If the daemon is unavailable, MCP falls back to local in-process computation.
- If `project_id` does not exist, `get_guidance` returns a versioned error payload rather than silently widening to workspace scope.

## Related Docs

- [MCP `get_guidance` Reference](./mcp-tool-reference-get-guidance.md)
- [Capability Index](./capability-index.md)
- [AI Playbook](./ai-playbook.md)
- [JSON Contracts](./json-contracts.md)
- [README](../README.md)
- [CLAUDE.md](../CLAUDE.md)
