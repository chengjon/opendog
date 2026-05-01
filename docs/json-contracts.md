# JSON Contracts

## Purpose

This document defines the recommended machine-consumption contract for OPENDOG JSON outputs.

It is not a formal schema registry. It tells downstream agents and scripts which fields should drive decisions first, which are explanatory, and which remain advisory.

Authority rule: `git`, tests, lint, and build remain external truth sources. Treat OPENDOG output as decision-support evidence and switch to shell or project-native validation when confirmation is required.

## Quick Navigation

- Product framing: [Positioning](./positioning.md)
- Capability-to-command map: [Capability Index](./capability-index.md)
- AI workflow and shell handoff: [AI Playbook](./ai-playbook.md)
- MCP request/response usage: [MCP Tool Reference](./mcp-tool-reference.md)

## Scope

Current CLI JSON entry points include `decision-brief`, `agent-guidance`, `config show/set/reload`, `report window/compare/trend`, `cleanup-data`, `workspace-data-risk`, `data-risk`, `verification`, `record-verification`, and `run-verification` with `--json`.

Related MCP entry points follow the same versioned-contract pattern: `get_decision_brief`, `get_agent_guidance`, `create_project`, config get/update/reload, evidence export, monitor start/stop, project list/delete, snapshot/stats/unused/report/compare/trend, cleanup, verification status/record/run, and data-risk overview tools.

When the daemon is live, treat CLI and MCP as entry surfaces that may reuse daemon-owned project state through the local control plane rather than standing up parallel monitor ownership.

Current MCP-only versioned utility outputs:

- `get_agent_guidance`: `guidance.schema_version = opendog.mcp.guidance.v1`
- `get_decision_brief`: `schema_version = opendog.mcp.decision-brief.v1`
- `create_project`: `schema_version = opendog.mcp.create-project.v1`
- `get_global_config`: `schema_version = opendog.mcp.global-config.v1`
- `get_project_config`: `schema_version = opendog.mcp.project-config.v1`
- `update_global_config`: `schema_version = opendog.mcp.update-global-config.v1`
- `update_project_config`: `schema_version = opendog.mcp.update-project-config.v1`
- `reload_project_config`: `schema_version = opendog.mcp.reload-project-config.v1`
- `export_project_evidence`: `schema_version = opendog.mcp.export-project-evidence.v1`
- `start_monitor`: `schema_version = opendog.mcp.start-monitor.v1`
- `stop_monitor`: `schema_version = opendog.mcp.stop-monitor.v1`
- `list_projects`: `schema_version = opendog.mcp.list-projects.v1`
- `delete_project`: `schema_version = opendog.mcp.delete-project.v1`
- `take_snapshot`: `schema_version = opendog.mcp.snapshot.v1`
- `get_stats`: `schema_version = opendog.mcp.stats.v1`
- `get_unused_files`: `schema_version = opendog.mcp.unused-files.v1`
- `get_time_window_report`: `schema_version = opendog.mcp.time-window-report.v1`
- `compare_snapshots`: `schema_version = opendog.mcp.snapshot-compare.v1`
- `get_usage_trends`: `schema_version = opendog.mcp.usage-trends.v1`
- `cleanup_project_data`: `schema_version = opendog.mcp.cleanup-project-data.v1`

## Contract Style

Always check `schema_version` first.

If the version is unknown, treat the payload as unsupported rather than guessing from field names. For MCP tools, this applies to both success and error responses. If `status = error`, branch on `error_code` first and only use freeform `error` text for display or debugging.

For scoped AI guidance tools, prefer explicit scope controls instead of post-filtering client-side:

- `get_agent_guidance({ "project_id": "<ID>", "top": N })`
- `get_decision_brief({ "project_id": "<ID>", "top": N })`

### Primary decision fields

Use these as the main machine-input signals for ranking, branching, and choosing the next command.

### Explanatory fields

Use these to justify or refine a decision after the primary fields narrow the path.

### Advisory context

Use these as supporting metadata only. Do not make high-risk decisions from them alone.

## `opendog agent-guidance --json`

Version marker:

