# OpenDog Field Notes — mystocks_spec

> Accumulated usage observations and optimization suggestions for the OpenDog project.
> Collected during real development sessions on the mystocks_spec repository.
> Submit as a batch to the OpenDog project when sufficient evidence accumulates.

---

## Session Log

| Date | Session Focus | Observer |
|------|--------------|----------|
| 2026-05-08 | MCP configuration, review workflow, initial monitoring | Claude (GLM-5.1) |
| 2026-05-11 | H/I retest and source-signal calibration | Claude (GLM-5.1) |

---

## MCP Integration Issues

### MCP-001: get_stats / get_unused_files connection drops after start_monitor

- **Date**: 2026-05-08
- **Context**: Called `start_monitor`, `get_stats`, `get_unused_files` in parallel. `start_monitor` succeeded but the other two returned `MCP error -32000: Connection closed`.
- **Reproduction**: Launch all three in same turn; the two read calls fail.
- **Workaround**: Use CLI (`opendog stats --id mystocks`) instead of MCP for read operations. CLI is stable.
- **Hypothesis**: The MCP stdio connection may be disrupted when the daemon starts or reconnects mid-session. Parallel MCP calls to the same server may also cause a race condition on the stdio pipe.
- **Severity**: Medium — MCP read tools are unusable after monitor start within the same session.

### MCP-002: start_monitor returns already_running=true but project had file_stats=0

- **Date**: 2026-05-08
- **Context**: First call to `start_monitor` returned `already_running: true` and `status: monitoring`. However, querying the SQLite DB directly showed `file_stats = 0`, `file_events = 0`, `file_sightings = 0`.
- **Observation**: The `already_running` field appears to reflect daemon-level state (daemon is alive) rather than project-level monitoring state (actually collecting inotify events for this project path). This is misleading.
- **Suggestion**: Distinguish between "daemon process is running" and "inotify watches are active for project path". Return different fields or status values.

### MCP-003: MCP tool surface not listed in Claude Code session after config

- **Date**: 2026-05-08
- **Context**: Added `mcpServers.opendog` to `.claude/settings.local.json`. The MCP tools appeared in the session where the config was written, but on session restart the tool list did not include `mcp__opendog__*` tools.
- **Workaround**: CLI usage works reliably regardless of MCP session state.
- **Suggestion**: Document the expected MCP lifecycle — does `opendog mcp` need the daemon pre-started, or does it auto-start? The quickstart says "auto-ensures daemon-backed monitoring reuse" but the behavior was inconsistent.

---

## Observation Quality Issues

### OBS-004: Claude Code source reads are transient and invisible to fd sampling

- **Date**: 2026-05-11
- **Context**: mystocks ran a source-heavy calibration window against `.py` and `.vue` files after Case H and Case I were fixed.
- **Evidence**: Touched Python source files remained at `access_count=0`; `web/frontend/src/App.vue` retained `access_count=1` but changed `modification_count` from 0 to 2 after edit/revert activity. Top `.claude/` infrastructure files continued to share near-identical high access counts.
- **Interpretation**: Claude Code Read operations close source file descriptors too quickly for `/proc/<pid>/fd` sampling to observe. Edit activity is visible through modification_count, but transient reads are not reliable access evidence.
- **Decision**: Do not reopen scanner attribution or `fix-fd-attribution` from this evidence. Treat the next improvement as source-first views, classification filters, and guidance boundary wording.
- **Resolution**: OpenDog shipped source-first `path_classification` filters for stats/unused, classification summaries, and guidance boundary wording. The shared issue is fixed as a product-presentation/guidance issue; the sampling limitation remains documented.
- **Tracking**: `ODX-20260511-source-signal-observation-calibration`; follow-up task `TASK-20260511-source-first-observation-views`.

### OBS-001: Hot-file results dominated by .claude/ config noise

- **Date**: 2026-05-08
- **Context**: After monitoring for ~30 minutes, `opendog stats` showed 31 accessed files. All 31 were `.claude/*.json`, `.claude/*.jsonc`, `.claude/*.md` files — zero source code files appeared.
- **Root Cause**: Claude Code reads `.claude/settings.json`, `.claude/CLAUDE.md`, and related config files on every tool call. These accumulate access counts far faster than any source file.
- **Impact**: Hot-file analysis is currently useless for identifying business-code hotspots. The signal is drowned by config-file noise.
- **Suggested Fix**: Add default ignore patterns for `.claude/`, `.omc/`, `.git/` internal directories (not just `.git`). These are tool-infrastructure files, not project source. The current default ignores (`node_modules`, `__pycache__`, etc.) don't cover AI-tool config directories.
- **Alternative**: Allow users to mark directories as "infrastructure" vs "source" so stats can filter by category.

