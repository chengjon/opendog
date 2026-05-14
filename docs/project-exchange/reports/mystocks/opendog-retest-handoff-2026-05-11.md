# OpenDog Retest Handoff - mystocks

Date: 2026-05-11

Purpose: provide the OpenDog-side evidence and exact mystocks retest focus for Case H and Case I.

## OpenDog-Side Status

- Release binary: `/opt/claude/opendog/target/release/opendog`
- Release binary timestamp: `2026-05-10 18:58:57 +0800`
- OpenDog local validation state: passed
- Relevant shared issues:
  - `ODX-20260510-mcp-large-payload-pagination`
  - `ODX-20260510-mcp-resources-not-discovered`

## Case H - Bounded MCP Stats / Unused Payloads

OpenDog local MCP probe against `mystocks-fd` confirmed:

- default `get_stats` returned 50 file rows
- explicit `get_stats` with `limit: 5` returned 5 file rows
- default `get_unused_files` returned 50 file rows
- explicit `get_unused_files` with `limit: 5` returned 5 file rows
- `result_window` reports `total_count`, `returned_count`, `limit`, and `truncated`

mystocks retest:

1. Restart or reconnect the Claude Code MCP host.
2. Confirm the MCP command points to `/opt/claude/opendog/target/release/opendog`.
3. Confirm the same `OPENDOG_HOME` is used by the MCP host.
4. Call `get_stats {"id":"mystocks"}`.
5. Call `get_stats {"id":"mystocks","limit":50}`.
6. Call `get_unused_files {"id":"mystocks"}`.
7. Call `get_unused_files {"id":"mystocks","limit":50}`.
8. Confirm `files.length <= limit` and `result_window.limit == limit`.

If MB-scale output still appears, capture:

- Claude Code MCP configured command
- connected binary path/process
- `OPENDOG_HOME`
- raw response envelope
- whether `structuredContent.files` is bounded or the host is persisting another payload field

## Case I - MCP Resources Discovery

OpenDog local MCP probe confirmed:

- `initialize` advertises `resources` and `tools`
- `resources/list` returns `opendog://projects`
- `resources/templates/list` returns `opendog://projects` and `opendog://project/{id}/verification`
- `resources/read` works for project-list and per-project verification resources

mystocks retest:

1. Restart or reconnect the Claude Code MCP host after the OpenDog release rebuild.
2. Inspect initialize capabilities if the host exposes them.
3. Run host-side resource discovery.
4. Confirm `opendog://projects` is visible.
5. Confirm `opendog://project/{id}/verification` is visible as a resource template or readable URI.
6. Read `opendog://projects`.
7. Read `opendog://project/mystocks/verification`.

If resources are still invisible, capture:

- initialize capabilities
- `resources/list` raw response
- binary path/process
- Claude Code host version
- MCP server name used by the host

## OpenDog Verification Gates

OpenDog side has passed:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/validate_planning_governance.py`

## References

- [Shared issue index](../../issues/INDEX.md)
- [mystocks feedback](./OPENDOG_USAGE_FEEDBACK.md)
- [mystocks MCP test report](./opendog-mcp-test-report-2026-05-10.md)
- [OpenDog QUICKSTART](../../../../QUICKSTART.md)
