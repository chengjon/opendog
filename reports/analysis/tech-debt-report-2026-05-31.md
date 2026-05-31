# Technical Debt Report - opendog

Generated: 2026-05-31T02:27:49Z

## Executive Summary

Overall status: WARN.

The current codebase is clean on the hard gates measured in this line: Rust check, clippy with denied warnings, production panic/unwrap/expect/suppression markers, ignored tests, `should_panic` tests, placeholder assertions, configured size budgets, internal dependency audit availability, and high-confidence secret findings. The warning status is retained because local external vulnerability/security audit tools are not installed and because `cargo tree -d` still reports duplicate transitive crates that do not have a low-risk local fix. External `cargo-audit` and `gitleaks` scans are now available through the independent `External Security Audit` workflow.

Scope constraints honored:

- Code files at or below 500 lines are not treated as split candidates.
- Document structure is limited to `docs/mcp-tool-reference.md` and `docs/json-contracts.md`.
- `FUNCTION_TREE.md` and historical documents are excluded from document-structure work.

Baseline written to: `reports/analysis/tech-debt-baseline.json`.

Executable drift gate: `python3 scripts/validate_tech_debt_baseline.py`.
Machine-readable drift report: `python3 scripts/validate_tech_debt_baseline.py --drift-report reports/analysis/tech-debt-baseline-drift-report.json`.

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
- Structural hygiene scan: 497 files within configured size budgets.
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
- Manifest dependency entries: 17.
- Locked dependency packages: 193.
- `cargo tree -d --depth 3` duplicate crate groups: `hashbrown`, `memchr`, `serde_core`, `serde_json`.
- Internal dependency audit: available via `internal-cargo-inventory`.
- Dependency audit issue count: 0.
- Cargo lockfile missing count: 0.
- External dependency audit workflow: available via `.github/workflows/external-security-audit.yml`.
- `cargo-audit`: unavailable.
- `cargo-deny`: unavailable.

Dependency interpretation:

- `hashbrown` duplication is transitive through `rusqlite/hashlink` and `process-wrap/indexmap`.
- `serde_json`, `serde_core`, and `memchr` appear as same-version graph duplication across normal and proc-macro/build contexts.
- No direct dependency deletion or version pin was identified as a low-risk local fix.

Recommended next step:

- Run the `External Security Audit` workflow manually or on its weekly schedule for CVE-backed `cargo-audit` coverage; the built-in gate covers inventory/lockfile presence in the standard repository gate.

## D6: Process And Security

Rating: B.

Measured state:

- Debt exception annotations: 0.
- Production TODO/FIXME/HACK/XXX comments: 0.
- Internal high-confidence secret scan: available.
- High-confidence secret findings: 0.
- External secret scan workflow: available via `.github/workflows/external-security-audit.yml`.
- `gitleaks`: unavailable.
- `trufflehog`: unavailable.

Notes:

- The built-in secret scan covers high-confidence token/private-key patterns without storing matched secret values. The external workflow runs pinned `gitleaks` for broader scanner coverage outside the standard repository gate.
- Governance and structural validators are passing and should remain part of the standard gate.

## Priority Plan

P0 - Must fix before merge:

- None currently identified.

P1 - Current iteration:

- Keep hard gates at zero for production panic/unwrap/expect/suppressions and skipped/panic-documenting tests.

P2 - Next iteration:

- Review the first scheduled/manual `External Security Audit` run and decide whether it should stay independent or become required before releases.
- Decide whether to add `cargo-deny` policy checks later for license/source policy, separate from `cargo-audit` CVE checks.
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
python3 scripts/validate_tech_debt_baseline.py --drift-report reports/analysis/tech-debt-baseline-drift-report.json
python3 scripts/validate_repository_gate.py
git diff --check
```

Debt measurements:

```bash
cargo tree -d --depth 3
python3 scripts/validate_tech_debt_baseline.py --drift-report reports/analysis/tech-debt-baseline-drift-report.json
```

Optional external measurements not available in this environment:

```bash
cargo deny check
trufflehog filesystem .
```

External workflow:

```bash
gh workflow run "External Security Audit"
```