### OBS-002: Backup files pollute stats

- **Date**: 2026-05-08
- **Context**: Files like `.claude/omc.jsonc.bak.20260311-120640`, `.claude/settings.local.json.backup-20260222_203403` appear as separate entries in stats with identical access counts to their active counterparts.
- **Suggested Fix**: Default ignore pattern for `*.bak.*`, `*.backup*`, `*.backup-*` would clean up the stats significantly.

### OBS-003: All .claude/ files show identical access counts and durations

- **Date**: 2026-05-08
- **Context**: Every `.claude/` file showed exactly 446 accesses and 1,349,000ms duration. This is suspicious — it suggests the monitoring is attributing a batch read (Claude Code loading all config files at session start) equally to every file, rather than tracking individual file-level access patterns.
- **Suggestion**: Investigate whether `/proc/<pid>/fd/` scanning is conflating directory-level access with file-level access. If the process has the `.claude/` directory open, all files in it may get credited.

---

## Feature Requests

### FEAT-001: Category-based filtering in get_stats

- **Date**: 2026-05-08
- **Description**: Allow `get_stats` to filter by file category (source, config, test, docs, infrastructure). This would let users ask "show me hot source files only" without manually filtering out `.claude/`, `config/`, `docs/` directories.
- **Use Case**: During code review sessions, I want to know which `.py` and `.vue` files are hot, not which `.json` config files are hot.

### FEAT-002: Relative time in stats output

- **Date**: 2026-05-08
- **Description**: Stats output shows `LAST ACCESS` as a Unix timestamp (e.g., `1778235340`). Converting this requires mental math or an external tool.
- **Suggestion**: Add a human-readable relative time column (e.g., "2 min ago", "1 hour ago") alongside the raw timestamp.

### FEAT-003: Snapshot diff summary

- **Date**: 2026-05-08
- **Description**: After running a second snapshot, it would be useful to see a summary like "12 files added, 3 files removed, 8 files modified since last snapshot" without having to call `compare_snapshots` and parse the full diff.

---

## CLI Observations

### CLI-001: CLI is more reliable than MCP for read operations

- **Date**: 2026-05-08
- **Context**: Throughout the session, CLI commands (`opendog stats`, `opendog unused`, `opendog list`) worked 100% of the time. MCP tools (`get_stats`, `get_unused_files`) failed with connection errors.
- **Recommendation**: For the quickstart guide, consider recommending CLI as the primary read interface and MCP as the primary write/control interface until the MCP connection stability issue is resolved.

### CLI-002: Stats table truncates long paths

- **Date**: 2026-05-08
- **Context**: File paths longer than ~45 characters are truncated with `...` prefix, making it hard to identify the actual file.
- **Suggestion**: Add a `--full-paths` or `--wide` flag to disable truncation, or output as JSON for programmatic consumption.

---

## Configuration Suggestions

### CFG-001: Default ignore patterns should include AI-tool directories

Current defaults are solid for traditional development:
```
node_modules, .git, dist, target, __pycache__, .cache, build, .next, .nuxt, vendor, .venv, venv, .tox, .mypy_cache, .pytest_cache, .gradle, .idea, .vscode, *.pyc, .DS_Store
```

Recommend adding for AI-assisted development:
```
.claude, .omc, .amazonq, .cursor, .agents, .zread, *.bak.*, *.backup*
```

### CFG-002: Process whitelist should document why each entry exists

Current whitelist: `claude, codex, node, python, python3, gpt, glm`

The `glm` entry was unexpected — it suggests the whitelist is tracking GLM API client processes. Documenting the purpose of each entry would help users decide whether to add entries like `cursor`, `windsurf`, or `aider`.

---

## Summary Statistics (as of 2026-05-08)

| Metric | Value |
|--------|-------|
| Snapshot files | 50,087 |
| file_stats records | 62 |
| Accessed files | 31 |
| Unused files | 50,056 |
| Source-code hot files | 0 |
| Config-file hot files | 31 |
| Monitoring duration | ~30 minutes |

