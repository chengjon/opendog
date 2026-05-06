# OPENDOG Overdesign Review — First-Principles Architect Assessment

Date: 2026-05-05
Reviewer: first-principles-fullstack-architect agent
Scope: independent verification of `docs/overdesign-assessment-2026-05-04.md`

## Purpose

This document is an independent first-principles review of whether OPENDOG's feature boundaries are overdesigned. It verifies or challenges the project's own self-assessment against verified code facts.

## Review Chain

Use the overdesign document set in this order:

- `docs/overdesign-assessment-2026-05-04.md` — pre-optimization self-assessment snapshot
- `docs/superpowers/reviews/overdesign-review-2026-05-05.md` — formal review against repository facts
- `docs/superpowers/reviews/overdesign-optimization-list-2026-05-05.md` — implementation disposition and post-cut status
- `docs/superpowers/reviews/overdesign-architect-review-opinion-2026-05-05.md` — independent follow-up opinion on the review chain

## Verified Facts

| Metric | Claimed | Actual | Match |
|--------|---------|--------|-------|
| MCP Rust files | 115 | 115 | Yes |
| MCP total lines | ~15,200 | 15,197 | Yes |
| Core Rust files | 16 | 16 | Yes |
| Core total lines | ~2,940 | 2,944 | Yes |
| Total src files | 179 | 179 | Yes |
| Total src lines | ~24,800 | 24,812 | Yes |
| MCP tools | 25 | 25 | Yes |
| CLI commands | 21 | 21 | Yes |

Additional facts discovered during review:

- MCP non-test code: ~10,191 lines
- MCP test code: ~5,006 lines
- `json!()` macro invocations in non-test MCP: 354
- Handler glue code: 899 lines across 7 files

## Claim-by-Claim Verification

### Claim 1: "The MCP tool surface is too large" (25 tools)

**PARTIALLY CONFIRMED**

25 is a large tool count for an MCP server. A typical focused MCP server exposes 5-10 tools. However:

- 8 tools are straightforward CRUD/observation (create, snapshot, start, stop, stats, unused, list, delete) — non-negotiable
- 5 tools are config management — the most clearly overexposed group
- 3 tools are guidance/decision — clearest overlap
- 2 tools are data-risk — reasonable to keep separate by scope
- 2 tools are verification execution — debatable

The real problem is not the number but that 5 config operations have no business being in an AI-facing MCP surface. Remove those and the count drops to 20.

**Verdict:** Surface is larger than necessary, but the self-assessment's target of 6-8 tools is itself too aggressive. A realistic target is 15-17.

### Claim 2: "Too many admin mutations in MCP"

**CONFIRMED**

These 5 MCP tools are operator/admin functions that should be CLI-only:

1. `update_global_config` — writes persistent state
2. `update_project_config` — writes persistent state
3. `reload_project_config` — triggers runtime reconfiguration
4. `export_project_evidence` — filesystem I/O producing artifact files
5. `cleanup_project_data` — deletes retained evidence rows and vacuums SQLite

Keep read-only config tools (`get_global_config`, `get_project_config`). Keep `delete_project` — an AI that creates a project needs to be able to clean it up. Keep verification recording tools — they are data intake, not admin.

### Claim 3: "Overlapping AI entrypoints" (guidance + decision_brief + workspace_data_risk)

**CONFIRMED, with nuance**

The code proves the overlap: `build_decision_brief_for_projects` literally calls `build_agent_guidance_for_projects` first, then wraps it with additional data-risk summaries and a decision envelope. The decision brief is a strict superset of agent guidance plus data-risk.

However, `get_workspace_data_risk_overview` answers a fundamentally different question ("which project has the most suspicious data?") and should remain separate.

**Verdict:** Merge guidance and decision_brief into one tool with a `detail` parameter. Keep workspace_data_risk separate.

### Claim 4: "CLI too flat" (21 commands)

**REJECTED**

The CLI already has subcommand grouping (`config` has subcommands, `report` has subcommands). The remaining flat commands are discoverable with shell autocomplete. CLI reorganization is a cosmetic change that does not reduce code complexity, maintenance burden, or the MCP surface. It is the lowest-value recommendation in the self-assessment.

### Claim 5: "MCP/product layer dominates" (15.2k vs 2.9k)

**CONFIRMED, but the diagnosis is incomplete**

The 5.2x ratio is real but overstated by Rust's `json!()` macro being a line-hungry way to build structured output. 354 `json!()` invocations inflate line count without proportional complexity. A single logical output structure might consume 40-80 lines of `json!()` calls.

