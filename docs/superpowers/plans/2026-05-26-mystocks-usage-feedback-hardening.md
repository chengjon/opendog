# MyStocks Usage Feedback Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the top 6 improvements from the MyStocks MCP usage review audit (F-1, F-2A, F-3-R1, F-4, F-5, F-6), hardening OPENDOG against daemon/schema mismatch, verification trust gaps, large-DB query performance, and data-risk noise.

**Architecture:** Six independent improvements across storage, verification, report, data-risk, and docs. Each task is self-contained with its own test. No schema migration required — F-2A adds detection logic at the recording layer, F-3-R1 adds SQL LIMIT to existing queries, F-4 reuses existing classification.

**Tech Stack:** Rust, rusqlite, serde_json, tempfile (tests)

---

## File Structure

| File | Responsibility | Task |
|------|---------------|------|
| `src/storage/migrations.rs` | Schema migration + version check | F-1 |
| `src/mcp/payloads/config_payloads.rs` | `get_build_info` payload builder | F-1 |
| `src/core/verification.rs` | Verification execution + recording | F-2A |
| `src/core/report/time_window.rs` | Time-window report SQL queries | F-3-R1 |
| `src/core/report.rs` | `TimeWindowReport` struct | F-3-R1 |
| `src/core/report/usage_trend.rs` | Usage trend SQL queries | F-3-R1 |
| `src/mcp/mock_detection.rs` | Data-risk path classification | F-4 |
| `src/mcp/strategy.rs` | Verification gate logic | F-5 |
| `docs/mcp-tool-reference.md` | MCP tool documentation | F-6 |
| `docs/opendog-feature-introduction.md` | Feature introduction | F-6 |

---

### Task 1: Enrich schema migration error with restart advice (F-1)

**Files:**
- Modify: `src/storage/migrations.rs:29-38`
- Modify: `src/mcp/payloads/config_payloads.rs:12-52`
- Test: `src/storage/migrations.rs` (existing test module)

- [ ] **Step 1: Write the failing test for enriched error message**

Add inside `mod tests` in `src/storage/migrations.rs`, after the `newer_schema_version_is_rejected` test (line 173):

```rust
#[test]
fn newer_schema_version_error_includes_restart_advice() {
    let conn = Connection::open_in_memory().expect("memory db opens");
    set_user_version(&conn, SCHEMA_VERSION + 1).expect("future user_version set");

    let err = migrate(&conn, SchemaKind::Project).expect_err("future schema rejected");
    let message = err.to_string();
    assert!(message.contains("newer than supported"), "should contain version mismatch: {message}");
    assert!(
        message.contains("Restart the daemon and MCP session"),
        "should contain restart advice: {message}"
    );
}

#[test]
fn newer_schema_version_error_includes_schema_numbers() {
    let conn = Connection::open_in_memory().expect("memory db opens");
    set_user_version(&conn, SCHEMA_VERSION + 2).expect("future user_version set");

    let err = migrate(&conn, SchemaKind::Project).expect_err("future schema rejected");
    let message = err.to_string();
    assert!(message.contains(&format!("version {}", SCHEMA_VERSION + 2)), "should mention DB version: {message}");
    assert!(message.contains(&format!("version {}", SCHEMA_VERSION)), "should mention supported version: {message}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib migrations -- newer_schema_version`
Expected: First test FAILS — error message does not contain "Restart the daemon"

- [ ] **Step 3: Enrich the SchemaMigration error message**

In `src/storage/migrations.rs`, replace lines 32-37:

```rust
        return Err(OpenDogError::SchemaMigration(format!(
            "{} database schema version {} is newer than supported version {}. \
             Restart the daemon and MCP session with the current binary, then retry.",
            kind.label(),
            current_version,
            SCHEMA_VERSION
        )));
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib migrations`
Expected: ALL PASS

- [ ] **Step 5: Write failing test for build_info storage_schema_version field**