- `guidance.schema_version = opendog.mcp.guidance.v1`

### Primary decision fields

- `guidance.recommended_flow`
- `guidance.layers.execution_strategy.global_strategy_mode`
- `guidance.layers.execution_strategy.preferred_primary_tool`
- `guidance.layers.execution_strategy.preferred_secondary_tool`
- `guidance.layers.workspace_observation.projects_missing_snapshot`
- `guidance.layers.workspace_observation.projects_missing_verification`
- `guidance.layers.workspace_observation.projects_with_stale_snapshot`
- `guidance.layers.workspace_observation.projects_with_stale_verification`
- `guidance.layers.multi_project_portfolio.priority_candidates`
- `guidance.layers.multi_project_portfolio.attention_queue`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_score`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_band`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_reasons`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].priority_basis`
- `guidance.layers.storage_maintenance.priority_projects`

### Explanatory fields

- `guidance.notes`
- `guidance.project_recommendations[*].reason`
- `guidance.project_recommendations[*].recommended_next_action`
- `guidance.project_recommendations[*].recommended_flow`
- `guidance.project_recommendations[*].verification_gate_levels.*`
- `guidance.project_recommendations[*].repo_truth_gaps`
- `guidance.project_recommendations[*].mandatory_shell_checks`
- `guidance.project_recommendations[*].execution_sequence`
- `guidance.layers.execution_strategy.projects_with_repo_truth_gaps`
- `guidance.layers.execution_strategy.repo_truth_gap_distribution`
- `guidance.layers.execution_strategy.mandatory_shell_check_examples`
- `guidance.layers.execution_strategy.projects_requiring_repo_stabilization`
- `guidance.layers.execution_strategy.repo_stabilization_priority_projects`
- `guidance.layers.workspace_observation.projects_with_storage_maintenance_candidates`
- `guidance.layers.multi_project_portfolio.project_overviews[*].observation.coverage_state`
- `guidance.layers.multi_project_portfolio.project_overviews[*].observation.freshness`
- `guidance.layers.multi_project_portfolio.project_overviews[*].observation.evidence_gaps`
- `guidance.layers.multi_project_portfolio.project_overviews[*].attention_score`
- `guidance.layers.multi_project_portfolio.project_overviews[*].attention_band`
- `guidance.layers.multi_project_portfolio.project_overviews[*].attention_reasons`
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.risk_findings`
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.highest_priority_finding`
- `guidance.layers.multi_project_portfolio.project_overviews[*].repo_status_risk.finding_counts`
- `guidance.layers.multi_project_portfolio.project_overviews[*].verification_gate_levels.*`
- `guidance.layers.storage_maintenance`
- `guidance.layers.verification_evidence`
- `guidance.layers.verification_evidence.cleanup_gate_distribution`
- `guidance.layers.verification_evidence.refactor_gate_distribution`
- `guidance.layers.constraints_boundaries`

### Advisory context

- `guidance.example_commands`
- `guidance.when_to_use_shell`
- `guidance.when_to_use_opendog`

### Recommended consumption pattern

1. Check `guidance.schema_version`.
2. Read `guidance.recommended_flow`.
3. Take the first item in `guidance.layers.multi_project_portfolio.priority_candidates`.
4. Use its `attention_score`, `attention_band`, `recommended_next_action`, and `recommended_flow` to choose the next command.
5. Read `guidance.layers.workspace_observation` to distinguish missing evidence from stale evidence before acting on any recommendation.
6. If `guidance.layers.storage_maintenance.priority_projects` is non-empty, consider a retained-evidence `cleanup-data --dry-run` pass before long cleanup/refactor sessions.
7. Read verification gate levels before broad modification: `allow` means ready, `caution` means advisory gaps remain, `blocked` means stop and fix evidence first.
8. Read constraints fields after verification so repo-risk blockers can further narrow what is safe.

## `opendog decision-brief --json`

Version marker:

- `schema_version = opendog.cli.decision-brief.v1`
- MCP equivalent: `schema_version = opendog.mcp.decision-brief.v1`

### Primary decision fields

