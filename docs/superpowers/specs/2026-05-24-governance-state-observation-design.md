# Governance State Observation Design

> Date: 2026-05-24
> Status: approved
> Maps to FT: FT-03.09 (new L2 under FT-03) + FT-03.09.01 (new L3 leaf)
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

CREATE TABLE IF NOT EXISTS governance_nodes (
    node_id           TEXT PRIMARY KEY,
    lane_id           TEXT NOT NULL,
    state             TEXT NOT NULL,               -- free text, no enforced vocabulary; required on create
    summary           TEXT,                        -- one-line factual summary
    evidence_refs     TEXT,                        -- JSON array of report/document paths
    artifact_refs     TEXT,                        -- JSON array of generated artifact paths
    reported_git_head TEXT,                        -- caller-reported HEAD anchor; not validated by OPENDOG
    suggested_next    TEXT,                        -- recommended next step
    forbidden_scope   TEXT,                        -- JSON array of semantic scope descriptions
    external_anchors  TEXT,                        -- JSON object: {"pr": "#186", "issue": "#79", "commit": "abc123"}
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_governance_nodes_lane ON governance_nodes(lane_id);
CREATE INDEX IF NOT EXISTS idx_governance_nodes_state ON governance_nodes(state);
```

Schema version: 4 → 5. Migration follows the existing `CREATE TABLE IF NOT EXISTS` pattern in `src/storage/schema.rs` — adding the new DDL statements to `PROJECT_SCHEMA` is sufficient because `migrate()` in `src/storage/migrations.rs` calls `conn.execute_batch(kind.schema_sql())`, which runs all `CREATE TABLE IF NOT EXISTS` statements idempotently. No separate versioned migration function is needed.

### Storage Location

Data is stored inside OPENDOG's own data directory, not in project source trees:

```text
$OPENDOG_HOME/data/
  └── projects/
        └── mystocks.db     # snapshot, file_stats, verification_runs, governance_lanes, steward_nodes
```

No files are created in the observed project's source directory.

### Scope Boundary

Governance state is **local coordination state only**. It lives in the OPENDOG database on the machine where it was created. There is no import/export, no cross-machine sync, and no mechanism for different checkouts to share governance state. If a project needs cross-machine coordination, the project should manage its own steward tree files in its source tree and use OPENDOG's governance tools as a local acceleration layer.

### Key Design Decisions

| Decision | Rationale |
|---|---|
| `state` is free text | No gate enforcement; projects use their own state vocabulary |
| No `node_transitions` audit table | Lightweight approach; audit history managed in project's own artifacts |
| No `authorized_paths` / `forbidden_paths` file globs | No path-level gate matching; `forbidden_scope` is semantic description only |
| Tables live in per-project SQLite | Follows OPENDOG's per-project isolation principle |
| No foreign keys on `lane_id` | SQLite FK enforcement requires `PRAGMA foreign_keys` per-connection; instead, referential integrity is enforced in `core::governance` application logic: upsert validates lane exists, close-lane deletes nodes in same transaction, get ignores orphan nodes |
| `reported_git_head` not auto-read | OPENDOG does not read the project's git state; the caller reports it as an anchor string. Renamed from `git_head` to `reported_git_head` to make this explicit |

## MCP Tools

4 new tools, following existing `McpToolSpec` pattern. Total MCP tools: 22 → 26 (+18.2%).

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

### `upsert_governance_node`

Create or update a governance node within a lane.

**Create vs update semantics:**
- On create (node does not exist): `state` is **required**. All other non-key fields are optional. Returns validation error if `state` is missing or `lane_id` does not reference an existing lane.
- On update (node exists): all non-key fields are optional; omitted fields retain their current value. `state` may be omitted to keep the current state.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",
  "node_id": "G2.46",
  "state": "evidence-prepared",           // required on create; optional on update
  "summary": "Found 8 singleton candidates, 3 with route dependencies",
  "evidence_refs": ["docs/reports/quality/candidates.md"],
  "artifact_refs": [".planning/codebase/generated/candidates.json"],
  "reported_git_head": "abc1234",
  "suggested_next": "Classify each candidate by ownership type before authorizing implementation",
  "forbidden_scope": ["backend source", "tests", "compatibility getter retirement"],
  "external_anchors": { "pr": "#186", "issue": "#79" }
}

// Response
{
  "node_id": "G2.46",
  "lane_id": "service-lifecycle-di",
  "state": "evidence-prepared",
  "created": false,
  "updated_at": "2026-05-24T10:30:00Z"
}
```

`created` is `true` when a new node was inserted, `false` when an existing node was updated.

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

`observation_hints` is automatically derived from existing OPENDOG project-level evidence (snapshot freshness, verification status, total unused files, total data-risk candidates). These are **project-level totals**, not scoped to the governance node's semantic `forbidden_scope` — OPENDOG does not interpret semantic scope strings into query filters.

### `close_governance_lane`

Close, defer, or hard-delete an entire lane and its nodes.

```json
// Params
{
  "id": "mystocks",
  "lane_id": "service-lifecycle-di",
  "action": "complete"   // "delete" (hard delete) | "complete" (mark completed) | "defer" (mark deferred)
}

// Response
{
  "lane_id": "service-lifecycle-di",
  "action_taken": "complete",
  "status": "completed",
  "nodes_affected": 5
}
```

## Guidance Integration

### New `governance` Layer in `get_guidance(detail=summary)`

The existing `get_guidance` response gains an 8th layer. The 7 existing layers (`workspace_observation`, `execution_strategy`, `multi_project_portfolio`, `storage_maintenance`, `verification_evidence`, `project_toolchain`, `constraints_boundaries`) are unchanged.

```json
{
  "layers": {
    "workspace_observation": { "... unchanged ..." },
    "execution_strategy": { "... unchanged ..." },
    "multi_project_portfolio": { "... unchanged ..." },
    "storage_maintenance": { "... unchanged ..." },
    "verification_evidence": { "... unchanged ..." },
    "project_toolchain": { "... unchanged ..." },
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
            "unused_files_total": 12,
            "data_risk_candidates_total": 8
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
| `observation_cross_reference` uses project-level totals | Fields are `unused_files_total` and `data_risk_candidates_total` — OPENDOG does not interpret `forbidden_scope` semantics into query filters |
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
  [--reported-git-head abc1234] \
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

New L2 module + L3 leaf under existing FT-03, following the tree structure where FT-03.01..08 are L2 with L3 children:

```yaml
- id: FT-03.09
  title: Governance State Observation
  level: L2
  parent: FT-03
  lifecycle: designing
  summary: Record, read, and cross-reference project governance work state with OPENDOG observation evidence.

- id: FT-03.09.01
  title: Store and surface governance lanes and nodes
  level: L3
  parent: FT-03.09
  lifecycle: designing
  requirement_ranges: [GOV-01..08]
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

## Contracts

4 new versioned contract IDs in `src/contracts.rs`, following the existing `MCP_*_V1` pattern:

| Tool | Contract ID |
|---|---|
| `create_governance_lane` | `MCP_CREATE_GOVERNANCE_LANE_V1` |
| `upsert_governance_node` | `MCP_UPSERT_GOVERNANCE_NODE_V1` |
| `get_governance_state` | `MCP_GET_GOVERNANCE_STATE_V1` |
| `close_governance_lane` | `MCP_CLOSE_GOVERNANCE_LANE_V1` |

All responses use the existing success envelope `{ "status": "ok", "data": { ... } }`. Errors use `{ "status": "error", "message": "..." }`. CLI `--json` output mirrors the MCP response shape. New contract sections will be added to `docs/json-contracts.md` and `docs/mcp-tool-reference.md`.

## Code Impact

### File Changes

| File | Action | Estimated Lines |
|---|---|---|
| `src/storage/schema.rs` | Modify (+2 tables, +2 indexes, version bump) | +25 |
| `src/storage/migrations.rs` | Modify (+v4→v5 fixture and test) | +40 |
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
| `tests/integration_test/cli_governance.rs` | **New** | ~350 |
| `FUNCTION_TREE.md` | Modify (+FT-03.09 L2 + FT-03.09.01 L3) | +15 |
| `.planning/REQUIREMENTS.md` | Modify (+GOV-01..08 section) | +80 |
| `docs/json-contracts.md` | Modify (+4 contract sections) | +60 |
| `docs/mcp-tool-reference.md` | Modify (+4 tool references) | +80 |
| **Total** | **7 new + 12 modified** | **~2,335 lines** |

### Scale Impact

| Metric | Before | After | Change |
|---|---|---|---|
| Source lines | ~24,800 | ~26,900 | +8.5% |
| MCP tools | 22 | 26 | +18.2% |
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
