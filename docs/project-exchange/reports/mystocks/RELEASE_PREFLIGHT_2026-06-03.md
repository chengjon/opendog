# Release Preflight - 2026-06-03

**Scope**: MyStocks feedback hardening, structural cleanup, documentation refresh, and implementation recheck.  
**Branch**: `master`  
**Head**: `7c5eb9c docs: refresh gitnexus guidance metadata`  
**Cargo version**: `0.1.0`  
**Existing tags**: none found at preflight time.

## Status

The repository is release-ready from the current local and CI evidence. No tag or GitHub release was created by this preflight.

## Verification Evidence

Local release readiness for `7c5eb9c` passed:

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
- External security audit: PASS for `7c5eb9c`
- Release readiness: PASS

Remote workflow evidence for `7c5eb9c`:

| Workflow | Event | Status | URL |
|----------|-------|--------|-----|
| Repository Gate | `workflow_dispatch` | success | <https://github.com/chengjon/opendog/actions/runs/26877028148> |
| Repository Gate | `push` | success | <https://github.com/chengjon/opendog/actions/runs/26876880785> |
| External Security Audit | `workflow_dispatch` | success | <https://github.com/chengjon/opendog/actions/runs/26877030079> |

## Included Work

This preflight covers the work line ending at `7c5eb9c`, including:

- MyStocks usage-feedback implementation recheck:
  - `docs/project-exchange/reports/mystocks/IMPLEMENTATION_SUMMARY_2026-05-26_RECHECK_2026-06-03.md`
- Project status refresh:
  - `FUNCTION_TREE.md`
  - `README.md`
  - `CHANGELOG.md`
- Low-risk structural cleanup and test fixture extraction captured in recent commits.
- GitNexus guidance metadata refreshed in `AGENTS.md` and `CLAUDE.md`.

## Candidate Release Action

Because `Cargo.toml` currently declares `version = "0.1.0"` and there are no existing tags, the natural first release tag candidate is:

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

If the release should represent only the MyStocks hardening line rather than the whole initial package version, use a dated or scoped tag instead, for example:

```bash
git tag -a mystocks-hardening-2026-06-03 -m "MyStocks hardening release 2026-06-03"
git push origin mystocks-hardening-2026-06-03
```

## Next Decision

Choose one release path before creating a tag:

1. Create first semver tag `v0.1.0`.
2. Create scoped milestone tag `mystocks-hardening-2026-06-03`.
3. Do not tag yet; keep `7c5eb9c` as the release-ready checkpoint.
