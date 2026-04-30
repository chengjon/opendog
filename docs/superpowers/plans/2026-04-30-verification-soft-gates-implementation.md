# Verification Soft Gates Implementation Plan

Goal: add additive verification soft gates that distinguish `allow`, `caution`, and `blocked`, then route MCP guidance and decision outputs through that shared evidence model without hard-blocking any CLI or MCP command.

## Constraints

- Keep `src/mcp/verification_evidence.rs` as the only source of truth for verification gate state.
- Preserve all existing top-level fields and schema versions. New fields are additive.
- Keep `safe_for_cleanup` and `safe_for_refactor` compatible with required-gate pass/fail semantics.
- Keep `cleanup_blockers` and `refactor_blockers` blocker-only. Advisory caution reasons must stay under gate-specific fields.
- Reuse one shared readiness helper for all higher layers instead of recomputing local rules.

## File Map

- `src/mcp/verification_evidence.rs`
- `src/mcp/constraints.rs`
- `src/mcp/project_recommendation.rs`
- `src/mcp/project_guidance/stats_unused/stats.rs`
- `src/mcp/project_guidance/stats_unused/unused.rs`
- `src/mcp/decision_support/profiles.rs`
- `docs/json-contracts.md`
- `docs/mcp-tool-reference.md`
- tests under `src/mcp/tests/`

## Task 1: Source-Of-Truth Gate Contract

Scope:

- Add `gate_assessment.cleanup` and `gate_assessment.refactor` to `verification_status_layer(...)`.
- Each gate exposes:
  - `allowed`
  - `level`
  - `required_kinds`
  - `advisory_kinds`
  - `missing_kinds`
  - `failing_kinds`
  - `stale_kinds`
  - `reasons`
  - `next_steps`

Rules:

- Cleanup requires fresh passing `test`; `lint` and `build` are advisory.
- Refactor requires fresh passing `test` and `build`; `lint` is advisory.
- `blocked` means required evidence is missing, stale, or failing.
- `caution` means required evidence passed but advisory evidence is missing or stale.
- `allow` means both required and advisory evidence are in good shape.

Compatibility:

- `safe_for_cleanup == gate_assessment.cleanup.allowed`
- `safe_for_refactor == gate_assessment.refactor.allowed`
- Legacy blocker arrays only include blocker reasons.

Tests:

- `src/mcp/tests/repo_and_readiness/verification_readiness.rs`
- `src/mcp/tests/payload_contracts/verification_payloads.rs`

## Task 2: Shared Readiness Snapshot

Add `project_readiness_snapshot(repo_risk, verification_layer)` in `src/mcp/constraints.rs`.

Required outputs:

- `verification_safe_for_cleanup`
- `verification_safe_for_refactor`
- `cleanup_gate_level`
- `refactor_gate_level`
- `safe_for_cleanup`
- `safe_for_cleanup_reason`
- `cleanup_blockers`
- `safe_for_refactor`
- `safe_for_refactor_reason`
- `refactor_blockers`

Semantics:

- `verification_safe_for_*` reflects verification-only gate pass/fail.
- `safe_for_*` reflects verification plus repo-risk blockers.
- `build_constraints_boundaries_layer(...)`, `project_overview(...)`, and `recommend_project_action(...)` must consume this shared snapshot instead of recomputing local readiness.

Tests:

- `src/mcp/tests/overview_constraints.rs`
- `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/verification_gates.rs`
- `src/mcp/tests/repo_and_readiness/action_recommendations/project_recommendations/readiness_progression.rs`

## Task 3: Guidance Consumers

Update stats and unused guidance to consume the shared readiness snapshot.

Required behavior:

- Reuse shared `safe_for_*`, blocker, and reason fields.
- Surface `cleanup_gate_level` and `refactor_gate_level` inside `layers.execution_strategy`.
- Keep existing recommendation flow intact; only sharpen readiness explanations.

Tests:

- `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs`

## Task 4: Workspace And Decision Outputs

Workspace verification aggregation must expose the new soft-gate shape without changing old counts.

Required behavior:

- Add `cleanup_gate_distribution.allow|caution|blocked`.
- Add `refactor_gate_distribution.allow|caution|blocked`.
- Include gate levels on blocking project summaries.
- Add `cleanup_gate_level` and `refactor_gate_level` to `decision.risk_profile`.
- For `review_unused_files` and `inspect_hot_files`, downgrade the risk profile from `low` to `medium` when the relevant gate is `caution` or `blocked`.

Tests:

- `src/mcp/tests/guidance_basics/toolchain_and_unused/workspace_aggregates/verification_evidence.rs`
- `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

## Task 5: Docs And Verification

Docs to update:

- `docs/json-contracts.md`
- `docs/mcp-tool-reference.md`

Required documentation points:

- `gate_assessment.*.level` is the new soft-gate signal.
- `safe_for_*` keeps legacy compatibility and can remain `true` while `level == "caution"`.
- Legacy blocker arrays remain blocker-only.
- Guidance and decision payloads now expose gate levels so consumers can distinguish advisory gaps from true blockers.

Verification sequence:

1. `cargo fmt --check`
2. `cargo check`
3. `cargo test repo_and_readiness --lib`
4. `cargo test guidance_basics --lib`
5. `cargo test portfolio_commands --lib`
6. `cargo test`
7. `python3 scripts/validate_planning_governance.py`

## Implementation Notes

- Do not add hard execution blocking to CLI or MCP handlers.
- Do not change schema version constants for this feature.
- Do not move advisory-only caution reasons into legacy blocker arrays.
- If a higher layer needs readiness information, route it through the shared snapshot instead of inventing another helper.
