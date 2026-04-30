# Capability Index

## Purpose

This is the single-page navigation map for OPENDOG's current shipped capability surface.

Use it when you need to answer one of these questions quickly:

- which capability area owns this behavior
- which MCP tool or CLI command should I call first
- which JSON contract explains the returned payload
- which planning artifact owns the capability structurally

Current scope at a glance:

- 3 capability layers
- 25 MCP tools
- 21 CLI top-level commands
- 26 `FT-*` leaf capabilities

Primary role of this page:

- lightweight consumption map first
- capability ownership and planning references second
- full product framing belongs in [Positioning](./positioning.md)

Authority rule:

- `git`, tests, lint, and build are external truth sources; treat OPENDOG output as decision-support evidence and switch to shell or project-native validation when confirmation is required.

## Quick Navigation

- Need current product framing first: [Positioning](./positioning.md)
- You are here: `Capability Index` — fastest capability-to-command and capability-to-doc map
- Need AI workflow order and shell handoff rules: [AI Playbook](./ai-playbook.md)
- Need MCP request/response usage: [MCP Tool Reference](./mcp-tool-reference.md)
- Need machine-readable output fields: [JSON Contracts](./json-contracts.md)

## Reading Order

Use these docs by intent:

| Need | Start Here | Then |
|---|---|---|
| Quick product positioning | [Positioning](./positioning.md) | [README](../README.md) |
| Quick current-state orientation | [README](../README.md) | [Capability Index](./capability-index.md) |
| AI usage order and safe workflow | [AI Playbook](./ai-playbook.md) | [MCP Tool Reference](./mcp-tool-reference.md) |
| MCP request/response shapes | [MCP Tool Reference](./mcp-tool-reference.md) | [JSON Contracts](./json-contracts.md) |
| CLI JSON payload consumption | [JSON Contracts](./json-contracts.md) | [AI Playbook](./ai-playbook.md) |
| Capability ownership and governance | [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md) | [.planning/REQUIREMENTS.md](../.planning/REQUIREMENTS.md) |
| Product framing and active priorities | [.planning/PROJECT.md](../.planning/PROJECT.md) | [.planning/STATE.md](../.planning/STATE.md) |

## Capability Map

| Capability area | FT ownership | Typical question | Preferred MCP first step | Preferred CLI first step | Contract / doc anchor |
|---|---|---|---|---|---|
| Project registry and isolation | `FT-01.01.01`, `FT-01.01.02` | Which projects exist and how are they isolated? | `list_projects`, `create_project`, `delete_project` | `opendog list`, `opendog create`, `opendog delete` | [MCP Tool Reference](./mcp-tool-reference.md), [FUNCTION_TREE](../.planning/FUNCTION_TREE.md) |
| Configuration policy and live reload | `FT-01.01.03` | What config is effective, and did runtime pick it up? | `get_global_config`, `get_project_config`, `update_*_config`, `reload_project_config` | `opendog config ...` | [JSON Contracts](./json-contracts.md), [MCP Tool Reference](./mcp-tool-reference.md) |
| Snapshot baseline management | `FT-01.02.01`, `FT-01.02.02` | Do I have a baseline, and what changed in the inventory? | `take_snapshot`, `compare_snapshots` | `opendog snapshot`, `opendog report compare` | [MCP Tool Reference](./mcp-tool-reference.md), [AI Playbook](./ai-playbook.md) |
| Monitoring and attribution | `FT-01.03.01`, `FT-01.03.02`, `FT-02.03.02` | Is monitoring running, and should I reuse daemon-owned state? | `start_monitor`, `stop_monitor` | `opendog start`, `opendog stop` | [README](../README.md), [CLAUDE.md](../CLAUDE.md) |
| Usage evidence and hotspot views | `FT-01.04.01`, `FT-01.04.02` | Which files are hot, cold, or never observed? | `get_stats`, `get_unused_files` | `opendog stats`, `opendog unused` | [MCP Tool Reference](./mcp-tool-reference.md), [JSON Contracts](./json-contracts.md) |
| Export and portable evidence | `FT-01.04.03` | How do I hand evidence to another tool or archive it? | `export_project_evidence` | `opendog export` | [MCP Tool Reference](./mcp-tool-reference.md), [JSON Contracts](./json-contracts.md) |
| Comparative and time-window analytics | `FT-01.04.04` | What changed recently, and which files are heating or cooling? | `get_time_window_report`, `compare_snapshots`, `get_usage_trends` | `opendog report window|compare|trend` | [AI Playbook](./ai-playbook.md), [MCP Tool Reference](./mcp-tool-reference.md) |
| Retained-evidence lifecycle | `FT-01.04.05` | Which OPENDOG-retained evidence should I prune, and is `VACUUM` worth it? | `cleanup_project_data` | `opendog cleanup-data` | [JSON Contracts](./json-contracts.md), [README](../README.md) |
| AI guidance and decision entry | `FT-03.01.01`, `FT-03.02.01`, `FT-03.02.02` | What should I do next overall or for one project? | `get_agent_guidance`, `get_decision_brief` | `opendog agent-guidance`, `opendog decision-brief` | [AI Playbook](./ai-playbook.md), [MCP Tool Reference](./mcp-tool-reference.md) |
| Verification evidence | `FT-03.03.01` | Do I already have test/lint/build evidence, or should I write some? | `get_verification_status`, `record_verification_result`, `run_verification_command` | `opendog verification`, `opendog record-verification`, `opendog run-verification` | [JSON Contracts](./json-contracts.md), [MCP Tool Reference](./mcp-tool-reference.md) |
| Multi-project prioritization | `FT-03.04.01` | Which project deserves attention first across the workspace, and why? | `get_workspace_data_risk_overview`, `get_agent_guidance` | `opendog workspace-data-risk`, `opendog agent-guidance` | [AI Playbook](./ai-playbook.md), [JSON Contracts](./json-contracts.md) |
| Cleanup/refactor review and data-risk | `FT-03.05.01`, `FT-03.08.01`, `FT-03.08.02` | Which files need review for unused, mixed, mock, or hardcoded-data reasons? | `get_data_risk_candidates` | `opendog data-risk` | [MCP Tool Reference](./mcp-tool-reference.md), [AI Playbook](./ai-playbook.md) |
| Toolchain guidance and authority boundaries | `FT-03.06.01`, `FT-03.07.01` | When should I trust OPENDOG, and when should I switch to shell or tests? | `get_agent_guidance`, `get_decision_brief` | `opendog agent-guidance`, `opendog decision-brief` | [AI Playbook](./ai-playbook.md), [JSON Contracts](./json-contracts.md) |

