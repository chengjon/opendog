# AI Playbook

## Purpose

This document tells AI agents how to use OPENDOG without guessing.

Treat OPENDOG as a three-layer system:

- observation core: project isolation, snapshot baseline, monitoring, usage evidence, reports, retained-evidence lifecycle
- service delivery and runtime coordination: CLI operator surface, MCP AI surface, daemon runtime, local control plane
- AI decision-support layer: guidance, evidence, boundaries, prioritization, cleanup review, and data-risk review

Use it when you need to decide:

- which project to inspect first
- whether a project is safe for cleanup or refactor
- whether to use MCP, CLI, or shell next
- whether suspicious mock or hardcoded pseudo-data needs review

If you want the shortest command-by-situation lookup first, start with [Capability Index](./capability-index.md).

Primary role of this page:

- workflow order, shell handoff, and safety rules for AI usage
- not full product framing; that belongs in [Positioning](./positioning.md)
- not capability ownership mapping; that belongs in `FUNCTION_TREE` and planning artifacts when structural scope changes

## Quick Navigation

- Need current product framing first: [Positioning](./positioning.md)
- Need the fastest capability-to-command map: [Capability Index](./capability-index.md)
- You are here: `AI Playbook` — recommended execution order, shell handoff, and safety workflow
- Need MCP request/response usage: [MCP Tool Reference](./mcp-tool-reference.md)
- Need machine-readable output fields: [JSON Contracts](./json-contracts.md)

## Core Rule

`git`, tests, lint, and build are external truth sources; treat OPENDOG output as decision-support evidence and switch to shell or project-native validation when confirmation is required.

CLI and MCP are entry surfaces over the same core capabilities. When the OPENDOG daemon is live, prefer the daemon-owned state that those surfaces reuse through the local control plane.

`FUNCTION_TREE`, requirement mappings, and task cards are governance artifacts for capability evolution, not the normal first stop for routine AI operation.

## Default Workflow

1. Make sure the project exists in OPENDOG.
   MCP: `create_project`
   CLI: `opendog create --id <ID> --path <DIR>`
2. Make sure a snapshot exists.
   MCP: `take_snapshot`
   CLI: `opendog snapshot --id <ID>`
3. Make sure monitoring is active when ongoing observation matters.
   MCP: `start_monitor`
   CLI: `opendog start --id <ID>`
4. Ask OPENDOG what kind of action is appropriate.
   MCP: `get_agent_guidance`
   CLI: `opendog agent-guidance`
   MCP/CLI both support project scoping and queue trimming:
   `project_id` or `--project <ID>`
   `top` or `--top <N>`
   If `layers.storage_maintenance.priority_projects` is non-empty, review retained OPENDOG evidence before a long cleanup/refactor pass.
   If you want one stable AI entry envelope first:
   MCP: `get_decision_brief`
   CLI: `opendog decision-brief`
5. If you need recent activity shape, snapshot deltas, or heating/cooling files, use reporting tools.
   MCP: `get_time_window_report`, `compare_snapshots`, `get_usage_trends`
   CLI: `opendog report window|compare|trend`
6. If OPENDOG retained evidence itself has grown too large, use selective cleanup first.
   MCP: `cleanup_project_data`
   CLI: `opendog cleanup-data`
   This is a retained-evidence lifecycle operation, not source cleanup.
   If the cleanup deletes a lot and `storage_before.approx_reclaimable_bytes` stays high, consider one explicit `vacuum` pass.
7. Before cleanup or refactor, check evidence and data-risk.
   MCP: `get_verification_status` then `get_data_risk_candidates`
   CLI: `opendog verification --id <ID>` then `opendog data-risk --id <ID>`
8. If multiple projects compete for attention, start at workspace scope.
   MCP: `get_workspace_data_risk_overview`
   CLI: `opendog workspace-data-risk`

## Decision Tree

### Which project should I inspect first?

Use:

- MCP: `get_workspace_data_risk_overview`
- CLI: `opendog workspace-data-risk`

Why:

- surfaces cross-project hardcoded-data risk
- shows priority ordering
- gives project-level reasons instead of raw counts only

### What should I do next in one project?

Use:

- MCP: `get_agent_guidance`
- CLI: `opendog agent-guidance`

Why:

- it combines observation state, repo risk, verification state, and execution suggestions
- it can run at workspace scope or single-project scope through `project_id`

### I want one stable AI entry envelope before choosing tools

Use:

- MCP: `get_decision_brief`
- CLI: `opendog decision-brief`

Why:

- it exposes the recommended next action, target project, next MCP/CLI entrypoints, and all 8 reusable OPENDOG layers in one payload

