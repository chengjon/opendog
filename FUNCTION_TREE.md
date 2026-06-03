---
function_tree_version: 1.4
last_updated: "2026-06-03"
canonical_role: business_capability_anchor
level_model:
  L1: domain_capability
  L2: module_capability
  L3: atomic_function
lifecycle_values:
  - designing
  - in_progress
  - shipped
  - deprecated
mapping_contract:
  requirements: every detailed requirement should map to at least one L3 function node
  roadmap: every phase or task card should declare the L3 function nodes it changes
  governance: function nodes are capability-first, not CLI-first or MCP-first
gate_intent:
  - no orphan requirements
  - no orphan task cards
  - no unowned leaf capabilities
  - no oversized structural files without an explicit size budget
task_card_workflow:
  directory: .planning/task-cards
  validator: scripts/validate_task_cards.py
requirement_mapping_workflow:
  file: .planning/REQUIREMENTS.md
  validator: scripts/validate_requirement_mappings.py
governance_workflow:
  guide: .planning/GOVERNANCE.md
  validator: scripts/validate_planning_governance.py
structural_hygiene_workflow:
  file: .planning/structural_hygiene_rules.json
  validator: scripts/validate_structural_hygiene.py
---

# Function Tree: OPENDOG

## Purpose

`FUNCTION_TREE.md` is the canonical business-capability hierarchy for OPENDOG.

It sits between:

- `.planning/PROJECT.md` as the product intent anchor
- `.planning/REQUIREMENTS.md` as the detailed requirement catalog
- `.planning/ROADMAP.md` and later task cards as execution planning artifacts

This file is not a feature wishlist. It is the structured capability tree used to answer:

- what business capability exists
- what capability is being changed
- which requirements belong to that capability
- which roadmap phase or task card is allowed to touch it

It should be read with two different intents:

- day-to-day consumption: understand which capability family owns an already shipped CLI, MCP, daemon, or guidance behavior
- structural change control: keep new requirements, task cards, and future capability edits anchored to the right `FT-*` leaves

Related review entrypoint:

- for the overdesign review chain and scope-reduction decisions, start at [docs/superpowers/reviews/README.md](/opt/claude/opendog/docs/superpowers/reviews/README.md)

This distinction matters because OPENDOG's runtime product is intentionally consumed as a lightweight three-layer system, while this file exists mainly as a governance overlay for capability evolution.

Current design posture:

- no product-direction drift: the tree still describes the original multi-project observation plus AI decision-support mission
- broad but bounded surface: the tree is wider than a simple monitoring backend, but each branch remains constrained by evidence, authority, and non-destructive advisory boundaries
- current priority is maintenance and quality: all declared capability leaves are shipped; future work should improve reliability, performance, and clarity of existing leaves rather than expand into new capability families
- current hardening baseline: all FT-01, FT-02, and FT-03 leaf nodes are shipped, covering fd-attribution credibility, soft verification gates, repository-truth boundary projection, bounded MCP observation payloads, read-only MCP Resources, data-risk noise reduction, and machine-readable resume sequencing
- current technical-debt posture: the latest cleanup line consolidated repeated low-risk constants and test fixtures while preserving behavior; `validate_repository_gate.py`, remote Repository Gate, External Security Audit, and release-readiness checks all pass on the current `master` lineage

Current capability investment priority:

- high: keep validating and hardening attribution, stats, unused-file review, and guidance credibility. These are OpenDog's core evidence foundation; attribution mistakes or misleading observation summaries contaminate every downstream recommendation.
- high: keep maintenance work evidence-backed and scoped to real drift reduction. Prefer test-only or local helper cleanup for incremental debt work, and treat production contract fields, data-risk rules, and cross-surface payload keys as separately scoped changes with broader verification.
- high: use `docs/project-exchange/` reports and the shared issue index to collect A/B/C project feedback before opening new task cards. Do not expand the capability surface from speculation.
- medium: deepen `FT-03.01`, `FT-03.03`, and `FT-03.04` so observation freshness, verification gates, and multi-project prioritization make it easier for AI to answer "which project now?" and "is it safe to change?".
- medium-low: continue tuning data-risk, toolchain guidance, and boundary messaging, but let real project reports drive the exact work.
- avoid: broad horizontal expansion. Keep engineering effort tied to validated pain points and shared issues.

