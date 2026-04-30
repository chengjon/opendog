---
title: "Harden Phase 6 guidance and boundary payloads"
id: "TASK-20260426-phase6-guidance-hardening"
status: completed
owner: "codex"
priority: medium
phase_hint: "Executed as iterative hardening for active Phase 6 guidance surfaces"
ft_ids_touched:
  - FT-03.01.01
  - FT-03.02.01
  - FT-03.02.02
  - FT-03.03.01
  - FT-03.06.01
  - FT-03.07.01
why_these_ft_ids:
  - "These leaves own the readiness, risk, strategy, evidence, toolchain, and boundary signals that Phase 6 exposes."
  - "The task tightens an existing reusable guidance surface instead of adding a new business capability."
requirement_ids:
  - OBS-01
  - OBS-02
  - OBS-03
  - RISK-01
  - RISK-02
  - STRAT-01
  - STRAT-02
  - EVID-01
  - EVID-02
  - STACKX-01
  - STACKX-02
  - BOUND-01
  - BOUND-02
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not introduce a new business-capability family outside the listed Phase 6 guidance leaves."
  - "Do not turn OPENDOG guidance into proof; shell and project-native verification remain authoritative."
verification_plan:
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Run the Phase 6 Rust test suite that covers guidance, risk, evidence, and boundary payloads."
  - "Inspect the CLI and MCP JSON samples for the updated guidance envelope."
evidence_outputs:
  - "Task-card validation output"
  - "`cargo test` output covering guidance, verification, data-risk, and control-plane paths"
  - "Updated guidance payload examples in README, CLAUDE.md, docs/ai-playbook.md, docs/mcp-tool-reference.md, docs/json-contracts.md, and docs/capability-index.md"
---

## Goal

Harden the reusable Phase 6 guidance envelope so it is easier for AI agents to choose the right next action without over-claiming certainty.

## Capability Scope

- `FT-03.01.01`
- `FT-03.02.01`
- `FT-03.02.02`
- `FT-03.03.01`
- `FT-03.06.01`
- `FT-03.07.01`

## Requirement Scope

This card covers the initial Phase 6 guidance families only. It does not expand into unrelated reporting or export features.

## Change Plan

1. Tighten the structured guidance payloads for readiness, risk, strategy, and boundaries.
2. Keep evidence references concise and machine-readable.
3. Verify the CLI and MCP outputs still agree on the same recommendation surface.

## Guardrails

- Do not broaden the card beyond the listed FT leaves.
- Do not add new capability families without updating `.planning/FUNCTION_TREE.md`.
- Do not mark the work done without validation output.

## Verification

- `python3 scripts/validate_planning_governance.py`
- Phase 6 test coverage for guidance payloads

## Completion Criteria

- guidance payloads remain structured and bounded
- evidence and boundary fields are still explicit
- task-card mapping remains valid

## Completion Note

This hardening slice is now reflected in the shipped operator and AI surfaces:

- `agent-guidance` / `get_agent_guidance`
- `decision-brief` / `get_decision_brief`
- `verification`, `record-verification`, and `run-verification`
- `data-risk` / `get_data_risk_candidates`
- `workspace-data-risk` / `get_workspace_data_risk_overview`

The surrounding docs and JSON-contract references were also aligned so AI agents can choose entrypoints without guessing.
