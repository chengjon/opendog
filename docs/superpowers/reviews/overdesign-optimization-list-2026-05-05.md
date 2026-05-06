# OPENDOG Overdesign Optimization List

Date: 2026-05-05
Basis:

- `docs/superpowers/reviews/overdesign-review-2026-05-05.md`
- current repository code and docs

## Purpose

This document turns the overdesign review into a concrete optimization list for this repository.

The goal is not to rewrite OPENDOG. The goal is to reduce MCP surface bloat while preserving the observation core and the useful AI-facing analysis layers.

This is a disposition list, not an implementation batch:

- which MCP tools to keep
- which MCP tools to move to CLI-only
- which MCP tools to merge
- which changes should be deferred

## Review Chain

Related overdesign documents:

- `docs/overdesign-assessment-2026-05-04.md` — pre-optimization self-assessment snapshot
- `docs/superpowers/reviews/overdesign-review-2026-05-05.md` — formal review and implementation acceptance/rejection decisions
- `docs/superpowers/reviews/overdesign-optimization-list-2026-05-05.md` — this implementation and disposition document
- `docs/superpowers/reviews/overdesign-architect-review-opinion-2026-05-05.md` — independent opinion and follow-up critique

## Status Update

Implementation status after the 2026-05-05 surface-reduction batch:

- Tier 1 is complete:
  - `update_global_config`
  - `update_project_config`
  - `reload_project_config`
  - `export_project_evidence`
  - `cleanup_project_data`
  - all removed from the MCP public surface and left on the CLI operator surface
- Tier 2 is complete:
  - `get_agent_guidance` and `get_decision_brief` were merged into `get_guidance`
  - MCP shape is now `get_guidance(detail = "summary" | "decision")`
  - legacy MCP aliases were not retained because the goal was actual menu reduction
- Current MCP surface after the implemented work: **19 tools**

Practical disposition after implementation:

- do **not** merge CLI `agent-guidance` and `decision-brief`
- do **not** fold `get_workspace_data_risk_overview` into `get_guidance`
- treat report-tool consolidation as optional future work, not a required follow-up

Independent-opinion follow-up:

- accept the internal-adapter-debt observation from the guidance merge
- accept that `json!()` payload assembly is meaningful maintainability debt
- keep config read tools only because they still expose a distinct resolved-config contract
- keep the repository position that report-tool consolidation is optional until usage evidence proves the extra merge is worth the blast radius

## Feasibility Summary

Overall feasibility: **high**, and Tier 1 / Tier 2 have already been completed.

Original recommended execution order:

1. Finish and review the `FUNCTION_TREE.md` path migration.
2. Reduce MCP surface by moving admin/operator mutations to CLI-only.
3. Merge overlapping guidance entrypoints.
4. Only then evaluate whether the reporting trio should be merged.

Why phased execution mattered:

- guidance/decision references appear broadly across source, tests, and docs
- report-tool references also have wide coverage
- the worktree was already dirty when this sequence was drafted, so surface-reduction work was better isolated into separate commits

## MCP Tool Disposition

### Keep As-Is

These tools are close to the core product value or are valid AI-facing intake/query surfaces.

| Tool | Decision | Reason |
|---|---|---|
| `create_project` | Keep | Core project lifecycle |
| `delete_project` | Keep | Needed cleanup path for created projects |
| `list_projects` | Keep | Fundamental discovery |
| `take_snapshot` | Keep | Core observation baseline |
| `start_monitor` | Keep | Core observation control |
| `stop_monitor` | Keep | Core observation control |
| `get_stats` | Keep | Highest-frequency observation query |
| `get_unused_files` | Keep | Highest-frequency cleanup review query |
| `get_data_risk_candidates` | Keep | Distinct project-level risk analysis |
| `get_workspace_data_risk_overview` | Keep | Distinct cross-project prioritization query |
| `get_verification_status` | Keep | Core safety/evidence query |
| `record_verification_result` | Keep | AI-facing evidence intake, not admin |
| `run_verification_command` | Keep | AI-facing evidence generation, not admin |
| `get_global_config` | Keep | Read-only config inspection is safe |
| `get_project_config` | Keep | Read-only config inspection is safe |

### Move To CLI-Only

These tools expose operator/admin behavior that is already fully available through CLI.

| Tool | Decision | Reason | Existing CLI Path |
|---|---|---|---|
| `update_global_config` | Move to CLI-only | Persistent mutation, low AI value | `opendog config set-global` |
| `update_project_config` | Move to CLI-only | Persistent mutation, low AI value | `opendog config set-project` |
| `reload_project_config` | Move to CLI-only | Runtime reconfiguration | `opendog config reload` |
| `export_project_evidence` | Move to CLI-only | Filesystem artifact generation | `opendog export` |
| `cleanup_project_data` | Move to CLI-only | Retained-evidence deletion / vacuum path | `opendog cleanup-data` |

### Merge

These tools overlap enough that one public surface is preferable.

| Current Tools | Proposed Tool | Decision | Reason |
|---|---|---|---|
| `get_agent_guidance` + `get_decision_brief` | `get_guidance` | Merge | `decision_brief` is a wrapper/superset path over guidance |

Recommended merged API shape:

```json
{
  "project_id": "demo",
  "top": 5,
  "detail": "summary"
}
```

Where:

- `detail = "summary"` means current `get_agent_guidance` behavior
- `detail = "decision"` means current `get_decision_brief` behavior

Implemented repository choice:

- add `get_guidance`
- remove `get_agent_guidance` and `get_decision_brief` from the public MCP surface
- keep CLI `agent-guidance` / `decision-brief` unchanged
- update docs to make `get_guidance` the preferred MCP entrypoint

### Merge Later

