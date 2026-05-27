# ADR 0004: MCP and CLI Contract Synchronization

Date: 2026-05-27

## Status

Accepted

## Context

OpenDog exposes overlapping capabilities through CLI commands, MCP tools, read-only MCP resources, daemon control protocol variants, JSON contract identifiers, and documentation. The project has already had drift in tool counts and documented surfaces.

## Decision

Treat `src/mcp/tool_inventory.rs`, contract constants, MCP documentation, `FUNCTION_TREE.md`, `QUICKSTART.md`, and `CLAUDE.md` as synchronized contract surfaces. Add tests where possible to compare implementation inventory against documentation.

## Consequences

- New MCP tools must update the central inventory and docs in the same change.
- Current MCP tool count is 27.
- CLI-only operator flows should stay explicitly documented as CLI-only.
- Future operation mapping should move toward a descriptor-style source of truth so CLI, MCP, and control-plane routing have fewer manual synchronization points.
