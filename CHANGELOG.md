# Changelog

All notable changes to OPENDOG are documented here.

## 2026-05-29

### Added

- Structural hygiene validation now rejects panic-like calls (`unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`) in production Rust while ignoring test modules and test files.

### Changed

- Structural hygiene contract checks for MCP surface documentation and OpenSpec archive placeholders now live in a focused helper module, leaving the main size-budget validator with room for future guards.
- Structural hygiene tests are split between generic size-budget coverage and contract-guard coverage so both files stay comfortably below script size limits.
- Daemon runtime and shutdown-signal setup failures now flow through logged `OpenDogError` handling instead of production `expect` panics.
- MCP `ServerHandler` resource wiring now lives in `src/mcp/server_handler.rs`, reducing the root MCP module below its structural size ceiling without changing the public tool surface.
- CLI argument definitions now live in `src/cli/args.rs`, keeping the root CLI module focused on command dispatch while preserving existing Clap parser behavior.
- CLI output facade wrappers now live in `src/cli/output/facade.rs`, keeping the root output module focused on submodule wiring and shared helpers.
- Monitor unit tests now live in `src/core/monitor/tests.rs`, reducing the production monitor module well below its structural size ceiling without changing monitor behavior.
- CLI error rendering now lives in `src/cli/error_output.rs`, keeping the root CLI module below its structural size ceiling while preserving existing error-output tests.
- Project-configuration integration tests now live in `tests/integration_test/storage_project_snapshot/project_config.rs`, keeping the storage/snapshot integration module below its structural size ceiling.
- Decision-brief execution-template rendering now lives in `src/cli/output/guidance_output/decision_brief_output/execution_templates.rs`, reducing the main decision-brief output formatter without changing rendered CLI output.
- MCP tool-router wrapper methods now live in `src/mcp/server_tools.rs`, keeping the root MCP module focused on module wiring and internal exports while preserving the existing RMCP tool surface.
- MCP guidance type serialization tests now live in `src/mcp/guidance_types/tests.rs`, reducing the production guidance type module while preserving the existing payload contract coverage.
- MCP storage-maintenance tests now live in `src/mcp/storage_maintenance/tests.rs`, reducing the production storage-maintenance module while preserving cleanup-planning coverage.
- Planning-governance parsing, function-tree coverage, and roadmap count helpers now live in `scripts/planning_governance_rules.py`, keeping the orchestration script focused on repository I/O and validation reporting.
- Task-card frontmatter parsing, status counting, and rule checks now live in `scripts/task_card_rules.py`, keeping the task-card validation script focused on repository paths and CLI reporting.
- Retention cleanup all-scope activity/history tests now live in `src/core/retention/tests/activity_all.rs`, keeping activity-scope rollup tests smaller and easier to scan.
- Monitor-controller project/global configuration reload methods now live in `src/control/config_reload.rs`, keeping the root control module focused on lifecycle wiring and shared query helpers.
- Daemon-process CLI smoke coverage now delegates report/cleanup and config-reload assertions to focused integration-test helper modules, making the long end-to-end scenario easier to scan.
- Storage snapshot scan, metadata, isolation, and incremental-update integration cases now live in `tests/integration_test/storage_project_snapshot/snapshot_cases.rs`, leaving the root storage/project snapshot module focused on registry and project-lifecycle coverage.
- Project CLI output formatter tests now live in `src/cli/output/project_output/tests.rs`, keeping the formatter module focused on operator-visible project, stats, unused-file, cleanup, and list output.
- Data-risk CLI output formatter tests now live in `src/cli/output/guidance_output/data_risk_output_tests.rs`, matching the surrounding guidance-output test-module pattern.
- Decision-brief workspace, repository, verification, toolchain, and signal context rendering now lives in `src/cli/output/guidance_output/decision_brief_output/context_sections.rs`, keeping the main formatter focused on brief and entrypoint orchestration.
- Monitor runtime helpers for process locks, inotify limit checks, thread cleanup, and timestamps now live in `src/core/monitor/runtime.rs`, keeping the monitor root focused on scanner/watcher orchestration and scan-result processing.
- CLI guidance JSON integration assertions now live in focused `guidance_and_risk` and `verification` helper modules under `tests/integration_test/cli_guidance/json_outputs/`, leaving the root test focused on fixture setup and project registration.
- Usage-trend bucket helper tests now live in `src/core/report/usage_trend/tests.rs`, keeping the report module focused on trend query construction and aggregation.
- Mock/data-risk text-file and path-kind classification helpers now live in `src/mcp/mock_detection/path_classification.rs`, keeping the detector root focused on candidate extraction and scoring flow.
- MCP attention scoring and portfolio regression tests now live in `src/mcp/attention/tests.rs`, keeping the attention module focused on scoring, batching, enrichment, portfolio, and recommendation sorting logic.
- Governance lifecycle and observation-hint regression tests now live in `src/core/governance/tests.rs`, keeping the core governance module focused on lane/node state transitions and query behavior.
- MCP guidance payload aggregation regression tests now live in `src/mcp/guidance_payload/tests.rs`, keeping the payload module focused on layer assembly and execution-strategy summaries.
- MCP verification-evidence gate and workspace-summary regression tests now live in `src/mcp/verification_evidence/tests.rs`, keeping the verification-evidence root focused on payload assembly and test-only helper wiring.
- Workspace data-risk priority and aggregation regression tests now live in `src/mcp/data_risk/workspace/tests.rs`, keeping the workspace data-risk module focused on project enrichment and aggregate summary construction.
- Extended MCP, verification, governance, orphan-scan, service, and error JSON contracts now live in `docs/json-contracts-mcp-governance.md`, keeping the root JSON contract index below its structural size ceiling.
- Detailed MCP `get_guidance` request shapes, schema notes, and response-field guidance now live in `docs/mcp-tool-reference-get-guidance.md`, keeping the root MCP tool reference below its structural size ceiling while preserving canonical tool headings.

