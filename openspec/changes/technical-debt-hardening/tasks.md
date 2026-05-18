## 1. Storage migrations

- [ ] 1.1 Add a migration runner that reads and writes `PRAGMA user_version` for registry and project databases.
- [ ] 1.2 Wire migration execution into `Database::open_registry` and `Database::open_project` before handles are returned.
- [ ] 1.3 Add fixture-based regression tests that open an older schema database and verify it migrates to the current `SCHEMA_VERSION`.

## 2. Runtime error boundaries

- [ ] 2.1 Convert MCP startup into a fallible `try_run_stdio()` boundary and keep fatal exit handling at the process edge.
- [ ] 2.2 Replace production `Mutex::lock().unwrap()` call sites in MCP/control code with structured error handling.
- [ ] 2.3 Replace the verification post-insert `expect` with a domain error path that callers can handle.
- [ ] 2.4 Replace production MCP payload/decision serialization `expect` calls with fallible conversion helpers or documented boundary errors.
- [ ] 2.5 Add regression coverage for poison/startup/error-path behavior where the runtime should fail cleanly instead of panicking.

## 3. Dependency hygiene

- [ ] 3.1 Remove or align the direct `schemars = "0.8"` dependency so the project uses one schema-derive path.
- [ ] 3.2 Verify the cleaned dependency tree with `cargo tree -d` and ensure schema derives still compile.

## 4. MCP tool inventory

- [ ] 4.1 Define a single inventory structure for MCP tool name, contract ID, params type, payload builder, handler module, and test owner.
- [ ] 4.2 Validate runtime MCP tool registration against the inventory so missing registrations fail fast in tests.
- [ ] 4.3 Update the relevant payload contract and tool-surface tests to consume the inventory as the source of truth.

## 5. Orphan module boundary

- [ ] 5.1 Split `src/core/orphan.rs` into focused submodules for types, classification, scanner contract, built-in scanners, and deletion plan logic.
- [ ] 5.2 Preserve the current public API through re-exports so MCP callers do not need import changes.
- [ ] 5.3 Keep existing orphan unit tests, MCP payload contract tests, and the MCP session regression passing without behavior changes.

## 6. MCP test fixtures

- [ ] 6.1 Extract reusable domain fixture builders for the largest MCP guidance and payload contract test clusters.
- [ ] 6.2 Refactor the repeated inline setup in those clusters to use the new fixture builders while keeping assertions explicit.
- [ ] 6.3 Verify that the fixture extraction does not weaken schema-field, command-string, or failure-state assertions.

## 7. Verification and rollout

- [ ] 7.1 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` after the code changes land.
- [ ] 7.2 Run `python3 scripts/validate_planning_governance.py` and record any rollback or migration notes required by the release.