The actual "product logic" is closer to 6,000 lines of meaningful code (scoring, detection, recommendation, gating), roughly 2x the core. That is still disproportionate but not the 5.2x catastrophe the raw numbers suggest.

### Claim 6: "The observation core is NOT the problem"

**CONFIRMED**

The core at 2,944 lines across 16 files is well-proportioned. The largest files (monitor.rs: 413, report.rs: 334, retention.rs: 317) are exactly what a file observation engine should contain. No bloat. No over-abstraction.

## 5-Why Root Cause Analysis

1. **Why so much MCP code?** Because the project built a comprehensive "AI decision support" layer beyond observation.
2. **Why was the decision support layer built?** Because the vision expanded from "monitor files for AI tools" to "tell AI tools what to do next, in what order, and when it is safe."
3. **Why did the vision expand?** Because the problem statement naturally cascades: "which files matter" → "which project matters most" → "is it safe to modify" → "what should I verify first." Each step adds a product surface.
4. **Why was each cascade step exposed as a separate MCP tool?** Because each new insight was wired directly to MCP as a new tool rather than being absorbed into existing tools.
5. **Why wasn't consolidation done incrementally?** Because the FT-* function tree treated each capability as a separate leaf node, making it easy to justify each tool as "mapping to a distinct requirement." The function tree became a rationalization mechanism rather than a constraint mechanism.

**Root cause:** The project conflated "the AI needs this information" with "the AI needs a dedicated MCP tool for this information." These are not the same thing. Information can be folded into existing tool responses without losing utility.

**Secondary cause:** Rust's JSON assembly pattern makes every output structure expensive in lines, creating a false sense that the MCP layer was "doing more" than it actually was.

## Real Cost Assessment

| Overdesign Area | Maintenance Cost | Discovery Cost | Code Complexity Cost | Real or Aesthetic? |
|----------------|-----------------|----------------|----------------------|---------------------|
| 5 config mutation tools in MCP | Low | Medium | Low | Real |
| 3 overlapping guidance tools | Medium | High | Medium | Real |
| 5 reporting tools instead of 1 | Low | Medium | Low | Marginal |
| 21 flat CLI commands | Negligible | Negligible | Zero | Aesthetic |
| FT-* governance tree | Negligible | Low | Zero | Governance theater |
| JSON assembly verbosity | Medium | Zero | Medium | Real structural issue |

## Prioritized Recommendations

### Tier 1: High Leverage, Low Disruption

**1. Merge `get_agent_guidance` and `get_decision_brief` into one tool.**

New tool: `get_guidance` with parameter `detail` accepting `summary` (current guidance behavior) or `decision` (current decision_brief behavior). The merge path is already in the code — `build_decision_brief_for_projects` already calls `build_agent_guidance_for_projects` internally.

**2. Remove 5 mutation tools from MCP.**

Move to CLI-only:
- `update_global_config`
- `update_project_config`
- `reload_project_config`
- `export_project_evidence`
- `cleanup_project_data`

Keep the read-only config tools. The handlers for removed tools already have CLI equivalents.

**3. Keep `get_workspace_data_risk_overview` separate.**

It answers a fundamentally different question than guidance.

### Tier 2: Moderate Leverage, Moderate Disruption

**4. Merge 3 time-series reporting tools into `get_report`.**

Current: `get_time_window_report`, `compare_snapshots`, `get_usage_trends`.
New: `get_report` with `mode` parameter accepting `window`, `compare`, `trend`.

Keep `get_stats` and `get_unused_files` separate — they are the highest-frequency tools and deserve direct-call convenience.

### Tier 3: Skip

- CLI reorganization — no structural value.
- FT-* consolidation — documentation-only, no code impact.

## What Should NOT Be Cut

1. **`mock_detection.rs` (371 lines)** — Data-risk detection engine. This IS the product differentiator.
2. **`verification_evidence.rs` (606 lines)** — Safety-critical gate assessment system.
3. **`attention.rs` (412 lines)** — Multi-factor scoring and prioritization engine.
4. **`toolchain.rs` (359 lines)** — Project type detection, makes all other guidance useful.
5. **The entire observation core (2,944 lines)** — Zero bloat.
6. **`record_verification_result` and `run_verification_command`** — AI data intake tools, not admin operations.
7. **`delete_project`** — Destructive but necessary for cleanup.

## Target Surface After Recommended Cuts

