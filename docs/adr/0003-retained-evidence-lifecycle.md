# ADR 0003: Retained-Evidence Lifecycle

Date: 2026-05-27

## Status

Accepted

## Context

Long-running observation can make OPENDOG project databases large. Deleting old raw activity rows without preserving summaries would reduce database size but harm later guidance, audit, and cleanup review.

## Decision

Before pruning raw activity evidence, preserve daily activity aggregates in `activity_daily_rollups`. Expose retained summaries through CLI `opendog report rollup` and MCP `get_activity_rollups`.

## Consequences

- Cleanup can reduce large raw tables while retaining aggregate evidence.
- Operators should dry-run cleanup before deleting retained evidence.
- VACUUM remains an explicit operator action because it can be expensive on large databases.
- Retained-evidence cleanup must never delete source project files.