## 2026-05-28

### Added

- MCP surface documentation coverage now has regression tests that check documented tool headings against the central MCP inventory, derive read-only Resource URI coverage from handlers, and reject removed guidance tool names in current public docs.
- `fd-attribution` now lives as a main OpenSpec contract after archiving the completed `fix-fd-attribution` change.
- Structural hygiene validation now rejects archived OpenSpec Purpose placeholders in main specs.

### Changed

- Mystocks re-audit and implementation-summary review records now reflect the verified current `master` state, including remediated review notes and committed follow-up evidence.
- OpenSpec change lifecycle references now point readers to `openspec/specs/fd-attribution/spec.md` and the archived `openspec/changes/archive/2026-05-28-fix-fd-attribution` change instead of the removed active change path.

## 2026-05-27

### Added

- Retained-evidence lifecycle support now preserves activity summaries in `activity_daily_rollups` before old raw activity rows are removed.
- CLI `opendog report rollup` and MCP `get_activity_rollups` expose retained daily activity summaries after cleanup.
- MCP `get_build_info` reports binary version/build metadata, storage schema version, daemon state, `OPENDOG_HOME`, and rebuild hints.
- Storage-retention operations documentation now covers dry-run cleanup, WAL checkpointing, vacuum behavior, and retained rollup verification.
- `CONTEXT.md` and ADR records now centralize domain language, process-attribution constraints, daemon-first routing, retained-evidence lifecycle, and contract synchronization decisions.
- Storage-maintenance MCP payload assembly now starts from typed `StorageMaintenanceAssessment` and `StorageMaintenanceWorkspaceSummary` models for candidate, pressure, mode, summary, workspace totals, and priority-project ordering before rendering JSON.

### Changed