| # | Tool | Source |
|---|------|--------|
| 1 | `create_project` | Unchanged |
| 2 | `delete_project` | Unchanged |
| 3 | `list_projects` | Unchanged |
| 4 | `take_snapshot` | Unchanged |
| 5 | `start_monitor` | Unchanged |
| 6 | `stop_monitor` | Unchanged |
| 7 | `get_stats` | Unchanged |
| 8 | `get_unused_files` | Unchanged |
| 9 | `get_report` | Merged from `get_time_window_report` + `compare_snapshots` + `get_usage_trends` |
| 10 | `get_guidance` | Merged from `get_agent_guidance` + `get_decision_brief` |
| 11 | `get_data_risk` | Renamed from `get_data_risk_candidates` |
| 12 | `get_workspace_data_risk` | Renamed from `get_workspace_data_risk_overview` |
| 13 | `get_verification_status` | Unchanged |
| 14 | `record_verification_result` | Unchanged |
| 15 | `run_verification_command` | Unchanged |
| 16 | `get_global_config` | Unchanged (read-only) |
| 17 | `get_project_config` | Unchanged (read-only) |

**17 tools.** Down from 25. Estimated effort: 2-3 days of annotation removal and parameter refactoring. No core changes required.

## Final Verdict

The self-assessment reached the right conclusion ("overdesigned") but for partially wrong reasons and with some recommendations that are counterproductive:

- **Correct:** Admin mutations in MCP, overlapping guidance entrypoints, and the 5.2x ratio are real problems.
- **Wrong:** The 6-8 tool target is too aggressive. The real target should be 15-17 tools.
- **Wrong:** CLI reorganization is a cosmetic distraction with no structural value.
- **Wrong:** Collapsing all 5 reporting tools into one loses the direct-call convenience of `get_stats` and `get_unused_files`.

The observation core is solid. The product logic (scoring, detection, gate assessment, toolchain detection) is justified. The problem is surface bloat: too many tools exposing operations that should be internal or CLI-only, and one unnecessary redundancy in the guidance/decision split.

## Implementation Follow-Up

Post-review implementation status in the repository:

- 5 MCP operator/admin mutation tools were removed from the MCP surface:
  - `update_global_config`
  - `update_project_config`
  - `reload_project_config`
  - `export_project_evidence`
  - `cleanup_project_data`
- MCP guidance was merged into one public entrypoint:
  - `get_guidance(detail = "summary")` replaces the old guidance-facing MCP entry behavior
  - `get_guidance(detail = "decision")` replaces the old decision-brief MCP entry behavior
- The old MCP public tool names `get_agent_guidance` and `get_decision_brief` were not kept as aliases because the goal of this batch was real surface reduction, not compatibility layering.
- Current MCP surface after the implemented cuts: **19 tools**
- CLI operator ergonomics were intentionally left unchanged:
  - `opendog agent-guidance`
  - `opendog decision-brief`

## Final Post-Implementation Assessment

### 1. Should CLI `agent-guidance` and `decision-brief` also be merged?

**No. Leave the CLI unchanged.**

Reasoning:

- CLI is an operator surface, not an AI tool-discovery surface.
- Shell autocomplete already makes the current command count manageable.
- A merged CLI command would add an indirect `--detail` layer without reducing internal complexity in a meaningful way.
- MCP and CLI do not need forced symmetry when their users and discovery costs differ.

### 2. Should `get_workspace_data_risk_overview` be folded into `get_guidance`?

**No. Keep it separate.**

Reasoning:

- It answers a different question: cross-project suspicious-data prioritization, not "what should I do next?"
- It carries a different payload shape than guidance.
- It is computationally heavier than normal guidance usage and should not become a mandatory part of every `get_guidance` call.
- The separate tool remains justified even after the broader MCP surface reduction.

## Independent Opinion Disposition

The follow-up independent opinion in `overdesign-architect-review-opinion-2026-05-05.md` surfaced three useful refinements:

- the merged guidance path still leaves internal adapter debt in the split helper handlers and param structs
- `json!()`-driven payload assembly is a real maintainability concern, not just a line-count artifact
- config read tools should only be reconsidered if another surface exposes the same resolved-config contract or usage evidence shows they are unnecessary

Disposition after reviewing that opinion:

- accept the factual refinements above
- keep the current decision that CLI guidance commands remain split
- keep the current decision that `get_workspace_data_risk_overview` remains separate
- keep report-tool consolidation as optional future work rather than an immediate required batch
- keep the current stopping point at 19 MCP tools unless later usage evidence justifies another merge

### Closure Judgment

The implemented reduction from **25 to 19 MCP tools** lands at a reasonable balance point.

- The highest-value surface bloat has been removed.
- The remaining direct tools still have clear usage boundaries.
- Further immediate consolidation would risk merging unlike workflows for cosmetic rather than structural benefit.

Current recommendation: stop MCP surface reduction here unless future usage proves that the report trio should also be merged.

---

*Reviewed by first-principles-fullstack-architect agent on 2026-05-05*