## Structural Rules

### Level Rules

- `L1` = domain capability
- `L2` = module capability
- `L3` = atomic function

Only `L3` nodes should normally be used as task-card mapping targets.

### Naming Rules

- Prefer business capability names, not implementation names
- Avoid naming nodes after CLI commands, MCP tools, structs, modules, or crates
- If multiple interfaces expose the same capability, keep one shared function node

### Lifecycle Rules

- `designing`: intended capability exists conceptually but is not yet implemented
- `in_progress`: capability is partially implemented or still under active hardening
- `shipped`: capability is live and considered part of the supported product surface
- `deprecated`: capability is being retired and should not receive new investment

### Mapping Rules

- Requirements should map downward into one or more `L3` nodes
- Roadmap phases and task cards should declare the `L3` nodes they affect
- Code, tests, and docs may reference `FT-*` IDs when helpful, but the primary governance mapping is requirement/task-card level
- Capability ownership should stay stable unless there is a real need to split or merge leaves; avoid inventing new `FT-*` branches just to mirror interface growth

## Machine-Readable Registry

```yaml
nodes:
  - id: FT-01
    title: Observation and Intelligence Capture
    level: L1
    parent: null
    lifecycle: shipped
    summary: Capture isolated project state, observation evidence, and file-usage intelligence.

  - id: FT-01.01
    title: Project Registry and Isolation
    level: L2
    parent: FT-01
    lifecycle: shipped
    summary: Manage projects as isolated monitoring and storage units.

  - id: FT-01.01.01
    title: Register and manage project records
    level: L3
    parent: FT-01.01
    lifecycle: shipped
    requirement_ranges: [PROJ-01..03]
    roadmap_phases: [1]
    summary: Create, list, and delete project records.

  - id: FT-01.01.02
    title: Isolate per-project state and configuration
    level: L3
    parent: FT-01.01
    lifecycle: shipped
    requirement_ranges: [PROJ-04..05]
    roadmap_phases: [1]
    summary: Keep storage, config, and namespace boundaries independent per project.

  - id: FT-01.01.03
    title: Manage configuration policy and live reload
    level: L3
    parent: FT-01.01
    lifecycle: shipped
    requirement_ranges: [CONF-01..03]
    roadmap_phases: [4, 5]
    summary: Manage per-project and global defaults and apply safe configuration reload behavior.

  - id: FT-01.02
    title: Snapshot Baseline Management
    level: L2
    parent: FT-01
    lifecycle: shipped
    summary: Build and refresh the baseline file inventory for each project.

  - id: FT-01.02.01
    title: Scan and filter baseline inventory
    level: L3
    parent: FT-01.02
    lifecycle: shipped
    requirement_ranges: [SNAP-01..04]
    roadmap_phases: [1]
    summary: Recursively scan project files and exclude known noise safely.

  - id: FT-01.02.02
    title: Refresh snapshot state incrementally
    level: L3
    parent: FT-01.02
    lifecycle: shipped
    requirement_ranges: [SNAP-05]
    roadmap_phases: [1]
    summary: Add, remove, and update snapshot entries without rebuilding everything manually.

  - id: FT-01.03
    title: Runtime Monitoring and Attribution
    level: L2
    parent: FT-01
    lifecycle: shipped
    summary: Observe AI-related file activity and approximate file ownership over time.

  - id: FT-01.03.01
    title: Detect AI-held project file activity
    level: L3
    parent: FT-01.03
    lifecycle: shipped
    requirement_ranges: [MON-03, PROC-01..04, PROC-06]
    roadmap_phases: [2]
    summary: Sample whitelisted AI processes, distinguish file-level descriptors from directory descriptors, and detect which project files they currently hold open without fan-out attribution.

  - id: FT-01.03.02
    title: Detect file changes and correlate attribution evidence
    level: L3
    parent: FT-01.03
    lifecycle: shipped
    requirement_ranges: [MON-01..02, MON-04..06, PROC-05]
    roadmap_phases: [2]
    summary: Track changes, starts, stops, and approximate attribution around change events.

  - id: FT-01.04
    title: Usage Analytics and Evidence Lifecycle
    level: L2
    parent: FT-01
    lifecycle: shipped
    summary: Turn raw observations into file-level evidence and review signals.

  - id: FT-01.04.01
    title: Record per-file usage evidence
    level: L3
    parent: FT-01.04
    lifecycle: shipped
    requirement_ranges: [STAT-01..05]
    roadmap_phases: [3]
    summary: Persist access count, estimated duration, modifications, and last-access evidence.

  - id: FT-01.04.02
    title: Produce unused and hotspot views
    level: L3
    parent: FT-01.04
    lifecycle: shipped
    requirement_ranges: [STAT-06..08]
    roadmap_phases: [3]
    summary: Expose project-level statistical queries such as unused files, hotspot candidates, and source/infrastructure/backup/project filtered observation views.

  - id: FT-01.04.03
    title: Export usage evidence in portable formats
    level: L3
    parent: FT-01.04
    lifecycle: shipped
    requirement_ranges: [EXPORT-01..02]
    roadmap_phases: [3, 4]
    summary: Export usage and analytics output in reusable machine-oriented formats.

  - id: FT-01.04.04
    title: Produce comparative and time-windowed analytics
    level: L3
    parent: FT-01.04
    lifecycle: shipped
    requirement_ranges: [RPT-01..03]
    roadmap_phases: [3, 4, 5]
    summary: Compare snapshots and summarize trends across time windows.

  - id: FT-01.04.05
    title: Manage retained evidence lifecycle and storage hygiene
    level: L3
    parent: FT-01.04
    lifecycle: shipped
    requirement_ranges: [RET-01..06]
    roadmap_phases: [5, 6]
    summary: Let users and AI prune retained OPENDOG evidence safely and inspect storage-maintenance signals.

  - id: FT-02
    title: Service Delivery, Runtime and Coordination
    level: L1
    parent: null
    lifecycle: shipped
    summary: Expose OPENDOG capabilities through operator and AI-facing interfaces.

  - id: FT-02.01
    title: Operator Workflow Surface
    level: L2
    parent: FT-02
    lifecycle: shipped
    summary: Provide terminal-native management and reporting commands.

  - id: FT-02.01.01
    title: Provide operator workflows and reporting
    level: L3
    parent: FT-02.01
    lifecycle: shipped
    requirement_ranges: [CLI-01..09]
    roadmap_phases: [4]
    summary: Let operators access OPENDOG through the `opendog` command surface, including explicit local maintenance workflows such as manual release-binary rebuild checks.

  - id: FT-02.02
    title: AI Workflow Surface
    level: L2
    parent: FT-02
    lifecycle: shipped
    summary: Provide machine-oriented tool access for AI clients.

  - id: FT-02.02.01
    title: Provide machine-invocable workflows and reporting
    level: L3
    parent: FT-02.02
    lifecycle: shipped
    requirement_ranges: [MCP-01..09]
    roadmap_phases: [4]
    summary: Let AI clients discover and invoke OPENDOG capabilities through MCP tools and read stable state through read-only MCP Resources.

  - id: FT-02.03
    title: Daemon Runtime and Coordination
    level: L2
    parent: FT-02
    lifecycle: shipped
    summary: Run OPENDOG as a stable background service in the target environment.

  - id: FT-02.03.01
    title: Run stable daemon lifecycle and deployment hooks
    level: L3
    parent: FT-02.03
    lifecycle: shipped
    requirement_ranges: [DAEM-01..05]
    roadmap_phases: [5]
    summary: Support daemon mode, systemd integration, graceful shutdown, and environment checks.

  - id: FT-02.03.02
    title: Coordinate daemon-owned project operations through local control plane
    level: L3
    parent: FT-02.03
    lifecycle: shipped
    requirement_ranges: [CTRL-01..05]
    roadmap_phases: [5]
    summary: Reuse daemon-owned monitor state and route project operations through the local control channel when available.

  - id: FT-03
    title: AI Decision Support and Governance
    level: L1
    parent: null
    lifecycle: shipped
    summary: Convert OPENDOG from a monitoring backend into a reusable AI decision-support layer.

  - id: FT-03.01
    title: Workspace Observation
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Express readiness, freshness, and evidence gaps at project and workspace scope.

  - id: FT-03.01.01
    title: Explain readiness and evidence gaps
    level: L3
    parent: FT-03.01
    lifecycle: shipped
    requirement_ranges: [OBS-01..04]
    roadmap_phases: [6]
    summary: Show whether OPENDOG has enough observation quality to support downstream conclusions, expose bounded and classification-filtered observation payload windows, and identify which bootstrap step is still missing.

  - id: FT-03.02
    title: Repository Risk and Execution Strategy
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Help AI decide whether to inspect, stabilize, verify, or modify next.

  - id: FT-03.02.01
    title: Summarize repository risk and confidence
    level: L3
    parent: FT-03.02
    lifecycle: shipped
    requirement_ranges: [RISK-01..04]
    roadmap_phases: [6]
    summary: Convert repository and observation state into evidence-backed risk summaries.

  - id: FT-03.02.02
    title: Recommend next-step execution strategy
    level: L3
    parent: FT-03.02
    lifecycle: shipped
    requirement_ranges: [STRAT-01..04]
    roadmap_phases: [6]
    summary: Recommend the next action, follow-up command path, and machine-readable resume sequence across OPENDOG, shell, verification, and observation workflows.

  - id: FT-03.03
    title: Verification Evidence
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Make verification evidence durable, queryable, and decision-relevant.

  - id: FT-03.03.01
    title: Record and reason over verification evidence
    level: L3
    parent: FT-03.03
    lifecycle: shipped
    requirement_ranges: [EVID-01..04]
    roadmap_phases: [6]
    summary: Attach recorded validation evidence, explicit freshness TTL policy, gate judgments, and verification-first sequencing context to recommendations and safety decisions.

  - id: FT-03.04
    title: Multi-Project Portfolio Prioritization
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Help AI choose which project deserves attention first.

  - id: FT-03.04.01
    title: Rank projects by attention and evidence quality
    level: L3
    parent: FT-03.04
    lifecycle: shipped
    requirement_ranges: [PORT-01..04]
    roadmap_phases: [6]
    summary: Compare projects by readiness, evidence, and review urgency.

  - id: FT-03.05
    title: Cleanup and Refactor Review
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Surface file-level review candidates without taking destructive action.

  - id: FT-03.05.01
    title: Surface cleanup and refactor candidates
    level: L3
    parent: FT-03.05
    lifecycle: shipped
    requirement_ranges: [CLEAN-01..04]
    roadmap_phases: [6]
    summary: Prioritize unused, hot, mixed, and suspicious files for later review.

  - id: FT-03.06
    title: Project Type and Toolchain Guidance
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Infer repository type and recommend appropriate validation commands.

  - id: FT-03.06.01
    title: Infer toolchain and recommend commands
    level: L3
    parent: FT-03.06
    lifecycle: shipped
    requirement_ranges: [STACKX-01..04]
    roadmap_phases: [6]
    summary: Suggest project-native validation commands, search paths, and fallback behavior from repository markers for review, verification, and sequencing handoff.

  - id: FT-03.07
    title: Constraints and Boundaries
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Prevent AI from over-claiming what OPENDOG actually knows.

  - id: FT-03.07.01
    title: State blind spots and authority boundaries
    level: L3
    parent: FT-03.07
    lifecycle: shipped
    requirement_ranges: [BOUND-01..04]
    roadmap_phases: [6]
    summary: Clarify what was observed, what was inferred, where transient reads or filtered views can hide evidence, where repository truth is missing, and when shell or project-native verification remains mandatory.

  - id: FT-03.08
    title: Mock and Hardcoded Data Review
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Detect test-only artifacts and riskier pseudo-business data across projects.

  - id: FT-03.08.01
    title: Detect mock and test-only data artifacts
    level: L3
    parent: FT-03.08
    lifecycle: shipped
    requirement_ranges: [MOCK-01, MOCK-03, MOCK-06, MOCK-07, MOCK-10]
    roadmap_phases: [6]
    summary: Detect and expose mock, fixture, stub, fake, demo, and sample data patterns.

  - id: FT-03.08.02
    title: Detect and prioritize hardcoded pseudo-business data
    level: L3
    parent: FT-03.08
    lifecycle: shipped
    requirement_ranges: [MOCK-02, MOCK-04, MOCK-05, MOCK-08, MOCK-09, MOCK-10]
    roadmap_phases: [6]
    summary: Flag risky business-like literals and mixed logic-plus-data files for review while down-ranking documentation and template-placeholder noise.

  - id: FT-03.09
    title: Governance State Observation
    level: L2
    parent: FT-03
    lifecycle: shipped
    summary: Record, read, and cross-reference project governance work state with OPENDOG observation evidence.

  - id: FT-03.09.01
    title: Store and surface governance lanes and nodes
    level: L3
    parent: FT-03.09
    lifecycle: shipped
    requirement_ranges: [GOV-01..08]
    roadmap_phases: [6]
    summary: >
      Let projects record and read their own governance work state
      (lanes and nodes), then cross-reference it with OPENDOG's
      observation evidence in guidance payloads. OPENDOG does not
      enforce governance rules — it observes and recommends.
```