- MCP tool count is now 27, covering guidance, config/build diagnostics, project lifecycle, observation, retained rollups, verification, orphan/deletion planning, data-risk, workspace overview, and governance state.
- `get_build_info` keeps the top-level response `schema_version` as the build-info contract identifier and exposes the SQLite schema separately as `storage_schema_version`.
- Mystocks project-exchange reports, audit responses, and feature-introduction counts were synced with the implemented retained-evidence, schema-contract, and documentation-coverage changes.
- `CLAUDE.md` and historical planning research now distinguish the current 27-tool MCP surface from the original 8-tool Phase 4 baseline.
- Storage-maintenance execution-template generation now consumes typed context for project placeholders, cleanup recommendations, cleanup-plan steps, and vacuum signals before rendering MCP JSON.
- Verification-evidence workspace aggregation now uses typed project summaries and gate distributions before rendering MCP JSON.
- Single-project verification status and gate-assessment payloads now use typed summaries before rendering MCP JSON.
- Execution-strategy workspace profiles now use a typed internal model for global mode selection, tool preference, evidence priority, and recommended-flow text before rendering MCP JSON.
- Guidance data-risk focus aggregation now uses a typed distribution model before rendering workspace and execution-strategy JSON.
- Guidance repo-truth gap aggregation now uses a typed dynamic distribution model before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling now uses a typed internal model for coupled/no-signal status, source project, repository-risk finding, and summary text before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling now uses concrete optional strings for action, strategy mode, and primary-tool fields before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling now uses a typed repository-risk finding model, including lockfile-anomaly details, before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling recommended-next-action now uses a typed action enum before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling strategy-mode now uses a typed mode enum before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling preferred-primary-tool now uses a typed tool enum before rendering execution-strategy JSON.
- Guidance repo-risk strategy coupling source now uses a typed source enum before rendering execution-strategy JSON.
- Guidance execution-strategy summary counts, required-action lists, and focus distributions now use concrete internal types before rendering MCP JSON.
- Guidance execution-strategy profile fields now use concrete strings and evidence-priority lists before rendering MCP JSON.
- Guidance execution-strategy global-strategy-mode now uses a typed mode enum before rendering MCP JSON.
- Guidance execution-strategy preferred-primary-tool now uses a typed tool enum before rendering MCP JSON.
- Guidance execution-strategy preferred-secondary-tool now uses a typed tool enum before rendering MCP JSON.
- Guidance execution-strategy evidence-priority now uses a typed priority enum before rendering MCP JSON.
- Guidance execution-strategy layer status now uses a typed status enum before rendering MCP JSON.
- Guidance workspace-observation layer status now uses a typed status enum before rendering MCP JSON.
- Guidance workspace-observation analysis-state now uses a typed state enum before rendering MCP JSON.
- Guidance constraints-boundaries layer status now uses a typed status enum before rendering MCP JSON.
- Guidance multi-project-portfolio layer status now uses a typed status enum before rendering MCP JSON.
- Guidance execution-strategy recommended-flow output now uses a concrete string list inside the typed layer before rendering MCP JSON.
- Guidance execution-strategy review-focus projection now uses a typed status/source model before rendering MCP JSON.
- Guidance execution-strategy review-focus projection now uses a concrete optional source-project string before rendering MCP JSON.
- Guidance execution-strategy external-truth boundary now uses a typed status/source/checks model before rendering MCP JSON.
- Guidance execution-strategy external-truth boundary now uses a concrete optional source-project string before rendering MCP JSON.
- Guidance execution-strategy external-truth boundary mode now uses a typed enum before rendering MCP JSON.
- Decision-support action profiles now use a typed internal model for action class, phase, mutability scope, verification requirement, and primary-goal text before rendering MCP JSON.
- Decision-support risk profiles now use a typed internal model for risk-tier selection, gate fallback, blockers, repo-risk findings, and manual-review flags before rendering MCP JSON.
- Decision-support entrypoint recommendations now use a typed plan for next MCP tools, CLI commands, selection reasons, and tool-selection policy before rendering MCP JSON.
- Constraints readiness snapshots now use a typed internal model for cleanup/refactor blockers, verification gate fallback, repository-risk signals, and readiness reasons before rendering MCP JSON.
- Project-recommendation review-focus payloads now use a typed internal model for candidate family, candidate basis, and repo-risk hints before rendering MCP JSON.
- Project-recommendation forced actions now use a typed builder for failing-verification, verification-before-high-risk, and repository-stabilization recommendations before rendering MCP JSON.
- Project-recommendation evidence-collection actions now use a typed builder for start-monitor, take-snapshot, and generate-activity recommendations before rendering MCP JSON.
- Project-recommendation review actions now use a typed builder for unused-file and hot-file review recommendations before rendering MCP JSON.

