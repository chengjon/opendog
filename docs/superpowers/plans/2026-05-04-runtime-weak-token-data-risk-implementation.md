# Runtime Weak-Token Data-Risk Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce false positives where runtime/shared source files are classified as mock or mixed-review risk only because their paths contain weak tokens such as `seed`, `demo`, or `sample`, while preserving current detection for real test/example mock assets.

**Architecture:** Keep `src/mcp/mock_detection.rs` as the only owner of weak-vs-strong path-token handling. Tighten mock candidate admission at the detection layer, let `mixed_review_files` and `data_risk_focus` become cleaner through upstream input changes, and avoid any schema or contract updates.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, existing MCP data-risk test modules

---

### Task 1: Tighten weak-token mock detection at the source

**Files:**
- Modify: `src/mcp/mock_detection.rs`
- Modify: `src/mcp/tests/data_risk_cases/report_detection.rs`

- [ ] **Step 1: Extend the detection tests with weak-token false-positive regressions**

```rust
// src/mcp/tests/data_risk_cases/report_detection.rs

#[test]
fn detect_mock_data_report_ignores_runtime_weak_path_tokens_without_mock_content() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/customer_seed.rs"),
        r#"const CUSTOMER: &str = "Acme Corp"; const EMAIL: &str = "ops@corp.com"; const ADDRESS: &str = "1 Market Street";"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/customer_seed.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 2,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
    assert_eq!(report.hardcoded_candidates.len(), 1);
    assert!(report.mixed_review_files.is_empty());
    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
}

#[test]
fn detect_mock_data_report_keeps_test_only_and_example_weak_tokens() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("tests/fixtures")).unwrap();
    std::fs::create_dir_all(dir.path().join("examples")).unwrap();
    std::fs::write(
        dir.path().join("tests/fixtures/demo.json"),
        r#"{"demo": true}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("examples/sample.json"),
        r#"{"sample": true}"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[
            StatsEntry {
                file_path: "tests/fixtures/demo.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
            StatsEntry {
                file_path: "examples/sample.json".to_string(),
                size: 10,
                file_type: "json".to_string(),
                access_count: 0,
                estimated_duration_ms: 0,
                modification_count: 0,
                last_access_time: None,
                first_seen_time: None,
            },
        ],
    );

    assert_eq!(report.mock_candidates.len(), 2);
    assert!(report
        .mock_candidates
        .iter()
        .all(|candidate| candidate.path_classification == "test_only"));
}

#[test]
fn detect_mock_data_report_allows_runtime_weak_tokens_with_strong_mock_content() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/demo_seed.rs"),
        r#"const FIXTURE_JSON: &str = "mock data fixture";"#,
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "src/demo_seed.rs".to_string(),
            size: 10,
            file_type: "rs".to_string(),
            access_count: 1,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert_eq!(report.mock_candidates.len(), 1);
    assert!(report.mock_candidates[0]
        .rule_hits
        .iter()
        .any(|hit| hit == "content.mock_token"));
}

#[test]
fn detect_mock_data_report_rejects_unknown_weak_tokens_without_mock_content() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("notes")).unwrap();
    std::fs::write(
        dir.path().join("notes/sample_notes.md"),
        "sample rollout notes for the onboarding flow",
    )
    .unwrap();

    let report = detect_mock_data_report(
        dir.path(),
        &[StatsEntry {
            file_path: "notes/sample_notes.md".to_string(),
            size: 10,
            file_type: "md".to_string(),
            access_count: 0,
            estimated_duration_ms: 0,
            modification_count: 0,
            last_access_time: None,
            first_seen_time: None,
        }],
    );

    assert!(report.mock_candidates.is_empty());
    assert!(report.hardcoded_candidates.is_empty());
    assert!(report.mixed_review_files.is_empty());
}
```

- [ ] **Step 2: Update the broad regression to reflect the new runtime weak-token behavior**

```rust
// src/mcp/tests/data_risk_cases/report_detection.rs

#[test]
fn detect_mock_data_report_distinguishes_mock_and_hardcoded_candidates() {
    // keep the existing fixture setup, but change the key expectations:
    // before: report.mock_candidates.len() == 3
    // after:  report.mock_candidates.len() == 2
    assert_eq!(report.mock_candidates.len(), 2);
    assert_eq!(report.hardcoded_candidates.len(), 1);
    // before: src/customer_seed.rs was asserted inside mixed_review_files
    // after:  weak runtime path tokens alone no longer create mock+hardcoded overlap
    assert!(report.mixed_review_files.is_empty());

    let rendered = report.to_value(5);
    assert_eq!(
        rendered["data_risk_focus"],
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
    assert!(report.mock_candidates.iter().any(|candidate| {
        candidate.path_classification == "generated_artifact" && candidate.review_priority == "low"
    }));
}
```

- [ ] **Step 3: Run the focused report-detection tests and confirm they fail**

Run: `cargo test detect_mock_data_report_ --lib`

Expected: FAIL because the current implementation still treats weak runtime path tokens as direct mock signals.

- [ ] **Step 4: Split strong and weak path tokens in `mock_detection.rs`**