## Entry Surface Map

Use this when you know the surface first and need to know what it is good at.

| Surface | Best for | Not the best first tool when |
|---|---|---|
| MCP | AI-driven structured workflows, guidance, reporting, config mutation, evidence recording | You only need a quick human-readable terminal summary |
| CLI | Human/operator usage, spot checks, JSON piping, shell-centric workflows | Another AI already consumes MCP directly |
| Local control plane | Reusing daemon-owned project state consistently | The daemon is not running and simple local fallback is enough |
| Shell | Git truth, semantic diff, project-native tests, direct inspection after OPENDOG narrows scope | You still do not know which project or file deserves attention |

## Consumption Guidance

Use the product in two different ways on purpose:

- Default consumption path: start from `get_decision_brief`, `get_agent_guidance`, `get_workspace_data_risk_overview`, or the matching CLI commands, then switch to shell only when OPENDOG explicitly points you to git, tests, or direct file inspection
- Governance path: open `FUNCTION_TREE`, `REQUIREMENTS`, task cards, and validation scripts when you are changing capability boundaries, adding a requirement family, or reviewing whether work still maps cleanly to existing `FT-*` leaves

This distinction keeps routine AI/operator use lightweight while preserving strong capability governance for structural changes.

## Baseline vs Current Surface

Keep this distinction explicit:

- Phase 4 baseline introduced the original 8 MCP control tools and 8 matching CLI commands.
- The current product surface is larger because later requirement families added reporting, config, export, retained-evidence cleanup, verification, guidance, and data-risk entrypoints.
- Do not treat `MCP-01..09` or `CLI-01..09` as the full current surface area.

## AI First-Command Matrix

Use this when you want the shortest practical route from question to action.

| Situation | First command | Second step | When to switch to shell |
|---|---|---|---|
| New project, no OPENDOG state yet | `create_project` / `opendog create` | `take_snapshot` / `opendog snapshot` | The project path, repo root, or registration target is still ambiguous |
| Project exists but no baseline | `take_snapshot` / `opendog snapshot` | `get_agent_guidance` or `get_decision_brief` | You need to inspect ignore patterns, repo layout, or tracked file classes directly |
| Baseline exists but no monitoring | `start_monitor` / `opendog start` | `get_time_window_report` or `get_stats` after some activity | You need direct repo truth instead of observation-state setup |
| Need one stable AI-facing decision envelope first | `get_decision_brief` / `opendog decision-brief` | Follow `entrypoints.next_mcp_tools` or `entrypoints.next_cli_commands` | The brief points to external truth that OPENDOG has not captured yet |
| Need a broader “what next” recommendation | `get_agent_guidance` / `opendog agent-guidance` | Drill into the highest-priority project by `attention_score` or run the suggested report/verification step | You need semantic diff, tests, or direct file inspection to confirm the recommendation |
| Need to distinguish missing evidence from stale evidence first | `get_agent_guidance` / `opendog agent-guidance` | Read `layers.workspace_observation` and `project_overviews[*].observation` before choosing the next tool | You still need repo-native validation after OPENDOG identifies a stale snapshot or stale verification path |
| Need cross-project prioritization | `get_workspace_data_risk_overview` / `opendog workspace-data-risk` | `get_data_risk_candidates` or `get_agent_guidance` on the top project | Project ranking alone is not enough and you need repo-native validation |
| Need cleanup or refactor safety signals | `get_verification_status` / `opendog verification` | `get_data_risk_candidates` / `opendog data-risk` | You are about to make broad edits, delete files, or rely on stale verification evidence |
| Need suspicious mock or hardcoded-data review | `get_data_risk_candidates` / `opendog data-risk` | `get_verification_status` or targeted report commands | You need `git diff`, `rg`, tests, or manual file review before changing risky files |
| Need recent activity shape | `get_time_window_report` / `opendog report window` | `get_stats` or `get_usage_trends` | You need semantic explanation for why the active files changed |
| Need snapshot delta or inventory change | `compare_snapshots` / `opendog report compare` | `get_time_window_report` for recency context | You need the actual code diff instead of baseline inventory delta |
| Need heating/cooling signals | `get_usage_trends` / `opendog report trend` | `get_stats` or `get_data_risk_candidates` on rising files | The trend is interesting but you still need code-level evidence |
| Need storage cleanup only | `cleanup_project_data --dry_run` / `opendog cleanup-data --dry-run` | Re-run without `--dry-run`, optionally with `--vacuum` | You are unsure whether the target is OPENDOG-retained evidence or source data |

## Related Docs

- [Positioning](./positioning.md)
- [README](../README.md)
- [AI Playbook](./ai-playbook.md)
- [MCP Tool Reference](./mcp-tool-reference.md)
- [JSON Contracts](./json-contracts.md)
- [CLAUDE.md](../CLAUDE.md)
- [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md)