---

## Document Review: OPENDOG_USAGE_FEEDBACK.md

> Review date: 2026-05-08
> Reviewer: Claude (GLM-5.1)
> Document reviewed: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md`
> Index reviewed: `/opt/claude/mystocks_spec/docs/operations/monitoring/INDEX.md`

### Overall Assessment

Well-structured feedback framework. The two-layer design (daily experience vs. tuning evidence) is the right split — it keeps the feedback loop low-friction for daily use while preserving engineering-grade evidence for OpenDog maintainers. The initial 2026-05-08 baseline entry accurately captures the bootstrap session's findings and matches my FIELD_NOTES observations.

### What Works Well

1. **Record rules are precise**. Requiring date, observer, entry type, project state, exact commands, expected vs actual behavior — this is exactly what an engineering feedback channel needs. The "do not paste secrets" reminder is practical.

2. **Case A-E taxonomy is actionable**. The five case types map cleanly to real problems I encountered:
   - Case A (large payload) — matches my MCP-001 finding
   - Case B (infrastructure noise) — matches my OBS-001, OBS-002
   - Case C (AI config dominance) — matches my OBS-003
   - Case D (attribution accuracy) — I haven't verified this yet but the identical-count anomaly in OBS-003 is relevant
   - Case E (guidance value) — the guidance responses I received were verbose but low-signal; worth tracking

3. **Interpretation boundary section** is important. Stating explicitly that `unused != safe to delete` prevents the most dangerous misreading of OpenDog's output. Good defensive documentation.

4. **Evidence command list** is practical. Having a stable set of commands to run when investigating makes it easy for multiple observers to produce comparable evidence.

### Suggestions for Improvement

#### SUG-001: Add a "payload size" evidence field to Case A

The Case A template asks whether CLI succeeds when MCP fails, but doesn't ask for the approximate response payload size. This is the single most useful diagnostic for determining if the issue is a transport limit (stdio pipe buffer, JSON serialization overhead) vs a logic error. Recommend adding:

```
- approximate response size:
  - CLI output: ~N KB / ~N lines
  - MCP JSON payload (if available): ~N KB
```

#### SUG-002: Clarify the "7 priority questions" scope overlap

Questions 2 and 3 in the "current most worth validating" list overlap significantly:
- Q2: unused over-reports `.claude/`, `.amazonq/` infrastructure files
- Q3: hot results dominated by AI config files

Both point to the same root cause: OpenDog's current classification treats all files equally, with no concept of "infrastructure" vs "source". Recommend merging into one question: "Does OpenDog need a file-classification layer that separates infrastructure (tool configs, hooks, agent prompts) from source code?"

#### SUG-003: Add a Case F for cross-session state persistence

Neither the case taxonomy nor the priority questions address the cross-session MCP reliability issue I encountered (MCP-003). The daemon persists across sessions, but the MCP stdio connection does not. This is a distinct problem from Case A (large payload) because it occurs even for small requests. Recommend adding:

```
#### Case F - MCP session lifecycle

- Trigger:
  - MCP tools available in one session but missing after restart
  - `opendog mcp` auto-starts daemon but tools still don't appear
- Why important:
  - If MCP is unreliable as a primary interface, users will default to CLI,
    which undermines the value of MCP integration
```

#### SUG-004: Monthly summary template should include "evidence volume"

The monthly summary template is missing a quantitative assessment of how much evidence was collected. Adding:

```
- Evidence volume:
  - Total tuning cases recorded:
  - Cases confirmed as reproducible:
  - Cases that self-resolved:
```

This helps OpenDog maintainers assess whether the feedback channel is producing enough signal to justify the documentation overhead.

#### SUG-005: INDEX.md should note the feedback document's role

The INDEX.md lists OPENDOG_USAGE_FEEDBACK.md as *"MyStocks 使用 OpenDog 的长期运行经验与调优证据主文档"*. This is accurate but doesn't communicate that this file is also the primary interface between the mystocks team and the OpenDog project. Consider adding a note like:

```
> This document serves as the feedback channel to the OpenDog project.
> OpenDog maintainers should read the "第二层" section for actionable tuning evidence.
```

### Alignment with FIELD_NOTES

My FIELD_NOTES entries map to the feedback document's cases as follows:

| FIELD_NOTES ID | Feedback Case | Status |
|----------------|---------------|--------|
| MCP-001 | Case A | Evidence collected, needs payload-size measurement |
| MCP-002 | — | No matching case; relates to state semantics, suggest Case F or new |
| MCP-003 | Case F (proposed) | Evidence collected |
| OBS-001 | Case B + C | Strong evidence, matches initial baseline |
| OBS-002 | Case B | Evidence collected |
| OBS-003 | Case C + D | Attribution anomaly — same access count across all files suggests `/proc/fd/` scanning over-attribution |
| FEAT-001 | — | Not covered in feedback doc; maps to Q6 (filtering) |
| FEAT-002 | — | UX polish, not covered |
| CFG-001 | Case B | Root cause fix — default ignore patterns |

### Priority Recommendation for Next Session

From my perspective as the primary AI observer, the highest-value next actions for the mystocks team are:

1. **Run a source-code-heavy session** (edit 5-10 `.py` or `.vue` files) and then check whether those files appear in `opendog stats`. This directly tests Case D (attribution accuracy) and Case C (source vs config dominance).

2. **Measure payload size** for `get_stats` MCP failure. Run `opendog stats --id mystocks` via CLI, capture output size, and compare to a known working MCP call. This tests Case A's hypothesis.

3. **Test OBS-003 hypothesis**: Check whether files in a directory that Claude Code has open via `/proc/<pid>/fd/` all get the same access count. If confirmed, this is a significant attribution accuracy bug.

---

## Follow-up: OPENDOG feedback doc updates

> Review date: 2026-05-08
> Scope: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md`, `INDEX.md`

### Current status

- Case F was added for MCP session lifecycle and state semantics.
- The INDEX now clearly points maintainers to the feedback document as the primary OpenDog tuning entry.
- The external docs now better separate daily usage notes from tuning evidence.

### Still worth tracking

- The main feedback doc still does not spell out the payload-size evidence field directly in the Case A template.
- The monthly summary template still lacks an explicit evidence-volume field.
- The Q2/Q3 overlap remains a useful simplification candidate, even though the core noise issue is now well described.

---

## Follow-up: fd attribution fix validation

> Review date: 2026-05-08
> Change: `openspec/changes/fix-fd-attribution` at implementation time; archived at `openspec/changes/archive/2026-05-28-fix-fd-attribution`, with the current contract in `openspec/specs/fd-attribution/spec.md`.

### Implementation outcome

- Added scanner-side directory-fd exclusion: only canonicalized regular files inside the snapshot set become per-file sightings.
- Added scan-cycle fd deduplication keyed by process id and fd number.
- Added unit tests for directory target exclusion and duplicate fd suppression.

### Large-repo validation

- Validation home: `/tmp/opendog-fd-test`
- Project: `mystocks-fd`
- Root: `/opt/claude/mystocks_spec`
- Snapshot size: 60638 files
- Workload: one Python process opened a repository directory fd plus `web/frontend_status.py` and `web/frontend/src/App.vue`; the `.vue` fd was closed earlier than the `.py` fd.
- Result:
  - `web/frontend_status.py`: 4 accesses, 9000ms
  - `web/frontend/src/App.vue`: 1 access, 0ms
- Interpretation: the directory fd did not force the two source files into the same access_count bucket, and the source files retained independent counts under a large-repo workload.

### Residual noise

- `.claude/` files still dominate the top stats because Claude Code opens them as actual file fds. That is a classification/default-ignore problem, not the directory-fd fan-out bug.

### Downstream consumer verification

- `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog stats --id mystocks-fd` exited 0 and still showed `web/frontend_status.py` and `web/frontend/src/App.vue` with distinct counts.
- `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog unused --id mystocks-fd` exited 0 and preserved the existing unused-file output contract.
- `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog report window --id mystocks-fd --window 24h --json` exited 0 and returned schema-shaped JSON with `schema_version`, `summary`, `guidance`, and `window` fields.
- `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog agent-guidance --project mystocks-fd --json` panicked in `src/mcp/mock_detection.rs:153` on a UTF-8 byte boundary bug while parsing local markdown content. This appears separate from fd attribution and should be tracked independently.

### Follow-up: agent-guidance UTF-8 panic closure

- Governance task: `.planning/task-cards/TASK-20260509-agent-guidance-utf8-panic.md`
- Fix: make mock-detection content previews slice on valid UTF-8 character boundaries.
- Regression test: `detect_mock_data_report_handles_non_ascii_preview_boundaries`
- Result: `env OPENDOG_HOME=/tmp/opendog-fd-test target/debug/opendog agent-guidance --project mystocks-fd --json` exits 0 and returns guidance JSON instead of panicking.