## Human Tree

```text
FT-01 Observation and Intelligence Capture
  FT-01.01 Project Registry and Isolation
    FT-01.01.01 Register and manage project records
    FT-01.01.02 Isolate per-project state and configuration
    FT-01.01.03 Manage configuration policy and live reload
  FT-01.02 Snapshot Baseline Management
    FT-01.02.01 Scan and filter baseline inventory
    FT-01.02.02 Refresh snapshot state incrementally
  FT-01.03 Runtime Monitoring and Attribution
    FT-01.03.01 Detect AI-held project file activity
    FT-01.03.02 Detect file changes and correlate attribution evidence
  FT-01.04 Usage Analytics and Evidence Lifecycle
    FT-01.04.01 Record per-file usage evidence
    FT-01.04.02 Produce unused and hotspot views
    FT-01.04.03 Export usage evidence in portable formats
    FT-01.04.04 Produce comparative and time-windowed analytics
    FT-01.04.05 Manage retained evidence lifecycle and storage hygiene

FT-02 Service Delivery, Runtime and Coordination
  FT-02.01 Operator Workflow Surface
    FT-02.01.01 Provide operator workflows and reporting
  FT-02.02 AI Workflow Surface
    FT-02.02.01 Provide machine-invocable workflows and reporting
  FT-02.03 Daemon Runtime and Coordination
    FT-02.03.01 Run stable daemon lifecycle and deployment hooks
    FT-02.03.02 Coordinate daemon-owned project operations through local control plane

FT-03 AI Decision Support and Governance
  FT-03.01 Workspace Observation
    FT-03.01.01 Explain readiness and evidence gaps
  FT-03.02 Repository Risk and Execution Strategy
    FT-03.02.01 Summarize repository risk and confidence
    FT-03.02.02 Recommend next-step execution strategy
  FT-03.03 Verification Evidence
    FT-03.03.01 Record and reason over verification evidence
  FT-03.04 Multi-Project Portfolio Prioritization
    FT-03.04.01 Rank projects by attention and evidence quality
  FT-03.05 Cleanup and Refactor Review
    FT-03.05.01 Surface cleanup and refactor candidates
  FT-03.06 Project Type and Toolchain Guidance
    FT-03.06.01 Infer toolchain and recommend commands
  FT-03.07 Constraints and Boundaries
    FT-03.07.01 State blind spots and authority boundaries
  FT-03.08 Mock and Hardcoded Data Review
    FT-03.08.01 Detect mock and test-only data artifacts
    FT-03.08.02 Detect and prioritize hardcoded pseudo-business data
  FT-03.09 Governance State Observation
    FT-03.09.01 Store and surface governance lanes and nodes
```

