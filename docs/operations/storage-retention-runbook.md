# Storage Retention Runbook

Use this runbook when an OPENDOG project database grows large, when guidance marks a project as a storage-maintenance candidate, or before long cleanup/refactor work where retained OPENDOG evidence may distort AI guidance.

This runbook is for OPENDOG-retained evidence only. It does not delete source files.

## Safety Principles

- Start with `--dry-run`; do not execute cleanup until the preview is understood.
- Treat a running daemon as part of the compatibility surface. If the daemon binary is stale, refresh it before relying on daemon-first CLI or MCP behavior.
- Prefer scoped cleanup first: `activity`, `snapshots`, or `verification`; use `all` only after the scoped preview is clear.
- Use `--vacuum` only when reclaimable bytes justify the SQLite rewrite cost.
- After activity cleanup, use rollup reports to inspect historical daily volume. Rollups preserve counts, not per-file or per-process raw detail.

## Preflight

1. Confirm the project and OPENDOG state directory.

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog list
   ```

2. Confirm the release binary is current.

   ```bash
   OPENDOG_HOME=/root/.opendog target/release/opendog self-update status --source /opt/claude/opendog --json
   ```

   Continue only when `needs_rebuild = false`.

3. Confirm daemon compatibility.

   If a command fails with a message like:

   ```text
   project database schema version N is newer than supported version M
   ```

   the running daemon or MCP process is older than the project DB. Rebuild the release binary, restart the daemon and MCP host sessions with that binary, then retry the dry-run. Do not treat this as a data cleanup result.

## Preview Workflow

Run the narrowest useful dry-run first.

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
  --id <PROJECT_ID> \
  --scope activity \
  --dry-run \
  --json
```

For a full retained-evidence preview:

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
  --id <PROJECT_ID> \
  --scope all \
  --dry-run \
  --json
```

Read these fields first:

- `storage_before.total_bytes`
- `storage_before.approx_reclaimable_bytes`
- `storage_before.evidence_counts`
- `deleted`
- `rolled_up`
- `maintenance.vacuum_recommended`
- `maintenance.recommended_command`

Decision guide:

- If `deleted` is near zero, no cleanup is needed.
- If `deleted.file_sightings` or `deleted.file_events` is large, verify `rolled_up.file_sightings` and `rolled_up.file_events` before execution.
- If `maintenance.vacuum_recommended = true`, consider adding `--vacuum` to the execution command.
- If `storage_before.approx_reclaimable_bytes` is small, skip `--vacuum`.

## Execute Workflow

Execute only after the dry-run result is acceptable.

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog cleanup-data \
  --id <PROJECT_ID> \
  --scope activity \
  --vacuum \
  --json
```

For snapshot or verification evidence, change `--scope` instead of using `all` by default.

## Post-Cleanup Checks

Inspect retained daily activity volume:

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog report rollup \
  --id <PROJECT_ID> \
  --window 30d \
  --json
```

Read:

- `summary.total_access_count`
- `summary.total_modification_count`
- `summary.total_event_count`
- `summary.rollup_days`
- `summary.returned_days`
- `summary.truncated`

If `summary.truncated = true`, rerun with a larger `--limit` or a narrower `--window`.

## Retention Policy Example

Use project-level retention when one repository needs different storage behavior from global defaults.

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog config set-project \
  --id <PROJECT_ID> \
  --retention-policy-json '{
    "cleanup_review_db_bytes_threshold": 16777216,
    "vacuum_reclaimable_bytes_threshold": 8388608,
    "vacuum_reclaim_ratio_threshold_percent": 20,
    "activity_rows_threshold": 1000000,
    "verification_runs_threshold": 10000,
    "snapshot_runs_threshold": 100,
    "activity_retention_days": 30,
    "verification_retention_days": 60,
    "keep_snapshot_runs": 20
  }' \
  --json
```

Return to global defaults:

```bash
OPENDOG_HOME=/root/.opendog target/release/opendog config set-project \
  --id <PROJECT_ID> \
  --inherit-retention \
  --json
```

## AI Usage Order

For AI or MCP-driven workflows:

1. Start with `get_guidance(detail=summary)`.
2. If storage maintenance is flagged, ask the operator to run `cleanup-data --dry-run --json`.
3. Use `get_activity_rollups` after cleanup to inspect retained daily activity volume.
4. Use `get_usage_trends` or `get_time_window_report` before cleanup when recent file-level detail is required.
5. Do not confuse retained-evidence cleanup with source-code cleanup.
