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

## Required Follow-Up

1. Restart the OPENDOG daemon and MCP host sessions so they use the rebuilt current binary.
2. Rerun:

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
     --id mystocks \
     --scope activity \
     --dry-run \
     --json
   ```

3. If the activity preview is reasonable, decide whether to execute with `--vacuum`.
4. After execution, rerun:

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog report rollup \
     --id mystocks \
     --window 30d \
     --json
   ```

5. Record the deletion counts, rolled-up counts, reclaimable bytes, and final rollup summary in a follow-up report.