```rust
// src/mcp/mock_detection.rs

fn path_has_any_token(path_lower: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| path_lower.contains(token))
}

fn allow_weak_path_token_as_mock_signal(
    path_classification: &str,
    has_content_mock_signal: bool,
) -> bool {
    matches!(path_classification, "test_only" | "generated_artifact") || has_content_mock_signal
}

pub(crate) fn detect_mock_data_report(root: &Path, entries: &[StatsEntry]) -> MockDataReport {
    let strong_mock_path_tokens = [
        "mock",
        "mocks",
        "fixture",
        "fixtures",
        "stub",
        "stubs",
        "fake",
        "fakes",
        "testdata",
        "__fixtures__",
    ];
    let weak_mock_path_tokens = ["seed", "seeds", "demo", "sample", "samples"];
    let mock_content_tokens = [
        "mock",
        "fixture",
        "stub",
        "fake",
        "sample data",
        "demo data",
        "seed data",
    ];

    // ...

    let content_mock_keywords = matched_keywords(&content_lower, &mock_content_tokens, 4);
    let has_content_mock_signal = !content_lower.is_empty() && !content_mock_keywords.is_empty();
    let strong_path_mock_hit = path_has_any_token(&path_lower, &strong_mock_path_tokens);
    let weak_path_mock_hit = path_has_any_token(&path_lower, &weak_mock_path_tokens);

    let mut mock_keywords = matched_keywords(&path_lower, &strong_mock_path_tokens, 4);
    if strong_path_mock_hit {
        mock_reasons
            .push("Path contains explicit mock/fixture/demo/test-data markers.".to_string());
        mock_evidence.push(entry.file_path.clone());
        mock_rule_hits.push("path.mock_token".to_string());
    }

    let weak_path_is_allowed =
        // path_classification must be the already-computed value from classify_path_kind()
        weak_path_mock_hit && allow_weak_path_token_as_mock_signal(path_classification, has_content_mock_signal);
    if weak_path_is_allowed {
        mock_reasons.push(
            "Path contains weak demo/sample/seed markers in a non-runtime or content-confirmed context."
                .to_string(),
        );
        mock_evidence.push(entry.file_path.clone());
        mock_rule_hits.push("path.mock_token".to_string());
        mock_keywords.extend(matched_keywords(&path_lower, &weak_mock_path_tokens, 4));
    }

    if has_content_mock_signal {
        mock_reasons
            .push("File content mentions mock/fixture/fake/sample data tokens.".to_string());
        mock_evidence.push(format!(
            "content token hit: {}",
            content_mock_keywords.join(", ")
        ));
        mock_rule_hits.push("content.mock_token".to_string());
        mock_keywords.extend(content_mock_keywords);
    }

    mock_keywords.sort();
    mock_keywords.dedup();

    // leave hardcoded detection unchanged
}
```

- [ ] **Step 5: Run the focused suite again and confirm the new behavior**

Run: `cargo test detect_mock_data_report_ --lib`

Expected: PASS with the new weak-token tests plus the existing broad regression passing.

Run: `cargo test mock_data_report_derives_ --lib`

Expected: PASS to confirm `data_risk_focus` still derives the same hardcoded/mixed/mock/none outcomes from the cleaned candidate sets.

- [ ] **Step 6: Commit the detection-tightening batch**

```bash
git add \
  src/mcp/mock_detection.rs \
  src/mcp/tests/data_risk_cases/report_detection.rs
git commit -m "feat: tighten weak runtime mock token detection"
```

### Task 2: Verify no contract drift and no broader regressions

**Files:**
- Modify: none expected
- Verify: `src/mcp/tests/data_risk_cases/report_detection.rs`
- Verify: `src/mcp/tests/data_risk_cases/single_project_guidance.rs`
- Verify: `src/mcp/tests/data_risk_cases/workspace_aggregation.rs`
- Verify: `docs/json-contracts.md`
- Verify: `docs/mcp-tool-reference.md`

- [ ] **Step 1: Run the broader data-risk suite**

Run: `cargo test data_risk_cases --lib`

Expected: PASS, including:
- project-level guidance tests
- workspace aggregation tests
- `data_risk_focus` regression tests

- [ ] **Step 2: Run full formatting, compile, and full tests**

Run: `cargo fmt --check`
Expected: PASS

Run: `cargo check`
Expected: PASS

Run: `cargo test`
Expected: PASS with all unit and integration tests green.

- [ ] **Step 3: Prove the contract docs did not need schema updates**

Run: `git diff --exit-code -- docs/json-contracts.md docs/mcp-tool-reference.md`

Expected: exit code `0` because this batch changes only detection values, not payload shape.

- [ ] **Step 4: Re-run planning governance validation**

Run: `python3 scripts/validate_planning_governance.py`

Expected: PASS with zero backlog and no structural hygiene regressions.

- [ ] **Step 5: Keep the Task 1 feature commit as the final implementation commit if no broader regressions require follow-up edits**

```bash
git log --oneline -1
# Expected:
# <sha> feat: tighten weak runtime mock token detection
```

If any Step 1-4 command exposes a real regression that needs code changes, fix it in the same files as Task 1 and create one final follow-up commit before branch handoff.
