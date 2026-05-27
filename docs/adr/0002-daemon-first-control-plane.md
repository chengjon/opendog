# ADR 0002: Daemon-First Control Plane

Date: 2026-05-27

## Status

Accepted

## Context

OPENDOG has three entry surfaces: daemon, CLI, and MCP. If each surface opens independent monitor/write paths, monitor state can diverge, SQLite writes can become harder to reason about, and MCP reconnects can lose runtime state.

## Decision

When the daemon is live, CLI and MCP operations should use the daemon-backed local control plane. Direct database access remains a fallback when the daemon is unavailable or a command is intentionally local-only.

## Consequences

- Monitoring state survives MCP reconnects.
- Runtime ownership is concentrated in one place.
- CLI/MCP handlers need consistent daemon-first fallback tests.
- The control protocol becomes a key interface and must be versioned, tested, and kept aligned with tool and command surfaces.