### Fixed

- Build-info payload consumers no longer need to distinguish contract schema from storage schema by interpreting one overloaded field.
- Root feature tree and quick-start guidance now reflect `report rollup`, `get_activity_rollups`, `get_build_info`, `scan_orphans`, and `verify_deletion_plan`.
- CLI data-risk option normalization now reports malformed error payloads without panicking.
- Monitor config and snapshot-path lock reads now recover from poisoned locks instead of panicking in observation loops.
- Storage-maintenance candidate rules now have focused typed-model tests in addition to JSON contract coverage.

## 2026-05-25

### Added

- Governance state observation feature (GOV-01..08, FT-03.09): 4 MCP tools (`create_governance_lane`, `upsert_governance_node`, `get_governance_state`, `close_governance_lane`) and 4 matching CLI commands (`opendog governance create-lane|upsert-node|show|close-lane`).
- Governance observation hints: `get_governance_state` cross-references snapshot freshness, verification status, unused file count, and data-risk candidate count from cache.
- `data_risk_cache` table (schema v6): persistent single-row cache for mock/hardcoded candidate counts, populated by `get_data_risk_candidates` and `get_workspace_data_risk_overview`.
- Daemon control plane support for governance operations: all 4 governance MCP handlers use daemon-first pattern via Unix socket IPC, falling back to direct DB when daemon is unavailable.
- Daemon control plane support for orphan operations: `scan_orphans` and `verify_deletion_plan` MCP handlers now use daemon-first pattern.
- Control plane roundtrip tests for governance operations (create lane → upsert node → get state → close lane).
- Governance design spec and implementation plan.

### Changed

- All MCP tool handlers now consistently use daemon-first pattern. When the daemon is running, requests route through Unix socket IPC; when the daemon is unavailable, handlers fall back to direct DB access.
- FT leaf count: 22 → 27 (added FT-03.09 Governance State Observation).
- MCP tool count: 20 → 26 (added 4 governance + 2 orphan tools).
- CLI command count: 22 → 23 (added `opendog governance` subcommand).
- Schema version: v5 → v6 (added `data_risk_cache` table).
- Test count: 298 → 300 → 308 (added control plane roundtrip test, data-risk cache test, orphan roundtrip test, governance payload contract tests, governance tool surface test, snapshot underflow regression test).
- `GovernanceState`, `UpsertNodeResult`, `ObservationHints`, `GovernanceLaneSummary` now implement `Deserialize` for control plane protocol serialization.
- README updated with accurate counts, daemon-first architecture note, governance and orphan tool tables, and governance CLI commands.
- AI playbook now covers governance CLI workflow entry points, JSON usage entries, and decision-critical field guides for governance and orphan tools.
- MCP tool reference updated with orphan scan and deletion plan documentation, governance cluster map, trimmed to fit 1000-line budget.
- JSON contracts updated with orphan scan and deletion plan contract sections.
- Extracted `GovernanceCommand` enum from `cli/mod.rs` to `governance_commands.rs` (482 lines, under 500-line budget).
- Fixed ROADMAP requirement count (114→122) and added missing `roadmap_phases` to FT-03.09.01.
- All 4 validation scripts pass: governance, structural hygiene, task cards, requirement mappings.

### Fixed

- Governance nodes query `get_governance_nodes` now correctly increments parameter index after the `node_id` WHERE branch (dormant bug — harmless now, would break with additional filters).
- Added composite index `governance_nodes(lane_id, updated_at DESC)` covering the primary governance state query pattern.
- Snapshot `new_files` calculation now uses `saturating_sub` for the inner subtraction to prevent panic/wrap when `removed > previous_count`.
- Project config JSON parse failures in `get_project`/`list_projects` now log a warning instead of silently discarding malformed config.
- Monitor lock acquisition uses atomic `create_new` to eliminate TOCTOU race between two concurrent `start_monitor` calls.
- Verification output tail truncation no longer allocates the full string before trimming — streams only the needed tail portion.

## 2026-05-11

### Added