### Is this project safe for cleanup or refactor?

Use this order:

1. `get_verification_status`
2. `get_data_risk_candidates`
3. shell verification such as `git status`, `git diff`, and project-native test/lint/build commands

Do not skip step 3 when changes are broad or risky.

### Which files look like mock/demo/seed data or hardcoded pseudo-business data?

Use:

- MCP: `get_data_risk_candidates`
- CLI: `opendog data-risk --id <ID>`

Pay extra attention to:

- `hardcoded_data_candidates`
- `mixed_review_files`
- runtime/shared paths
- high review priority findings

### I only need file activity and unused-file signals

Use:

- `get_stats`
- `get_unused_files`
- `get_time_window_report`
- `compare_snapshots`
- `get_usage_trends`
- `cleanup_project_data` when retained OPENDOG evidence itself must be pruned
- `opendog stats --id <ID>`
- `opendog unused --id <ID>`

### I need to know what changed recently, not just what is active now

Use:

- MCP: `compare_snapshots`
- CLI: `opendog report compare --id <ID>`

Pair it with:

- `get_time_window_report` when you need recent concentration
- shell diff when you need semantic code review

### I need to know which files are heating up or cooling down

Use:

- MCP: `get_usage_trends`
- CLI: `opendog report trend --id <ID>`

Useful follow-up:

- `get_time_window_report` for a simpler recent summary
- `get_stats` for full current per-file totals

## MCP vs CLI vs Shell

### Prefer MCP when

- the caller is an AI agent already integrated with OPENDOG
- you need structured output
- you want reusable guidance or evidence layers
- you want project/workspace prioritization rather than raw file listings
- you want the MCP AI surface over daemon-coordinated project state

### Prefer CLI when

- you are operating from a terminal
- you want a quick human-readable summary
- MCP integration is not active
- you want the same capability surface with explicit shell-oriented invocation

### Prefer shell when

- you need repository truth such as git state or test results not yet captured
- you need direct file inspection after OPENDOG identifies targets
- you need project-native validation commands

Typical shell follow-ups:

- `git status`
- `git diff`
- `cargo test`
- `cargo clippy`
- `npm test`
- `pnpm test`
- `pytest`
- `rg "mock|fixture|fake|stub|sample|demo|seed" .`
- `rg "customer|invoice|email|address|payment|tenant" .`

## Safety Rules

- Do not claim a project is safe for cleanup just because OPENDOG found unused files.
- Do not treat mock-data detection as perfect classification.
- Do not start a second independent monitor path if daemon already owns monitoring state.
- Do not bypass daemon-owned project state with ad hoc parallel monitoring when CLI/MCP can reuse the local control plane.
- Do not use OPENDOG as the sole basis for destructive edits.
- Do not ignore verification evidence before broad changes.
- Do not confuse `cleanup_project_data` with source-code cleanup; it only prunes OPENDOG-retained evidence and storage history.

## High-Value Patterns

### Pattern: choose a project to work on

1. Run `opendog agent-guidance` or `get_agent_guidance`.
2. If the question is still cross-project prioritization, run workspace data-risk overview.
3. Pick the project with the strongest hardcoded-data or mixed-file review reason.
4. Enter that project and run project-specific review commands.

### Pattern: prepare for cleanup

1. Run `opendog agent-guidance` or `get_agent_guidance`.
2. Confirm snapshot exists.
3. Confirm monitor has enough recent activity.
4. Check verification status.
5. Check data-risk candidates.
6. If storage maintenance is flagged, review retained-evidence cleanup first.
7. Only then inspect unused files and draft cleanup candidates.

## CLI Workflow Entry Points

- `opendog agent-guidance`
  Use first when you want the top-level answer to "what should I do next?"
  Useful options:
  `--project <ID>` for one project, `--top <N>` to trim lists, `--json` for machine-readable output
- `opendog decision-brief`
  Use first when another AI should consume one stable decision envelope and then choose narrower tools itself
  Useful options:
  `--project <ID>` for one project, `--top <N>` to trim lists, `--json` for machine-readable output
- `opendog workspace-data-risk`
  Use when the top-level question is "which project should I inspect first?"
  Useful option:
  `--json` for machine-readable output
- `opendog data-risk --id <ID>`
  Use when the question is "which suspicious files in this project need review first?"
  Useful option:
  `--json` for machine-readable output
- `opendog report window --id <ID>`
  Use when the question is "what was active in the last 24h / 7d / 30d?"
  Useful options:
  `--window 24h|7d|30d`, `--json`
- `opendog report compare --id <ID>`
  Use when the question is "what changed between snapshot baselines?"
  Useful options:
  `--base-run-id`, `--head-run-id`, `--json`
