## Why

OpenDog's current `/proc/<pid>/fd` attribution can fan out a single directory-level fd into identical access counts for every snapshot file beneath that directory. That breaks the trust boundary for hotspots, unused candidates, and downstream guidance because the core signal is no longer per-file.

## What Changes

- Distinguish file-level fds from directory-level fds in the scanner attribution path.
- Deduplicate sightings within a single scan cycle so one fd cannot inflate counts by being observed repeatedly.
- Preserve per-file access counts for independently opened source files instead of collapsing them under a shared directory fd.
- Add a regression validation pass against the `mystocks` 50087-file repository to confirm `.py` and `.vue` files receive independent access counts after the fix.

## Capabilities

### New Capabilities
- `fd-attribution`: accurate per-file process attribution from `/proc/<pid>/fd` scanning, including scan-cycle deduplication and directory-fd exclusion.

### Modified Capabilities
- None

## Impact

- `src/core/scanner.rs`: fd classification and sighting selection.
- `src/core/monitor.rs`: scan result consumption and attribution accounting.
- Core stats/report consumers that rely on `file_sightings` and `file_stats`.
- Regression tests for attribution behavior, including large-repo validation in `mystocks`.