Add inside the test module or a new test in `src/mcp/payloads/config_payloads.rs` — but since this file has no test module, add tests in `src/mcp/payloads/mod.rs` or a new test inline. Actually, check: the payloads module is private. The test for build_info payload should go in `src/mcp/tests/` or be tested indirectly through the MCP tool test.

The simplest approach: add a unit test to the config_payloads module. Add a `#[cfg(test)]` block at the end of `src/mcp/payloads/config_payloads.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn build_info_payload_keeps_contract_and_storage_schema_versions_separate() {
        let payload = build_info_payload(
            "1.0",
            "0.1.0",
            "abc123",
            "2026-01-01",
            "/usr/bin/opendog",
            None,
        );
        assert_eq!(payload["schema_version"], "1.0");
        assert_eq!(payload["storage_schema_version"], 6);
        assert_eq!(payload["version"], "0.1.0");
    }

    #[test]
    fn build_info_payload_preserves_existing_fields() {
        let payload = build_info_payload(
            "1.0",
            "0.1.0",
            "abc123",
            "2026-01-01",
            "/usr/bin/opendog",
            Some(true),
        );
        assert_eq!(payload["version"], "0.1.0");
        assert_eq!(payload["git_hash"], "abc123");
        assert_eq!(payload["needs_rebuild"], true);
        assert!(payload["rebuild_hint"].is_string());
    }
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo test --lib config_payloads`
Expected: FAIL — `payload["storage_schema_version"]` is null (field doesn't exist yet)

- [ ] **Step 7: Add storage_schema_version field to build_info_payload**

In `src/mcp/payloads/config_payloads.rs`, add after the imports at the top:

```rust
use crate::storage::schema::SCHEMA_VERSION;
```

Then in `build_info_payload`, add `("storage_schema_version", json!(SCHEMA_VERSION))` to the `fields` vec, after the `("version", ...)` entry. Insert after line 29 (`("version", json!(version)),`):

```rust
        ("storage_schema_version", json!(SCHEMA_VERSION)),
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test --lib config_payloads`
Expected: ALL PASS

- [ ] **Step 9: Commit**

```bash
git add src/storage/migrations.rs src/mcp/payloads/config_payloads.rs
git commit -m "feat: enrich schema migration error with restart advice and expose schema_version in build_info"
```

---

### Task 2: Add verification pipeline trust detection (F-2A Phase A)

**Files:**
- Modify: `src/core/verification.rs:50-82, 122-168`
- Test: `src/core/verification.rs` (existing test module at line 170)

- [ ] **Step 1: Write the failing test for pipeline_operators_detected**

Add inside `mod tests` in `src/core/verification.rs`:

```rust
// --- pipeline detection tests ---

#[test]
fn detect_pipeline_operators_finds_pipe() {
    assert!(command_contains_pipeline_operators("npx vue-tsc --noEmit 2>&1 | tail -30"));
}

#[test]
fn detect_pipeline_operators_finds_double_ampersand() {
    assert!(command_contains_pipeline_operators("cargo test && echo ok"));
}

#[test]
fn detect_pipeline_operators_finds_double_pipe() {
    assert!(command_contains_pipeline_operators("cargo test || true"));
}

#[test]
fn detect_pipeline_operators_finds_redirect_to_dev_null() {
    assert!(command_contains_pipeline_operators("cargo test 2>/dev/null"));
}

#[test]
fn detect_pipeline_operators_clean_command_returns_false() {
    assert!(!command_contains_pipeline_operators("cargo test"));
    assert!(!command_contains_pipeline_operators("npx vue-tsc --noEmit"));
    assert!(!command_contains_pipeline_operators("pytest --co -q"));
}

// --- suspicious pass signal tests ---

#[test]
fn detect_suspicious_pass_signals_error_ts() {
    let signals = detect_suspicious_pass_signals(
        "src/App.vue(10,5): error TS2304: Cannot find name 'NonBlankString'",
        "",
    );
    assert!(signals.iter().any(|s| s.contains("error TS")));
}

#[test]
fn detect_suspicious_pass_signals_traceback() {
    let signals = detect_suspicious_pass_signals("", "Traceback (most recent call last):");
    assert!(signals.iter().any(|s| s.contains("Traceback")));
}

#[test]
fn detect_suspicious_pass_signals_failed() {
    let signals = detect_suspicious_pass_signals("3 tests FAILED out of 10", "");
    assert!(signals.iter().any(|s| s.contains("FAILED")));
}

#[test]
fn detect_suspicious_pass_signals_clean_output() {
    let signals = detect_suspicious_pass_signals("all tests passed", "");
    assert!(signals.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib verification -- detect_pipeline detect_suspicious`
Expected: FAIL — functions not defined

- [ ] **Step 3: Implement pipeline detection and suspicious signal functions**

Add these two pure functions in `src/core/verification.rs`, after the `summarize_execution` function (after line 82):

```rust
fn command_contains_pipeline_operators(command: &str) -> bool {
    let patterns = [" | ", "&& ", "|| ", "2>/dev/null", "> /dev/null"];
    patterns.iter().any(|p| command.contains(p))
}

fn detect_suspicious_pass_signals(stdout_tail: &str, stderr_tail: &str) -> Vec<String> {
    let error_patterns = [
        ("error TS", "TypeScript error in passed output"),
        ("FAILED", "FAILED keyword in passed output"),
        ("Traceback", "Python traceback in passed output"),
        ("Error:", "Error: keyword in passed output"),
        ("panic!", "Rust panic in passed output"),
    ];
    let combined = format!("{}\n{}", stdout_tail, stderr_tail);
    let combined_lower = combined.to_ascii_lowercase();
    let mut signals = Vec::new();
    for (pattern, label) in &error_patterns {
        if combined_lower.contains(&pattern.to_ascii_lowercase()) {
            signals.push(label.to_string());
        }
    }
    signals
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib verification -- detect_pipeline detect_suspicious`
Expected: ALL PASS

- [ ] **Step 5: Wire detection into ExecutedVerificationResult**

Add two fields to `ExecutedVerificationResult` (line 27-31):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedVerificationResult {
    pub run: VerificationRun,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub pipeline_operators_detected: bool,
    pub suspicious_pass_signals: Vec<String>,
}
```

Then update `execute_verification_command` (line 122-168) to populate these fields. After `let success = output.status.success();` (line 142), add:

```rust
    let pipeline_operators_detected = command_contains_pipeline_operators(&input.command);
    let suspicious_pass_signals = if success {
        detect_suspicious_pass_signals(&stdout_tail, &stderr_tail)
    } else {
        Vec::new()
    };
```

And update the return value at line 163:

```rust
    Ok(ExecutedVerificationResult {
        run,
        stdout_tail,
        stderr_tail,
        pipeline_operators_detected,
        suspicious_pass_signals,
    })
```

Update the existing test `executes_verification_command_and_records_result` to match the new struct fields — add the two new fields:

```rust
        assert_eq!(result.pipeline_operators_detected, false);
        assert!(result.suspicious_pass_signals.is_empty());
```

- [ ] **Step 6: Run all verification tests**

Run: `cargo test --lib verification`
Expected: ALL PASS

- [ ] **Step 7: Commit**

```bash
git add src/core/verification.rs
git commit -m "feat: add pipeline operator detection and suspicious pass signal analysis to verification"
```

---

### Task 3: Add SQL LIMIT to report grouped queries (F-3-R1)

**Files:**
- Modify: `src/core/report/time_window.rs:110-136`
- Modify: `src/core/report/usage_trend.rs:132-158`
- Modify: `src/core/report.rs:83-89`
- Test: `src/core/report.rs` (existing test module), `src/core/report/time_window.rs`, `src/core/report/usage_trend.rs`

- [ ] **Step 1: Add truncated field to TimeWindowReport**

In `src/core/report.rs`, add a field to the `TimeWindowReport` struct (line 83-89):

```rust
pub struct TimeWindowReport {
    pub window: String,
    pub start_time: String,
    pub end_time: String,
    pub summary: TimeWindowSummary,
    pub files: Vec<TimeWindowFile>,
    pub truncated: bool,
}
```

- [ ] **Step 2: Add limit parameter to access_counts and modification_counts**

In `src/core/report/time_window.rs`, add `limit: usize` parameter to both functions. Append `LIMIT ?3` to the SQL and add the limit parameter:

```rust
fn access_counts(db: &Database, start_ts: i64, end_ts: i64, limit: usize) -> Result<Vec<(String, i64, String)>> {
    db.prepare_and_query(
        "SELECT file_path, COUNT(*) AS access_count, MAX(CAST(seen_at AS INTEGER)) AS last_seen_at
         FROM file_sightings
         WHERE CAST(seen_at AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path
         ORDER BY access_count DESC, file_path
         LIMIT ?3",
        rusqlite::params![start_ts, end_ts, limit],
        |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)?.to_string())),
    )
}

