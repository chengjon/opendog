---
title: "Add configuration policy management and safe live reload"
id: "TASK-20260427-configuration-policy-and-live-reload"
status: completed
owner: "codex"
priority: medium
phase_hint: "Executed as cross-phase hardening for Phase 4/5 surfaces"
ft_ids_touched:
  - FT-01.01.03
why_these_ft_ids:
  - "FT-01.01.03 owns per-project/global configuration policy, ignore-pattern mutation, and safe reload behavior."
requirement_ids:
  - CONF-01
  - CONF-02
  - CONF-03
lifecycle_impact:
  - ft_id: FT-01.01.03
    from: designing
    to: shipped
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not redesign the project registry or snapshot model beyond configuration ownership needs."
  - "Do not add a visual configuration editor."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Add tests for per-project ignore-pattern updates, global-default resolution, and live reload while a monitor is active."
  - "Verify CLI and MCP both expose the same effective configuration and reload result."
evidence_outputs:
  - "Governance validation output"
  - "Config precedence and reload test output"
  - "Example CLI/MCP config payloads"
---

## Goal

Make configuration a first-class managed capability so AI or operators can adjust ignore patterns and defaults without forcing a daemon restart or manual file edits.

## Capability Scope

- `FT-01.01.03`

## Requirement Scope

This card covers the full configuration management family:

- `CONF-01`
- `CONF-02`
- `CONF-03`

## Change Plan

1. Define the effective configuration model: project overrides, global defaults, and resolved runtime view.
2. Add configuration read/write surfaces for CLI and MCP with clear safety limits.
3. Add safe reload behavior so monitor-owned runtime state can refresh configuration without process replacement.

## Guardrails

- Do not expand into unrelated snapshot or analytics redesign.
- Do not treat reload as silent mutation; surface success, failure, and skipped fields explicitly.
- Do not mark the capability active without daemon-path verification.

## Verification

- `python3 scripts/validate_planning_governance.py`
- configuration precedence tests
- monitor live-reload integration test

## Completion Criteria

- configuration defaults and project overrides are queryable and changeable through supported interfaces
- reload behavior is explicit and safe for running monitor state
- requirement and FT mappings remain valid
