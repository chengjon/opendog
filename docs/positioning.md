# OPENDOG Positioning

> Current-state framing aligned on 2026-04-28

## Quick Navigation

- You are here: `Positioning` — current product framing, scope boundaries, and design judgment
- Need the fastest capability-to-command map: [Capability Index](./capability-index.md)
- Need AI workflow order and shell handoff rules: [AI Playbook](./ai-playbook.md)
- Need MCP request/response usage: [MCP Tool Reference](./mcp-tool-reference.md)
- Need machine-readable output fields: [JSON Contracts](./json-contracts.md)

## One Sentence

OPENDOG is a multi-project observation and AI decision-support system that watches how AI tools interact with project files, stores reusable evidence, and helps users or agents decide what to inspect, verify, review, or clean up next.

## What It Is

- A workspace-level observation layer above individual repositories
- A reusable MCP/CLI/daemon-backed decision-support surface for AI workflows
- A bounded engineering aid that summarizes evidence, prioritizes attention, and exposes explicit next-step guidance

## What It Is Not

- `git`, tests, lint, and build are external truth sources; treat OPENDOG output as decision-support evidence and switch to shell or project-native validation when confirmation is required.
- Not an auto-cleanup or destructive repo-management tool
- Not a generic governance platform that should sit in front of every routine operation

## Current Design Judgment

- No direction drift: the project still centers on multi-project observation plus AI decision support
- Broad but bounded scope: the surface is wider than a simple monitoring backend, but it stays constrained by evidence limits, authority boundaries, and non-destructive behavior
- Selective deepening over expansion: the near-term priority is to harden existing `FT-03` decision-support leaves rather than keep opening unrelated capability families

## Runtime Shape

Treat the runtime product as three layers:

1. Observation core
2. Service delivery and runtime coordination
3. AI decision-support layer

Treat planning governance as an overlay:

- `FUNCTION_TREE`, requirement mappings, validators, and task cards exist to keep capability ownership clean when the product changes
- They are not the default first stop for routine CLI, MCP, or daemon usage

## How To Read The Project

- Start with [README](../README.md) for current implementation shape
- Use [Capability Index](./capability-index.md) for the fastest capability-to-command map
- Use [AI Playbook](./ai-playbook.md) when an AI needs execution order and shell handoff rules
- Use [MCP Tool Reference](./mcp-tool-reference.md) for request/response usage
- Use [.planning/FUNCTION_TREE.md](../.planning/FUNCTION_TREE.md) only when capability ownership, requirement mapping, or structural scope is changing
