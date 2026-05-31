# Technical Debt Report - opendog

Generated: 2026-05-31T02:27:49Z

## Executive Summary

Overall status: WARN.

The current codebase is clean on the hard gates measured in this line: Rust check, clippy with denied warnings, production panic/unwrap/expect/suppression markers, ignored tests, `should_panic` tests, placeholder assertions, and configured size budgets. The warning status is retained because dependency/security audit tools are not installed in the local environment and because `cargo tree -d` still reports duplicate transitive crates that do not have a low-risk local fix.

Scope constraints honored:

- Code files at or below 500 lines are not treated as split candidates.
- Document structure is limited to `docs/mcp-tool-reference.md` and `docs/json-contracts.md`.
- `FUNCTION_TREE.md` and historical documents are excluded from document-structure work.

Baseline written to: `reports/analysis/tech-debt-baseline.json`.

Executable drift gate: `python3 scripts/validate_tech_debt_baseline.py`.

## D1: Code Quality

Rating: A.

Measured state:

- `cargo check --all-targets --all-features --quiet`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Production Rust escape hatches: 0 `panic!`, 0 `.unwrap()`, 0 `.expect()`, 0 `#[allow(...)]`, 0 `todo!`, 0 `unimplemented!`, 0 `dbg!`.
- Production TODO/FIXME/HACK/XXX comments: 0.
- Code files over 500 lines under `src/**/*.rs`, `tests/**/*.rs`, and `scripts/**/*.py`: 0.

Remediation completed in this line:

- Removed production clippy suppressions by packaging data-risk payload inputs and decision-brief build inputs.
- Removed `truncate_str(max < 3)` underflow behavior and replaced panic-documenting tests with explicit boundary expectations.

## D2: Architecture

Rating: A.

Measured state:

- `python3 scripts/validate_structural_hygiene.py`: passed.
- `python3 scripts/validate_planning_governance.py`: passed.
- `python3 scripts/validate_tech_debt_baseline.py`: passed.
- Structural hygiene scan: 492 files within configured size budgets.
- No code file currently exceeds the agreed 500-line split threshold.

Notes:

- The prior structural split line has removed all currently eligible code-file size violations.
- No additional split review is recommended for files at or below 500 lines unless a future change creates a concrete coupling or testability problem.

## D3: Testing

Rating: B.

Measured state:

- Full suite: 1821 unit/module tests and 31 integration tests passed in the latest full gate before this report line.
- Targeted `truncate_str` boundary tests: 11 passed after removing `should_panic`.
- Ignored tests: 0.
- `should_panic` tests: 0.
- Placeholder assertions: 0.
- Test TODO/FIXME/HACK/XXX comments: 0.
- Sleep calls in test-bearing files: 10.

Observed sleep call locations:

- `src/daemon.rs`: 2.
- `src/control/transport.rs`: 2.
- `src/core/monitor.rs`: 1.
- `tests/integration_test/daemon_process_cli.rs`: 2.
- `tests/integration_test/mcp_session_reuse.rs`: 2.
- `tests/integration_test/daemon_control.rs`: 1.

Notes:

- The remaining sleep calls are observed rather than gated because several are polling or timing-bound daemon/control tests. They should be revisited only when a deterministic readiness/event primitive is available.

## D4: Documentation

Rating: A.

Measured state under the current document policy:

- `docs/mcp-tool-reference.md`: 846 lines, under the 1000-line split threshold.
- `docs/json-contracts.md`: 773 lines, under the 1000-line split threshold.
- Policy document files over 1000 lines: 0.

Notes:

- `FUNCTION_TREE.md` is excluded by the current policy.
- Historical documents are excluded by the current policy and were not modified.

## D5: Dependencies

Rating: C.

Measured state:

- Direct dependencies: 15.
- Dev dependencies: 2.
- `cargo tree -d --depth 3` duplicate crate groups: `hashbrown`, `memchr`, `serde_core`, `serde_json`.
- `cargo-audit`: unavailable.
- `cargo-deny`: unavailable.

Dependency interpretation:

- `hashbrown` duplication is transitive through `rusqlite/hashlink` and `process-wrap/indexmap`.
- `serde_json`, `serde_core`, and `memchr` appear as same-version graph duplication across normal and proc-macro/build contexts.
- No direct dependency deletion or version pin was identified as a low-risk local fix.

Recommended next step:

- Add an optional dependency audit tool in a separate environment/tooling task if vulnerability gating is required.

## D6: Process And Security

Rating: B.

Measured state:

- Debt exception annotations: 0.
- Production TODO/FIXME/HACK/XXX comments: 0.
- `gitleaks`: unavailable.
- `trufflehog`: unavailable.

Notes:

- No secrets claim is made from this report because the local secret-scanning tools are not installed.
- Governance and structural validators are passing and should remain part of the standard gate.

## Priority Plan

P0 - Must fix before merge:

- None currently identified.

P1 - Current iteration:

- Keep hard gates at zero for production panic/unwrap/expect/suppressions and skipped/panic-documenting tests.

P2 - Next iteration:

- Decide whether to install and gate `cargo-audit` or `cargo-deny`.
- Decide whether to install and gate `gitleaks` or another secret scanner.
- Revisit test sleep calls only where a deterministic readiness/event primitive can replace timing waits without increasing flakiness.

P3 - Backlog:

- Monitor upstream dependency graph for a future opportunity to collapse the transitive `hashbrown` split.

## Reproducible Commands

Core gates:

```bash
cargo check --all-targets --all-features --quiet
cargo fmt --check
cargo test --quiet
cargo clippy --all-targets --all-features -- -D warnings
python3 -m unittest scripts.test_validate_structural_hygiene scripts.test_structural_contract_guards scripts.test_structural_rust_guards
python3 -m unittest scripts.test_validate_tech_debt_baseline
python3 scripts/validate_planning_governance.py
python3 scripts/validate_structural_hygiene.py
python3 scripts/validate_tech_debt_baseline.py
git diff --check
```

Debt measurements:

```bash
cargo tree -d --depth 3
```

Optional measurements not available in this environment:

```bash
cargo audit
cargo deny check
gitleaks detect
trufflehog filesystem .
```
