# Source-First Observation Views Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task after approval. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add source-first observation affordances so large AI-assisted repositories can inspect source hotness and unused candidates without manually filtering `.claude/` infrastructure noise.

**Architecture:** Keep the existing scanner and storage model unchanged. Add a presentation-layer filter parameter for MCP stats/unused payloads, mirror the same option in CLI stats/unused commands, and strengthen guidance boundaries around transient-read blind spots.

**Tech Stack:** Rust 2021, `serde`, `schemars`, `clap`, existing MCP payload builders, existing file classification helper.

---

## Approval Boundary

This plan is not approved for implementation until reviewed. It explicitly does not change `/proc/<pid>/fd` scanner attribution, directory-fd versus file-fd semantics, default ignore patterns, registration, snapshot, monitor, daemon, or storage schema.

## Evidence Basis

Mystocks 2026-05-11 calibration showed Claude Code Read operations are too transient for fd sampling, edit/revert activity is visible through `modification_count`, and sustained `.claude/` infrastructure reads can dominate hot stats. Therefore, the next governed fix is view/filter/guidance work, not scanner attribution.

Authoritative local evidence:

- `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md`
- `docs/project-exchange/issues/INDEX.md`
- `.planning/task-cards/TASK-20260511-source-first-observation-views.md`

## Proposed Interface Contract

### MCP `get_stats`

Request shape after implementation:

```json
{
  "id": "mystocks",
  "limit": 50,
  "path_classification": "source"
}
```

Allowed `path_classification` values: `all` default/current behavior, `source`, `infrastructure`, `backup`, `project`.

Response contract:

- `classification_summary` remains calculated from the full unfiltered input set.
- `result_window.total_count` means count after filter.
- `result_window.returned_count` means returned rows after filter and limit.
- `result_window.limit` remains normalized limit.
- `result_window.truncated` means filtered count is larger than returned count.
- Add `result_window.path_classification` with the normalized filter value.
- `files[*].path_classification` remains present.
- `guidance.layers.workspace_observation` should state the selected filter when one is active.
- If no rows match the selected filter, return an empty `files` array with `result_window.total_count=0`, `returned_count=0`, `truncated=false`, and the requested `result_window.path_classification`; guidance must state that the filtered view is empty, not that the project has no files.

### MCP `get_unused_files`

Request shape after implementation:

```json
{
  "id": "mystocks",
  "limit": 50,
  "path_classification": "source"
}
```

Same allowed values and result-window semantics as `get_stats`.

`unused_count` remains the full unfiltered unused candidate count for backward compatibility.

Add this field when `path_classification` is not `all`:

```json
{
  "filtered_unused_count": 18868
}
```

### CLI

Add optional classification filters:

```bash
opendog stats --id mystocks --path-classification source
opendog unused --id mystocks --path-classification source
```

Accepted values match MCP.

CLI output should remain human-readable:

- stats header includes `filter=source` when non-default
- unused header includes `filter=source` and shows both filtered and total unused counts

## File Structure

### Modify

- `src/core/file_classification.rs` - parse user-facing filter values without changing classification rules.
- `src/mcp/params.rs` - extend `ObservationRowsParams` with `path_classification`.
- `src/mcp/analysis_handlers.rs` - pass normalized filters into payload builders.
- `src/mcp/payloads/analysis_payloads.rs` - filter rows, preserve full classification summary, add filter metadata.
- `src/mcp/project_guidance/stats_unused/stats.rs` - accept selected filter, align recommendations with filtered rows, replace transient-read wording.
- `src/mcp/project_guidance/stats_unused/unused.rs` - accept selected filter, align recommendations with filtered rows, replace unused-safety wording.
- `src/cli/mod.rs`, `src/cli/project_commands.rs`, `src/cli/output/project_output.rs` - add and print CLI filters.
- `docs/mcp-tool-reference.md`, `docs/json-contracts.md`, `QUICKSTART.md`, `CHANGELOG.md` - document the public interface.

### Test

- `src/mcp/tests/payload_contracts/analysis_payloads.rs` - filtered stats/unused payload contracts.
- `src/mcp/tests/tool_surface.rs` - schema exposes `path_classification`.
- `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs` - transient-read, source-filter, and empty-filter-result boundaries.
- Existing CLI tests, if present, should be extended; otherwise add unit coverage around filtering helper logic.

## Task 1: Add Filter Type

**Files:**

