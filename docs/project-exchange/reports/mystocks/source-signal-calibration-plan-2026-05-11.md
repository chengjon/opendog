# Source Signal Calibration Plan - mystocks

Date: 2026-05-11

Purpose: collect evidence for `ODX-20260511-source-signal-observation-calibration` before changing scanner attribution, default ignore behavior, or MCP/CLI presentation.

## Problem Boundary

Case H and Case I are fixed. This plan only investigates the residual observation-quality issue:

- `.claude/` infrastructure files dominate hot stats.
- Top `.claude/` files share near-identical access counts and durations.
- `.py` / `.vue` source files still show `access_count=0` in the latest mystocks retest.

Do not treat this as a payload-size, MCP Resources, or already-closed fd directory-fan-out issue unless new evidence proves it.

## Hypotheses To Separate

1. Expected tool behavior: Claude Code repeatedly opens `.claude/` files as real file fds and does not keep source files open long enough for sampling.
2. Scanner behavior: source file fds are opened but missed, mis-normalized, or not mapped into snapshot-relative paths.
3. Presentation/filtering debt: source signal exists but is buried behind infrastructure rows or not surfaced by current default views.
4. Workflow mismatch: edits produce modification events, but no read/open activity, so source files legitimately have `modification_count > 0` and `access_count = 0`.

## Sampling Setup

- Use the same `OPENDOG_HOME` as the active mystocks MCP host.
- Confirm project id: `mystocks`.
- Confirm monitor is active before the source-heavy session.
- Record binary path and timestamp for the OpenDog binary used by MCP.
- Record the current top stats before starting the sample.

## Source-Heavy Activity Sample

During one monitored window, intentionally perform real source-code work:

1. Open and inspect at least 3 Python files under `src/` or `web/`.
2. Open and inspect at least 2 Vue files under `web/frontend/src/`.
3. Run source searches with `rg` against symbols from those files.
4. Make a small reversible edit or formatting-only touch in at least one `.py` and one `.vue` file, then revert if needed.
5. Run a project-native check if available.
6. Wait at least one monitor sampling interval after the activity.

## Required Evidence

Capture these outputs after the sample:

1. `get_stats {"id":"mystocks","limit":100}`
2. `get_unused_files {"id":"mystocks","limit":100}`
3. CLI `opendog stats --id mystocks`
4. CLI `opendog unused --id mystocks`
5. A direct DB query or equivalent export showing rows for the touched `.py` and `.vue` files:
   - file path
   - access_count
   - estimated_duration_ms
   - modification_count
   - path_classification
6. Top grouped counts for high-frequency identical access patterns.
7. A short list of the exact source files intentionally opened or edited.

## Interpretation Rules

- If touched source files receive distinct `access_count` values, scanner attribution is likely working and the next fix should be view/filter/guidance oriented.
- If source files remain `access_count=0` but have `modification_count > 0`, investigate whether the workflow only writes files without holding read fds long enough for sampling.
- If source file fds are proven open during sampling but not recorded, open a new OpenSpec proposal before changing scanner attribution semantics.
- If `.claude/` files remain dominant but are classified as infrastructure, consider source-first view/filter improvements instead of hiding infrastructure evidence.

## Completion Output

Write results back to:

- `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`
- `docs/project-exchange/issues/INDEX.md`

If implementation is needed, update:

- `.planning/task-cards/TASK-20260511-source-signal-observation-calibration.md`

If scanner semantics are implicated, create a new OpenSpec change before code changes.
