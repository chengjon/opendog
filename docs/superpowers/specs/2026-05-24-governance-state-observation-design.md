# Governance State Observation Design

> Date: 2026-05-24
> Status: approved
> Maps to FT: FT-03.09 (new L3 leaf under FT-03 AI Decision Support and Governance)
> Requirement family: GOV (new, ~6-8 requirements)

## Purpose

Let projects record and read their own governance work state (lanes and nodes) through OPENDOG, then cross-reference it with OPENDOG's observation evidence in guidance payloads. OPENDOG does not enforce governance rules — it observes and recommends.

This is motivated by the Steward Tree practice pattern proven in mystocks architecture remediation: a recoverable state machine connecting evidence packages, decision packages, implementation authorizations, source changes, verification, closeout, and next-candidate selection across multiple agents, branches, and review rounds.

OPENDOG's role is not to replace that state machine but to give projects a place to record it and to connect it with OPENDOG's file-observation, verification, and data-risk evidence.

## Design Principles

1. **Observe, don't enforce.** OPENDOG stores governance state and cross-references it with observation evidence. It does not validate state transitions, block actions, or enforce gates.
2. **Project owns its vocabulary.** The `state` field on steward nodes is free text. Projects choose their own lifecycle vocabulary (5 states, 11 states, or anything else). OPENDOG never rejects a state value.
3. **Lightweight storage, few tools.** 2 new tables, 4 new MCP tools, 1 new CLI command group. ~2,100 lines of new code.
4. **Reference anchors, not sync.** External artifacts (PRs, issues, commits, reports) are stored as string references, not synchronized from external tools.
5. **Per-project isolation.** Governance data lives in the same per-project SQLite database as snapshots, stats, and verification runs. No cross-project data leakage.

## What OPENDOG Does NOT Do

| Does not do | Why | If the project needs it |
|---|---|---|
| State machine validation / gate blocking | OPENDOG observes and recommends, not executes or blocks | Implement gate logic in AI agent prompts or CI |
| Path-level authorization matching | `forbidden_scope` is semantic description, not file globs | Use GitNexus, linters, or pre-commit hooks for path protection |
| Automatic closeout report generation | Requires understanding PR diff semantics, beyond observation capability | AI agent reads `get_governance_state` + `compare_snapshots` and writes closeout itself |
| External tool sync (GitHub/Linear/Jira) | OPENDOG is not an integration bus | `external_anchors` stores reference strings; sync is the project's responsibility |
| Automatic governance state inference | OPENDOG should not guess "this project is doing architecture remediation" | Project explicitly calls `create_governance_lane` |
| Audit history tracking | Lightweight design; avoids `node_transitions` table growth | Project points to its own audit files in `artifact_refs` |

## Data Model

### New Tables (in per-project SQLite)

```sql
CREATE TABLE IF NOT EXISTS governance_lanes (
    lane_id     TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'active',   -- active / completed / deferred
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS steward_nodes (
    node_id           TEXT PRIMARY KEY,
    lane_id           TEXT NOT NULL,
    state             TEXT NOT NULL,               -- free text, no enforced vocabulary
    summary           TEXT,                        -- one-line factual summary
    evidence_refs     TEXT,                        -- JSON array of report/document paths
    artifact_refs     TEXT,                        -- JSON array of generated artifact paths
    git_head          TEXT,                        -- HEAD commit when node was created/updated
    suggested_next    TEXT,                        -- recommended next step
    forbidden_scope   TEXT,                        -- JSON array of semantic scope descriptions
    external_anchors  TEXT,                        -- JSON object: {"pr": "#186", "issue": "#79", "commit": "abc123"}
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_steward_nodes_lane ON steward_nodes(lane_id);
CREATE INDEX IF NOT EXISTS idx_steward_nodes_state ON steward_nodes(state);
```

Schema version: 4 → 5. Migration adds these tables to existing project databases.

### Storage Location

Data is stored inside OPENDOG's own data directory, not in project source trees:

```text
$OPENDOG_HOME/data/
  └── projects/
        └── mystocks.db     # snapshot, file_stats, verification_runs, governance_lanes, steward_nodes
```

No files are created in the observed project's source directory.

### Key Design Decisions

| Decision | Rationale |
|---|---|
| `state` is free text | No gate enforcement; projects use their own state vocabulary |
| No `node_transitions` audit table | Lightweight approach; audit history managed in project's own artifacts |
| No `authorized_paths` / `forbidden_paths` file globs | No path-level gate matching; `forbidden_scope` is semantic description only |
| Tables live in per-project SQLite | Follows OPENDOG's per-project isolation principle |

