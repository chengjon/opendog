# MyStocks Implementation Follow-Up Recheck - 2026-06-03

**Source**: `docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26.md`  
**Related reviews**:

- `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_MCP_USAGE_REVIEW_2026-05-26.md`
- `docs/project-exchange/reports/mystocks/OPENDOG_MCP_USAGE_REVIEW_2026-05-26_AUDIT.md`
- `docs/project-exchange/reports/mystocks/USAGE_REVIEW_AUDIT_RESPONSE.md`
- `docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26_REAUDIT_2026-05-28.md`

## Verdict

The F-1 through F-7 implementation claims remain consistent with the current codebase. No new functional gap was found in this recheck.

The remaining items are intentional product-boundary or prioritization decisions, not unimplemented commitments:

- `host_tools_visible` automatic detection remains outside OPENDOG's direct responsibility because the AI host owns tool visibility.
- Automatic capability-matrix generation remains a useful future documentation automation, but current `docs/capability-index.md`, `docs/mcp-tool-reference.md`, and `docs/json-contracts.md` already describe the corrected surfaces.
- A separate `opendog doctor mcp` CLI remains deferred; `get_build_info` plus MCP/JSON contract docs expose the smaller diagnostic envelope.

## Evidence Matrix

| Item | Status | Evidence |
|------|--------|----------|
| F-1 daemon/schema mismatch diagnostics | PASS | `src/storage/migrations.rs:33` reports a newer-than-supported schema and `src/storage/migrations.rs:34` tells operators to restart the daemon and MCP session. `src/storage/migrations/tests.rs:131` asserts the restart hint. `src/mcp/payloads/config_payloads.rs:35` exposes `storage_schema_version`; `src/mcp/payloads/config_payloads.rs:40-41` expose `daemon_running` and `opendog_home`. `docs/mcp-tool-reference.md:306-309` documents the fields and host boundary. |
| F-2 verification trust detection and gate integration | PASS | `src/core/verification.rs:86` exposes `command_contains_pipeline_operators`; `src/core/verification.rs:91` exposes `detect_suspicious_pass_signals`; `src/core/verification.rs:171-200` stores `pipeline_operators_detected` and `suspicious_pass_signals`. `src/core/verification/tests.rs:237-266` covers spaced and unspaced shell operators such as `cmd|tail`, `cmd&&echo`, and `cmd||true`. `src/mcp/verification_evidence/model/gate/rules.rs:54` and `src/mcp/verification_evidence/model/gate/rules.rs:72` compute pipeline and suspicious-summary caution kinds. `src/mcp/verification_evidence/tests/gate_assessment.rs:92`, `:129`, and `:164` assert gate output fields. |
| F-3 large-database report and cleanup protection | PASS | `src/core/report/time_window.rs:127` and `src/core/report/time_window.rs:145` add SQL `LIMIT ?3`; `src/core/report/usage_trend.rs:163` adds `LIMIT ?4`; `src/core/report/time_window.rs:98-107` computes `truncated`. `src/mcp/tests/payload_contracts/analysis_payloads.rs:131`, `:147`, `:175`, and `:198` assert `result_window.truncated`. `src/core/retention/executor.rs:76-88` switches dry-run cleanup to `EstimateMode::ScopeCountsOnly` above the snapshot-run threshold while leaving real deletion on full mode. `src/core/retention/tests/estimate.rs:36` and `:82` cover estimate mode and real cleanup boundaries. |
| F-4 data-risk path classification noise reduction | PASS | `src/mcp/mock_detection/path_classification.rs:97-112` classifies `.claude/`, `.cursor/`, `.agents/`, `.amazonq/`, and `.zread/` as `infrastructure`. `src/mcp/data_risk/rules.rs:30-35` validates `min_review_priority`; `src/mcp/data_risk/rules.rs:43-47` scores review priority; `src/mcp/data_risk/rules.rs:57-58` down-ranks documentation and generated-artifact path kinds. `docs/mcp-tool-reference.md:234-252` documents payload bounds and `path_classification`; `docs/mcp-tool-reference.md:566`, `:581`, and `:601` document `min_review_priority`, `review_priority`, and operator interpretation. |
| F-5 advisory-boundary regression protection | PASS | `src/mcp/strategy/tests.rs:327` covers stale-verification cleanup blocking. `src/mcp/strategy/tests.rs:345` covers `destructive_change_recommended = false` for weak evidence. `src/mcp/decision_support/profiles/model/risk.rs:202` and `:241` preserve non-destructive recommendation semantics. |
| F-6 CLI/MCP/documentation capability surface | PASS | `docs/opendog-feature-introduction.md:69`, `:86`, and `:158` describe the MCP decision entrypoint as `get_guidance(detail = "decision")`, while `docs/opendog-feature-introduction.md:164` keeps the CLI equivalent `opendog decision-brief`. `docs/capability-index.md:113` and `docs/mcp-tool-reference.md:34-35` present the same MCP/CLI pairing. |
| F-7 MCP host and daemon diagnostics | PASS | `src/mcp/config_handlers.rs:53-54` probes daemon reachability and resolves the OPENDOG data root. `src/mcp/payloads/config_payloads.rs:20-21` defines `daemon_running` and `opendog_home`; `src/mcp/payloads/config_payloads.rs:40-41` emits both fields. `src/mcp/payloads/config_payloads/tests.rs:215-222` asserts `daemon_running`. `docs/json-contracts.md:457-458` documents the consumption rules and warns not to infer AI-host tool visibility from this payload. |

## Current Verification

The 2026-06-03 release-readiness run for the latest branch head passed after the project-status document refresh:

- Repository gate: PASS
- OpenSpec strict validation: 7 passed, 0 failed
- Rust tests: 1822 lib tests passed, 32 integration tests passed
- `cargo fmt --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `ruff check scripts`: PASS
- Python unit tests: PASS
- Technical-debt baseline: PASS
- Planning governance: PASS
- Structural hygiene: PASS
- External security audit: PASS for commit `3d7a8ab`
- Release readiness: PASS

GitHub Actions confirmed for commit `3d7a8ab`:

- Repository Gate: <https://github.com/chengjon/opendog/actions/runs/26875493515>
- External Security Audit: <https://github.com/chengjon/opendog/actions/runs/26875503104>

## Follow-Up Plan

1. Keep monitoring `scripts/validate_tech_debt_baseline.py` and `scripts/validate_structural_hygiene.py` for drift after future feature work.
2. Do not split historical project-exchange documents. Current documentation policy only considers `docs/mcp-tool-reference.md` and `docs/json-contracts.md` for splitting after they exceed 1000 lines.
3. If MyStocks or another consuming project reports renewed large-database pressure, prioritize operator-visible telemetry around cleanup estimates, retained evidence volume, and SQLite maintenance status before adding broader timeout machinery.
4. If AI-host visibility becomes a recurring support problem, document a host-side checklist first; only add OPENDOG code after the host exposes a reliable visibility API.
