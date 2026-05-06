# OPENDOG Overdesign Assessment

Date: 2026-05-04
Scope: current shipped surface in `src/`, `docs/`, `.planning/`

Historical note:

- this document is a pre-optimization self-assessment snapshot from 2026-05-04
- counts such as MCP tool totals reflect the repository state before the 2026-05-05 MCP surface reduction
- for the post-implementation review chain and current disposition, see:
  - `docs/superpowers/reviews/overdesign-review-2026-05-05.md`
  - `docs/superpowers/reviews/overdesign-optimization-list-2026-05-05.md`
  - `docs/superpowers/reviews/overdesign-architect-review-opinion-2026-05-05.md`

## Purpose

This note captures a focused review of whether OPENDOG has grown beyond a reasonable scope for an MCP-oriented tool.

Review criteria:

- Is the public surface too large for an MCP-first product?
- Are there too many top-level user-facing entrypoints?
- Are some capabilities better treated as internal implementation details or CLI/admin-only operations?
- Is the project spending too much complexity on guidance/product layers relative to its observation core?

## Executive Summary

Conclusion: yes, the project is currently overdesigned for an MCP.

The main issue is not raw code volume. The main issue is that OPENDOG exposes too many first-class capabilities and has started behaving like a full product platform with admin, reporting, decision-support, verification orchestration, workspace prioritization, and governance layers all exposed together.

In practice this creates four problems:

1. The MCP tool surface is too large.
2. The CLI top-level surface is too large.
3. Several entrypoints overlap heavily in purpose and payload.
4. Too much administrator/operator behavior is exposed to AI-facing surfaces.

## Evidence

### 1. Public surface exceeds a reasonable MCP boundary

Current published scope in [docs/capability-index.md](./capability-index.md):

- 25 MCP tools
- 21 CLI top-level commands
- 26 `FT-*` leaf capabilities

Source:

- [docs/capability-index.md](./capability-index.md)

For a focused MCP, this is too broad. Even if the internals are well-factored, the exposed control surface is already large enough to create discovery, maintenance, and overlap costs.

### 2. The project is dominated by the MCP/product layer

Rough source breakdown from the repository:

- `src/mcp`: 115 files, about 15.2k lines
- `src/core`: 16 files, about 2.9k lines
- `src/control`: 11 files, about 1.6k lines
- full `src`: 179 Rust files, about 24.8k lines

Interpretation:

- the observation core exists, but a disproportionate amount of complexity now sits in MCP orchestration, guidance payloads, ranking, sequencing, summaries, and supporting glue
- that is a sign the project has expanded from "monitoring backend with AI access" into "AI workflow platform"

### 3. Entry duplication is real

The current capability map treats the following as separate first-class user-facing areas:

- guidance and decision entry
- verification evidence
- multi-project prioritization
- cleanup/refactor review and data-risk
- toolchain guidance and authority boundaries

Source:

- [docs/capability-index.md](./capability-index.md)

This is already too much conceptual branching for an MCP. Several of these are not separate product families from the user's perspective; they are slices of one guidance problem.

Examples:

- `get_agent_guidance`
- `get_decision_brief`
- `get_workspace_data_risk_overview`

These are all close variants of "tell me where to look next and why". The implementation also shows composition and merging across these layers rather than clean independence:

- [src/mcp/workspace_decision.rs](/opt/claude/opendog/src/mcp/workspace_decision.rs:108)

### 4. Too many admin-style operations are exposed as MCP tools

The current MCP surface includes:

- config read/update/reload
- project export
- retained-evidence cleanup
- verification recording
- verification command execution
- project deletion

These are valid operations in a system, but they are not all good default MCP capabilities.

From an AI-tooling perspective, the safest and highest-value MCP surface should usually be:

- read-heavy
- narrow
- low-side-effect
- easy to explain

By contrast, this project exposes many mutable or operational actions directly to the MCP layer.

### 5. The CLI top level is already too wide

Current top-level CLI commands in [src/cli/mod.rs](/opt/claude/opendog/src/cli/mod.rs:23):

- `create`
- `snapshot`
- `start`
- `stop`
- `config`
- `export`
- `cleanup-data`
- `report`
- `mcp`
- `stats`
- `unused`
- `list`
- `agent-guidance`
- `decision-brief`
- `data-risk`
- `workspace-data-risk`
- `record-verification`
- `verification`
- `run-verification`
- `delete`
- `daemon`

That count alone is enough to say the surface should be compressed. This is beyond a lightweight command interface for an MCP companion.

### 6. Disk size is not the main problem

Repository disk usage is misleading because `target/` dominates the size:

- `target/`: about 2.5G
- `src/`: about 1.4M
- `docs/`: about 548K
- `.planning/`: about 208K

