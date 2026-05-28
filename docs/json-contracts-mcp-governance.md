# JSON Contracts: MCP and Governance

This companion document contains extended MCP, verification, governance, orphan-scan, service, and error contracts split from [JSON Contracts](./json-contracts.md).

## MCP Resources

Version markers:

- Resources return JSON content with the same schema families as their tool equivalents.
- `opendog://projects` mirrors project-list state.
- `opendog://project/{id}/verification` mirrors verification-status state for one project.

### Recommended consumption pattern

1. Use resources only for read-only state.
2. Use tools for registration, snapshot, monitoring, verification execution, deletion, export, or cleanup.
3. If resources are not visible after a rebuild, reconnect the MCP host and follow QUICKSTART retest steps.

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

## `opendog create-governance-lane --json` / MCP `create_governance_lane`

Version markers:

- CLI: `schema_version = opendog.cli.create-governance-lane.v1`
- MCP: `schema_version = opendog.mcp.create-governance-lane.v1`

### Primary decision fields

- `lane_id`
- `title`
- `status`
- `created_at`

### Explanatory fields

- `description`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Confirm `status = active`.
3. Use `lane_id` in subsequent `upsert_governance_node` and `get_governance_state` calls.

## `opendog upsert-governance-node --json` / MCP `upsert_governance_node`

Version markers:

- CLI: `schema_version = opendog.cli.upsert-governance-node.v1`
- MCP: `schema_version = opendog.mcp.upsert-governance-node.v1`

### Primary decision fields

- `node_id`
- `lane_id`
- `state`
- `created`

### Explanatory fields

- `summary`
- `evidence_refs`
- `artifact_refs`
- `reported_git_head`
- `suggested_next`
- `forbidden_scope`
- `external_anchors`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Read `created` to distinguish insert versus update.
3. Use `state` to confirm the node landed in the intended lifecycle state.
4. Read `forbidden_scope` back to verify safety boundaries are persisted.

## `opendog get-governance-state --json` / MCP `get_governance_state`

Version markers:

- CLI: `schema_version = opendog.cli.get-governance-state.v1`
- MCP: `schema_version = opendog.mcp.get-governance-state.v1`

### Primary decision fields

- `lanes`
- `nodes`
- `observation_hints`

### Explanatory fields

- `lanes[*].title`
- `lanes[*].status`
- `nodes[*].state`
- `nodes[*].summary`
- `nodes[*].suggested_next`
- `observation_hints.stale_snapshot`
- `observation_hints.missing_verification`
- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Filter by `lane_id` or `node_id` when the query scope is narrow.
3. Use `active_only` to exclude closed or deferred lanes.
4. Read `observation_hints` before trusting governance state as current.

## `opendog close-governance-lane --json` / MCP `close_governance_lane`

Version markers:

- CLI: `schema_version = opendog.cli.close-governance-lane.v1`
- MCP: `schema_version = opendog.mcp.close-governance-lane.v1`

### Primary decision fields

- `lane_id`
- `action_taken`
- `status`
- `nodes_affected`

### Explanatory fields

- `guidance`

### Recommended consumption pattern

1. Check `schema_version`.
2. Confirm `action_taken` matches the requested `action`.
3. Read `nodes_affected` to understand cascade scope.
4. Treat `status = closed` as terminal; `deferred` lanes may be reopened later.

## MCP `scan_orphans`

Schema versions:

- MCP: `schema_version = opendog.mcp.orphan-scan.v1`

Top-level contract:

```json
{
  "schema_version": "opendog.mcp.orphan-scan.v1",
  "project_id": "demo",
  "status": "ok",
  "scan_run_id": null,
  "scanner_health": [...],
  "summary": {
    "total_candidates": 0,
    "remove_candidate_count": 0,
    "review_required_count": 0,
    "blocked_count": 0
  },
  "candidates": [],
  "warnings": [],
  "recommended_next_actions": []
}
```

Consumption notes:

1. Check `status` first: `"ok"` or `"partial"`.
2. Read `scanner_health` to understand which scanners ran and their health.
3. Each candidate has `classification` (remove / review / blocked), `confidence`, and `reasons`.
4. Use `recommended_next_actions` for workflow guidance after the scan.

## MCP `verify_deletion_plan`

Schema versions:

- MCP: `schema_version = opendog.mcp.orphan-deletion-plan.v1`

Top-level contract:

```json
{
  "schema_version": "opendog.mcp.orphan-deletion-plan.v1",
  "project_id": "demo",
  "status": "blocked",
  "safe_to_plan_deletion": false,
  "blocked_targets": [],
  "review_required_targets": [],
  "remove_candidates": [],
  "required_project_verification_commands": [],
  "evidence_gaps": []
}
```

Consumption notes:

1. Check `safe_to_plan_deletion` first — only `true` means the evidence supports a deletion plan.
2. `blocked_targets` have vetoes or strong counter-evidence.
3. `required_project_verification_commands` lists commands that should pass before deletion.
4. Treat the result as decision-support evidence, not as authority to delete files.

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

Daemon-backed MCP calls may return `daemon_response_integrity_error` when the socket returns empty or incomplete JSON. Treat it as transport failure: retry once, compare with CLI, then restart the daemon if repeated.

Recommended consumption pattern:

1. Check top-level `schema_version`.
2. If `status = error`, branch on `error_code`.
3. Use `project_id` or request echo fields to correlate the failure with the attempted action.
4. Read `remediation` only after the error class is known.
