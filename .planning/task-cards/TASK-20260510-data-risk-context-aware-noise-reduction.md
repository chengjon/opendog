---
title: "Reduce context-insensitive data-risk false positives"
id: "TASK-20260510-data-risk-context-aware-noise-reduction"
status: completed
owner: "codex"
priority: high
phase_hint: "Phase 6 data-risk quality hardening"
ft_ids_touched:
  - FT-03.08.01
  - FT-03.08.02
  - FT-03.07.01
why_these_ft_ids:
  - "FT-03.08.01 owns mock/test-only artifact detection and must distinguish benign examples from runtime data risk."
  - "FT-03.08.02 owns hardcoded pseudo-business data prioritization and is currently too sensitive to documentation and template variables."
  - "FT-03.07.01 owns evidence boundaries; data-risk output must explain confidence, context, and false-positive boundaries."
requirement_ids:
  - MOCK-01
  - MOCK-02
  - MOCK-03
  - MOCK-04
  - MOCK-05
  - MOCK-06
  - MOCK-07
  - BOUND-01
  - BOUND-03
interface_surfaces:
  - cli
  - mcp
non_goals:
  - "Do not suppress all markdown or YAML findings blindly."
  - "Do not introduce secret scanning or compliance scanning as a new product family."
  - "Do not require target projects to add OpenDog-specific annotations to reduce false positives."
verification_plan:
  - "Add regression fixtures for markdown documentation, YAML `${VAR}` templates, runtime hardcoded literals, and test/mock artifacts."
  - "Run targeted data-risk tests and verify documentation/template candidates are downgraded while runtime hardcoded candidates remain high-priority."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Before/after candidate priority examples from `quantix-rust` report patterns."
  - "Updated data-risk contract docs if confidence or priority fields change."
  - "Root `CHANGELOG.md` entry when implemented."
completion_notes:
  - "Added documentation path classification for data-risk candidates."
  - "Added template-placeholder rule evidence and lowered hardcoded review priority/confidence for documentation or placeholder-heavy candidates."
  - "Kept runtime-shared source hardcoded candidates high priority when strong literal evidence is present."
  - "Updated MCP and JSON contract docs to clarify down-ranking rather than hiding."
---

## Goal

Reduce false positives in `get_data_risk_candidates` and workspace data-risk guidance by making hardcoded/mock detection more context-aware for documentation, template variables, and runtime-shared source paths.

## Evidence Source

`docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md` reports hardcoded candidates in markdown and deployment documentation where business keywords and literal markers are likely examples or templates rather than runtime liabilities.

## Change Plan

1. Add failing regression tests for markdown/template false positives and runtime-shared true positives.
2. Introduce file-type and path-context weighting for markdown, docs, examples, config templates, and runtime source files.
3. Detect placeholder/template variable patterns such as `${VAR}` and lower priority unless paired with stronger runtime evidence.
4. Preserve rule-hit transparency so users can see why a candidate was downgraded or retained.

## Guardrails

- Keep data-risk output advisory and auditable.
- Prefer priority/confidence reduction over silent removal.
- Preserve existing high-priority detection for runtime-shared source files with strong literal evidence.

## Completion Criteria

- Documentation and template examples no longer dominate hardcoded review priority.
- Runtime-shared hardcoded candidates remain visible and high-priority.
- Guidance explains downgrade basis through existing or extended evidence fields.

## Closure

This task is closed as a context-aware down-ranking change. Documentation and template-placeholder findings remain visible for auditability, but they no longer carry the same priority/confidence as runtime-shared hardcoded source candidates.