- Synced mystocks 2026-05-11 source-signal calibration results back into OpenDog project-exchange evidence.
- Added `TASK-20260511-source-first-observation-views` as the governed follow-up for source-first stats/unused views and transient-read guidance boundaries.
- Added an approval-ready implementation plan for source-first MCP/CLI observation filters without scanner changes.
- Revised the source-first observation implementation plan after review to cover empty filtered results, filter-aware guidance, conditional payload fields, and replacement-only boundary wording.
- Added `path_classification` filters for MCP and CLI stats/unused views with `all`, `source`, `infrastructure`, `backup`, and `project` modes.
- Added mystocks retest handoff for source-first observation filters and rebuilt the release MCP binary for validation.
- QUICKSTART now documents that MCP hosts use the configured binary path and require `cargo build --release` plus host reconnect to pick up OpenDog code updates.
- QUICKSTART now clarifies that OpenDog update commands are WSL/Linux shell maintenance operations run against the OpenDog source tree, not automatic MCP actions or business-project commands.
- Added governance task card and implementation plan for a CLI-only manual self-update workflow.
- Added CLI-only `opendog self-update status/build --source <opendog-source>` for explicit local release-binary maintenance.
- README, capability index, requirements, roadmap, and governance docs now consistently reference the root `FUNCTION_TREE.md`, `register_project`, 22 CLI top-level commands, and the CLI-only `self-update` maintenance entrypoint.

### Changed

- `ODX-20260511-source-signal-observation-calibration` is now classified as expected fd-sampling behavior plus filtering/presentation debt, not a scanner-attribution regression.
- Field notes now record the 2026-05-11 mystocks calibration decision and route follow-up work to source-first views rather than scanner changes.
- Stats/unused guidance now clarifies transient-read blind spots and `access_count=0` open-descriptor boundaries while keeping infrastructure evidence visible on request.
- `ODX-20260511-source-signal-observation-calibration` is now fixed as a product issue by source-first filters and guidance boundaries; transient Claude Code reads remain an explicit sampling limitation rather than a scanner-attribution regression.

## 2026-05-10

### Added

- Central project-exchange directory under `docs/project-exchange/` for OpenDog usage reports, tuning feedback, and cross-project communication.
- Reusable OpenDog usage feedback template at `docs/project-exchange/templates/OPENDOG_USAGE_FEEDBACK_TEMPLATE.md`.
- Shared project-exchange issue index for cross-project OpenDog feedback and resolution tracking.
- Shared issue entries for mystocks MCP regression results: fixed A/G cases and accepted H/I follow-ups.
- Imported project reports for `mystocks` and `quantix-rust` so OpenDog operating evidence is collected in this repository.
- Added a mystocks-specific 2026-05-11 MCP retest handoff for Case H and Case I validation.
- Added a mystocks-specific 2026-05-11 OpenDog change summary for retest handoff context.
- Imported mystocks 2026-05-11 MCP retest results confirming Case H and Case I are fixed.
- Opened `ODX-20260511-source-signal-observation-calibration` and a proposed task card for the remaining mystocks `.claude/` dominance / source-signal visibility issue.
- Added a mystocks source-signal calibration sampling plan for distinguishing scanner, filtering, workflow, and expected-tool-behavior causes.
- Copied the mystocks source-signal calibration plan into `mystocks_spec` for direct execution from the target project.
- QUICKSTART now indexes the mystocks retest handoff alongside archived project reports.
- mystocks feedback now references the 2026-05-11 retest handoff as the execution checklist for Case H and Case I.
- mystocks feedback now also references the 2026-05-11 OpenDog change summary before retest.
- `.gitignore` now excludes local `.claude/` integration files to avoid accidental product commits.
- Governed follow-up task cards for data-risk false-positive reduction, MCP regression coverage, verification evidence TTL policy, and read-only MCP Resources.
- OpenSpec governance artifacts for `fix-fd-attribution`, including acceptance evidence, review plan, and future scanner attribution change gate.
- Data-risk rule metadata for documentation paths and template-placeholder content.
- Source/infrastructure/backup file classification plumbing for stats, unused-file review, CLI output, MCP payloads, and guidance.
- Regression coverage for daemon-backed explicit snapshot comparison and verification command execution paths used by MCP when the daemon is live.
- Machine-readable verification freshness TTL policy fields in verification status and gate assessment payloads.
- Read-only MCP Resources for stable project-list and per-project verification state reads.
- Dedicated governance task cards for UTF-8 guidance panic handling, daemon IPC response integrity, MCP observation payload bounds, infrastructure file classification, data-risk noise reduction, regression coverage, TTL transparency, and read-only MCP Resources.