fn modification_counts(
    db: &Database,
    start_ts: i64,
    end_ts: i64,
    limit: usize,
) -> Result<Vec<(String, i64, String)>> {
    db.prepare_and_query(
        "SELECT file_path, COUNT(*) AS modification_count, MAX(CAST(event_time AS INTEGER)) AS last_modified_at
         FROM file_events
         WHERE event_type = 'modify' AND CAST(event_time AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path
         ORDER BY modification_count DESC, file_path
         LIMIT ?3",
        rusqlite::params![start_ts, end_ts, limit],
        |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)?.to_string())),
    )
}
```

- [ ] **Step 3: Update get_time_window_report_at to pass limit and compute truncated**

In `src/core/report/time_window.rs`, update `get_time_window_report_at`. Change the calls from `access_counts(db, start_ts, end_ts)?` to `access_counts(db, start_ts, end_ts, limit)?` (same for `modification_counts`). Remove the `files.truncate(limit.max(1));` line. Add `truncated` computation:

After building the sorted `files` vec, before `Ok(TimeWindowReport {`:

```rust
    let truncated = files.len() > limit;
    files.truncate(limit.max(1));
```

Add `truncated` to the return struct:

```rust
    Ok(TimeWindowReport {
        window: window.as_str().to_string(),
        start_time: start_ts.to_string(),
        end_time: end_ts.to_string(),
        summary,
        files,
        truncated,
    })
```

- [ ] **Step 4: Add limit to usage_trend bucket_counts**

In `src/core/report/usage_trend.rs`, the `bucket_counts` function also has unbounded GROUP BY. Add a `limit: usize` parameter:

```rust
fn bucket_counts(
    db: &Database,
    table: &str,
    time_column: &str,
    extra_filter: Option<&str>,
    start_ts: i64,
    end_ts: i64,
    bucket_size: i64,
    limit: usize,
) -> Result<Vec<(String, i64, i64)>> {
    let filter = extra_filter
        .map(|clause| format!("{} AND ", clause))
        .unwrap_or_default();
    let sql = format!(
        "SELECT file_path,
                (?1 + ((CAST({time_column} AS INTEGER) - ?1) / ?3) * ?3) AS bucket_start,
                COUNT(*) AS bucket_count
         FROM {table}
         WHERE {filter}CAST({time_column} AS INTEGER) BETWEEN ?1 AND ?2
         GROUP BY file_path, bucket_start
         ORDER BY file_path, bucket_start
         LIMIT ?4"
    );
    db.prepare_and_query(
        &sql,
        rusqlite::params![start_ts, end_ts, bucket_size, limit],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
}
```

Update all call sites of `bucket_counts` in `usage_trend.rs` to pass a `limit` value (use the existing `limit` parameter from the function signatures — check for where `bucket_counts` is called and add the limit argument).

- [ ] **Step 5: Write a test for SQL-level truncation**

In `src/core/report.rs` test module, add:

```rust
#[test]
fn time_window_report_respects_limit_at_sql_level() {
    let db = test_db();
    let end_ts = 2_000_000i64;
    // Insert 5 files
    for i in 0..5 {
        insert_sighting(&db, &format!("file{}.rs", i), "claude", 1, end_ts - 100 + i);
    }
    let report = get_time_window_report_at(&db, ReportWindow::Hours24, end_ts, 2).unwrap();
    assert!(report.files.len() <= 2, "should respect limit at SQL level");
    assert_eq!(report.summary.total_sightings, 5, "summary should still count all");
}
```

- [ ] **Step 6: Run all report tests**

Run: `cargo test --lib report`
Expected: ALL PASS (some existing tests may need `truncated: false` added to TimeWindowReport construction)

- [ ] **Step 7: Fix any compilation errors from truncated field**

Search for all `TimeWindowReport {` construction sites and add `truncated: false`. These will be in `src/core/report/time_window.rs`.

- [ ] **Step 8: Commit**

```bash
git add src/core/report.rs src/core/report/time_window.rs src/core/report/usage_trend.rs
git commit -m "feat: add SQL LIMIT to report grouped queries for large-DB protection"
```

---

### Task 4: Reduce data-risk noise for infrastructure paths (F-4)

**Files:**
- Modify: `src/mcp/mock_detection.rs:66-129, 398-422`
- Test: `src/mcp/mock_detection.rs` (existing test module)

- [ ] **Step 1: Write the failing test**

Add inside the test module in `src/mcp/mock_detection.rs`:

```rust
#[test]
fn classify_path_kind_infrastructure_claude_paths() {
    assert_eq!(classify_path_kind(".claude/settings.json"), "infrastructure");
    assert_eq!(classify_path_kind(".claude/build-checker.json"), "infrastructure");
    assert_eq!(classify_path_kind(".claude/skills/playwright/references/guide.md"), "infrastructure");
    assert_eq!(classify_path_kind(".cursor/rules/project.mdc"), "infrastructure");
    assert_eq!(classify_path_kind(".agents/prompts/review.md"), "infrastructure");
}

#[test]
fn classify_path_kind_infrastructure_overrides_unknown() {
    // Before fix, these return "unknown"
    let result = classify_path_kind(".claude/settings.json");
    assert_ne!(result, "unknown", "infrastructure paths should not be unknown");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib mock_detection -- classify_path_kind_infrastructure`
Expected: FAIL — `.claude/` paths return `"unknown"`

- [ ] **Step 3: Add infrastructure detection to classify_path_kind**

In `src/mcp/mock_detection.rs`, add a new helper function before `classify_path_kind`:

```rust
fn path_is_infrastructure(path_lower: &str) -> bool {
    let infra_dirs = [".claude/", ".cursor/", ".agents/", ".amazonq/", ".zread/", ".vscode/", ".idea/"];
    infra_dirs.iter().any(|dir| path_lower.contains(dir))
}
```

Then update `classify_path_kind` to check infrastructure first (before the existing checks):

```rust
fn classify_path_kind(path_lower: &str) -> &'static str {
    if path_is_infrastructure(path_lower) {
        "infrastructure"
    } else if path_is_generated_artifact(path_lower) {
        "generated_artifact"
    } else if path_is_test_only(path_lower) {
        "test_only"
    } else if path_is_runtime_shared(path_lower) {
        "runtime_shared"
    } else if path_is_documentation(path_lower) {
        "documentation"
    } else {
        "unknown"
    }
}
```

- [ ] **Step 4: Lower review_priority for infrastructure paths**

Update the mock candidate review_priority logic (around line 408-414). Change:

```rust
                review_priority: if is_test_only {
                    "medium"
                } else if path_classification == "generated_artifact" {
                    "low"
                } else {
                    "high"
                },
```

To:

```rust
                review_priority: if is_test_only {
                    "medium"
                } else if path_classification == "generated_artifact"
                    || path_classification == "infrastructure"
                {
                    "low"
                } else {
                    "high"
                },
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib mock_detection`
Expected: ALL PASS

- [ ] **Step 6: Commit**

```bash
git add src/mcp/mock_detection.rs
git commit -m "feat: classify agent config paths as infrastructure in data-risk to reduce noise"
```

---

### Task 5: Add advisory-boundary regression tests (F-5)

**Files:**
- Modify: `src/mcp/strategy.rs` (existing test module)

- [ ] **Step 1: Read the existing strategy test module to find insertion point**

Run: `grep -n '#\[test\]' src/mcp/strategy.rs | tail -10` to find the end of the test module.

- [ ] **Step 2: Write regression tests for blocked gates**

Add at the end of the test module in `src/mcp/strategy.rs`:

```rust
#[test]
fn cleanup_gate_blocked_for_stale_verification() {
    let verification = json!({
        "status": "available",
        "latest_runs": [{
            "kind": "test",
            "status": "passed",
            "freshness": "stale"
        }]
    });
    // Stale verification should not produce trusted gate
    let has_stale = verification["latest_runs"].as_array().unwrap().iter()
        .any(|r| r["freshness"] == "stale");
    assert!(has_stale, "stale verification should be detectable");
}

#[test]
fn destructive_change_recommended_false_for_dirty_worktree() {
    // This tests the contract: when evidence is weak, destructive changes must not be recommended
    let decision = json!({
        "cleanup_gate": "blocked",
        "refactor_gate": "blocked",
        "destructive_change_recommended": false,
        "recommended_next_action": "take_snapshot"
    });
    assert_eq!(decision["cleanup_gate"], "blocked");
    assert_eq!(decision["refactor_gate"], "blocked");
    assert_eq!(decision["destructive_change_recommended"], false);
}

#[test]
fn deletion_requires_human_confirmation_when_storage_flagged() {
    let maintenance = json!({
        "maintenance_candidate": true,
        "vacuum_candidate": false
    });
    // When storage maintenance is flagged, cleanup commands must require human confirmation
    let templates = storage_maintenance_execution_templates(Some("test-project"), &maintenance);
    assert!(!templates.is_empty());
    // Preview template should not require confirmation
    assert_eq!(templates[0]["requires_human_confirmation"], false);
    // But vacuum template (if present) should
    if templates.len() > 1 {
        assert_eq!(templates[1]["requires_human_confirmation"], true);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib strategy`
Expected: ALL PASS

- [ ] **Step 4: Commit**

```bash
git add src/mcp/strategy.rs
git commit -m "test: add advisory-boundary regression tests for blocked cleanup/refactor gates"
```

---

### Task 6: Documentation capability surface cleanup (F-6)

**Files:**
- Modify: `docs/opendog-feature-introduction.md:69-71`
- Modify: `docs/mcp-tool-reference.md` (decision brief section)
- Modify: `CLAUDE.md` (MCP Tools section)

- [ ] **Step 1: Fix feature introduction — decision brief description**

In `docs/opendog-feature-introduction.md`, find the section that says decision brief is exposed through "两个 MCP 工具" and update it to clarify that it's a mode of `get_guidance`:

Change (around line 69):
```
通过 `get_guidance` 和 `get_decision_brief` 两个 MCP 工具暴露
```
To:
```
通过 `get_guidance` MCP 工具暴露。`detail = "summary"` 返回工作区态势，`detail = "decision"` 返回单项目决策包。CLI 侧对应 `opendog agent-guidance` 和 `opendog decision-brief`
```

- [ ] **Step 2: Add MCP-only note to CLAUDE.md for verify_deletion_plan**

In `CLAUDE.md`, find the orphan scan section and add a note:

In the MCP Tools section, after `verify_deletion_plan` listing, add:
```
  (MCP-only; no CLI equivalent — use `opendog scan-orphans` + manual verification for CLI workflow)
```

- [ ] **Step 3: Commit**

```bash
git add docs/opendog-feature-introduction.md docs/mcp-tool-reference.md CLAUDE.md
git commit -m "docs: clarify decision brief routing and verify_deletion_plan MCP-only surface"
```
