# Toolchain Confidence Refinement Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.06.01` so OPENDOG expresses toolchain confidence more precisely for `mixed_workspace` and `mono_repo` projects without broadening toolchain detection or changing any MCP/CLI contract.

The target is intentionally narrow:

- refine only `mixed_workspace` and `mono_repo` confidence scoring
- introduce one additional trusted confidence band: `medium-high`
- keep existing project-type detection rules unchanged
- keep existing recommended command lists unchanged
- keep MCP/CLI payload structure and field names unchanged
- avoid any new config, parser, AST, or scanning subsystem

This is confidence hardening, not a toolchain-detection expansion.

## Capability Scope

FT IDs touched:

- `FT-03.06.01` Identify project type and toolchain
- consumer-side effect for `FT-03.02.02` Recommend next-step execution strategy
- consumer-side effect for `FT-03.04.01` Aggregate and prioritize across projects

Primary requirement family:

- `STACKX-01..04`

This batch tightens the trust semantics of an existing Phase 6 layer. It does not expand the set of detectable stacks or add new recommendation commands.

## Current Problem

`src/mcp/toolchain.rs` currently uses coarse confidence values:

- single-stack `rust` / `node` / `python` / `go`: `high`
- `mixed_workspace`: always `medium`
- generic `mono_repo`: always `medium`
- `unknown`: `low`

That is too coarse for two recurring cases:

1. multi-stack roots with real workspace corroboration
- example: `Cargo.toml + package.json + pnpm-workspace.yaml`
- current `mixed_workspace` remains `medium` even though the workspace shape is strongly corroborated

2. mono-repo roots with only weak outer markers
- example: `pnpm-workspace.yaml` exists but no current stack manifest is present at root
- current generic `mono_repo` still lands on `medium` even though command confidence is weak

This causes two problems:

- the `confidence` field under-explains whether the current recommendation commands are trustworthy
- workspace aggregation overstates or understates which projects still need toolchain review

Phase 6 already requires toolchain confidence and fallback behavior when repository signals are ambiguous. The current `medium` bucket is too broad to satisfy that cleanly.

## Design

### 1. Keep Detection And Output Surface Stable

This batch does not change:

- stack marker detection
- workspace marker detection
- project-type names
- command lists
- MCP payload structure
- CLI output structure

The only outward value change is the `confidence` string for `mixed_workspace` and `mono_repo`, plus downstream workspace aggregation behavior that consumes that value.

### 2. Introduce A Trusted `medium-high` Band

The confidence ladder for this batch becomes:

- `high`
- `medium-high`
- `medium`
- `low`

Semantics:

- `high`
  - strong, self-consistent workspace evidence
  - current implementation already supports concrete workspace-level command recommendations
- `medium-high`
  - strong enough to be treated as broadly trusted
  - existing marker sets corroborate the workspace/mixed interpretation
  - current command set is still less canonical than the strongest single-stack workspace paths
- `medium`
  - plausible but not fully corroborated
  - interpretation is useful, but still deserves review
- `low`
  - weak outer shape only
  - current marker set is insufficient to treat toolchain commands as broadly reliable

For this batch, `medium-high` belongs to the trusted interval and should not be grouped with low-confidence projects in workspace aggregation.

### 3. Refine `mixed_workspace` Confidence Using Existing Workspace Corroboration

`mixed_workspace` should no longer be always `medium`.

Use only current signals:

- `detected_stack_markers(root)`
- `cargo_toml_has_workspace(root)`
- `node_workspace_marker_exists(root)`
- `file_exists(root, "go.work")`

Rules:

- `medium-high`
  - `stacks.len() > 1`
  - and at least one existing workspace corroboration signal is present
- `medium`
  - `stacks.len() > 1`
  - and no current workspace corroboration signal is present

Examples:

- `Cargo.toml + package.json + package.json.workspaces` -> `mixed_workspace`, `medium-high`
- `Cargo.toml + package.json` only -> `mixed_workspace`, `medium`

This raises confidence only when the current code already has independent corroborating workspace evidence.

### 4. Refine Generic `mono_repo` Confidence Without Adding New Detection Rules

The existing special high-confidence mono-repo paths stay unchanged:

- Rust workspace with `[workspace]`
- Node workspace with current node workspace markers

For the remaining generic `mono_repo` fallback, use only current signals:

- `stacks.len()`
- current workspace-marker checks

Rules:

- `medium`
  - at least one recognized stack is present
  - workspace shape exists
  - but the project does not fall into an existing high-confidence mono-repo specialization
- `low`
  - workspace shape exists
  - but no recognized stack marker is present at root

Examples:

- `go.work` -> `mono_repo`, `medium`
- `pnpm-workspace.yaml` without `package.json` -> `mono_repo`, `low`

This keeps the fallback bounded: no new toolchain knowledge is introduced, only a more honest confidence label.

### 5. Keep Commands And Project Types Unchanged

This batch must not mutate the current command lists.

That means:

- `mixed_workspace` keeps its existing merged command list behavior
- generic `mono_repo` keeps its current fallback command list behavior
- no additional build/test/lint command derivation is added

The point is to align `confidence` with current command trust, not to widen command generation.

### 6. Tighten Workspace Aggregation Semantics

`workspace_toolchain_layer(...)` currently groups any non-`high` confidence project into `low_confidence_projects`.

That is too broad once `medium-high` exists.

New aggregation rule:

- trusted interval:
  - `high`
  - `medium-high`
- review-needed interval:
  - `medium`
  - `low`
  - `unknown`

Effects:

- `medium-high` projects stay out of `low_confidence_projects`
- `projects_without_detected_toolchain` remains unchanged
- summary counts and command aggregation remain unchanged

This preserves the structure of the workspace layer while making its review queue more precise.

## Implementation Shape

Primary implementation file:

- `src/mcp/toolchain.rs`

Preferred helper structure:

- `workspace_signal_present(root: &Path) -> bool`
- `mixed_workspace_confidence(root: &Path, stacks: &[&'static str]) -> &'static str`
- `generic_mono_repo_confidence(stacks: &[&'static str]) -> &'static str`
- `toolchain_confidence_is_trusted(confidence: &str) -> bool`

Rules:

- reuse existing marker helpers
- do not add new marker sources
- do not rewrite command generation
- do not change the JSON schema

## Test Strategy

Primary test files:

- `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

Add or tighten coverage for five scenarios:

1. `mixed_workspace` without workspace corroboration
- expected `confidence = "medium"`

2. `mixed_workspace` with current workspace corroboration
- expected `confidence = "medium-high"`

3. existing high-confidence `mono_repo` paths
- expected to remain `high`

4. generic `mono_repo` with only weak outer workspace marker
- expected `confidence = "low"`

5. workspace aggregation
- `medium-high` projects should not appear under `low_confidence_projects`

## Non-Goals

Do not:

- add new stack markers
- inspect nested workspaces beyond current root-level checks
- infer additional commands
- change `unknown`, `docs_only`, or single-stack confidence rules in this batch
- change CLI rendering beyond whatever existing text already prints from the `confidence` value
