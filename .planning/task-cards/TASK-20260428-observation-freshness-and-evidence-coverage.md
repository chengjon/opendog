---
title: "Normalize observation freshness and evidence coverage metadata"
id: "TASK-20260428-observation-freshness-and-evidence-coverage"
status: completed
owner: "codex"
priority: medium
phase_hint: "Active Phase 6 refinement for observation, strategy, evidence, and boundaries"
ft_ids_touched:
  - FT-03.01.01
  - FT-03.02.02
  - FT-03.03.01
  - FT-03.07.01
why_these_ft_ids:
  - "Freshness and coverage metadata tighten readiness/evidence-gap reporting at the workspace and project level."
  - "The same metadata should directly influence next-step execution strategy instead of living only as descriptive text."
  - "Verification freshness and evidence coverage belong to durable evidence reasoning and explicit AI boundaries."
requirement_ids:
  - OBS-02
  - OBS-03
  - OBS-04
  - STRAT-02
  - STRAT-04
  - EVID-03
  - EVID-04
  - BOUND-02
  - BOUND-04
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not introduce a new capability family outside the existing Phase 6 observation/strategy/evidence/boundary leaves."
  - "Do not pretend freshness heuristics are authoritative proof; shell and project-native verification remain the final authority."
verification_plan:
  - "Run `cargo test`."
  - "Run `python3 scripts/validate_planning_governance.py`."
  - "Inspect guidance/decision-brief JSON to confirm freshness, coverage, and evidence-gap fields are machine-readable."
evidence_outputs:
  - "Rust test output covering guidance, verification, and decision-brief payloads"
  - "Governance validation output"
  - "Representative guidance JSON with freshness/coverage metadata"
---

## Goal

Make OPENDOG say not only what it knows, but how fresh and how complete that evidence is, so AI can sequence snapshot, monitoring, verification, and review work more reliably.

## Capability Scope

- `FT-03.01.01`
- `FT-03.02.02`
- `FT-03.03.01`
- `FT-03.07.01`

## Requirement Scope

This card focuses on machine-readable freshness, coverage, and evidence-gap metadata for already shipped guidance surfaces.

## Change Plan

1. Add project-level freshness and coverage metadata for snapshot, activity, and verification evidence.
2. Feed freshness/coverage state into next-step recommendation logic where stale evidence should change sequencing.
3. Expose the resulting metadata consistently through guidance and decision-oriented JSON outputs.

## Guardrails

- Do not collapse freshness heuristics into binary “safe/unsafe” claims.
- Do not hide missing or stale evidence behind human-only prose.
- Do not change capability ownership away from the listed `FT-*` leaves.

## Verification

- `cargo test`
- `python3 scripts/validate_planning_governance.py`

## Completion Criteria

- guidance and decision payloads expose freshness and coverage state explicitly
- stale evidence can change recommended next actions
- governance mappings and task-card validation remain clean

## Completion Note

This slice is now reflected in shipped guidance and decision payloads:

- project overviews expose machine-readable observation freshness, coverage state, and evidence-gap metadata
- stale snapshot or verification evidence can change recommended next actions before AI proceeds
- CLI JSON, daemon-backed responses, and MCP guidance share the same freshness/coverage contract
