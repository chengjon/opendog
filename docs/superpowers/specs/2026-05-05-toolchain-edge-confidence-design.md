# Toolchain Edge-Confidence Tightening Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.06.01` so OPENDOG treats `docs_only` and `unknown` toolchain profiles more honestly at the confidence boundary without adding any new toolchain detection logic or changing any output contract.

The target is intentionally narrow:

- tighten only `docs_only` and `unknown` confidence semantics
- keep the current detection conditions unchanged
- keep recommended command lists unchanged
- keep MCP/CLI payload structure unchanged
- keep workspace aggregation structure unchanged
- avoid any new marker checks, config, parser, or scan behavior

This is edge-confidence hardening, not toolchain expansion.

## Capability Scope

FT IDs touched:

- `FT-03.06.01` Identify project type and toolchain
- consumer-side effect for `FT-03.04.01` Aggregate and prioritize across projects

Primary requirement family:

- `STACKX-01..04`

This batch sharpens the trust semantics at the edge of the existing toolchain layer. It does not add new project types or new recommended commands.

## Current Problem

After the previous confidence refinement batch:

- trusted interval = `high | medium-high`
- review-needed interval = `medium | low`

Two edge profiles still need tightening:

1. `docs_only`
- current detection already requires both docs configuration and docs content
- current command is narrowly scoped to docs search
- current confidence is still only `medium`

2. `unknown`
- current detection has no reliable stack evidence
- current fallback commands are generic shell truth commands
- current confidence is already `low`

This creates an asymmetry:

- `docs_only` is stronger than a speculative medium-confidence guess because the current detector already has two corroborating signals
- but it is still grouped alongside projects that genuinely need toolchain review

At the same time, `unknown` should remain the bottom-confidence fallback because this batch must not invent new evidence.

## Design

### 1. Keep Detection, Commands, And Schema Stable

This batch does not change:

- `docs_only_marker_exists(root)`
- `unknown_profile()`
- command lists for either profile
- any toolchain field names
- any MCP or CLI payload structure

Only `confidence` values and the downstream meaning of that confidence in workspace aggregation are affected.

### 2. Move `docs_only` Into The Trusted Interval

`docs_only` should move from `medium` to `medium-high`.

Rationale:

- current detection already requires docs configuration markers
- current detection already requires docs content markers
- the recommended command is narrow and aligned with that project type
- this is a bounded, documentation-oriented interpretation, not a speculative full-stack guess

This does not mean docs repositories become `high`.

`medium-high` is the correct ceiling because:

- OPENDOG still is not inferring a full build/test toolchain
- the project may still contain non-docs behavior outside the detected docs surface

### 3. Keep `unknown` At `low`

`unknown` should remain:

- `project_type = "unknown"`
- `confidence = "low"`

Rationale:

- there is still no trustworthy stack signal
- the fallback command set remains generic repository-inspection guidance
- raising `unknown` would effectively widen inference scope without new evidence

This batch is intentionally conservative: it tightens trusted edges without softening the unknown case.

### 4. Preserve Fallback Command Semantics

This batch must not alter commands:

- `docs_only`
  - keep only the current docs-oriented search command
- `unknown`
  - keep the current shell fallback:
    - `rg "<pattern>" .`
    - `git diff`
    - `git status`

The confidence change should explain command trust more accurately, not mutate the fallback command surface.

### 5. Tighten Workspace Aggregation Using Existing Trusted Semantics

Trusted interval remains:

- `high`
- `medium-high`

Therefore:

- `docs_only` should no longer appear under `low_confidence_projects`
- `unknown` should continue to appear under `low_confidence_projects`
- `projects_without_detected_toolchain` should continue to count only `unknown`

No other aggregation field needs to change.

## Implementation Shape

Primary implementation file:

- `src/mcp/toolchain.rs`

Preferred helper structure:

- `docs_only_profile() -> ProjectToolchainProfile`
- reuse `unknown_profile()`
- reuse `toolchain_confidence_is_trusted(confidence: &str) -> bool`

Rules:

- do not add new marker helpers
- do not change `detect_project_commands(...)`
- do not change workspace summary field names

## Test Strategy

Primary test files:

- `src/mcp/tests/guidance_basics/toolchain_and_unused/project_toolchain_detection.rs`
- `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/toolchain_signals.rs`

Add or tighten coverage for five scenarios:

1. `docs_only` confidence
- expected `confidence = "medium-high"`

2. `docs_only` command surface
- expected no test/lint/build commands
- expected docs search command only

3. `unknown` confidence
- expected `confidence = "low"`

4. `unknown` fallback commands
- expected current `rg / git diff / git status` fallback unchanged

5. workspace aggregation
- `docs_only` with `medium-high` should stay out of `low_confidence_projects`
- `unknown` should remain inside

## Non-Goals

Do not:

- change `mixed_workspace` or `mono_repo` again in this batch
- infer docs-specific build or lint commands
- raise `unknown` above `low`
- add new docs markers or new unknown heuristics