- `scope`
- `decision.recommended_next_action`
- `decision.target_project_id`
- `decision.strategy_mode`
- `decision.action_profile`
- `decision.risk_profile`
- `entrypoints.next_mcp_tools`
- `entrypoints.next_cli_commands`
- `entrypoints.selection_reasons`
- `entrypoints.execution_templates`
- `decision.signals`
- `decision.signals.storage_maintenance_candidate`
- `decision.signals.storage_reclaimable_bytes`
- `decision.signals.attention_score`
- `decision.signals.attention_band`
- `decision.signals.attention_reasons`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.repo_risk_findings`
- `decision.risk_profile.repo_risk_finding_counts`

### Explanatory fields

- `decision.summary`
- `decision.reason`
- `decision.recommended_flow`
- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`
- `decision.execution_sequence`
- `decision.safe_for_cleanup`
- `decision.safe_for_refactor`
- `decision.verification_status`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.cleanup_gate_level`
- `decision.risk_profile.refactor_gate_level`

### Advisory context

- `top`
- `selected_project_id`
- `layers`

Key layer fields worth checking first:

- `layers.workspace_observation.projects_missing_snapshot`
- `layers.workspace_observation.projects_with_stale_snapshot`
- `layers.workspace_observation.projects_missing_verification`
- `layers.workspace_observation.projects_with_stale_verification`
- `layers.multi_project_portfolio.project_overviews[*].observation.coverage_state`
- `layers.multi_project_portfolio.project_overviews[*].observation.freshness`

### Recommended consumption pattern

1. Check top-level `schema_version`.
2. Read `decision.recommended_next_action` and `decision.target_project_id`.
3. Pick from `entrypoints.next_mcp_tools` or `entrypoints.next_cli_commands`.
4. Use `entrypoints.selection_reasons`, `decision.signals.attention_score`, and `decision.signals.attention_reasons` to understand why those entrypoints were chosen.
5. If `decision.signals.storage_maintenance_candidate = true`, inspect the injected retained-evidence `cleanup_project_data` preview template before broader cleanup/refactor work.
6. Use `entrypoints.execution_templates` for argument skeletons, parameter constraints, defaults, placeholders, priorities, run conditions, preconditions, blocking conditions, expected output fields, and follow-up routing.
7. Read `layers.workspace_observation` first so stale or missing evidence can change the execution order.
8. Read `decision.risk_profile.cleanup_gate_level` and `decision.risk_profile.refactor_gate_level` before broad edits; `caution` is advisory-only, while `blocked` means verification evidence is not ready.
9. Read `decision.repo_truth_gaps` before broad edits when repository truth is uncertain; use `decision.mandatory_shell_checks` as the minimum shell handoff set before treating OPENDOG guidance as sufficient.
10. When `decision.recommended_next_action = stabilize_repository_state`, read `decision.execution_sequence` to keep shell stabilization first and refresh OPENDOG guidance only after repo state is stable again.
11. Read the relevant layer in `layers` before making broad edits.
12. Treat this as the unified AI entry envelope, then descend into narrower MCP/CLI tools.

Compatibility rule: `repo_truth_gaps` and `mandatory_shell_checks` are machine-readable boundary projections. Legacy `blind_spots`, `requires_shell_verification`, and human-readable `reason` fields remain available and unchanged.

## `opendog report window --json`

Version marker:

- `schema_version = opendog.cli.time-window-report.v1`
- MCP equivalent: `schema_version = opendog.mcp.time-window-report.v1`

### Primary decision fields

- `window`
- `range`
- `summary.total_sightings`
- `summary.unique_files_accessed`
- `summary.modification_events`
- `files`

### Explanatory fields

- `files[*].access_count`
- `files[*].modification_count`
- `files[*].last_seen_at`
- `files[*].last_modified_at`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Compare the same project across `24h`, `7d`, and `30d`.
3. Use `files` to identify the hottest recent targets.
4. Before cleanup or broad edits, pair this with snapshot comparison or verification evidence.

## `opendog report compare --json`

Version marker:

- `schema_version = opendog.cli.snapshot-compare.v1`
- MCP equivalent: `schema_version = opendog.mcp.snapshot-compare.v1`

### Primary decision fields

- `base_run`
- `head_run`
- `summary.added_files`
- `summary.removed_files`
- `summary.modified_files`
- `changes`

### Explanatory fields

- `changes[*].change_type`
- `changes[*].before`
- `changes[*].after`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Read `summary` first to understand structural change volume.
3. Use `changes` to identify exact paths for shell diff or review.
4. Treat unchanged files as omitted from the change list unless surfaced by `summary`.

## `opendog report trend --json`

Version marker:

- `schema_version = opendog.cli.usage-trends.v1`
- MCP equivalent: `schema_version = opendog.mcp.usage-trends.v1`

### Primary decision fields

- `window`
- `summary.bucket_size`
- `summary.bucket_count`
- `files[*].current_bucket_access_count`
- `files[*].previous_bucket_access_count`
- `files[*].delta_access_count`

### Explanatory fields

- `summary.total_access_count`
- `summary.total_modification_count`
- `files[*].buckets`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Sort attention using `delta_access_count`.
3. Use `buckets` only after `delta_access_count` narrows the candidate set.
4. Pair with `get_time_window_report` when you need a simpler recent summary.

## `opendog cleanup-data --json`

Version marker:

- `schema_version = opendog.cli.cleanup-project-data.v1`
- MCP equivalent: `schema_version = opendog.mcp.cleanup-project-data.v1`

### Primary decision fields

- `scope`
- `dry_run`
- `older_than_days`
- `keep_snapshot_runs`
- `vacuum`
- `deleted`
- `storage_before.approx_reclaimable_bytes`

### Explanatory fields

- `storage_before`
- `storage_after`
- `maintenance`
- `notes`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Run with `dry_run=true` first.
3. Read `deleted` before executing a destructive retained-evidence action.
4. Use `storage_before.approx_reclaimable_bytes` to decide whether an explicit `vacuum` pass is worth the rewrite cost.
5. Treat this as OPENDOG retained-evidence lifecycle only; it does not delete source files.

## `opendog config show --json`

Version marker:

- global scope: `schema_version = opendog.cli.global-config.v1`
- project scope: `schema_version = opendog.cli.project-config.v1`

### Primary decision fields

- `global_defaults`
- `project_overrides`
- `effective`
- `inherits`

### Explanatory fields

- `project_id`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Distinguish global versus project scope first.
3. For project scope, read `effective` before reasoning about monitor behavior.
4. Use `inherits` to decide whether a project is still following global defaults.

## `opendog config set-project --json`

Version marker:

- `schema_version = opendog.cli.update-project-config.v1`

### Primary decision fields

- `project_id`
- `status`
- `project_overrides`
- `effective`
- `reload`

### Explanatory fields

- `global_defaults`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Confirm `status = updated`.
3. Read `reload.runtime_reloaded` before assuming a running monitor changed behavior immediately.
4. Read `effective` after mutation, not just `project_overrides`.

## `opendog config set-global --json`

Version marker:

- `schema_version = opendog.cli.update-global-config.v1`

### Primary decision fields

- `status`
- `global_defaults`
- `reloaded_projects`

### Explanatory fields

- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Confirm `status = updated`.
3. Read `reloaded_projects` to see which running monitors picked up the new defaults already.
4. Use project-scoped config show/reload when any critical project still needs explicit runtime confirmation.

## `opendog config reload --json`

Version marker:

- `schema_version = opendog.cli.reload-project-config.v1`

### Primary decision fields

- `project_id`
- `status`
- `reload`
- `effective`

### Explanatory fields

- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Confirm `status = reloaded`.
3. Read `reload.changed_fields` and `reload.runtime_reloaded`.
4. Treat `effective` as the post-reload truth for later monitoring or cleanup decisions.

## Execution Template Plan Fragments

Each item in `entrypoints.execution_templates` should be treated as a machine-consumable plan fragment, not just a command example.

- `plan_stage`: where this step belongs in the broader flow, such as `observe`, `inspect`, `analyze`, `verify`, or `decide`
- `terminality`: whether the step is `non_terminal`, a `decision_gate`, or `terminal_on_success`
- `can_run_in_parallel`: whether the step can safely run beside other non-conflicting inspection steps
- `requires_human_confirmation`: whether an AI should ask for explicit confirmation before executing the step
- `evidence_written_to_opendog`: whether success is expected to persist new evidence inside OPENDOG
- `retry_policy`: retry envelope with `allowed`, `max_attempts`, `strategy`, and `retry_when`

Example shape:

```json
{
  "template_id": "verification.execute",
  "kind": "mcp_tool",
  "plan_stage": "verify",
  "terminality": "non_terminal",
  "can_run_in_parallel": false,
  "requires_human_confirmation": true,
  "evidence_written_to_opendog": true,
  "retry_policy": {
    "allowed": true,
    "max_attempts": 2,
    "strategy": "rerun_once_after_fix_or_command_adjustment",
    "retry_when": ["verification command was corrected"]
  }
}
```

## `opendog workspace-data-risk --json`

Version marker:

- `schema_version = opendog.cli.workspace-data-risk.v1`
- `guidance.schema_version = opendog.mcp.guidance.v1`

### Primary decision fields

- `guidance.recommended_flow`
- `guidance.layers.multi_project_portfolio.priority_projects`
- `guidance.layers.workspace_observation.projects_with_hardcoded_candidates`
- `guidance.layers.workspace_observation.total_hardcoded_candidates`
- `matched_project_count`

### Explanatory fields

- `guidance.layers.workspace_observation.rule_groups_summary`
- `guidance.layers.workspace_observation.rule_hits_summary`
- `guidance.layers.multi_project_portfolio.priority_projects[*].priority_reason`
- `guidance.layers.multi_project_portfolio.priority_projects[*].dominant_rule_group`
- `projects[*].top_hardcoded_candidates`
- `projects[*].top_mock_candidates`

### Advisory context

- `project_limit`
- `candidate_type`
- `min_review_priority`

### Recommended consumption pattern

1. Check top-level `schema_version`.
2. Read `guidance.recommended_flow`.
3. Select the first item in `guidance.layers.multi_project_portfolio.priority_projects`.
4. Use `priority_reason` to explain why that project was chosen.
5. Escalate into project-level review after the project is selected.

## `opendog data-risk --json`

Version marker:

- `schema_version = opendog.cli.data-risk.v1`
- `guidance.schema_version = opendog.mcp.guidance.v1`

### Primary decision fields

- `guidance.recommended_flow`
- `hardcoded_data_candidates`
- `mock_data_candidates`
- `mixed_review_files`
- `hardcoded_candidate_count`
- `mock_candidate_count`

### Explanatory fields

- `rule_groups_summary`
- `rule_hits_summary`
- `hardcoded_data_candidates[*].rule_hits`
- `hardcoded_data_candidates[*].matched_keywords`
- `hardcoded_data_candidates[*].suggested_commands`
- `guidance.layers.constraints_boundaries`

### Advisory context

- `candidate_type`
- `min_review_priority`
- `guidance.summary`

### Recommended consumption pattern

1. Check top-level `schema_version`.
2. Read `guidance.recommended_flow`.
3. Inspect `hardcoded_data_candidates` before `mock_data_candidates`.
4. Treat `mixed_review_files` as elevated review targets.
5. Use suggested commands and rule hits to move into shell verification.

## `opendog verification --json`

Version marker:

- `schema_version = opendog.cli.verification-status.v1`
- MCP equivalent: `schema_version = opendog.mcp.verification-status.v1`

### Primary decision fields

- `verification.status`
- `verification.latest_runs`
- `verification.gate_assessment.cleanup.level`
- `verification.gate_assessment.refactor.level`
- `verification.safe_for_cleanup`
- `verification.safe_for_refactor`
- `verification.missing_kinds`

### Explanatory fields

- `verification.summary`
- `verification.gate_assessment.*`
- `verification.cleanup_blockers`
- `verification.refactor_blockers`
- `verification.safe_for_cleanup_reason`
- `verification.safe_for_refactor_reason`
- `verification.failing_runs`

Compatibility rule: `verification.safe_for_*` stays compatible with required-gate pass/fail semantics and can remain `true` when `verification.gate_assessment.*.level = "caution"`. Legacy blocker arrays stay blocker-only; advisory caution reasons live under `verification.gate_assessment.*.reasons`.

### Recommended consumption pattern

1. Check top-level `schema_version`.
2. Read `verification.gate_assessment.cleanup.level` and `verification.gate_assessment.refactor.level` first.
3. Treat `allow` as ready, `caution` as advisory-only, and `blocked` as stop-and-fix.
4. Read `verification.safe_for_*` next when you need the legacy boolean contract.
5. If a gate is blocked, read blocker and reason fields before editing.
6. Use `missing_kinds`, `failing_kinds`, `stale_kinds`, and `next_steps` under `verification.gate_assessment.*` to decide what evidence to refresh next.

## `opendog record-verification --json`

Version marker:

- `schema_version = opendog.cli.record-verification.v1`
- MCP equivalent: `schema_version = opendog.mcp.record-verification.v1`

### Primary decision fields

- `recorded.kind`
- `recorded.status`
- `recorded.exit_code`
- `recorded.finished_at`

### Explanatory fields

- `recorded.command`
- `recorded.summary`
- `recorded.source`
- `recorded.started_at`

## `opendog run-verification --json`

Version marker:

- `schema_version = opendog.cli.run-verification.v1`
- MCP equivalent: `schema_version = opendog.mcp.run-verification.v1`

### Primary decision fields

- `executed.run.kind`
- `executed.run.status`
- `executed.run.exit_code`
- `executed.run.summary`

### Explanatory fields

- `executed.stdout_tail`
- `executed.stderr_tail`
- `executed.run.command`
- `executed.run.finished_at`

## MCP Service Contracts

Use this section for contract-level MCP rules, not per-tool request/response walkthroughs.

Per-tool MCP request shapes and highlighted response fields live in [mcp-tool-reference.md](./mcp-tool-reference.md).

For MCP service-style tools:

- check top-level `schema_version` first
- branch on `status` second
- then read the tool-specific payload fields

The schema-version mappings for individual MCP tools are already listed earlier in this document under the related CLI/MCP contract sections.

## MCP Error Contract

Versioned MCP tools now keep the same top-level contract on both success and failure paths.

Common error fields:

- `schema_version`
- `status = error`
- `error_code`
- `error`

Project-scoped MCP tools also include:

- `project_id`

Some tools may include extra diagnostic fields such as:

- `remediation`
- request echo fields such as `id` or `requested_path`

Recommended consumption pattern:

1. Check top-level `schema_version`.
2. If `status = error`, branch on `error_code`.
3. Use `project_id` or request echo fields to correlate the failure with the attempted action.
4. Read `remediation` only after the error class is known.

## Stability Guidance

### More stable

These are the fields downstream consumers should prefer first:

- top-level counts
- `recommended_flow`
- `recommended_next_action`
- priority queues and candidate arrays
- explicit risk or readiness booleans and reasons
- `gate_assessment.*.level` and `verification_gate_levels.*`

### Less stable

These may change more as OPENDOG evolves:

- freeform summary strings
- exact wording inside explanatory text
- presentation-oriented arrays intended mainly for humans

## Safe Consumption Rules

- Do not treat any OPENDOG JSON output as proof of safety by itself.
- Do not confuse retained-evidence lifecycle output with repository cleanup authority.
- Do not bypass daemon-coordinated project state with parallel monitoring when a local-control-plane path already exists.
- Prefer verification and repository-state fields before broad edits.
- Prefer `gate_assessment.*.level` before treating `safe_for_* = true` as an all-clear for broad edits.
- Treat heuristic data-risk output as review input, not automatic classification truth.
- Use explanatory fields to justify actions, not to override primary safety signals.

## Related Docs

- [Capability Index](./capability-index.md)
- [AI Playbook](./ai-playbook.md)
- [README](../README.md)
- [CLAUDE.md](../CLAUDE.md)