### Changed

- `/proc/<pid>/fd` attribution now distinguishes file descriptors from directory descriptors and deduplicates `(pid, fd)` sightings within one scan cycle, preventing directory-fd fan-out from inflating per-file access counts.
- Project feedback and report templates are now managed from the OpenDog repository instead of being scattered across target project directories.
- Project-exchange reports now distinguish one-to-one project response routing from cross-project shared issue visibility.
- Structural hygiene now treats project-exchange reports as evidence artifacts with a larger size budget than reference docs.
- Structural hygiene now treats `QUICKSTART.md` as the canonical detailed usage guide with a 1000-line budget, while `README.md`, `CLAUDE.md`, and `REVIEW.md` have a 600-line root-doc budget.
- Structural hygiene now gives `docs/mcp-tool-reference.md` a dedicated MCP contract-reference budget instead of forcing it under the default docs-reference byte cap.
- Structural hygiene now gives `docs/json-contracts.md` a dedicated machine-contract budget.
- Governance docs now state the root-document and QUICKSTART line-budget policy explicitly.
- Root `CHANGELOG.md` updates are now required for substantial project changes before closure.
- Data-risk hardcoded candidates in documentation or template-heavy files are now down-ranked instead of being treated like runtime-shared source literals.
- MCP `get_stats` and `get_unused_files` now default to bounded file rows while preserving full project counts and result-window metadata.
- QUICKSTART now documents the required large-payload MCP retest contract for default and explicit `limit` calls.
- QUICKSTART now documents MCP resource-discovery retest evidence after OpenDog rebuilds.
- QUICKSTART project-exchange guidance is now expanded into readable report-routing, hygiene, and retest checklists.
- MCP tool reference now includes resource-discovery troubleshooting for host reconnect and raw capability checks.
- Capability index now lists read-only MCP Resources as the preferred no-operation read path for project-list and verification state.
- JSON contracts now document bounded MCP stats/unused result windows and read-only MCP Resource consumption.
- Daemon empty or truncated socket responses now surface as transport-integrity errors with remediation guidance.
- Agent guidance and mock-data preview generation now tolerate non-ASCII/UTF-8 content without panicking.
- Verification freshness now exposes the default policy thresholds: fresh within 24h, aging through 7d, stale after 7d.
- MCP server capabilities now advertise read-only resources in addition to tools; mutations remain on tools or CLI.
- Rebuilt and protocol-verified the release MCP binary for read-only resource capability discovery.
- `README.md`, `QUICKSTART.md`, `FUNCTION_TREE.md`, and MCP documentation now point readers to the current governance, MCP, and cross-project exchange surfaces.

## 2026-05-02

### Added

- Verification-driven soft gates for cleanup and refactor decisions, including machine-readable `gate_assessment.cleanup` and `gate_assessment.refactor` outputs.
- Repository-truth boundary projection fields such as `repo_truth_gaps` and `mandatory_shell_checks` in project recommendations, decision briefs, and guidance summaries.
- Machine-readable `execution_sequence` payloads for repository stabilization, verification-first, and observation-first workflows, including resume conditions and suggested follow-up commands.

### Changed

- Project action selection now uses shared priority gating, scoring, and stable reasoning while preserving the existing `recommended_next_action` enum.
- Guidance and decision payloads now summarize which projects require repository stabilization, fresh verification runs, failing-verification repair, monitor start, snapshot refresh, or activity generation before broader cleanup or refactor review.
- MCP contract docs now describe the new sequencing and boundary fields so AI consumers can follow the same recommendation chain consistently.

### Scope

- This hardening line completed the current selective-deepening slice across `FT-03.01.01`, `FT-03.02.02`, `FT-03.03.01`, `FT-03.06.01`, and `FT-03.07.01`.