## Current Implementation Distribution

This section complements the capability tree with the current code and interface shape.

### Code Module Distribution

Current implementation is concentrated in these directories:

- `src/core/` — observation core, snapshot, monitoring, reporting, export, retention, verification primitives
- `src/storage/` — SQLite schema and query layer
- `src/control/` — daemon-local control plane, protocol, request routing, client reuse
- `src/config/` — configuration loading, validation, patching, ignore rules, path helpers
- `src/cli/` — operator-facing command parsing and output
- `src/mcp/` — MCP routing, payloads, guidance, decision logic, data-risk, toolchain, workspace aggregation

Current rough weight by source area:

- `src/mcp`: 130 files, about 33.5k lines
- `src/core`: 36 files, about 9.2k lines
- `src/storage`: 12 files, about 3.1k lines
- `src/control`: 14 files, about 3.2k lines
- `src/config`: 5 files, about 1.2k lines
- `src/cli`: 21 files, about 4.9k lines
- full `src`: 226 Rust files, about 56.9k lines

Interpretation:

- `src/core/` is the observation kernel
- `src/storage/` is the SQLite persistence and migration layer
- `src/control/` is the runtime coordination layer
- `src/mcp/` is the largest current surface and carries most of the AI-facing orchestration complexity

### CLI Menu Distribution

