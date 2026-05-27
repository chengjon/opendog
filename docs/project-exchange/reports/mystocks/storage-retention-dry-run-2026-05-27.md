# Storage Retention Dry-Run - mystocks

Date: 2026-05-27

Project:

- id: `mystocks`
- path: `/opt/claude/mystocks_spec`
- OPENDOG_HOME: `/root/.opendog`
- observed status: `monitoring`

## Scope

This record captures the first real-project storage-retention check after OPENDOG added retained-evidence cleanup governance, activity rollups, and retention policy configuration.

No destructive cleanup was executed.

## Commands Attempted

Project lookup:

```bash
OPENDOG_HOME=/root/.opendog cargo run --quiet -- list
```

Relevant result:

```text
mystocks /opt/claude/mystocks_spec monitoring
```

Activity cleanup preview:

```bash
OPENDOG_HOME=/root/.opendog cargo run --quiet -- cleanup-data \
  --id mystocks \
  --scope activity \
  --dry-run \
  --json
```

Full retained-evidence preview:

```bash
OPENDOG_HOME=/root/.opendog cargo run --quiet -- cleanup-data \
  --id mystocks \
  --scope all \
  --dry-run \
  --json
```

Both cleanup previews failed before producing deletion estimates:

```text
Error: Remote control error: Schema migration error: project database schema version 6 is newer than supported version 4
```

Interpretation:

- The CLI reached an older daemon through the daemon-first local control path.
- The running daemon supports project DB schema v4, while the mystocks DB is already schema v6.
- This is a runtime compatibility prerequisite failure, not a storage-retention result.
- No cleanup or vacuum was executed.

## Rollup Query

Command:

```bash
OPENDOG_HOME=/root/.opendog cargo run --quiet -- report rollup \
  --id mystocks \
  --window 30d \
  --json
```

Summarized result:

```json
{
  "schema_version": "opendog.cli.activity-rollups.v1",
  "project_id": "mystocks",
  "window": "30d",
  "summary": {
    "bucket_size": "1d",
    "bucket_count": 31,
    "returned_days": 0,
    "rollup_days": 0,
    "total_access_count": 0,
    "total_modification_count": 0,
    "total_event_count": 0,
    "truncated": false
  }
}
```

Interpretation:

- `report rollup` is callable with the current code path.
- No retained daily activity rollups exist yet for mystocks.
- That is expected before the first successful activity cleanup compacts raw rows into `activity_daily_rollups`.

## Release Binary Status

Before rebuilding, `self-update status --json` returned:

```json
{
  "schema_version": "opendog.cli.self-update-status.v1",
  "needs_rebuild": true
}
```

The release binary was rebuilt successfully:

```bash
cargo build --release
```

Result:

```text
Finished `release` profile [optimized]
```

After rebuilding, `self-update status --json` returned:

```json
{
  "schema_version": "opendog.cli.self-update-status.v1",
  "needs_rebuild": false
}
```

The running daemon was not restarted during this check.

## Daemon Refresh

The first cleanup previews were blocked by an older daemon. The daemon was restarted with the current rebuilt release binary.

Restart details:

- old daemon pid: `13329`
- new daemon pid: `3026893`
- daemon socket: `/root/.opendog/data/daemon.sock`
- release binary status after rebuild: `needs_rebuild = false`

No MCP host processes were killed during this check.

## Post-Restart Dry-Run Results

After daemon restart, cleanup validation reached the current retention argument checks. `cleanup-data` requires explicit retention parameters.

These commands succeeded:

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
  --id mystocks \
  --scope activity \
  --older-than-days 30 \
  --dry-run \
  --json
```

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
  --id mystocks \
  --scope all \
  --older-than-days 30 \
  --keep-snapshot-runs 20 \
  --dry-run \
  --json
```

30-day retention result:

```json
{
  "activity": {
    "deleted": {
      "file_events": 0,
      "file_sightings": 0,
      "snapshot_history": 0,
      "snapshot_runs": 0,
      "verification_runs": 0
    },
    "rolled_up": {
      "file_events": 0,
      "file_sightings": 0
    }
  },
  "all": {
    "deleted": {
      "file_events": 0,
      "file_sightings": 0,
      "snapshot_history": 0,
      "snapshot_runs": 0,
      "verification_runs": 0
    },
    "rolled_up": {
      "file_events": 0,
      "file_sightings": 0
    }
  }
}
```

Interpretation:

- The daemon/schema compatibility blocker is resolved.
- With 30-day retention and 20 snapshot runs retained, mystocks currently has no eligible retained rows to delete.
- The large DB is therefore not caused by stale rows older than 30 days or by SQLite freelist bloat.

## Size Attribution

The project DB is large but active:

```json
{
  "database": "/root/.opendog/data/projects/mystocks.db",
  "bytes": 9964892160,
  "page_size": 4096,
  "page_count": 2435438,
  "freelist_count": 0,
  "key_counts": {
    "file_events": 33427636,
    "file_sightings": 10240832,
    "snapshot_history": 111280,
    "snapshot_runs": 2,
    "verification_runs": 4,
    "activity_daily_rollups": 0
  }
}
```

Most size pressure comes from recent `file_events` and `file_sightings`, not from old snapshots, verification runs, or reclaimable SQLite pages.

## Retention Sensitivity

Non-destructive activity dry-runs with different retention windows:

| Retention window | `file_events` eligible | `file_sightings` eligible | Interpretation |
|---|---:|---:|---|
| 7 days | 7,940,335 | 7,066,380 | High impact, aggressive for an active project |
| 14 days | 321,843 | 2,810,981 | Moderate impact |
| 21 days | 0 | 0 | No impact |
| 30 days | 0 | 0 | No impact under current default-style policy |

Dry-run `rolled_up` remains zero because no write is performed. A real activity cleanup would roll up daily counts before deleting raw rows, then `report rollup` should show retained daily volume.

## Required Follow-Up

1. Restart any long-lived MCP host sessions before testing MCP tools against the refreshed daemon.
2. Rerun the preferred activity-retention preview:

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
     --id mystocks \
     --scope activity \
     --older-than-days 14 \
     --dry-run \
     --json
   ```

3. Decide whether mystocks should use 14-day or 7-day activity retention. The 30-day policy does not reduce this DB today.
4. If the selected activity preview is reasonable, execute with `--vacuum` only during an accepted maintenance window.
5. After execution, rerun:

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog report rollup \
     --id mystocks \
     --window 30d \
     --json
   ```

6. Record the deletion counts, rolled-up counts, reclaimable bytes, and final rollup summary in a follow-up report.
