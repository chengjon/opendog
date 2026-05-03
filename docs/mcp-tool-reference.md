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

- OPENDOG currently ships 25 MCP tools
- this file is the practical request/response registry for that current inventory
- if an AI only needs one first entrypoint, prefer `get_decision_brief` or `get_agent_guidance`
- versioned contract style and stability guidance are documented separately in `docs/json-contracts.md`

Capability clusters covered here:

- baseline control and observation: project creation, snapshotting, monitor lifecycle, stats, unused files, project listing, deletion
- configuration and export: global/project config inspection and mutation, runtime reload, portable evidence export
- observation and usage evidence: stats, unused files, time windows, snapshot comparison, usage trends
- decision-support: agent guidance, decision brief, workspace and project review prioritization
- retained-evidence lifecycle: cleanup and storage-maintenance signals
- verification and data-risk review: verification status/record/execute plus mock/hardcoded-data review
- runtime coordination: daemon-backed state reuse through the local control plane when available

## Recommended First Stops

Most AI clients should not read this file top-to-bottom in storage/setup order.

Start from the tool that matches the decision you need now, then drill into lower-level setup or control tools only when necessary.

| If you need... | Read this section first | Why |
|---|---|---|
| One stable AI-facing decision envelope | [`get_decision_brief`](#get_decision_brief) | Best first stop when the AI should choose narrower tools from one structured summary |
| A broader "what should I do next?" recommendation | [`get_agent_guidance`](#get_agent_guidance) | Best first stop for workspace or project-level sequencing and evidence gaps |
| Cross-project prioritization | [`get_workspace_data_risk_overview`](#get_workspace_data_risk_overview) | Best first stop when the question is which project deserves attention first |
| Safety before cleanup or refactor | [`get_verification_status`](#get_verification_status) | Best first stop before broad edits or deletion candidates |
| Suspicious files to review | [`get_data_risk_candidates`](#get_data_risk_candidates) | Best first stop for mock / hardcoded / mixed-review candidate inspection |
| Recent activity shape | [`get_time_window_report`](#get_time_window_report) | Best first stop for short-term concentration and active-file review |
| Baseline or inventory change | [`compare_snapshots`](#compare_snapshots) | Best first stop for added / removed / modified file inventory changes |
| Heating / cooling trends | [`get_usage_trends`](#get_usage_trends) | Best first stop for activity momentum over time |
| Retained OPENDOG evidence cleanup | [`cleanup_project_data`](#cleanup_project_data) | Best first stop for storage maintenance without touching source files |
| Project setup and monitor lifecycle | [`create_project`](#create_project), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor) | Use these when OPENDOG state does not exist yet or observation is not running |

## Reading By Cluster

- Decision and prioritization: [`get_agent_guidance`](#get_agent_guidance), [`get_decision_brief`](#get_decision_brief), [`get_workspace_data_risk_overview`](#get_workspace_data_risk_overview)
- Review and safety: [`get_verification_status`](#get_verification_status), [`get_data_risk_candidates`](#get_data_risk_candidates), [`cleanup_project_data`](#cleanup_project_data)
- Observation and reporting: [`get_time_window_report`](#get_time_window_report), [`compare_snapshots`](#compare_snapshots), [`get_usage_trends`](#get_usage_trends), [`get_stats`](#get_stats), [`get_unused_files`](#get_unused_files)
- Setup and lifecycle: [`create_project`](#create_project), [`list_projects`](#list_projects), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor), [`stop_monitor`](#stop_monitor), [`delete_project`](#delete_project)
- Configuration and export: [`get_global_config`](#get_global_config), [`get_project_config`](#get_project_config), [`update_global_config`](#update_global_config), [`update_project_config`](#update_project_config), [`reload_project_config`](#reload_project_config), [`export_project_evidence`](#export_project_evidence)

## `create_project`

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

- begin daemon-backed or local monitoring for one project
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
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `summary.total_files`
- `summary.accessed`
- `summary.unused`
- `files`
- `guidance`
- `guidance.file_recommendations[*].candidate_basis`
- `guidance.file_recommendations[*].candidate_risk_hints`
- `guidance.file_recommendations[*].candidate_priority`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_basis`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_risk_hints`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_priority`

These candidate fields are review aids, not delete/refactor permission fields. Use the parent cleanup/refactor gate state for safety decisions.

## `get_unused_files`

Purpose:

- answer "which snapshot-tracked files have never been observed as accessed?"
- expose cleanup candidates without deleting anything

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `unused_count`
- `files`
- `guidance`
- `guidance.file_recommendations[*].candidate_basis`
- `guidance.file_recommendations[*].candidate_risk_hints`
- `guidance.file_recommendations[*].candidate_priority`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_basis`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_risk_hints`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_priority`

These candidate fields are review aids, not delete/refactor permission fields. Use the parent cleanup/refactor gate state for safety decisions.

## `get_global_config`

Purpose:

- inspect OPENDOG global defaults such as ignore patterns and process whitelist
- understand what newly created or inheriting projects will use by default

Request shape:

```json
{}
```

Useful response fields:

- `schema_version`
- `global_defaults`
- `guidance`

## `get_project_config`

Purpose:

- inspect resolved config for one project
- compare global defaults, project overrides, and effective runtime values

Request shape:

```json
{
  "id": "demo"
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `global_defaults`
- `project_overrides`
- `effective`
- `inherits`
- `guidance`

## `update_global_config`

Purpose:

- change OPENDOG-wide default policy for ignore patterns or process whitelist
- observe which running projects reloaded automatically

Minimal request shape:

```json
{
  "ignore_patterns": [
    ".cache",
    "dist"
  ]
}
```

Extended request shape:

```json
{
  "ignore_patterns": [
    ".cache",
    "dist"
  ],
  "process_whitelist": [
    "claude",
    "codex",
    "node",
    "python"
  ]
}
```

Useful response fields:

- `schema_version`
- `status`
- `global_defaults`
- `reloaded_projects`
- `guidance`

## `update_project_config`

Purpose:

- override ignore patterns or process whitelist for one project
- control whether the project keeps inheriting global defaults

Minimal request shape:

```json
{
  "id": "demo",
  "ignore_patterns": [
    "generated"
  ]
}
```

Extended request shape:

```json
{
  "id": "demo",
  "ignore_patterns": [
    "generated"
  ],
  "process_whitelist": [
    "claude",
    "codex"
  ],
  "inherit_ignore_patterns": false,
  "inherit_process_whitelist": false
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `status`
- `global_defaults`
- `project_overrides`
- `effective`
- `reload`
- `guidance`

## `reload_project_config`

Purpose:

- re-apply persisted project config to a running monitor
- confirm whether runtime state actually changed without restarting the daemon

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
- `reload`
- `effective`
- `guidance`

## `export_project_evidence`

Purpose:

- write project evidence rows into a portable JSON or CSV artifact
- support downstream automation, review handoff, or archival workflows

Request shape:

```json
{
  "id": "demo",
  "format": "json",
  "output_path": "/tmp/demo-evidence.json"
}
```

Extended request shape:

```json
{
  "id": "demo",
  "format": "csv",
  "view": "core",
  "output_path": "/tmp/demo-core.csv",
  "min_access_count": 5
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `status`
- `format`
- `view`
- `output_path`
- `bytes_written`
- `row_count`
- `summary`
- `content`
- `guidance`

## `get_agent_guidance`

Purpose:

- answer "what should I do next overall?"
- answer "what should I do next in this one project?"
- return recommendation ordering, execution strategy, and safety context
- expose the decision-support surface without forcing the AI to guess which narrower tool to call first

Request shapes:

```json
{}
```

Workspace-scoped guidance with the default queue length.

```json
{
  "project_id": "demo"
}
```

Single-project guidance with the default queue length.

```json
{
  "project_id": "demo",
  "top": 3
}
```

Single-project guidance with a shorter recommendation queue.

Useful response fields:

- `guidance.schema_version`
- `guidance.recommended_flow`
- `guidance.project_recommendations`
- `guidance.project_recommendations[*].review_focus`
- `guidance.project_recommendations[*].verification_gate_levels`
- `guidance.project_recommendations[*].repo_truth_gaps`
- `guidance.project_recommendations[*].mandatory_shell_checks`
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_basis`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_risk_hints`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_priority`
- `guidance.layers.execution_strategy`
- `guidance.layers.execution_strategy.cleanup_gate_level`
- `guidance.layers.execution_strategy.refactor_gate_level`
- `guidance.layers.execution_strategy.projects_with_repo_truth_gaps`
- `guidance.layers.execution_strategy.repo_truth_gap_distribution`
- `guidance.layers.execution_strategy.mandatory_shell_check_examples`
- `guidance.layers.execution_strategy.projects_requiring_verification_run`
- `guidance.layers.execution_strategy.projects_requiring_failing_verification_repair`
- `guidance.layers.execution_strategy.projects_requiring_repo_stabilization`
- `guidance.layers.execution_strategy.repo_stabilization_priority_projects`
- `guidance.layers.execution_strategy.projects_requiring_monitor_start`
- `guidance.layers.execution_strategy.projects_requiring_snapshot_refresh`
- `guidance.layers.execution_strategy.projects_requiring_activity_generation`
- `guidance.layers.multi_project_portfolio`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_score`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_band`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_reasons`

`review_focus` and candidate-level `candidate_*` fields are review aids only. They do not replace the parent cleanup/refactor gate fields.
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.risk_findings`
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.highest_priority_finding`
- `guidance.layers.multi_project_portfolio.project_overviews[*].verification_gate_levels`
- `guidance.layers.verification_evidence.cleanup_gate_distribution`
- `guidance.layers.verification_evidence.refactor_gate_distribution`
- `guidance.layers.storage_maintenance`

## `get_decision_brief`

Purpose:

- give the AI one stable decision envelope before it chooses tools
- return the recommended next action, target project, risk profile, and execution templates
- expose one AI-consumable entrypoint into OPENDOG's guidance, evidence, and storage-maintenance layers

Request shapes:

```json
{}
```

Workspace-scoped decision brief with the default queue length.

```json
{
  "project_id": "demo"
}
```

Single-project decision brief with the default queue length.

```json
{
  "project_id": "demo",
  "top": 1
}
```

Single-project decision brief with the shortest queue.

Useful response fields:

- `schema_version`
- `decision.recommended_next_action`
- `decision.target_project_id`
- `decision.action_profile`
- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`
- `decision.execution_sequence`
- `decision.risk_profile`
- `decision.risk_profile.cleanup_gate_level`
- `decision.risk_profile.refactor_gate_level`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.repo_risk_finding_counts`
- `decision.signals.attention_score`
- `decision.signals.attention_band`
- `decision.signals.attention_reasons`
- `decision.signals.storage_maintenance_candidate`
- `decision.signals.storage_reclaimable_bytes`
- `entrypoints.next_mcp_tools`
- `entrypoints.execution_templates`
- `layers`

Read `repo_truth_gaps` before broad edits when repository truth is uncertain. Use `mandatory_shell_checks` as the minimum shell handoff set before treating OPENDOG guidance as sufficient.

When `decision.recommended_next_action = run_verification_before_high_risk_changes`, treat `execution_sequence.verification_commands` as the project-native verification steps to run before refreshing OPENDOG guidance.

When `decision.recommended_next_action = review_failing_verification`, treat `execution_sequence.verification_commands` as the repair-and-rerun command set for the current failing verification path.

When `decision.recommended_next_action = stabilize_repository_state`, treat `execution_sequence` as machine-readable ordering metadata: `strategy_mode` still names the high-level strategy, while `execution_sequence` tells the consumer to stabilize in shell first and refresh OPENDOG guidance after repository state is stable again.

When `decision.recommended_next_action = start_monitor`, treat `execution_sequence.observation_steps` as the observation bootstrap order: enable monitoring first, then let real project activity happen before refreshing OPENDOG guidance.

When `decision.recommended_next_action = take_snapshot`, treat `execution_sequence.observation_steps` as the snapshot refresh order: take or refresh the baseline snapshot first, then refresh OPENDOG guidance after snapshot evidence is fresh again.

When `decision.recommended_next_action = generate_activity_then_stats`, treat `execution_sequence.observation_steps` as the activity-generation order: produce meaningful project activity and refresh stats before asking OPENDOG for the next recommendation.

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

## `cleanup_project_data`

Purpose:

- answer "what retained OPENDOG evidence can I prune safely?"
- let the user or AI selectively delete old activity, verification, or historical snapshot data
- expose retained-evidence lifecycle and storage-hygiene operations without touching source files

Dry-run request shape:

```json
{
  "id": "demo",
  "scope": "activity",
  "older_than_days": 30,
  "dry_run": true
}
```

Snapshot-retention request shape:

```json
{
  "id": "demo",
  "scope": "snapshots",
  "keep_snapshot_runs": 5,
  "dry_run": false
}
```

Compaction request shape:

```json
{
  "id": "demo",
  "scope": "all",
  "older_than_days": 30,
  "keep_snapshot_runs": 5,
  "vacuum": true,
  "dry_run": false
}
```

Useful response fields:

- `schema_version`
- `project_id`
- `scope`
- `dry_run`
- `vacuum`
- `deleted`
- `storage_before`
- `storage_after`
- `maintenance`
- `notes`

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
- `verification.safe_for_cleanup`
- `verification.safe_for_refactor`

Interpretation notes:

- Read `verification.gate_assessment.cleanup.level` and `verification.gate_assessment.refactor.level` first.
- `allow` means required evidence is present and fresh enough for that review mode.
- `caution` means required evidence passed, but advisory evidence is missing or stale.
- `blocked` means required evidence is missing, stale, or failing.
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
- `mock_candidate_count`
- `hardcoded_candidate_count`
- `mixed_review_file_count`
- `mock_data_candidates`
- `hardcoded_data_candidates`
- `guidance`

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
- `guidance.layers.workspace_observation`
- `guidance.layers.multi_project_portfolio.priority_projects`
- `projects`

## Runtime behavior

- MCP tools should be understood as the AI-facing entry surface over shared OPENDOG capabilities, not as a separate ownership layer.
- `get_agent_guidance` and `get_decision_brief` prefer daemon-backed state through the local control plane when the OPENDOG daemon is live.
- `get_time_window_report`, `compare_snapshots`, and `get_usage_trends` also prefer daemon-backed state through the local control plane when the daemon is live.
- `cleanup_project_data` uses the same daemon-first local control path so retained-evidence cleanup applies to daemon-owned project state consistently.
- Other MCP tools use the same daemon-first pattern where remote control support already exists.
- If the daemon is unavailable, MCP falls back to local in-process computation.
- If `project_id` does not exist, the scoped guidance tools return a versioned error payload rather than silently widening to workspace scope.

## Related Docs

- [Capability Index](./capability-index.md)
- [AI Playbook](./ai-playbook.md)
- [JSON Contracts](./json-contracts.md)
- [README](../README.md)
- [CLAUDE.md](../CLAUDE.md)
