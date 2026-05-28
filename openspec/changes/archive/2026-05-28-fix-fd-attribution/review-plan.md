# Attribution Credibility Fix Review Plan

## Current State

This document now serves as a review and audit checklist for the implemented `fix-fd-attribution` change.

Current implementation status:

- Governance status: accepted closed-loop baseline.
- OpenSpec artifacts are complete: `proposal.md`, `design.md`, `specs/fd-attribution/spec.md`, and `tasks.md`.
- Scanner implementation is present in `src/core/scanner.rs`.
- Unit coverage exists for directory target exclusion and scan-cycle fd deduplication.
- Large-repository validation has been recorded in `FIELD_NOTES.md`.
- Downstream checks cover `stats`, `unused`, and `report window`; the observed `agent-guidance` UTF-8 panic is split into an independent task and is not part of this fd attribution closure.
- Any further implementation or behavioral edits still require explicit approval from the project owner before execution.

## Governance Outcome

- `fix-fd-attribution` is accepted as the formal scanner attribution baseline.
- The shipped baseline is: only canonicalized regular file fd targets inside the project snapshot set create file sightings, directory fd targets do not fan out, and duplicate `(pid, fd)` observations are suppressed within one scan cycle.
- Future changes to scanner attribution semantics must start with OpenSpec governance and must include a review plan, task mapping, regression tests, and large-repo validation evidence before acceptance.
- The independent `agent-guidance` UTF-8 boundary panic is governed by `.planning/task-cards/TASK-20260509-agent-guidance-utf8-panic.md`.

## Goal

Fix the attribution credibility bug where `/proc/<pid>/fd` scanning can treat a directory-level fd as evidence for every snapshot file under that directory, producing identical `access_count` values across unrelated files.

This is a core OpenDog signal-quality fix because hotspots, unused candidates, reports, and guidance all depend on trustworthy per-file attribution.

## Scope

This plan focuses on the existing OpenSpec change:

- `openspec/changes/fix-fd-attribution`

Primary implementation area:

- `src/core/scanner.rs`

Expected downstream consumers:

- `src/core/monitor.rs`
- stats, unused-file detection, reports, and guidance layers that consume `file_sightings` and `file_stats`

## Proposed Work

1. Confirm the current baseline and completed work.
   - Inspect the existing `fix-fd-attribution` OpenSpec artifacts.
   - Inspect the current `src/core/scanner.rs` diff.
   - Confirm `tasks.md` accurately reflects completed and remaining work.
   - Confirm current tests and field-note evidence.

2. Review the `/proc/<pid>/fd` scanner attribution path.
   - Only regular file fd targets that resolve inside the project root and exist in the snapshot set should emit `FileSighting`.
   - Directory fd targets must be skipped and must not fan out to files under that directory.
   - Non-path fd targets such as sockets, pipes, memfd, anon inode, and unknown targets remain ignored.
   - Symlink-to-directory targets must also be skipped after canonicalization because `metadata.is_file()` is false.

3. Review scan-cycle fd deduplication.
   - Within one scan cycle, the same process fd should contribute at most one sighting.
   - The dedup key should be based on process id and fd number: `(pid, fd)`.
   - This key is scoped only to a single scan cycle. It does not claim fd numbers are stable across scan cycles.
   - This should not change the existing monitor/stat storage contract.

4. Confirm regular file attribution is preserved.
   - A valid regular file fd should still produce a per-file sighting.
   - The fix must not suppress normal file access accounting.

5. Confirm test coverage.
   - Directory fd targets do not produce file sightings.
   - Duplicate `(pid, fd)` observations in one scan cycle are deduplicated.
   - Regular file fd targets still count.
   - Existing monitor/stat tests continue to pass.

6. Confirm large-repository regression validation with `mystocks`.
   - Use an isolated `OPENDOG_HOME` so production/user state is not polluted.
   - Register `/opt/claude/mystocks_spec`.
   - Take a snapshot.
   - Start monitoring.
   - Run a controlled process that opens:
     - one repository directory fd
     - one `.py` file fd
     - one `.vue` file fd
   - Keep the `.py` and `.vue` fds open for different durations.
   - Verify that `.py` and `.vue` files receive independent `access_count` values.
   - Verify that the directory fd does not create the prior identical-count fan-out pattern.

7. Confirm downstream consumer behavior.
   - `opendog stats` should report the independently opened `.py` and `.vue` files with distinct counts.
   - `opendog unused` should remain well-formed and continue to consume the same snapshot/stat data model.
   - `opendog report window` should remain well-formed because it reads from `file_sightings`.
   - `get_guidance` / `opendog agent-guidance` should remain contract-compatible because downstream payload contracts are unchanged.

8. Record evidence.
   - Update `FIELD_NOTES.md` with the exact validation setup and result.
   - Clearly separate residual `.claude/` file noise from this directory-fd attribution bug.

## Acceptance Criteria

- Directory fds do not create per-file sightings.
  Evidence: scanner unit test `resolve_snapshot_relative_file_path_ignores_directory_targets`.
- Regular file fds still create per-file sightings.
  Evidence: the same scanner unit test asserts a regular file resolves to `src/main.rs`.
- One `(pid, fd)` contributes at most one sighting per scan cycle.
  Evidence: scanner unit test `mark_fd_seen_deduplicates_per_pid_and_fd`.
- `mystocks` large-repo validation shows independently opened `.py` and `.vue` files can receive distinct `access_count` values.
  Evidence: `FIELD_NOTES.md` records `web/frontend_status.py` with 4 accesses and `web/frontend/src/App.vue` with 1 access under isolated validation.
- Existing downstream APIs and payload contracts remain unchanged.
  Evidence: no MCP/CLI schema change is part of this OpenSpec change, and full Rust tests pass.
- Verification commands pass:
  - `openspec validate fix-fd-attribution` exits 0 and reports `Change 'fix-fd-attribution' is valid`
  - `cargo fmt --check` exits 0
  - `cargo test` exits 0
  - `cargo clippy --all-targets --all-features -- -D warnings` exits 0

## Risks

- Over-filtering fd targets could hide legitimate file access.
- Some tools may hold unusual fd target types that are not regular files but still represent useful activity.
- The large-repo validation may still show `.claude/` files as hot because Claude Code opens those as real file fds.

## Risk Controls

- Filter only targets whose resolved metadata is not a regular file.
- Keep non-path fd targets ignored as before.
- Keep all downstream contracts unchanged.
- Treat `.claude/` dominance as a separate classification/default-ignore issue, not as evidence that this attribution fix failed.
- Use `(pid, fd)` only for scan-cycle-local deduplication, not cross-cycle identity.

## Out Of Scope

- Default ignore pattern changes for `.claude/`, `.amazonq/`, `.cursor/`, or backup files.
- New source-only or infrastructure-filtered stats views.
- MCP payload pagination or size handling.
- Changes to guidance ranking logic.

## Approval Gate

Project owner approval is required before any further implementation or behavioral edits.

For this review cycle, approval means an explicit user message approving the next action. Review notes alone do not authorize additional code changes.
