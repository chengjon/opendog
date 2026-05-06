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

- OPENDOG currently ships 19 MCP tools
- this file is the practical request/response registry for that current inventory
- if an AI only needs one first entrypoint, prefer `get_guidance`
- operator mutations, export artifacts, and retained-evidence cleanup intentionally live on the CLI surface
- versioned contract style and stability guidance are documented separately in `docs/json-contracts.md`

Capability clusters covered here:

- baseline control and observation: project creation, snapshotting, monitor lifecycle, stats, unused files, project listing, deletion
- configuration inspection: global/project config inspection and effective runtime visibility
- observation and usage evidence: stats, unused files, time windows, snapshot comparison, usage trends
- decision-support: merged guidance, workspace and project review prioritization
- storage-maintenance signals: guidance may recommend CLI cleanup flows when retained evidence grows
- verification and data-risk review: verification status/record/execute plus mock/hardcoded-data review
- runtime coordination: daemon-backed state reuse through the local control plane when available

## Recommended First Stops

Most AI clients should not read this file top-to-bottom in storage/setup order.

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
| Project setup and monitor lifecycle | [`create_project`](#create_project), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor) | Use these when OPENDOG state does not exist yet or observation is not running |

## Reading By Cluster

- Decision and prioritization: [`get_guidance`](#get_guidance), [`get_workspace_data_risk_overview`](#get_workspace_data_risk_overview)
- Review and safety: [`get_verification_status`](#get_verification_status), [`get_data_risk_candidates`](#get_data_risk_candidates)
- Observation and reporting: [`get_time_window_report`](#get_time_window_report), [`compare_snapshots`](#compare_snapshots), [`get_usage_trends`](#get_usage_trends), [`get_stats`](#get_stats), [`get_unused_files`](#get_unused_files)
- Setup and lifecycle: [`create_project`](#create_project), [`list_projects`](#list_projects), [`take_snapshot`](#take_snapshot), [`start_monitor`](#start_monitor), [`stop_monitor`](#stop_monitor), [`delete_project`](#delete_project)
- Configuration inspection: [`get_global_config`](#get_global_config), [`get_project_config`](#get_project_config)

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

Review-candidate aids: `guidance.file_recommendations[*].candidate_{basis,risk_hints,priority}` plus matching cleanup/refactor candidates. Use parent cleanup/refactor gate state for safety decisions.

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

Review-candidate aids: `guidance.file_recommendations[*].candidate_{basis,risk_hints,priority}` plus matching cleanup/refactor candidates. Use parent cleanup/refactor gate state for safety decisions.

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

Operator note:

- config mutation, config reload, evidence export, and retained-evidence cleanup are intentionally CLI-only flows
- use `opendog config set-global`, `opendog config set-project`, `opendog config reload`, `opendog export`, and `opendog cleanup-data`

## `get_guidance`

Purpose:

- provide the single MCP guidance entry surface for workspace or project scope
- support both the broader recommendation view and the stable decision-envelope view
- return recommendation ordering, execution strategy, risk signals, and storage-maintenance hints without forcing the AI to guess between multiple MCP guidance tools

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

Single-project summary guidance with a shorter recommendation queue.

```json
{
  "project_id": "demo",
  "detail": "decision",
  "top": 1
}
```

Single-project decision envelope with the shortest queue.

Mode guide:

- omit `detail` or set `detail = "summary"` for the broader "what should I do next?" guidance payload
- set `detail = "decision"` for the stable decision envelope that returns the former decision-brief payload

Schema-version note:

- `detail = "summary"` returns `guidance.schema_version = opendog.mcp.guidance.v1`
- `detail = "decision"` returns top-level `schema_version = opendog.mcp.decision-brief.v1`

Useful response fields when `detail = "summary"`:

- `guidance.schema_version`
- `guidance.recommended_flow`
- `guidance.project_recommendations`
- `guidance.project_recommendations[*].review_focus`
- `guidance.project_recommendations[*].verification_gate_levels`
- `guidance.project_recommendations[*].repo_truth_gaps`
- `guidance.project_recommendations[*].mandatory_shell_checks`
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy`
- `guidance.layers.execution_strategy.review_focus_projection`
- `guidance.layers.execution_strategy.{cleanup_gate_level,refactor_gate_level}`
- `guidance.layers.execution_strategy.{projects_with_repo_truth_gaps,repo_truth_gap_distribution,mandatory_shell_check_examples}`
- `guidance.layers.execution_strategy.risk_strategy_coupling`
- `guidance.layers.execution_strategy.external_truth_boundary`
- `guidance.layers.execution_strategy.{projects_requiring_verification_run,projects_requiring_failing_verification_repair}`
- `guidance.layers.execution_strategy.{projects_requiring_repo_stabilization,repo_stabilization_priority_projects}`
- `guidance.layers.execution_strategy.{projects_requiring_monitor_start,projects_requiring_snapshot_refresh,projects_requiring_activity_generation}`
- `guidance.layers.execution_strategy.{data_risk_focus_distribution,projects_requiring_hardcoded_review,projects_requiring_mock_review,projects_requiring_mixed_file_review}`
- `guidance.layers.multi_project_portfolio`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].{attention_score,attention_band,attention_reasons}`
- `guidance.layers.multi_project_portfolio.attention_batches`
- `guidance.layers.multi_project_portfolio.attention_batches.{batched_project_count,unbatched_project_count}`
- `guidance.layers.multi_project_portfolio.attention_batches.{immediate,next,later}[*].{project_id,recommended_next_action,attention_score,attention_band}`
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.{risk_findings,highest_priority_finding}`
- `guidance.layers.multi_project_portfolio.project_overviews[*].verification_gate_levels`
- `guidance.layers.multi_project_portfolio.project_overviews[*].mock_data_summary.data_risk_focus`
- `guidance.layers.verification_evidence.{cleanup_gate_distribution,refactor_gate_distribution}`
- `guidance.layers.cleanup_refactor_candidates.candidates[*].candidate_{basis,risk_hints,priority}`
- `guidance.layers.storage_maintenance`

`review_focus` and candidate-level `candidate_*` fields are review aids only, not replacements for the parent cleanup/refactor gate fields.

`guidance.layers.execution_strategy.risk_strategy_coupling` is a read-only explanation of how the top workspace repository-risk finding reinforces the current strategy mode and primary tool choice. It does not change the underlying project recommendation logic.

`guidance.layers.execution_strategy.external_truth_boundary` is a read-only top-project boundary projection. It summarizes whether OPENDOG guidance can continue or whether the AI must first switch to direct repository truth or project-native verification truth, using existing `repo_truth_gaps`, `mandatory_shell_checks`, and `execution_sequence.verification_commands`.

`guidance.layers.execution_strategy.review_focus_projection` is a read-only top-project projection of the current review family intent. It does not expose file-level previews or widen the `candidate_*` surface.

`guidance.layers.multi_project_portfolio.attention_batches` is a read-only batching projection derived from `guidance.layers.multi_project_portfolio.attention_queue`; it groups the current queue into `immediate / next / later` handling buckets and is not a scheduling engine.

Useful response fields when `detail = "decision"`:

- `schema_version`
- `decision.recommended_next_action`
- `decision.target_project_id`
- `decision.action_profile`
- `decision.review_focus`
- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`
- `decision.external_truth_boundary`
- `decision.execution_sequence`
- `decision.data_risk_focus`
- `decision.risk_profile`
- `decision.risk_profile.cleanup_gate_level`
- `decision.risk_profile.refactor_gate_level`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.repo_risk_finding_counts`
- `decision.signals.attention_score`
- `decision.signals.attention_band`
- `decision.signals.mixed_review_file_count`
- `decision.signals.attention_reasons`
- `decision.signals.storage_maintenance_candidate`
- `decision.signals.storage_reclaimable_bytes`
- `entrypoints.next_mcp_tools`
- `entrypoints.execution_templates`
- `layers`

Read `decision.external_truth_boundary` before broad edits. If `mode = must_switch_to_external_truth`, use `minimum_external_checks` as the minimum repo or project-native verification handoff before treating OPENDOG guidance as sufficient.

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
- `data_risk_focus`
- `mock_candidate_count`, `hardcoded_candidate_count`, `mixed_review_file_count`
- `mock_data_candidates`, `hardcoded_data_candidates`
- `guidance`
- `data_risk_focus` uses `primary_focus` (`none | mock | hardcoded | mixed`), `priority_order`, and stable `basis` keys such as `hardcoded_candidates_present` or `mixed_review_files_present`

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

## Runtime behavior

- MCP tools should be understood as the AI-facing entry surface over shared OPENDOG capabilities, not as a separate ownership layer.
- `get_guidance` prefers daemon-backed state through the local control plane when the OPENDOG daemon is live, regardless of `detail`.
- `get_time_window_report`, `compare_snapshots`, and `get_usage_trends` also prefer daemon-backed state through the local control plane when the daemon is live.
- CLI-only operator flows such as config mutation, evidence export, and retained-evidence cleanup still reuse the same daemon-first local control path where available.
- Other MCP tools use the same daemon-first pattern where remote control support already exists.
- If the daemon is unavailable, MCP falls back to local in-process computation.
- If `project_id` does not exist, `get_guidance` returns a versioned error payload rather than silently widening to workspace scope.

## Related Docs

- [Capability Index](./capability-index.md)
- [AI Playbook](./ai-playbook.md)
- [JSON Contracts](./json-contracts.md)
- [README](../README.md)
- [CLAUDE.md](../CLAUDE.md)