These tools are mergeable, but the benefit is lower than the guidance merge.

| Current Tools | Proposed Tool | Decision | Reason |
|---|---|---|---|
| `get_time_window_report` + `compare_snapshots` + `get_usage_trends` | `get_report` | Merge later | Same report family, but parameter and payload models differ enough that this is a second-phase cleanup |

Recommended merged API shape:

```json
{
  "id": "demo",
  "mode": "window",
  "window": "24h",
  "limit": 10
}
```

Allowed `mode` values:

- `window`
- `compare`
- `trend`

Important boundary:

- do **not** fold `get_stats` or `get_unused_files` into this merge
- they are frequent, direct queries and still deserve dedicated names

## Resulting MCP Surface

After the implemented Tier 1 and Tier 2 reductions, the MCP surface is:

| # | Tool |
|---|---|
| 1 | `create_project` |
| 2 | `delete_project` |
| 3 | `list_projects` |
| 4 | `take_snapshot` |
| 5 | `start_monitor` |
| 6 | `stop_monitor` |
| 7 | `get_stats` |
| 8 | `get_unused_files` |
| 9 | `get_guidance` |
| 10 | `get_data_risk_candidates` |
| 11 | `get_workspace_data_risk_overview` |
| 12 | `get_verification_status` |
| 13 | `record_verification_result` |
| 14 | `run_verification_command` |
| 15 | `get_global_config` |
| 16 | `get_project_config` |
| 17 | `get_time_window_report` |
| 18 | `compare_snapshots` |
| 19 | `get_usage_trends` |

This yields a 19-tool MCP surface.

If the later report merge also lands, the MCP surface becomes:

| # | Tool |
|---|---|
| 1 | `create_project` |
| 2 | `delete_project` |
| 3 | `list_projects` |
| 4 | `take_snapshot` |
| 5 | `start_monitor` |
| 6 | `stop_monitor` |
| 7 | `get_stats` |
| 8 | `get_unused_files` |
| 9 | `get_report` |
| 10 | `get_guidance` |
| 11 | `get_data_risk_candidates` |
| 12 | `get_workspace_data_risk_overview` |
| 13 | `get_verification_status` |
| 14 | `record_verification_result` |
| 15 | `run_verification_command` |
| 16 | `get_global_config` |
| 17 | `get_project_config` |

This yields a 17-tool MCP surface if `get_report` replaces three report tools but read-only config remains.

## Original Implementation Priority

### Tier 1: High Value, Lowest Architectural Risk

1. Move the 5 admin/operator mutation tools to CLI-only:
   - `update_global_config`
   - `update_project_config`
   - `reload_project_config`
   - `export_project_evidence`
   - `cleanup_project_data`

Why first:

- lowest product ambiguity
- no observation-core changes
- CLI equivalents already exist
- biggest immediate signal reduction in MCP

### Tier 2: High Value, Medium Refactor Cost

2. Merge guidance entrypoints:
   - `get_agent_guidance`
   - `get_decision_brief`

Why second:

- strong conceptual overlap
- already layered that way in code
- large doc/test blast radius, so should be isolated into its own batch

### Tier 3: Optional / Nice-to-Have

3. Merge the report trio:
   - `get_time_window_report`
   - `compare_snapshots`
   - `get_usage_trends`

Why third:

- lower leverage than Tier 1 and Tier 2
- more contract-shape cleanup than conceptual simplification
- current 19-tool surface is already acceptable, so this is not a required continuation item

## Blast Radius Notes

The following change families have visible repository-wide impact:

### Guidance merge

Broad references exist across:

- MCP handler layer
- CLI guidance commands
- docs (`ai-playbook`, `capability-index`, `mcp-tool-reference`, `json-contracts`)
- payload contracts and guidance tests

This should be treated as a public-surface migration with broad doc/test fallout, not as a simple rename.

### Report merge

Broad references exist across:

- MCP analysis handlers
- CLI report commands
- analysis payload docs and tests
- AI playbook and capability index

This is feasible, but only after guidance consolidation or if menu pressure remains a problem.

### MCP admin-tool removal

Impact is broad in docs/tests but shallow in logic:

- MCP router
- config/maintenance handlers
- MCP tool reference
- JSON contract docs
- capability index

This is the cleanest first reduction pass.

## Non-Goals

These are explicitly **not** recommended as current optimization goals:

- rewriting the observation core
- removing data-risk detection
- removing verification evidence
- removing workspace prioritization
- reorganizing the CLI just for cosmetic menu grouping
- collapsing `get_stats` and `get_unused_files` into a generic mega-report tool

## Suggested Execution Sequence

### Batch A

MCP-only surface reduction:

- remove 5 admin/operator mutation tools from MCP routing
- keep CLI behavior unchanged
- update MCP docs and contracts accordingly

### Batch B

Guidance consolidation:

- add `get_guidance`
- implement `detail=summary|decision`
- remove `get_agent_guidance` and `get_decision_brief` from the public MCP surface
- update docs to prefer `get_guidance`

### Batch C

Optional report consolidation:

- add `get_report`
- keep `get_stats` and `get_unused_files`
- decide whether to retain legacy report tools as aliases or remove them after migration

## Final Recommendation

Original approved optimization scope was:

1. **Do now**
   - move 5 admin/operator mutation tools to CLI-only

2. **Do next**
   - merge `get_agent_guidance` and `get_decision_brief` into `get_guidance`

3. **Do only if still needed after the above**
   - merge the 3 report tools into `get_report`

Current repository state:

- step 1 is complete
- step 2 is complete
- step 3 remains optional

This gave the repository a clear path to reduce MCP surface bloat without touching the observation core or deleting the most product-defining analysis capabilities.
