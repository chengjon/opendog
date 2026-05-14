# OPENDOG Retest Results - mystocks (2026-05-11)

> Retest per handoff: `/opt/claude/opendog/docs/project-exchange/reports/mystocks/opendog-retest-handoff-2026-05-11.md`
> OpenDog release binary: `/opt/claude/opendog/target/release/opendog` (2026-05-10 18:58:57 +0800)
> MCP host: Claude Code CLI (GLM-5.1)
> Project: mystocks, status: monitoring, 50087 files

---

## Case H - Bounded MCP Stats / Unused Payloads

**Status: PASS**

| Call | limit | returned_count | total_count | truncated | files.length | files.length <= limit | result_window.limit == limit |
|------|-------|----------------|-------------|-----------|--------------|----------------------|------------------------------|
| `get_stats {id:"mystocks"}` | 50 (default) | 50 | 50087 | true | 50 | 50 <= 50 PASS | 50 == 50 PASS |
| `get_stats {id:"mystocks", limit:50}` | 50 | 50 | 50087 | true | 50 | 50 <= 50 PASS | 50 == 50 PASS |
| `get_unused_files {id:"mystocks"}` | 50 (default) | 50 | 50041 | true | 50 | 50 <= 50 PASS | 50 == 50 PASS |
| `get_unused_files {id:"mystocks", limit:50}` | 50 | 50 | 50041 | true | 50 | 50 <= 50 PASS | 50 == 50 PASS |

Evidence:
- No MCP error -32000 (Connection closed). Previous Case A root cause resolved.
- No MB-scale output. Payloads are bounded by default limit of 50.
- `result_window` includes `limit`, `returned_count`, `total_count`, `truncated` — all correct.
- `classification_summary` present in both responses: 50087 total (backup: 23, infrastructure: 463/430, project: 30730/30720, source: 18871/18868).
- `guidance` layers embedded in response: cleanup_refactor_candidates, constraints_boundaries, execution_strategy, repo_status_risk, verification_evidence all populated.

### Remaining observations (non-blocking)

- Case C+D attribution anomaly persists: top 28 `.claude/` files share identical access_count (88981/88982) and estimated_duration_ms (270452000/270455000). Source files still at access_count=0. This is a known observation issue, not a payload issue.
- `schema_version` fields present: `opendog.mcp.stats.v1` and `opendog.mcp.unused-files.v1`.

---

## Case I - MCP Resources Discovery

**Status: PASS**

| Test | Result | Detail |
|------|--------|--------|
| `resources/list` (server=opendog) | PASS | Returns 1 resource: `opendog://projects` (mimeType: application/json, title: "OpenDog Projects") |
| `resources/read opendog://projects` | PASS | Returns full project list: count=1, mystocks (status: monitoring), with embedded guidance layers |
| `resources/read opendog://project/mystocks/verification` | PASS | Returns complete verification status JSON: freshness (aging, age_seconds: 88600), gate_assessment (cleanup: caution, refactor: blocked), latest_runs (pytest passed) |
| Resources capability active | PASS | ListMcpResourcesTool and ReadMcpResourceTool both functional, confirming `resources` capability advertised in initialize |

Evidence:
- `opendog://projects` URI returns `schema_version: opendog.mcp.list-projects.v1`
- `opendog://project/mystocks/verification` returns `schema_version: opendog.mcp.verification-status.v1`
- Both resources return `mimeType: application/json` with valid JSON in `contents[].text`
- Resource template `opendog://project/{id}/verification` confirmed working for mystocks

---

## Shared Issue Status Update

| Issue ID | Previous Status | New Status | Reason |
|----------|----------------|------------|--------|
| `ODX-20260510-mcp-large-payload-pagination` | validating | **fixed** | Case H fully passes: bounded payloads, correct result_window, no connection errors |
| `ODX-20260510-mcp-resources-not-discovered` | validating | **fixed** | Case I fully passes: resources/list and resources/read both functional |
| `ODX-20260511-source-signal-observation-calibration` | validating | **fixed** | OpenDog resolved the product issue with source-first `path_classification` filters and guidance boundaries; transient Claude Code reads remain a documented sampling limitation |

Updated in: `/opt/claude/opendog/docs/project-exchange/issues/INDEX.md`

---

## Source Signal Calibration (ODX-20260511-source-signal-observation-calibration)

**Date: 2026-05-11**

### Activity Performed

Intentional source-heavy activity during monitored window:
- Read 3 Python files: `src/adapters/data_source_manager.py`, `src/adapters/base_adapter.py`, `src/advanced_analysis/capital_flow_analyzer.py`
- Read 2 Vue files: `web/frontend/src/App.vue`, `web/frontend/src/components/DynamicSidebar.vue`
- Ran 4 `rg` searches against symbols from those files
- Edited `src/adapters/base_adapter.py` (reverted) and `web/frontend/src/App.vue` (reverted)
- Ran `py_compile` + `ruff check` on touched Python file

### Evidence: Touched Source Files After Activity

| File | access_count (before) | access_count (after) | modification_count (after) | path_classification |
|------|-----------------------|----------------------|----------------------------|---------------------|
| `src/adapters/data_source_manager.py` | 0 | **0** | 0 | source |
| `src/adapters/base_adapter.py` | 0 | **0** | 0 (reverted) | source |
| `src/advanced_analysis/capital_flow_analyzer.py` | 0 | **0** | 0 | source |
| `web/frontend/src/App.vue` | 1 | **1** (unchanged) | **2** (was 0) | source |
| `web/frontend/src/components/DynamicSidebar.vue` | 0 | **0** | 0 | source |

### Evidence: Infrastructure Dominance

- Top 28 `.claude/` files all share identical `access_count=89914` and `estimated_duration_ms=273290000`
- `web/frontend_status.py`: only source file with `access_count > 0` (count=4, from much earlier)

### Interpretation (per plan rules)

**Hypothesis 4 confirmed: workflow mismatch.** Claude Code's Read tool does not hold file descriptors open long enough for `/proc`-based fd sampling. The Edit tool produces modification events (App.vue `modification_count` 0→2) but no access events.

**Hypothesis 1 partially confirmed: expected tool behavior.** Claude Code reads files into memory and closes fds quickly; the sampling interval cannot catch transient reads.

**Conclusion: Scanner attribution is working correctly for sustained fd holders, but Claude Code's read pattern is invisible to sampling-based observation.** The next fix should be **view/filter/guidance oriented** (source-first views, classification-based filtering) rather than scanner attribution changes. No OpenSpec proposal needed for scanner semantics — this is expected behavior for the observation method.

### OpenDog Response

OpenDog implemented the view/filter/guidance fix identified by this calibration:
- MCP and CLI `stats` / `unused` support `path_classification`: `all`, `source`, `infrastructure`, `backup`, `project`.
- Observation payloads expose `result_window.path_classification`, per-row `files[*].path_classification`, and full `classification_summary`.
- Guidance now states transient-read and `access_count=0` boundaries so AI agents do not over-claim from sampled fd evidence.

The shared issue is fixed as a product-presentation and guidance issue. Scanner attribution remains unchanged because this evidence shows an expected `/proc/<pid>/fd` sampling boundary, not directory-fd fan-out or source-path misattribution.

---

## Environment

- host: WSL2 Linux 6.6.87.2-microsoft-standard-WSL2
- MCP host: Claude Code CLI
- OpenDog binary: `/opt/claude/opendog/target/release/opendog` (2026-05-10 18:58:57)
- OPENDOG_HOME: `/root/.opendog`
- Project: mystocks at `/opt/claude/mystocks_spec`, status: monitoring, 50087 files
- Retest date: 2026-05-11