---

## Follow-up: mystocks MCP release validation feedback

> Review date: 2026-05-10
> Source: `/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md`

### New evidence received

- `mystocks` release rebuild validation used `/opt/claude/opendog/target/release/opendog` built on 2026-05-09.
- MCP `list_projects` succeeded with embedded guidance.
- MCP `get_stats {id: "mystocks"}` still closed the connection on a 50,087-file project.
- MCP `get_unused_files {id: "mystocks"}` produced about 5.9M characters, confirming the output shape is too large for routine AI consumption.
- MCP `get_guidance {project_id: "mystocks", detail: "decision", top: 3}` returned `serialization_error` while CLI `decision-brief --json` produced valid JSON around 101KB.
- CLI `stats`, `unused`, `decision-brief`, and `agent-guidance` all succeeded, so the highest-priority remaining defects are MCP payload/transport reliability rather than core stats or guidance business logic.

### Governance tasks opened

- `.planning/task-cards/TASK-20260510-mcp-observation-payload-bounds.md`
  - Scope: bounded `get_stats` / `get_unused_files` MCP responses for large repositories.
- `.planning/task-cards/TASK-20260510-daemon-ipc-response-integrity.md`
  - Scope: daemon socket empty/truncated response detection and decision guidance IPC reliability.
- `.planning/task-cards/TASK-20260510-infrastructure-file-classification.md`
  - Scope: source vs infrastructure separation for stats, unused, and guidance readability.

### Status separation

- The fd attribution fan-out bug remains closed under `fix-fd-attribution`.
- The UTF-8 preview panic remains closed under `TASK-20260509-agent-guidance-utf8-panic`.
- The new open issues are MCP payload shaping, daemon IPC integrity, and infrastructure-file classification.

### Follow-up: MCP observation payload bounds closure

- Governance task: `.planning/task-cards/TASK-20260510-mcp-observation-payload-bounds.md`
- Fix: `get_stats` and `get_unused_files` MCP payloads now return at most 50 file rows by default.
- Caller override: both tools accept optional `limit`.
- Contract metadata: responses include `result_window.total_count`, `returned_count`, `limit`, and `truncated`.
- Release binary: `cargo build --release` completed after the bounded payload change.
- Residual scope: daemon IPC response integrity and infrastructure classification remain separate proposed tasks.

### Follow-up: daemon IPC response integrity closure

- Governance task: `.planning/task-cards/TASK-20260510-daemon-ipc-response-integrity.md`
- Fix: empty daemon responses and EOF/truncated daemon JSON now return `DaemonResponseIntegrity` instead of generic `Serialization`.
- MCP error code: `daemon_response_integrity_error`
- Contract metadata: error responses include remediation actions for retrying, comparing with CLI, and restarting daemon if repeated.
- Validation: `env OPENDOG_HOME=/root/.opendog target/debug/opendog decision-brief --project mystocks --top 3 --json` exits 0 against the existing `mystocks` daemon state.
- Release binary: `cargo build --release` completed after the daemon IPC response integrity change.
- Residual scope: infrastructure file classification remains a separate proposed task.

### Follow-up: infrastructure file classification closure

- Governance task: `.planning/task-cards/TASK-20260510-infrastructure-file-classification.md`
- Fix: stats and unused payloads now expose soft `path_classification` values and full-result `classification_summary` counts for source, infrastructure, backup, and project files.
- Guidance behavior: unused-file recommendations prefer source-classified candidates before AI/tool infrastructure noise while preserving infrastructure evidence in the output.
- CLI behavior: stats and unused tables display classification so operators can distinguish source signal from `.claude/`, `.amazonq/`, `.cursor/`, `.agents/`, `.zread/`, and backup-file patterns.
- Boundary: this does not change snapshot ignore patterns or scanner attribution semantics.

### Follow-up: mystocks retest completion