Current CLI top-level commands:

- `register` (`create` remains a CLI alias)
- `snapshot`
- `start`
- `stop`
- `config`
- `export`
- `cleanup-data`
- `report`
- `self-update`
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
- `governance`

Current CLI subcommands:

- `config`
  - `show`
  - `set-project`
  - `set-global`
  - `reload`
- `report`
  - `window`
  - `compare`
  - `trend`
  - `rollup`
- `governance`
  - `create-lane`
  - `upsert-node`
  - `show`
  - `close-lane`

Re-grouped by intent, the CLI currently covers:

- project lifecycle: `register` / `create` alias, `list`, `delete`
- observation: `snapshot`, `start`, `stop`, `stats`, `unused`
- reporting: `report window`, `report compare`, `report trend`, `report rollup`, `export`
- AI guidance: `agent-guidance`, `decision-brief`, `data-risk`, `workspace-data-risk`
- verification: `verification`, `record-verification`, `run-verification`
- operations/runtime: `config *`, `cleanup-data`, `self-update`, `daemon`, `mcp`
- governance: `governance create-lane`, `governance upsert-node`, `governance show`, `governance close-lane`

### MCP Tool Distribution

Current MCP tool set (27 tools):

- project and monitoring
  - `register_project`
  - `list_projects`
  - `delete_project`
  - `take_snapshot`
  - `start_monitor`
  - `stop_monitor`
