---
title: "Make verification evidence freshness TTL explicit and configurable"
id: "TASK-20260510-verification-evidence-ttl-policy"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 verification evidence hardening"
ft_ids_touched:
  - FT-03.03.01
  - FT-03.01.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-03.03.01 owns durable verification evidence and safety-gate reasoning."
  - "FT-03.01.01 owns freshness and evidence gaps in observation/readiness summaries."
  - "FT-03.07.01 owns boundary language when evidence is stale or insufficient."
requirement_ids:
  - EVID-01
  - EVID-02
  - EVID-03
  - EVID-04
  - OBS-01
  - OBS-02
  - OBS-03
  - BOUND-03
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not execute verification commands automatically on TTL expiry."
  - "Do not make stale evidence disappear from history."
  - "Do not make TTL policy project-destructive or source-mutating."
verification_plan:
  - "Add tests for default TTL behavior across test/lint/build evidence."
  - "Add tests for project/global config overrides if TTL becomes configurable through existing config surfaces."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Updated verification JSON contract fields or docs for TTL/freshness behavior."
  - "Root `CHANGELOG.md` entry when implemented."
completion_notes:
  - "Exposed default freshness TTL policy in verification freshness payloads and gate assessments."
  - "Added regression coverage for policy fields and existing stale-evidence blocking behavior."
  - "Kept stale evidence visible as history; TTL affects gate interpretation only."
  - "Did not add new config fields because threading user-configurable TTL through existing config surfaces would widen this task beyond policy transparency."
---

## Goal

Make verification evidence freshness explicit enough that long-lived branches do not rely on old `test`, `lint`, or `build` results as if they were current.

## Evidence Source

`docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md` notes that recorded verification results affect cleanup/refactor gates, but their expiry policy is not clearly configurable or operator-visible enough.

## Change Plan

1. Audit current stale-evidence behavior and document the existing baseline.
2. Define default TTL policy for verification kinds such as `test`, `lint`, and `build`.
3. Expose stale-vs-fresh policy in verification status, guidance, and decision-support outputs.
4. Add config-path support only if it fits existing global/project config semantics without widening scope.

## Guardrails

- Keep stale evidence visible as history.
- Stale means "refresh before relying on this", not "verification failed".
- Do not run commands automatically.

## Completion Criteria

- Verification freshness policy is machine-readable and documented.
- Safety gates consistently account for stale evidence.
- Operators can understand what must be rerun before cleanup/refactor decisions.

## Closure

This task is closed as a default-policy transparency change. The current TTL policy is now machine-readable in verification payloads. User-configurable TTL remains a separate compatibility decision if future evidence requires project-specific policy.