## MCP Tools

4 new tools, following existing `McpToolSpec` pattern. Total MCP tools: 22 → 26 (+18%).

### `create_governance_lane`

Create a governance work lane for the project.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",
  "title": "Service lifecycle DI remediation",
  "description": "Extract singleton services into DI-provided seams"
}

// Response
{
  "lane_id": "service-lifecycle-di",
  "status": "active",
  "created_at": "2026-05-24T10:00:00Z"
}
```

### `upsert_steward_node`

Create or update a governance node within a lane. All non-key fields are optional.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",
  "node_id": "G2.46",
  "state": "evidence-prepared",
  "summary": "Found 8 singleton candidates, 3 with route dependencies",
  "evidence_refs": ["docs/reports/quality/candidates.md"],
  "artifact_refs": [".planning/codebase/generated/candidates.json"],
  "git_head": "abc1234",
  "suggested_next": "Classify each candidate by ownership type before authorizing implementation",
  "forbidden_scope": ["backend source", "tests", "compatibility getter retirement"],
  "external_anchors": { "pr": "#186", "issue": "#79" }
}
```

### `get_governance_state`

Read governance state for a project. Optionally filter by lane, specific node, or active-only.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",  // optional
  "node_id": "G2.46",                 // optional
  "active_only": true                  // optional
}

// Response
{
  "lanes": [
    {
      "lane_id": "service-lifecycle-di",
      "title": "Service lifecycle DI remediation",
      "status": "active",
      "node_count": 5,
      "active_nodes": 3
    }
  ],
  "nodes": [
    {
      "node_id": "G2.46",
      "lane_id": "service-lifecycle-di",
      "state": "evidence-prepared",
      "summary": "Found 8 singleton candidates, 3 with route dependencies",
      "suggested_next": "Classify each candidate by ownership type...",
      "forbidden_scope": ["backend source", "tests"],
      "external_anchors": { "pr": "#186" },
      "updated_at": "2026-05-24T10:30:00Z"
    }
  ],
  "observation_hints": {
    "snapshot_freshness": "fresh",
    "verification_status": "passed",
    "data_risk_candidates": 12,
    "unused_files": 34
  }
}
```

`observation_hints` is automatically derived from existing OPENDOG evidence (snapshot freshness, verification status, stats, data-risk). This is the core integration value: governance state and observation evidence presented together.

### `delete_governance_lane`

Delete or archive an entire lane and its nodes.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",
  "action": "complete"   // "delete" (hard delete) | "complete" (mark completed) | "defer" (mark deferred)
}
```

## Guidance Integration

### New `governance` Layer in `get_guidance(detail=summary)`

The existing `get_guidance` response gains a 5th layer. Existing layers are unchanged.

```json
{
  "layers": {
    "workspace_observation": { "... unchanged ..." },
    "execution_strategy": { "... unchanged ..." },
    "constraints_boundaries": { "... unchanged ..." },
    "governance": {
      "has_governance_state": true,
      "project_governance": [
        {
          "project_id": "mystocks",
          "lanes": [
            {
              "lane_id": "service-lifecycle-di",
              "title": "Service lifecycle DI remediation",
              "status": "active",
              "active_nodes": 3,
              "latest_node": {
                "node_id": "G2.46",
                "state": "evidence-prepared",
                "summary": "Found 8 singleton candidates",
                "suggested_next": "Classify each candidate by ownership type",
                "forbidden_scope": ["backend source", "tests"],
                "updated_at": "2026-05-24T10:30:00Z"
              }
            }
          ],
          "observation_cross_reference": {
            "snapshot_freshness": "fresh",
            "verification_status": "passed",
            "unused_files_in_scope": 12,
            "data_risk_candidates": 8
          }
        }
      ],
      "workspace_summary": {
        "total_active_lanes": 2,
        "total_active_nodes": 7,
        "projects_with_governance": 1,
        "projects_without_governance": 3
      }
    }
  }
}
```

### Design Rules

| Rule | Reason |
|---|---|
| `observation_cross_reference` is auto-generated | Extracted from existing snapshot freshness, verification status, stats, data-risk — projects do not fill this in |
| Projects without governance state | `has_governance_state: false`, `project_governance` is empty array — no impact on existing guidance consumers |
| `forbidden_scope` passed through verbatim | OPENDOG does not interpret or enforce it — just presents it to AI agents |
| Only in `detail=summary` mode | `detail=decision` mode is unchanged, keeping the decision brief compact |

## CLI Commands

1 new top-level command group `opendog governance` with 4 subcommands. Total CLI commands: 22 → 23.

