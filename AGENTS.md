# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` is the binary entrypoint and delegates to `opendog::cli::run()`. Core product logic lives in `src/core/` (`snapshot`, `monitor`, `report`, `retention`, `verification`), persistence lives in `src/storage/`, and daemon/local-control-plane code lives in `src/control/`. MCP tool surfaces and payload builders are under `src/mcp/`; CLI handlers are under `src/cli/`; config loading and validation are under `src/config/`. Integration tests live in `tests/integration_test/`, while module-scoped tests stay near the code in `src/**/tests.rs`. Planning and scope-governance artifacts live in `.planning/`; longer operator docs live in `docs/`; deployment assets live in `deploy/`.

## Build, Test, and Development Commands
`cargo build --release` builds `target/release/opendog`.
`cargo run -- mcp` starts the MCP stdio server; `cargo run -- daemon` starts the background coordinator.
`cargo test` runs the full suite; `cargo test test_snapshot` narrows to snapshot coverage; `RUST_LOG=debug cargo test -- --nocapture` helps debug failures.
`cargo clippy --all-targets --all-features -- -D warnings` is the lint gate used throughout the repo.
`cargo fmt --check` enforces standard Rust formatting.
`python3 scripts/validate_tech_debt_baseline.py` validates the full technical-debt baseline, including Rust check/clippy and dependency observations.
`python3 scripts/validate_tech_debt_baseline.py --drift-report reports/analysis/tech-debt-baseline-drift-report.json` also writes a machine-readable baseline drift report.
`python3 scripts/validate_planning_governance.py` validates task cards, FT mappings, roadmap counts, structural hygiene, and the lightweight technical-debt baseline gate.
`python3 scripts/validate_repository_gate.py` runs the full local repository gate used before commits.

## Coding Style & Naming Conventions
Follow Rust 2021 defaults: 4-space indentation, `snake_case` for files/functions/modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep CLI/MCP handlers thin and move reusable logic into `core`, `control`, `storage`, or focused `mcp` helpers. Prefer extending small submodules over growing catch-all files, especially around `src/mcp/` and planning-related validators.

## Testing Guidelines
Add unit tests beside the code they exercise and integration tests under `tests/integration_test/`. Use `test_*` naming for test functions. Reuse shared helpers from `tests/integration_test/common.rs` when adding CLI or daemon scenarios. When changing JSON payloads or command recommendations, assert schema fields and representative command strings, not only success paths.

## Commit & Pull Request Guidelines
Recent history uses conventional prefixes such as `feat:`, `fix:`, `docs:`, `refactor:`, and `chore:` with short imperative summaries. Keep each commit scoped to one concern. PRs should describe the operator-visible effect, list the verification commands you ran, and reference the relevant `.planning/task-cards/TASK-YYYYMMDD-<slug>.md` or `FT-*` ownership when capability boundaries or requirements change. For CLI/MCP output changes, include sample commands or JSON snippets instead of screenshots.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **opendog** (8793 symbols, 19340 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/opendog/context` | Codebase overview, check index freshness |
| `gitnexus://repo/opendog/clusters` | All functional areas |
| `gitnexus://repo/opendog/processes` | All execution flows |
| `gitnexus://repo/opendog/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
