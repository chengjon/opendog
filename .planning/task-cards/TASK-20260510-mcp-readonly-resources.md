---
title: "Expose stable read-only MCP resources for project state"
id: "TASK-20260510-mcp-readonly-resources"
status: completed
owner: "codex"
priority: medium
phase_hint: "Phase 6 MCP consumption hardening"
ft_ids_touched:
  - FT-02.02.01
  - FT-03.01.01
  - FT-03.03.01
  - FT-03.04.01
why_these_ft_ids:
  - "FT-02.02.01 owns AI-facing machine-invocable surfaces, including future read-only MCP resources."
  - "FT-03.01.01 owns readiness and evidence-gap summaries that are good resource candidates."
  - "FT-03.03.01 owns verification status, a stable read-only state surface."
  - "FT-03.04.01 owns multi-project portfolio state, which could be exposed as a low-token read resource."
requirement_ids:
  - MCP-01
  - OBS-01
  - OBS-02
  - EVID-01
  - EVID-03
  - PORT-01
  - PORT-02
interface_surfaces:
  - mcp
non_goals:
  - "Do not replace existing MCP tools."
  - "Do not expose write or mutation operations as resources."
  - "Do not add a network transport or HTTP server."
verification_plan:
  - "Confirm rmcp resource support and design minimal URI set."
  - "Add tests for resource listing and reading if the current MCP stack supports it cleanly."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Documented resource URI contract such as `opendog://projects` and `opendog://project/{id}/verification`."
  - "Compatibility notes explaining when tools remain the preferred entrypoint."
  - "Root `CHANGELOG.md` entry when implemented."
---

## Goal

Add a governed path for read-only MCP Resources so AI clients can fetch stable OpenDog state without repeatedly invoking heavier tools.

## Evidence Source

`docs/project-exchange/reports/quantix-rust/opendog-mcp-test-report-2026-05-10.md` recommends resource URIs for project lists, health/readiness, and verification status to reduce token and tool-call overhead.

## Candidate Resources

- `opendog://projects`
- `opendog://project/{id}/observation`
- `opendog://project/{id}/verification`
- `opendog://project/{id}/guidance-summary`

## Change Plan

1. Verify current MCP crate support and client compatibility for resources.
2. Define the smallest stable read-only URI set.
3. Reuse existing payload builders and versioned contracts where possible.
4. Document resources separately from tools so clients know when each surface is appropriate.

## Guardrails

- Resources are read-only snapshots of state.
- Existing tools remain authoritative for operations and parameterized analysis.
- Resource payloads should be smaller and more stable than broad guidance tools.

## Completion Criteria

- MCP resource support is either implemented with tests or explicitly deferred with technical evidence.
- Resource URI contracts are documented.
- Existing MCP tools remain backward compatible.

## Completion Notes

- Implemented read-only MCP Resources support through the existing `rmcp` stdio server without adding a network transport.
- Added static resource discovery for `opendog://projects` via `resources/list`.
- Added resource-template discovery for `opendog://project/{id}/verification` via `resources/templates/list`.
- Added resource reads for `opendog://projects` and `opendog://project/{id}/verification`, reusing existing versioned JSON payload builders.
- Kept write/mutation operations on tool or CLI surfaces only.
- Documented the URI contract in `docs/mcp-tool-reference.md`.
- Updated root `CHANGELOG.md` as required by project governance.
