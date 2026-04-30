# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` is the binary entrypoint and delegates to `opendog::cli::run()`. Core product logic lives in `src/core/` (`snapshot`, `monitor`, `report`, `retention`, `verification`), persistence lives in `src/storage/`, and daemon/local-control-plane code lives in `src/control/`. MCP tool surfaces and payload builders are under `src/mcp/`; CLI handlers are under `src/cli/`; config loading and validation are under `src/config/`. Integration tests live in `tests/integration_test/`, while module-scoped tests stay near the code in `src/**/tests.rs`. Planning and scope-governance artifacts live in `.planning/`; longer operator docs live in `docs/`; deployment assets live in `deploy/`.

## Build, Test, and Development Commands
`cargo build --release` builds `target/release/opendog`.
`cargo run -- mcp` starts the MCP stdio server; `cargo run -- daemon` starts the background coordinator.
`cargo test` runs the full suite; `cargo test test_snapshot` narrows to snapshot coverage; `RUST_LOG=debug cargo test -- --nocapture` helps debug failures.
`cargo clippy --all-targets --all-features -- -D warnings` is the lint gate used throughout the repo.
`cargo fmt --check` enforces standard Rust formatting.
`python3 scripts/validate_planning_governance.py` validates task cards, FT mappings, roadmap counts, and structural hygiene.

## Coding Style & Naming Conventions
Follow Rust 2021 defaults: 4-space indentation, `snake_case` for files/functions/modules, `UpperCamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep CLI/MCP handlers thin and move reusable logic into `core`, `control`, `storage`, or focused `mcp` helpers. Prefer extending small submodules over growing catch-all files, especially around `src/mcp/` and planning-related validators.

## Testing Guidelines
Add unit tests beside the code they exercise and integration tests under `tests/integration_test/`. Use `test_*` naming for test functions. Reuse shared helpers from `tests/integration_test/common.rs` when adding CLI or daemon scenarios. When changing JSON payloads or command recommendations, assert schema fields and representative command strings, not only success paths.

## Commit & Pull Request Guidelines
Recent history uses conventional prefixes such as `feat:`, `fix:`, `docs:`, `refactor:`, and `chore:` with short imperative summaries. Keep each commit scoped to one concern. PRs should describe the operator-visible effect, list the verification commands you ran, and reference the relevant `.planning/task-cards/TASK-YYYYMMDD-<slug>.md` or `FT-*` ownership when capability boundaries or requirements change. For CLI/MCP output changes, include sample commands or JSON snippets instead of screenshots.