- Source: `/opt/claude/mystocks_spec/docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`
- Imported canonical copy: `docs/project-exchange/reports/mystocks/opendog-mcp-retest-results-2026-05-11.md`
- Case H result: PASS. mystocks confirmed default and explicit `limit:50` calls for `get_stats` and `get_unused_files` return bounded 50-row payloads with correct `result_window`; no MB-scale output and no connection-close error.
- Case I result: PASS. mystocks confirmed Claude Code MCP can discover `opendog://projects` and read both project-list and per-project verification resources.
- Shared issue updates: `ODX-20260510-mcp-large-payload-pagination` and `ODX-20260510-mcp-resources-not-discovered` are now fixed in `docs/project-exchange/issues/INDEX.md`.
- Residual issue tracked: `ODX-20260511-source-signal-observation-calibration` separated `.claude/` dominance and source `access_count=0` from fixed Case H/I.
- Governance task opened and completed: `.planning/task-cards/TASK-20260511-source-signal-observation-calibration.md`; calibration evidence did not implicate scanner attribution semantics.
- Sampling plan created: `docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md`.
- Sampling plan copied to mystocks for execution: `/opt/claude/mystocks_spec/docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md`.

### Follow-up: MCP release retest readiness

- Review date: 2026-05-11
- Release binary checked: `target/release/opendog`, built `2026-05-10 18:58:57 +0800`.
- Case H local MCP probe against `mystocks-fd` confirmed bounded observation payloads:
  - default `get_stats` returned 50 file rows with `result_window.limit=50` and `total_count=60638`
  - explicit `get_stats` with `limit: 5` returned 5 file rows
  - default `get_unused_files` returned 50 file rows with `result_window.limit=50` and `total_count=60600`
  - explicit `get_unused_files` with `limit: 5` returned 5 file rows
- Case I local MCP probe confirmed release resource support:
  - `initialize` advertises both `resources` and `tools`
  - `resources/list` returns `opendog://projects`
  - `resources/templates/list` returns `opendog://projects` and `opendog://project/{id}/verification`
  - `resources/read` works for both project-list and per-project verification resources
- Final gates run after documentation and governance updates:
  - `cargo fmt --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `python3 scripts/validate_planning_governance.py`
  - `openspec validate fix-fd-attribution`
  - `git diff --check`
- `cargo build --release` was re-run on 2026-05-11 and reported the release target up to date; `target/release/opendog` remains newer than the touched MCP/resource/scanner source files.
- Project-exchange handoff links were checked after adding the mystocks retest handoff; all local markdown links under `docs/project-exchange/` resolved.
- Worktree note: `.claude/` is a local Claude/OpenSpec integration directory and is ignored by `.gitignore`; it should not be treated as part of the OpenDog product change unless explicitly promoted.
- Untracked-file triage: untracked `.planning/task-cards/**`, `docs/project-exchange/**`, `openspec/**`, `FIELD_NOTES.md`, `src/core/file_classification.rs`, and `src/mcp/resource_handlers.rs` are product/governance change candidates.
- After adding `.claude/` to `.gitignore`, `git ls-files --others --exclude-standard` no longer reports local Claude integration files.
- Result: mystocks retest feedback has been received and mirrored. Case H, Case I, and `ODX-20260511-source-signal-observation-calibration` are fixed in the shared issue index; future work should start from new project-exchange evidence.

### Follow-up: public surface inventory check

- MCP source registration check: `src/mcp/mod.rs` exposes 19 `#[tool(name = ...)]` entries.
- MCP reference check: `docs/mcp-tool-reference.md` has 19 matching `## \`tool_name\`` sections.
- CLI help check: `target/release/opendog --help` lists 22 operational top-level commands plus Clap's generated `help`.
- Documentation alignment: README, QUICKSTART, FUNCTION_TREE, and capability index use 19 MCP tools, 22 CLI commands, root `FUNCTION_TREE.md`, and current `register_project` naming.

### Follow-up: 2026-05-12 documentation integrity check

- Date handling: 2026-05-10 and 2026-05-11 dates remain historical evidence/report/task dates and should not be mechanically rewritten to 2026-05-12.
- Link check: a local Markdown link scan covering README, QUICKSTART, FUNCTION_TREE, CHANGELOG, FIELD_NOTES, `docs/`, `.planning/`, and `openspec/` checked 109 Markdown files with no missing local markdown/file links.
- Historical exception: `docs/overdesign-assessment-2026-05-04.md` keeps its 2026-05-04 inventory numbers as a pre-optimization snapshot; current inventory lives in README, QUICKSTART, FUNCTION_TREE, capability index, and MCP reference.

---

*End of field notes. Append new observations in chronological order.*
