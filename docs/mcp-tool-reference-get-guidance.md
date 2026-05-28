# MCP `get_guidance` Reference

This page carries the detailed request shapes, mode guide, response field map, and usage guidance for the `get_guidance` MCP tool. Keep the root [MCP Tool Reference](./mcp-tool-reference.md) as the canonical MCP tool inventory.

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

## Related Docs

- [MCP Tool Reference](./mcp-tool-reference.md)
- [JSON Contracts](./json-contracts.md)