### `opendog governance create-lane`

```bash
opendog governance create-lane \
  --id mystocks \
  --lane-id service-lifecycle-di \
  --title "Service lifecycle DI remediation" \
  [--description "..."] \
  [--json]
```

### `opendog governance upsert-node`

```bash
opendog governance upsert-node \
  --id mystocks \
  --lane-id service-lifecycle-di \
  --node-id G2.46 \
  --state evidence-prepared \
  --summary "Found 8 singleton candidates" \
  [--evidence-refs '["docs/reports/x.md"]'] \
  [--artifact-refs '["generated/x.json"]'] \
  [--git-head abc1234] \
  [--suggested-next "Classify each candidate"] \
  [--forbidden-scope '["backend source","tests"]'] \
  [--external-anchors '{"pr":"#186"}'] \
  [--json]
```

### `opendog governance show`

Default output is human-readable table. `--json` for machine consumption.

```bash
opendog governance show \
  --id mystocks \
  [--lane-id service-lifecycle-di] \
  [--node-id G2.46] \
  [--active-only] \
  [--json]
```

Human-readable output example:

```text
Governance: mystocks

Lane: service-lifecycle-di
  Title: Service lifecycle DI remediation | Status: active

  Node    State               Summary                          Suggested Next
  G2.44   closeout-merged     StockSearch provider merged      Select next candidate
  G2.45   implementation-merged AdvancedAnalysis provider merged  Prepare closeout
  G2.46   evidence-prepared   Found 8 singleton candidates     Classify by ownership

  Observation: snapshot=fresh | verification=passed | unused=12 | data_risk=8
```

### `opendog governance close-lane`

```bash
opendog governance close-lane \
  --id mystocks \
  --lane-id service-lifecycle-di \
  --action complete \   # complete | defer | delete
  [--json]
```

## FT-* Mapping

New L3 leaf under existing FT-03:

```yaml
- id: FT-03.09
  title: Governance State Observation
  level: L3
  parent: FT-03
  lifecycle: designing
  summary: >
    Let projects record and read their own governance work state
    (lanes and nodes), then cross-reference it with OPENDOG's
    observation evidence in guidance payloads. OPENDOG does not
    enforce governance rules — it observes and recommends.
```

Placed under FT-03 rather than a new FT-04 because:
- FT-03 is already defined as "AI Decision Support and Governance"
- Governance state observation is a form of decision support
- Does not warrant a new L1 domain

## Code Impact

### File Changes

| File | Action | Estimated Lines |
|---|---|---|
| `src/storage/schema.rs` | Modify (+2 tables, +2 indexes, version bump) | +25 |
| `src/storage/migrations.rs` | Modify (v4→v5 migration) | +40 |
| `src/storage/governance_queries.rs` | **New** | ~350 |
| `src/core/governance.rs` | **New** (lane/node CRUD logic) | ~300 |
| `src/mcp/governance_handlers.rs` | **New** (4 tool handlers) | ~250 |
| `src/mcp/governance_payload.rs` | **New** (4 payload builders) | ~300 |
| `src/mcp/guidance_types.rs` | Modify (+3 structs) | +50 |
| `src/mcp/guidance_payload.rs` | Modify (+governance layer) | +150 |
| `src/mcp/params.rs` | Modify (+4 Params structs) | +80 |
| `src/mcp/tool_inventory.rs` | Modify (+4 McpToolSpec entries) | +80 |
| `src/mcp/mod.rs` | Modify (+module registration) | +10 |
| `src/contracts.rs` | Modify (+4 contract IDs) | +8 |
| `src/cli/mod.rs` | Modify (+governance subcommands) | +120 |
| `tests/governance_test.rs` | **New** | ~350 |
| **Total** | **7 new + 8 modified** | **~2,100 lines** |

### Scale Impact

| Metric | Before | After | Change |
|---|---|---|---|
| Source lines | ~24,800 | ~26,900 | +8.5% |
| MCP tools | 22 | 26 | +18% |
| CLI top-level commands | 22 | 23 | +4.5% |
| FT-* leaf nodes | 26 | 27 | +3.8% |
| New requirement family | 0 | GOV (~6-8 reqs) | +1 family |

## Design Intent Verification

| OPENDOG Core Principle | Compliance |
|---|---|
| Observe what happened | Yes — observes project governance work state |
| Recommend next steps | Yes — `suggested_next` + guidance layer cross-reference |
| Don't act on your behalf | Yes — no enforcement, no blocking, no auto-generation |
| Don't replace external truth sources | Yes — git/tests/lint remain external truth; OPENDOG stores reference anchors only |
