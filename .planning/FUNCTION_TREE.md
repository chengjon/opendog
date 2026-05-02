---
function_tree_version: 1.3
last_updated: "2026-05-02"
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

This distinction matters because OPENDOG's runtime product is intentionally consumed as a lightweight three-layer system, while this file exists mainly as a governance overlay for capability evolution.

Current design posture:

- no product-direction drift: the tree still describes the original multi-project observation plus AI decision-support mission
- broad but bounded surface: the tree is wider than a simple monitoring backend, but each branch remains constrained by evidence, authority, and non-destructive advisory boundaries
- current priority is selective deepening: future work should mainly improve the trustworthiness and clarity of existing `FT-03` leaves before opening unrelated new capability families
- current hardening baseline: `FT-03.02.02`, `FT-03.03.01`, `FT-03.06.01`, and `FT-03.07.01` now cover soft verification gates, repository-truth boundary projection, and machine-readable resume sequencing for repository stabilization and verification workflows

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
    lifecycle: in_progress
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
    summary: Sample whitelisted AI processes and detect which project files they currently hold open.

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
    summary: Expose project-level statistical queries such as unused files and core-file candidates.

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
    summary: Let operators access OPENDOG through the `opendog` command surface.

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
    summary: Let AI clients discover and invoke OPENDOG capabilities through MCP.

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
    lifecycle: in_progress
    summary: Convert OPENDOG from a monitoring backend into a reusable AI decision-support layer.

  - id: FT-03.01
    title: Workspace Observation
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Express readiness, freshness, and evidence gaps at project and workspace scope.

  - id: FT-03.01.01
    title: Explain readiness and evidence gaps
    level: L3
    parent: FT-03.01
    lifecycle: in_progress
    requirement_ranges: [OBS-01..04]
    roadmap_phases: [6]
    summary: Show whether OPENDOG has enough observation quality to support downstream conclusions.

  - id: FT-03.02
    title: Repository Risk and Execution Strategy
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Help AI decide whether to inspect, stabilize, verify, or modify next.

  - id: FT-03.02.01
    title: Summarize repository risk and confidence
    level: L3
    parent: FT-03.02
    lifecycle: in_progress
    requirement_ranges: [RISK-01..04]
    roadmap_phases: [6]
    summary: Convert repository and observation state into evidence-backed risk summaries.

  - id: FT-03.02.02
    title: Recommend next-step execution strategy
    level: L3
    parent: FT-03.02
    lifecycle: in_progress
    requirement_ranges: [STRAT-01..04]
    roadmap_phases: [6]
    summary: Recommend the next action, follow-up command path, and machine-readable resume sequence across OPENDOG, shell, and verification workflows.

  - id: FT-03.03
    title: Verification Evidence
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Make verification evidence durable, queryable, and decision-relevant.

  - id: FT-03.03.01
    title: Record and reason over verification evidence
    level: L3
    parent: FT-03.03
    lifecycle: in_progress
    requirement_ranges: [EVID-01..04]
    roadmap_phases: [6]
    summary: Attach recorded validation evidence, freshness, and gate judgments to recommendations and safety decisions.

  - id: FT-03.04
    title: Multi-Project Portfolio Prioritization
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Help AI choose which project deserves attention first.

  - id: FT-03.04.01
    title: Rank projects by attention and evidence quality
    level: L3
    parent: FT-03.04
    lifecycle: in_progress
    requirement_ranges: [PORT-01..04]
    roadmap_phases: [6]
    summary: Compare projects by readiness, evidence, and review urgency.

  - id: FT-03.05
    title: Cleanup and Refactor Review
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Surface file-level review candidates without taking destructive action.

  - id: FT-03.05.01
    title: Surface cleanup and refactor candidates
    level: L3
    parent: FT-03.05
    lifecycle: in_progress
    requirement_ranges: [CLEAN-01..04]
    roadmap_phases: [6]
    summary: Prioritize unused, hot, mixed, and suspicious files for later review.

  - id: FT-03.06
    title: Project Type and Toolchain Guidance
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Infer repository type and recommend appropriate validation commands.

  - id: FT-03.06.01
    title: Infer toolchain and recommend commands
    level: L3
    parent: FT-03.06
    lifecycle: in_progress
    requirement_ranges: [STACKX-01..04]
    roadmap_phases: [6]
    summary: Suggest project-native validation commands, search paths, and fallback behavior from repository markers.

  - id: FT-03.07
    title: Constraints and Boundaries
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Prevent AI from over-claiming what OPENDOG actually knows.

  - id: FT-03.07.01
    title: State blind spots and authority boundaries
    level: L3
    parent: FT-03.07
    lifecycle: in_progress
    requirement_ranges: [BOUND-01..04]
    roadmap_phases: [6]
    summary: Clarify what was observed, what was inferred, where repository truth is missing, and when external verification is mandatory.

  - id: FT-03.08
    title: Mock and Hardcoded Data Review
    level: L2
    parent: FT-03
    lifecycle: in_progress
    summary: Detect test-only artifacts and riskier pseudo-business data across projects.

  - id: FT-03.08.01
    title: Detect mock and test-only data artifacts
    level: L3
    parent: FT-03.08
    lifecycle: in_progress
    requirement_ranges: [MOCK-01, MOCK-03, MOCK-06, MOCK-07, MOCK-10]
    roadmap_phases: [6]
    summary: Detect and expose mock, fixture, stub, fake, demo, and sample data patterns.

  - id: FT-03.08.02
    title: Detect and prioritize hardcoded pseudo-business data
    level: L3
    parent: FT-03.08
    lifecycle: in_progress
    requirement_ranges: [MOCK-02, MOCK-04, MOCK-05, MOCK-08, MOCK-09, MOCK-10]
    roadmap_phases: [6]
    summary: Flag risky business-like literals and mixed logic-plus-data files for review.
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
```

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
- Runtime readers should not mistake this file for the primary product entrypoint; normal usage should usually start from README, capability index, AI playbook, CLI help, or MCP tool docs
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