- `opendog report trend --id <ID>`
  Use when the question is "which files are heating up or cooling down?"
  Useful options:
  `--window 24h|7d|30d`, `--json`
- `opendog cleanup-data --id <ID>`
  Use when the question is "which OPENDOG-retained evidence or storage history should I prune?"
  Useful options:
  `--scope activity|snapshots|verification|all`, `--older-than-days`, `--keep-snapshot-runs`, `--dry-run`, `--vacuum`, `--json`

## JSON Use For AI

Use JSON output when another AI or script will consume the result directly.

Most useful entrypoints:

- `opendog agent-guidance --json`
- `opendog decision-brief --json`
- `opendog workspace-data-risk --json`
- `opendog data-risk --id <ID> --json`
- `opendog verification --id <ID> --json`
- `opendog report window|compare|trend --json`
- `opendog cleanup-data --json`

For exhaustive field-by-field contracts, schema-version guidance, error shapes, and stability rules, use [json-contracts.md](./json-contracts.md).

## Decision-Critical JSON Fields

Read these first when an AI is making decisions rather than just displaying output.

### `opendog agent-guidance --json`

Read first:

- `guidance.schema_version`
- `guidance.recommended_flow`
- `guidance.layers.execution_strategy.global_strategy_mode`
- `guidance.layers.execution_strategy.preferred_primary_tool`
- `guidance.layers.workspace_observation.projects_missing_snapshot`
- `guidance.layers.workspace_observation.projects_with_stale_snapshot`
- `guidance.layers.workspace_observation.projects_missing_verification`
- `guidance.layers.workspace_observation.projects_with_stale_verification`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_score`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_band`
- `guidance.layers.multi_project_portfolio.priority_candidates[*].attention_reasons`
- `guidance.layers.storage_maintenance.priority_projects`

Then use explanation fields such as `guidance.notes`, `guidance.project_recommendations[*].reason`, `guidance.layers.verification_evidence`, and `guidance.layers.constraints_boundaries`.

### `opendog decision-brief --json`

Read first:

- `schema_version`
- `decision.recommended_next_action`
- `decision.target_project_id`
- `decision.strategy_mode`
- `entrypoints.next_mcp_tools`
- `entrypoints.next_cli_commands`
- `entrypoints.selection_reasons`
- `entrypoints.execution_templates`
- `decision.signals.attention_score`
- `decision.signals.attention_band`
- `decision.signals.attention_reasons`
- `decision.risk_profile.primary_repo_risk_finding`
- `decision.risk_profile.repo_risk_finding_counts`

When `entrypoints.execution_templates` is present, read in this order:

1. `template_id`, `kind`, `plan_stage`
2. `requires_human_confirmation`, `blocking_conditions`
3. `parameter_schema`, `default_values`, `placeholder_hints`
4. `expected_output_fields`, `evidence_written_to_opendog`
5. `follow_up_on_success`, `follow_up_on_failure`, `retry_policy`

### `opendog workspace-data-risk --json`

Read first:

- `schema_version`
- `guidance.recommended_flow`
- `guidance.layers.multi_project_portfolio.priority_projects`
- `guidance.layers.workspace_observation.projects_with_hardcoded_candidates`
- `guidance.layers.workspace_observation.total_hardcoded_candidates`

### `opendog data-risk --json`

Read first:

- `schema_version`
- `guidance.recommended_flow`
- `hardcoded_data_candidates`
- `mock_data_candidates`
- `mixed_review_files`
- `hardcoded_candidate_count`
- `mock_candidate_count`

Treat `mixed_review_files` as high-caution review targets because mock-like and business-like signals overlap there.

### `opendog verification --json`

Read first:

- `schema_version`
- `verification.safe_for_cleanup`
- `verification.safe_for_refactor`
- `verification.missing_kinds`
- `verification.latest_runs`

`record-verification --json` and `run-verification --json` are mainly evidence-write paths; the key fields are `kind`, `status`, `exit_code`, and summary/output metadata.

## JSON Interpretation Rules

- Treat JSON results as decision-support evidence, not final truth
- Always distinguish missing evidence from stale evidence before acting on recommendations
- Switch to shell, git, tests, or build tools when OPENDOG indicates external confirmation is required
- Use `json-contracts.md` when you need complete field coverage instead of decision-first reading

## Related Docs

- [Capability Index](./capability-index.md)
- [README](../README.md)
- [CLAUDE.md](../CLAUDE.md)
- [.planning/PROJECT.md](../.planning/PROJECT.md)
- [.planning/ROADMAP.md](../.planning/ROADMAP.md)
