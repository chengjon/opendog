## Context

OpenDog currently scans whitelisted processes via `/proc/<pid>/fd`, canonicalizes each path target, and records a sighting whenever the path lands inside the project snapshot set. That works for file fds, but a directory fd can also resolve into the project root and incorrectly credit every file under that directory.

The observed result in `mystocks` was a group of 30 `.claude/` files sharing the same `access_count` and duration. That is a signal integrity problem, not just a presentation issue, because the same attribution feeds stats, unused-file ranking, time-window reports, and guidance.

## Goals / Non-Goals

**Goals:**
- Attribute only actual file-level opens to per-file usage counts.
- Ignore directory-level fds for per-file access accounting.
- Prevent repeated observations of the same fd in one scan cycle from inflating counts.
- Provide a regression path that can be run against the `mystocks` repository to validate that independently opened source files keep distinct counts.

**Non-Goals:**
- Replacing `/proc`-based attribution with a different monitoring model.
- Solving every noisy or ambiguous attribution case outside the directory-fd fan-out bug.
- Changing the external MCP/CLI contract.

## Decisions

1. **Classify fd targets before sighting emission.**
   Use procfs target metadata to separate regular files from directories, and only emit file sightings for regular files.
   Alternative: infer file-vs-directory from canonicalized paths alone. Rejected because path shape is not enough to distinguish file and directory fds reliably.

2. **Make deduplication scan-local and fd-based.**
   Track accepted sightings within a scan cycle using the process identity plus fd identity so one fd can contribute at most one sighting per cycle.
   Alternative: dedupe only by resolved path. Rejected because a path-based key would still allow repeated credit from the same fd across resolution variants and would not directly address the duplicated-fd problem.

3. **Keep attribution normalization at the scanner boundary.**
   The scanner should emit a clean stream of accepted sightings and leave the monitor/stat layers unchanged except for consuming better inputs.
   Alternative: patch the counting logic later in `monitor.rs`. Rejected because the bug is introduced before the database write, so fixing it at the source keeps the data model consistent.

4. **Use a large-repo regression as the acceptance check.**
   Validate on `mystocks` because it exposed the bug, has enough scale to reproduce the pattern, and can show whether `.py` / `.vue` source files receive independent access counts.
   Alternative: use only synthetic unit tests. Rejected because synthetic tests can prove the helper logic but not the real-world fan-out failure.

## Risks / Trade-offs

- [Risk] Some directory-backed workflows may lose coarse attribution signal -> Mitigation: document the behavior as intentional and keep directory access out of per-file counts rather than silently inflating them.
- [Risk] fd identity may be hard to surface cleanly from procfs APIs -> Mitigation: isolate the procfs-specific handling in the scanner and cover it with unit tests around the extracted classification helper.
- [Risk] Regressions may appear only in large repositories -> Mitigation: add a `mystocks`-scale verification step after unit and integration tests.

## Migration Plan

1. Implement file-vs-directory classification and scan-cycle dedup in the scanner.
2. Add targeted tests for directory fds, duplicate fd observations, and preserved file-level counts.
3. Run the full Rust test suite plus a `mystocks`-scale verification pass.
4. If the regression shows remaining fan-out, tighten the fd identity handling before release.

Rollback strategy:
- Revert the scanner change and tests if directory-only workflows regress unexpectedly.

## Open Questions

- Does procfs expose enough fd identity to dedupe exactly by fd number in all supported environments, or does the implementation need a fallback key?
- Should the scanner emit a diagnostic when it skips a directory fd, or stay silent to avoid log noise?