- observation and reporting
  - `get_stats` (`path_classification`: `all`, `source`, `infrastructure`, `backup`, `project`)
  - `get_unused_files` (`path_classification`: `all`, `source`, `infrastructure`, `backup`, `project`)
  - `get_time_window_report`
  - `compare_snapshots`
  - `get_usage_trends`
  - `get_activity_rollups`
- configuration inspection
  - `get_global_config`
  - `get_build_info`
  - `get_project_config`
- guidance and decision
  - `get_guidance`
- verification
  - `get_verification_status`
  - `record_verification_result`
  - `run_verification_command`
- data risk and workspace prioritization
  - `get_data_risk_candidates`
  - `get_workspace_data_risk_overview`
- orphan and deletion planning
  - `scan_orphans`
  - `verify_deletion_plan`
- governance state observation
  - `create_governance_lane`
  - `upsert_governance_node`
  - `get_governance_state`
  - `close_governance_lane`

Current MCP read-only resources (2 resources):

- static resources
  - `opendog://projects`
- resource templates
  - `opendog://project/{id}/verification`

### Interface Relationship Notes

From a capability perspective, several public entrypoints are close variants of the same capability family:

- guidance family
  - `agent-guidance`
  - `decision-brief`
  - `get_guidance`
  - `workspace-data-risk`
  - `get_workspace_data_risk_overview`
- reporting family
  - `stats`
  - `unused`
  - `report window`
  - `report compare`
  - `report trend`
  - `report rollup`
  - `get_stats`
  - `get_unused_files`
  - `get_time_window_report`
  - `compare_snapshots`
  - `get_usage_trends`
  - `get_activity_rollups`
