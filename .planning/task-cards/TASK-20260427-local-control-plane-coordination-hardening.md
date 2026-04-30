---
title: "Formalize local control-plane coordination"
id: "TASK-20260427-local-control-plane-coordination-hardening"
status: completed
owner: "codex"
priority: medium
phase_hint: "Executed as retrospective hardening for Phase 5 daemon/runtime coordination"
ft_ids_touched:
  - FT-02.03.02
why_these_ft_ids:
  - "FT-02.03.02 owns daemon-backed reuse of project operations through the local control plane, including fallback and remediation behavior."
requirement_ids:
  - CTRL-01
  - CTRL-02
  - CTRL-03
  - CTRL-04
  - CTRL-05
lifecycle_impact:
  - ft_id: FT-02.03.02
    from: designing
    to: shipped
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not add remote network control or a new transport beyond the local control plane."
  - "Do not create a second monitor-ownership path beside the daemon-backed coordination flow."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Run Rust tests that cover daemon-client coordination, fallback/remediation behavior, and daemon-backed report/guidance/cleanup requests."
  - "Inspect representative CLI/MCP responses to confirm daemon-backed and local fallback result shapes stay aligned."
evidence_outputs:
  - "Governance validation output"
  - "Daemon coordination test output"
  - "Example remediation and daemon-unavailable payloads"
---

## Goal

Make daemon, CLI, and MCP share one consistent coordination path for project operations so AI agents and operators do not silently drift into duplicate monitor ownership or mismatched runtime state.

## Capability Scope

- `FT-02.03.02`

## Requirement Scope

This card covers the complete local runtime-coordination family:

- `CTRL-01`
- `CTRL-02`
- `CTRL-03`
- `CTRL-04`
- `CTRL-05`

## Change Plan

1. Route daemon-owned project operations through the local control plane instead of ad hoc interface-specific ownership.
2. Keep fallback and remediation behavior explicit when daemon coordination is unavailable or unsafe.
3. Verify that monitor state, reporting, guidance, verification, cleanup, and configuration reload stay consistent across daemon-backed and local execution paths.

## Guardrails

- Do not broaden this card into new business capability outside `FT-02.03.02`.
- Do not hide daemon-unavailable behavior; return explicit remediation instead of silent divergence.
- Do not allow local and daemon-backed result contracts to drift for the same user-visible operation.

## Verification

- `python3 scripts/validate_planning_governance.py`
- daemon coordination and fallback Rust tests
- CLI/MCP response-shape review for daemon-backed operations

## Completion Criteria

- daemon-backed project operations reuse the local control plane consistently
- duplicate monitor ownership risk is reduced across CLI, MCP, and daemon paths
- remediation and fallback behavior remain explicit and bounded
- requirement and FT mappings remain valid