- Modify: `src/core/file_classification.rs`

- [ ] **Step 1: Add a user-facing filter enum**

Add a type separate from `FilePathClassification` so `all` can be represented without changing the existing classifier:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePathClassificationFilter {
    All,
    Source,
    Infrastructure,
    Backup,
    Project,
}

impl FilePathClassificationFilter {
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("all").trim().to_ascii_lowercase().as_str() {
            "all" => Ok(Self::All),
            "source" => Ok(Self::Source),
            "infrastructure" => Ok(Self::Infrastructure),
            "backup" => Ok(Self::Backup),
            "project" => Ok(Self::Project),
            other => Err(format!(
                "path_classification must be one of: all, source, infrastructure, backup, project; got '{}'",
                other
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Source => "source",
            Self::Infrastructure => "infrastructure",
            Self::Backup => "backup",
            Self::Project => "project",
        }
    }

    pub fn matches(self, classification: FilePathClassification) -> bool {
        match self {
            Self::All => true,
            Self::Source => classification == FilePathClassification::Source,
            Self::Infrastructure => classification == FilePathClassification::Infrastructure,
            Self::Backup => classification == FilePathClassification::Backup,
            Self::Project => classification == FilePathClassification::Project,
        }
    }
}
```

- [ ] **Step 2: Add parser tests**

Add tests for default, uppercase normalization, each accepted value, and invalid input.

- [ ] **Step 3: Run targeted tests**

Run:

```bash
cargo test core::file_classification
```

Expected: PASS.

## Task 2: Extend MCP Params and Handlers

**Files:**

- Modify: `src/mcp/params.rs`
- Modify: `src/mcp/analysis_handlers.rs`

- [ ] **Step 1: Extend request params**

Add to `ObservationRowsParams`:

```rust
/// Optional row classification filter: "all" (default), "source", "infrastructure", "backup", or "project".
pub path_classification: Option<String>,
```

- [ ] **Step 2: Normalize in handlers**

In both `handle_get_stats` and `handle_get_unused_files`, parse once:

```rust
let path_filter = match FilePathClassificationFilter::parse(path_classification.as_deref()) {
    Ok(filter) => filter,
    Err(message) => {
        return error_json_for(
            MCP_STATS_V1,
            Some(id),
            &OpenDogError::InvalidInput(message),
        );
    }
};
```

Use `MCP_UNUSED_FILES_V1` for the unused handler.

- [ ] **Step 3: Pass filter to all payload-builder call sites**

Update all four calls to `stats_payload_with_limit` and `unused_files_payload_with_limit` in `src/mcp/analysis_handlers.rs` to include `path_filter`:

- daemon-backed stats path
- direct stats fallback path
- daemon-backed unused path
- direct unused fallback path

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check
```

Expected: PASS after Task 3 updates payload signatures.

## Task 3: Filter MCP Payload Rows

**Files:**

- Modify: `src/mcp/payloads/analysis_payloads.rs`
- Test: `src/mcp/tests/payload_contracts/analysis_payloads.rs`

- [ ] **Step 1: Update payload signatures**

Change both payload builders to accept:

```rust
path_filter: FilePathClassificationFilter
```

Keep test-only wrappers defaulting to `FilePathClassificationFilter::All`.

- [ ] **Step 2: Filter display rows before limit**

For stats:

```rust
let filtered_entries: Vec<StatsEntry> = entries
    .iter()
    .filter(|entry| path_filter.matches(classify_file_path(&entry.file_path)))
    .cloned()
    .collect();
let files: Vec<Value> = filtered_entries
    .iter()
    .take(limit)
    .map(|e| {
        json!({
            "path": e.file_path,
            "size": e.size,
            "file_type": e.file_type,
            "access_count": e.access_count,
            "estimated_duration_ms": e.estimated_duration_ms,
            "modification_count": e.modification_count,
            "last_access_time": e.last_access_time,
            "path_classification": classify_file_path(&e.file_path).as_str(),
        })
    })
    .collect();
```

Use `filtered_entries.len()` in `observation_result_window`.

Pass `&filtered_entries` to `stats_guidance` so `file_recommendations` and "hottest file" wording match the displayed filtered rows. Keep `summary` unchanged so full-project counts remain visible.

- [ ] **Step 3: Add filter metadata**

Extend `observation_result_window`:

```rust
fn observation_result_window(
    total_count: usize,
    returned_count: usize,
    limit: usize,
    path_filter: FilePathClassificationFilter,
) -> Value
```

and include:

```rust
"path_classification": path_filter.as_str()
```

- [ ] **Step 4: Preserve full classification summary**

Keep:

```rust
("classification_summary", classification_summary(entries))
```

not `classification_summary(filtered_entries)`.

- [ ] **Step 5: Add `filtered_unused_count` only for non-default unused filters**

For `unused_files_payload_with_limit`, replace the current array-literal payload field construction with a `Vec<(&str, Value)>` builder so the filtered count can be conditional:

```rust
let mut fields: Vec<(&str, Value)> = vec![
    ("unused_count", json!(unused.len())),
    (
        "result_window",
        observation_result_window(
            filtered_entries.len(),
            files.len(),
            limit,
            path_filter,
        ),
    ),
    ("classification_summary", classification_summary(unused)),
    ("files", json!(files)),
    (
        "guidance",
        unused_guidance(root_path, &filtered_entries, verification_runs, path_filter),
    ),
];

if path_filter != FilePathClassificationFilter::All {
    fields.push(("filtered_unused_count", json!(filtered_entries.len())));
}

versioned_project_payload(MCP_UNUSED_FILES_V1, id, fields)
```

Pass `&filtered_entries` to `unused_guidance` so candidate recommendations match the displayed filtered rows. Keep `unused_count` and `classification_summary` based on the unfiltered `unused` slice for backward compatibility.

- [ ] **Step 6: Add payload contract tests**

Test dataset:

```rust
vec![
    stats_entry("src/main.rs", 7, 1),
    stats_entry(".claude/settings.json", 99, 0),
    stats_entry("notes.txt", 1, 0),
    stats_entry("src/main.rs.bak.20260511", 0, 0),
]
```

Assertions:

- `path_classification=source` returns only `src/main.rs`.
- `classification_summary` still counts all four categories from the full set.
- `result_window.total_count == 1`.
- `result_window.path_classification == "source"`.
- `path_classification=infrastructure` returns only `.claude/settings.json` and `result_window.total_count == 1`.
- `path_classification=source` with no matching rows returns `files=[]`, `total_count=0`, `returned_count=0`, `truncated=false`, and `path_classification="source"`.
- `unused_count` remains full unused count for unused payload.
- `filtered_unused_count` appears only when non-default filter is used.

- [ ] **Step 7: Run targeted tests**

Run:

```bash
cargo test mcp::tests::payload_contracts::analysis_payloads
```

Expected: PASS.

## Task 4: Add CLI Filters

**Files:**

- Modify: `src/cli/mod.rs`
- Modify: `src/cli/project_commands.rs`
- Modify: `src/cli/output/project_output.rs`

- [ ] **Step 1: Add CLI args**

Add to both `Stats` and `Unused`:

```rust
/// Optional row classification filter: all, source, infrastructure, backup, or project.
#[arg(long, default_value = "all")]
path_classification: String,
```

- [ ] **Step 2: Update command dispatch**

Pass the string into:

```rust
project_commands::cmd_stats(&pm, &id, &path_classification)
project_commands::cmd_unused(&pm, &id, &path_classification)
```

- [ ] **Step 3: Filter rows before printing**

In `project_commands`, parse with `FilePathClassificationFilter::parse(Some(path_classification))`.

Filter entries using:

```rust
let filtered: Vec<StatsEntry> = entries
    .into_iter()
    .filter(|entry| filter.matches(classify_file_path(&entry.file_path)))
    .collect();
```

Use the same logic for unused.

- [ ] **Step 4: Print filter context**

Change output functions to accept:

```rust
filter: FilePathClassificationFilter,
unfiltered_count: usize
```

For stats, print:

```text
Project 'mystocks' — 50087 files | 46 accessed | 50041 unused | filter=source | shown=50/18871
```

For unused, print:

```text
Unused files for project 'mystocks' — filter=source | shown=100/18868 | total_unused=50041
```

- [ ] **Step 5: Run CLI-targeted tests or checks**

Run:

```bash
cargo test cli
```

Expected: PASS. If no CLI tests cover output, run full `cargo test` in final verification.

## Task 5: Guidance Filter Awareness and Boundary Wording

**Files:**

- Modify: `src/mcp/project_guidance/stats_unused/stats.rs`
- Modify: `src/mcp/project_guidance/stats_unused/unused.rs`
- Test: `src/mcp/tests/guidance_basics/toolchain_and_unused/stats_and_unused.rs`

- [ ] **Step 1: Pass selected filter into guidance builders**

Update signatures:

```rust
pub(in crate::mcp) fn stats_guidance(
    root_path: &Path,
    summary: &stats::ProjectSummary,
    entries: &[StatsEntry],
    verification_runs: &[VerificationRun],
    path_filter: FilePathClassificationFilter,
) -> Value
```

```rust
pub(in crate::mcp) fn unused_guidance(
    root_path: &Path,
    unused_entries: &[StatsEntry],
    verification_runs: &[VerificationRun],
    path_filter: FilePathClassificationFilter,
) -> Value
```

Update existing test-only and production call sites. Existing wrappers that do not expose filtering should pass `FilePathClassificationFilter::All`.

- [ ] **Step 2: Add filter metadata to `workspace_observation`**

When `path_filter != FilePathClassificationFilter::All`, add this field to every `workspace_observation` object built by stats and unused guidance:

```rust
"path_classification_filter": path_filter.as_str()
```

When the filtered `entries` / `unused_entries` slice is empty but full project summary or full unused count is non-zero, add an inference or blind spot string equivalent to:

```rust
"The selected path_classification filter returned no rows; this does not mean the project has no files or no unused candidates."
```

- [ ] **Step 3: Replace transient-read blind spot**

In stats constraints boundaries, replace the existing `"Sampling-based monitoring may miss very brief file accesses."` boundary string with:

```rust
"Sampling-based monitoring may miss very brief file reads, including MCP host or AI assistant reads that open and close source files quickly.".to_string()
```

- [ ] **Step 4: Replace unused safety boundary**

In unused constraints boundaries, replace the existing `"Lack of observed access is not proof that a file is safe to delete."` boundary string with:

```rust
"access_count=0 means OPENDOG did not observe an open descriptor; it is not proof that the file was never read or is safe to delete.".to_string()
```

- [ ] **Step 5: Add guidance tests**

Assert the JSON guidance contains:

- `very brief file reads`
- `open descriptor`
- `never read`
- `path_classification_filter`
- `selected path_classification filter returned no rows`

- [ ] **Step 6: Run targeted tests**

Run:

```bash
cargo test mcp::tests::guidance_basics::toolchain_and_unused
```

Expected: PASS.

## Task 6: Tool Schema and Docs

**Files:**

- Modify: `src/mcp/tests/tool_surface.rs`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/json-contracts.md`
- Modify: `QUICKSTART.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update tool schema test**

Assert `get_stats` and `get_unused_files` input schemas expose `path_classification`.

- [ ] **Step 2: Update MCP tool reference**

Document:

```json
{
  "id": "demo",
  "limit": 50,
  "path_classification": "source"
}
```

and the allowed values.

- [ ] **Step 3: Update JSON contracts**

Document:

- `result_window.path_classification`
- full versus filtered count semantics
- `filtered_unused_count`

- [ ] **Step 4: Update QUICKSTART**

Add an AI-facing source-first example:

```json
get_stats {"id":"mystocks","path_classification":"source","limit":50}
get_unused_files {"id":"mystocks","path_classification":"source","limit":50}
```

State that this does not hide infrastructure globally.

- [ ] **Step 5: Update CHANGELOG**

Add a 2026-05-11 entry after implementation:

```md
- Added source-first `path_classification` filters for MCP and CLI stats/unused views.
- Clarified transient-read blind spots in stats/unused guidance.
```

## Final Verification

Run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
python3 scripts/validate_task_cards.py
python3 scripts/validate_planning_governance.py
git diff --check
```

Expected:

- all commands pass
- no scanner files changed except imports caused by compiler formatting if any
- H/I bounded payload behavior remains intact
- `TASK-20260511-source-first-observation-views.md` can move from `proposed` to `completed` only after implementation and verification evidence are recorded

## Retest Handoff After Implementation

Ask mystocks to run:

```json
get_stats {"id":"mystocks","path_classification":"source","limit":50}
get_unused_files {"id":"mystocks","path_classification":"source","limit":50}
get_stats {"id":"mystocks","path_classification":"infrastructure","limit":10}
```

Expected:

- source calls return source-classified rows or an empty bounded set with clear metadata
- infrastructure call still exposes `.claude/` activity when requested
- `classification_summary` keeps full-project visibility
- guidance warns that transient reads may be missed