Interpretation:

- this is not a case where the project is obviously too large because the codebase itself is massive
- this is a case where the public product shape is too broad relative to its core purpose

## Design Judgment

If OPENDOG is meant to be an MCP-first tool, then its ideal shape is closer to:

- one observation core
- one reporting interface
- one guidance interface
- one verification/status interface
- minimal mutation/admin operations

The current shape is broader than that. It looks more like a platform that grew several adjacent product lines without re-compressing the public surface.

The key distinction:

- the monitoring core is not the overdesigned part
- the exposed product surface around that core is the overdesigned part

## Recommended Scope Reduction

### A. Collapse overlapping AI entrypoints

Recommendation:

- keep one primary AI guidance entrypoint
- remove the distinction between `agent_guidance` and `decision_brief` at the public surface
- fold workspace-level data-risk prioritization into the same guidance surface rather than keeping it as a separate first-stop tool

Preferred shape:

- `get_guidance`
  - `scope=project|workspace`
  - optional detail level
  - guidance includes prioritized next step, risks, evidence gaps, and target project/file hints

Why:

- users do not need three ways to ask "what next?"
- this is the biggest single surface simplification available

### B. Collapse reporting tools into one tool family

Current fragmentation:

- `get_stats`
- `get_unused_files`
- `get_time_window_report`
- `compare_snapshots`
- `get_usage_trends`

Recommendation:

- expose one public reporting tool family, for example `get_report`
- select mode with a parameter:
  - `stats`
  - `unused`
  - `window`
  - `compare`
  - `trend`

Why:

- these are all observation queries over the same evidence base
- splitting them as separate MCP tools inflates the menu without adding real conceptual clarity

### C. Move operator/admin mutations out of the MCP default surface

Good candidates to downscope to CLI/admin-only:

- config update/reload
- export
- cleanup retained evidence
- record verification result
- run verification command
- delete project

Why:

- these are operational controls, not core AI-consumption tools
- they add risk and conceptual overhead
- they make the MCP feel like a system administration API instead of a focused decision-support API

### D. Merge data-risk and cleanup/refactor review into guidance/reporting

Current split:

- `get_data_risk_candidates`
- `get_workspace_data_risk_overview`
- review-focused guidance embedded elsewhere

Recommendation:

- keep data-risk as a report mode or a guidance sub-layer, not as multiple first-class surfaces

Why:

- suspicious data review is a specialized analysis mode, not a top-level product line
- the current split creates more branching than value

### E. Compress the CLI into domain groups

Current CLI is too flat.

Recommended top-level groups:

- `project`
- `observe`
- `report`
- `guide`
- `verify`
- `admin`
- `daemon`
- `mcp`

Example mapping:

- `project`: create, list, delete
- `observe`: snapshot, start, stop
- `report`: stats, unused, compare, trend, window, data-risk
- `guide`: guidance, decision
- `verify`: status, record, run
- `admin`: config, export, cleanup-data

This would bring the top-level menu back into a range that is easier to learn and maintain.

## Proposed Target Surface

### MCP target

Recommended MCP public surface should be around 6-8 tools, for example:

1. `list_projects`
2. `create_project`
3. `take_snapshot`
4. `set_monitoring_state`
5. `get_report`
6. `get_guidance`
7. `get_verification_status`
8. `get_data_risk`

Everything else should either:

- become an internal helper
- move to CLI/admin-only
- or be folded into one of the tools above

### CLI target

Recommended CLI top level should stay under 8 groups:

1. `project`
2. `observe`
3. `report`
4. `guide`
5. `verify`
6. `admin`
7. `daemon`
8. `mcp`

## Priority Cuts

If the project wants to reduce complexity without rewriting the core, the highest-value cuts are:

1. Merge `get_agent_guidance`, `get_decision_brief`, and workspace-first prioritization into one guidance surface.
2. Merge reporting tools into one public report tool family.
3. Move config/export/cleanup/delete/verification execution out of the MCP default tool list.
4. Compress CLI top-level commands into grouped domains.

These four changes would materially improve focus without discarding the observation engine or the useful analysis work already done.

## What Should Not Be Cut First

The following are not the main problem and should not be the first cuts:

- per-project isolation
- snapshotting
- monitoring
- basic stats
- unused-file detection
- repository risk detection itself

Those are close to the core purpose. The problem starts when too many derivative decision layers become separate user-facing product surfaces.

## Final Judgment

OPENDOG currently behaves less like "an MCP for project observation" and more like "a small AI workflow platform built around project observation".

That is the overdesign signal.

If the goal is to keep this as a strong MCP, the right move is not to rewrite the core. The right move is to aggressively compress the public surface and demote several capabilities from first-class product features to internal or CLI/admin-only responsibilities.