- verification family
  - `verification`
  - `record-verification`
  - `run-verification`
  - `get_verification_status`
  - `record_verification_result`
  - `run_verification_command`
- operations family
  - `config *`
  - `get_build_info`
  - `cleanup-data`
  - `export`
  - `daemon`
  - `mcp`
- read-only state family
  - `opendog://projects`
  - `opendog://project/{id}/verification`
- orphan and deletion-planning family
  - `scan_orphans`
  - `verify_deletion_plan`
- governance family
  - `governance create-lane`
  - `governance upsert-node`
  - `governance show`
  - `governance close-lane`
  - `create_governance_lane`
  - `upsert_governance_node`
  - `get_governance_state`
  - `close_governance_lane`

Operator-only downscope now applied:

- CLI-only operator mutations and artifact flows
  - `opendog config set-global`
  - `opendog config set-project`
  - `opendog config reload`
  - `opendog export`
  - `opendog cleanup-data`

## Governance Usage

### Requirement Authoring

When adding or changing a requirement, declare which `FT-*` leaf node owns it.

### Roadmap and Task Cards

Every future phase note, task card, or implementation ticket should include:

- `FT IDs touched`
- `why these FT IDs are changing`
- `verification plan`

Apply this most strictly to substantial capability work, structural refactors, new requirement families, and any change that could blur ownership boundaries. Do not force the full planning ceremony onto every routine CLI/MCP usage or tiny wording-only adjustment.

Canonical starter template:

- `.planning/TASK_CARD_TEMPLATE.md`
- `.planning/task-cards/` for concrete task cards
- `.planning/GOVERNANCE.md` for the full workflow

Lightweight checks:

- `python3 scripts/validate_planning_governance.py`
- `python3 scripts/validate_task_cards.py`
- `python3 scripts/validate_requirement_mappings.py`
- `python3 scripts/validate_structural_hygiene.py`

### Review and Gate Checks

The intended gate behavior for structural changes is:

- reject requirements with no `FT-*` leaf ownership
- reject task cards with no `FT-*` leaf mapping
- reject new capability work that changes behavior but does not update this file
- reject capability deletion or renaming without remapping its requirements and tasks
- reject structural files that exceed configured line-count or byte-size budgets unless they are covered by an explicit temporary rule

In other words: keep governance strong where capability boundaries move, and keep routine product consumption lightweight where they do not.

## Adoption Notes

- The function tree remains the canonical capability anchor even though current requirement families now also have inline `Maps to FT:` ownership
- Existing requirement families are still summarized here by capability leaf so the tree can be audited without scanning the whole requirement file
- Task cards should adopt `FT-*` mapping immediately through `.planning/TASK_CARD_TEMPLATE.md`
- Task cards should live under `.planning/task-cards/` so they can be validated consistently
- Requirement sections should carry inline `Maps to FT:` lines and can be checked with `scripts/validate_requirement_mappings.py`
- Runtime readers should not mistake this file for the only product entrypoint; normal usage can still start from README, capability index, AI playbook, CLI help, or MCP tool docs
- The current evolution focus is still vertical improvement inside `FT-03`, not horizontal expansion into unrelated capability families

## Gradual Requirement Mapping Plan

Priority order:

1. Task cards and execution tickets must use `FT-*` fields immediately
2. New requirements must include explicit `Maps to FT:` annotations when they are added
3. Existing requirement families can then be normalized section by section

Current state:

- current requirement families have inline `Maps to FT:` ownership
- remaining work is maintenance for future edits and finer-grained remapping if leaf ownership ever needs to split

Recommended rollout:

- Step 1: use the centralized mapping in this file as the temporary source of truth
- Step 2: keep inline `Maps to FT:` lines current whenever a requirement section changes
- Step 3: if capability ownership becomes more granular, refine either the requirement mappings or the tree leaf boundaries instead of letting drift accumulate

Do not pause delivery work for a full-document requirement remap. The migration should piggyback on real edits.

---
*Function tree established: 2026-04-26; positioning updated 2026-04-28 to reflect proportional governance and selective `FT-03` deepening.*
