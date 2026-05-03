# Data-Risk Focus Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add machine-readable `data_risk_focus` for mock, hardcoded, and mixed-review findings, then project it through project payloads, workspace guidance, `agent_guidance`, and `decision_brief` without changing detection heuristics.

**Architecture:** Keep `MockDataReport` as the canonical source by adding `data_risk_focus(&self) -> Value` in `src/mcp/data_risk/report.rs`. Project and workspace payloads render that object directly; `agent_guidance` and `decision_brief` only consume rendered focus data from `mock_data_summary` or workspace data-risk guidance.

**Tech Stack:** Rust, `serde_json`, Cargo unit/integration tests, Markdown docs

---

### Task 1: Canonical `MockDataReport::data_risk_focus`

**Files:**
- Modify: `src/mcp/data_risk/report.rs`
- Modify: `src/mcp/tests/data_risk_cases/report_detection.rs`

- [ ] **Step 1: Write the failing data-risk focus derivation tests**

```rust
// src/mcp/tests/data_risk_cases/report_detection.rs

#[test]
fn mock_data_report_derives_hardcoded_focus_from_runtime_shared_high_severity_hits() {
    let report = super::MockDataReport {
        mock_candidates: vec![],
        hardcoded_candidates: vec![super::DataCandidate {
            file_path: "src/customer_seed.rs".to_string(),
            confidence: "high",
            review_priority: "high",
            path_classification: "runtime_shared",
            rule_hits: vec![
                "path.runtime_shared".to_string(),
                "content.business_literal_combo".to_string(),
            ],
            matched_keywords: vec!["customer".to_string(), "email".to_string()],
            reasons: vec!["hardcoded".to_string()],
            evidence: vec!["runtime".to_string()],
            access_count: 1,
            file_type: "rs".to_string(),
        }],
        mixed_review_files: vec!["src/customer_seed.rs".to_string()],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "hardcoded",
            "priority_order": ["hardcoded", "mixed", "mock"],
            "basis": [
                "hardcoded_candidates_present",
                "mixed_review_files_present",
                "runtime_shared_candidates_present",
                "high_severity_content_hits_present"
            ]
        })
    );
}

#[test]
fn mock_data_report_derives_mixed_focus_when_mixed_files_exist_without_hardcoded_dominance() {
    let report = super::MockDataReport {
        mock_candidates: vec![super::DataCandidate {
            file_path: "src/demo.rs".to_string(),
            confidence: "medium",
            review_priority: "high",
            path_classification: "unknown",
            rule_hits: vec!["path.mock_token".to_string()],
            matched_keywords: vec!["demo".to_string()],
            reasons: vec!["mock".to_string()],
            evidence: vec!["demo".to_string()],
            access_count: 0,
            file_type: "rs".to_string(),
        }],
        hardcoded_candidates: vec![super::DataCandidate {
            file_path: "src/demo.rs".to_string(),
            confidence: "medium",
            review_priority: "medium",
            path_classification: "unknown",
            rule_hits: vec![],
            matched_keywords: vec!["customer".to_string()],
            reasons: vec!["mixed".to_string()],
            evidence: vec!["mixed".to_string()],
            access_count: 0,
            file_type: "rs".to_string(),
        }],
        mixed_review_files: vec!["src/demo.rs".to_string()],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "mixed",
            "priority_order": ["mixed", "hardcoded", "mock"],
            "basis": ["mixed_review_files_present"]
        })
    );
}

#[test]
fn mock_data_report_derives_mock_focus_when_only_mock_candidates_exist() {
    let report = super::MockDataReport {
        mock_candidates: vec![super::DataCandidate {
            file_path: "tests/fixtures/demo.json".to_string(),
            confidence: "high",
            review_priority: "medium",
            path_classification: "test_only",
            rule_hits: vec!["path.test_only".to_string()],
            matched_keywords: vec!["demo".to_string()],
            reasons: vec!["mock".to_string()],
            evidence: vec!["fixture".to_string()],
            access_count: 0,
            file_type: "json".to_string(),
        }],
        hardcoded_candidates: vec![],
        mixed_review_files: vec![],
    };

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "mock",
            "priority_order": ["mock", "hardcoded", "mixed"],
            "basis": ["mock_candidates_present"]
        })
    );
}

#[test]
fn mock_data_report_derives_none_focus_when_no_candidates_exist() {
    let report = super::MockDataReport::default();

    assert_eq!(
        report.data_risk_focus(),
        json!({
            "primary_focus": "none",
            "priority_order": [],
            "basis": ["no_candidates_detected"]
        })
    );
}
```

- [ ] **Step 2: Run the focused derivation tests and confirm they fail**

Run: `cargo test mock_data_report_derives_ --lib`

Expected: FAIL because `MockDataReport` does not yet expose `data_risk_focus()`.

- [ ] **Step 3: Add the canonical focus helper and render it in `to_value()`**

```rust
// src/mcp/data_risk/report.rs
fn candidate_has_rule(candidates: &[DataCandidate], rule: &str) -> bool {
    candidates
        .iter()
        .any(|candidate| candidate.rule_hits.iter().any(|hit| hit == rule))
}

impl MockDataReport {
    pub(crate) fn data_risk_focus(&self) -> Value {
        let hardcoded = !self.hardcoded_candidates.is_empty();
        let mock = !self.mock_candidates.is_empty();
        let mixed = !self.mixed_review_files.is_empty();
        let runtime_shared = candidate_has_rule(&self.hardcoded_candidates, "path.runtime_shared");
        let high_content =
            candidate_has_rule(&self.hardcoded_candidates, "content.business_literal_combo");

        let (primary_focus, priority_order, basis) = if hardcoded && (mixed || runtime_shared || high_content) {
            let mut basis = vec!["hardcoded_candidates_present"];
            if mixed {
                basis.push("mixed_review_files_present");
            }
            if runtime_shared {
                basis.push("runtime_shared_candidates_present");
            }
            if high_content {
                basis.push("high_severity_content_hits_present");
            }
            ("hardcoded", json!(["hardcoded", "mixed", "mock"]), json!(basis))
        } else if mixed {
            ("mixed", json!(["mixed", "hardcoded", "mock"]), json!(["mixed_review_files_present"]))
        } else if mock {
            ("mock", json!(["mock", "hardcoded", "mixed"]), json!(["mock_candidates_present"]))
        } else {
            ("none", json!([]), json!(["no_candidates_detected"]))
        };

        json!({
            "primary_focus": primary_focus,
            "priority_order": priority_order,
            "basis": basis,
        })
    }

    pub(crate) fn to_value(&self, limit: usize) -> Value {
        json!({
            "mock_candidate_count": self.mock_candidates.len(),
            "hardcoded_candidate_count": self.hardcoded_candidates.len(),
            "mixed_review_file_count": self.mixed_review_files.len(),
            "data_risk_focus": self.data_risk_focus(),
            "rule_groups_summary": self.rule_groups_summary(),
            "rule_hits_summary": self.rule_hits_summary(),
            "mock_data_candidates": self.mock_candidates.iter().take(limit).map(data_candidate_value).collect::<Vec<_>>(),
            "hardcoded_data_candidates": self.hardcoded_candidates.iter().take(limit).map(data_candidate_value).collect::<Vec<_>>(),
            "mixed_review_files": self.mixed_review_files.iter().take(limit).cloned().collect::<Vec<_>>(),
        })
    }
}
```

- [ ] **Step 4: Run the focused derivation tests and existing detection regression tests**

Run: `cargo test mock_data_report_derives_ --lib`
Expected: PASS with 4 tests passing.

Run: `cargo test detect_mock_data_report_distinguishes_mock_and_hardcoded_candidates --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/data_risk/report.rs \
  src/mcp/tests/data_risk_cases/report_detection.rs
git commit -m "feat: add data risk focus helper"
```

### Task 2: Project Data-Risk Payload And Guidance Projection

**Files:**
- Modify: `src/mcp/data_risk/guidance.rs`
- Modify: `src/mcp/tests/data_risk_cases/single_project_guidance.rs`

- [ ] **Step 1: Write the failing project-level payload and guidance tests**

```rust
// src/mcp/tests/data_risk_cases/single_project_guidance.rs

// extend the existing hardcoded fixture setup from
// `data_risk_guidance_surfaces_counts_and_candidates()`
let payload = project_data_risk_payload(
    MCP_PROJECT_V1,
    "demo",
    "all",
    "low",
    10,
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

assert_eq!(
    guidance["layers"]["cleanup_refactor_candidates"]["data_risk_focus"]["primary_focus"],
    json!("hardcoded")
);
assert_eq!(
    payload["data_risk_focus"],
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
assert_eq!(
    payload["guidance"]["layers"]["cleanup_refactor_candidates"]["data_risk_focus"],
    payload["data_risk_focus"]
);
```

- [ ] **Step 2: Run the focused project-level tests and confirm they fail**

Run: `cargo test data_risk_guidance_and_payload_include_data_risk_focus --lib`

Expected: FAIL because project data-risk output does not yet expose `data_risk_focus`.

- [ ] **Step 3: Mirror the canonical focus into project payload and project guidance**

```rust
// src/mcp/data_risk/guidance.rs
pub(crate) fn data_risk_guidance(root_path: &Path, report: &MockDataReport) -> Value {
    let boundary_hints = common_boundary_hints(root_path);
    let rendered = report.to_value(10);
    let focus = rendered["data_risk_focus"].clone();
    let summary = match focus["primary_focus"].as_str().unwrap_or("none") {
        "hardcoded" => {
            "Hardcoded data candidates detected. Review runtime-shared files before cleanup or refactor work."
        }
        "mock" => {
            "Mock-style data candidates detected. Confirm whether they are test-only artifacts before acting on cleanup suggestions."
        }
        _ => {
            "No mock or hardcoded data candidates were detected in the current snapshot-derived file set."
        }
    };

    guidance["layers"]["cleanup_refactor_candidates"] = json!({
        "status": "available",
        "data_risk_focus": focus,
        "mock_data_candidates": rendered["mock_data_candidates"].clone(),
        "hardcoded_data_candidates": rendered["hardcoded_data_candidates"].clone(),
        "mixed_review_files": rendered["mixed_review_files"].clone(),
    });
}
```

```rust
// src/mcp/data_risk/guidance.rs
pub(crate) fn project_data_risk_payload(...) -> Value {
    let report = detect_mock_data_report(root_path, entries);
    let filtered = report.filtered(candidate_type, Some(min_review_priority));
    let rendered = filtered.to_value(limit.max(1));
    versioned_project_payload(
        schema_version,
        id,
        [
            ("candidate_type", json!(candidate_type)),
            ("min_review_priority", json!(min_review_priority)),
            ("mock_candidate_count", rendered["mock_candidate_count"].clone()),
            ("hardcoded_candidate_count", rendered["hardcoded_candidate_count"].clone()),
            ("mixed_review_file_count", rendered["mixed_review_file_count"].clone()),
            ("data_risk_focus", rendered["data_risk_focus"].clone()),
            ("rule_groups_summary", rendered["rule_groups_summary"].clone()),
            ("rule_hits_summary", rendered["rule_hits_summary"].clone()),
            ("mock_data_candidates", rendered["mock_data_candidates"].clone()),
            ("hardcoded_data_candidates", rendered["hardcoded_data_candidates"].clone()),
            ("mixed_review_files", rendered["mixed_review_files"].clone()),
            ("guidance", data_risk_guidance(root_path, &filtered)),
        ],
    )
}
```

- [ ] **Step 4: Run the focused project-level tests and the existing project guidance regression**

Run: `cargo test data_risk_guidance_and_payload_include_data_risk_focus --lib`
Expected: PASS.

Run: `cargo test data_risk_guidance_surfaces_counts_and_candidates --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/data_risk/guidance.rs \
  src/mcp/tests/data_risk_cases/single_project_guidance.rs
git commit -m "feat: project data risk focus"
```

### Task 3: Workspace Aggregation, `agent_guidance`, And `decision_brief`

**Files:**
- Modify: `src/mcp/data_risk/workspace.rs`
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/workspace_decision.rs`
- Modify: `src/mcp/tests/data_risk_cases/workspace_aggregation.rs`
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs`

- [ ] **Step 1: Write the failing workspace, guidance, and decision projection tests**

```rust
// src/mcp/tests/data_risk_cases/workspace_aggregation.rs

// add `data_risk_focus` to the two existing project summaries
assert_eq!(
    payload["layers"]["workspace_observation"]["data_risk_focus_distribution"],
    json!({
        "hardcoded": 1,
        "mixed": 0,
        "mock": 1,
        "none": 0
    })
);
assert_eq!(
    payload["layers"]["workspace_observation"]["projects_requiring_hardcoded_review"],
    json!(1)
);
assert_eq!(
    payload["layers"]["multi_project_portfolio"]["priority_projects"][0]["data_risk_focus"]["primary_focus"],
    json!("hardcoded")
);
```

```rust
// src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs

// extend existing overview inputs with `mock_data_summary.data_risk_focus`
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["data_risk_focus_distribution"],
    json!({
        "hardcoded": 1,
        "mixed": 0,
        "mock": 1,
        "none": 0
    })
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"],
    json!(1)
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["projects_requiring_mock_review"],
    json!(1)
);
```

```rust
// src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs

// extend a decision-brief test with these assertions
assert_eq!(
    brief["decision"]["data_risk_focus"],
    json!({
        "primary_focus": "hardcoded",
        "priority_order": ["hardcoded", "mixed", "mock"],
        "basis": [
            "hardcoded_candidates_present",
            "mixed_review_files_present",
            "high_severity_content_hits_present"
        ]
    })
);
assert_eq!(brief["decision"]["signals"]["mixed_review_file_count"], json!(1));
assert_eq!(
    brief["layers"]["workspace_observation"]["data_risk_focus_distribution"],
    json!({
        "hardcoded": 1,
        "mixed": 0,
        "mock": 0,
        "none": 0
    })
);
```

- [ ] **Step 2: Run the focused projection tests and confirm they fail**

Run: `cargo test workspace_data_risk_overview_payload_exposes_focus_distribution --lib`
Expected: FAIL.

Run: `cargo test agent_guidance_summarizes_data_risk_focus_distribution --lib`
Expected: FAIL.

Run: `cargo test decision_brief_payload_projects_selected_data_risk_focus --lib`
Expected: FAIL.

- [ ] **Step 3: Add workspace focus aggregation and guidance/decision projection**

```rust
// src/mcp/data_risk/workspace.rs
fn data_risk_focus_distribution(project_summaries: &[Value]) -> Value {
    let mut counts = json!({"hardcoded": 0_u64, "mixed": 0_u64, "mock": 0_u64, "none": 0_u64});
    for summary in project_summaries {
        if let Some(focus) = summary["data_risk_focus"]["primary_focus"].as_str() {
            counts[focus] = json!(counts[focus].as_u64().unwrap_or(0) + 1);
        }
    }
    counts
}
```

```rust
// src/mcp/data_risk/workspace.rs
let focus_distribution = data_risk_focus_distribution(project_summaries);
let projects_requiring_hardcoded_review =
    focus_distribution["hardcoded"].as_u64().unwrap_or(0);
let projects_requiring_mixed_file_review =
    focus_distribution["mixed"].as_u64().unwrap_or(0);
let projects_requiring_mock_review =
    focus_distribution["mock"].as_u64().unwrap_or(0);

guidance["layers"]["workspace_observation"]["data_risk_focus_distribution"] =
    focus_distribution.clone();
guidance["layers"]["workspace_observation"]["projects_requiring_hardcoded_review"] =
    json!(projects_requiring_hardcoded_review);
guidance["layers"]["workspace_observation"]["projects_requiring_mock_review"] =
    json!(projects_requiring_mock_review);
guidance["layers"]["workspace_observation"]["projects_requiring_mixed_file_review"] =
    json!(projects_requiring_mixed_file_review);

guidance["layers"]["execution_strategy"]["data_risk_focus_distribution"] =
    focus_distribution.clone();
guidance["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"] =
    json!(projects_requiring_hardcoded_review);
guidance["layers"]["execution_strategy"]["projects_requiring_mock_review"] =
    json!(projects_requiring_mock_review);
guidance["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"] =
    json!(projects_requiring_mixed_file_review);
```

```rust
// src/mcp/guidance_payload.rs
fn execution_strategy_data_risk_focus_summary(project_overviews: &[Value]) -> Value {
    let mut distribution = json!({"hardcoded": 0_u64, "mixed": 0_u64, "mock": 0_u64, "none": 0_u64});
    for overview in project_overviews {
        if let Some(focus) = overview["mock_data_summary"]["data_risk_focus"]["primary_focus"].as_str() {
            distribution[focus] = json!(distribution[focus].as_u64().unwrap_or(0) + 1);
        }
    }

    json!({
        "data_risk_focus_distribution": distribution,
        "projects_requiring_hardcoded_review": distribution["hardcoded"].clone(),
        "projects_requiring_mixed_file_review": distribution["mixed"].clone(),
        "projects_requiring_mock_review": distribution["mock"].clone(),
    })
}
```

```rust
// src/mcp/guidance_payload.rs
let data_risk_summary = execution_strategy_data_risk_focus_summary(project_overviews);
value["guidance"]["layers"]["execution_strategy"]["data_risk_focus_distribution"] =
    data_risk_summary["data_risk_focus_distribution"].clone();
value["guidance"]["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"] =
    data_risk_summary["projects_requiring_hardcoded_review"].clone();
value["guidance"]["layers"]["execution_strategy"]["projects_requiring_mock_review"] =
    data_risk_summary["projects_requiring_mock_review"].clone();
value["guidance"]["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"] =
    data_risk_summary["projects_requiring_mixed_file_review"].clone();
```

```rust
// src/mcp/workspace_decision.rs
"decision": {
    // existing fields
    "data_risk_focus": matched_overview["mock_data_summary"]["data_risk_focus"].clone(),
    "signals": {
        // existing fields
        "mixed_review_file_count": matched_overview["mock_data_summary"]["mixed_review_file_count"]
            .as_u64()
            .unwrap_or(0),
    }
}

// inside the workspace_data_guidance merge block
layers["workspace_observation"]["data_risk_focus_distribution"] =
    risk_observation["data_risk_focus_distribution"].clone();
layers["workspace_observation"]["projects_requiring_hardcoded_review"] =
    risk_observation["projects_requiring_hardcoded_review"].clone();
layers["workspace_observation"]["projects_requiring_mock_review"] =
    risk_observation["projects_requiring_mock_review"].clone();
layers["workspace_observation"]["projects_requiring_mixed_file_review"] =
    risk_observation["projects_requiring_mixed_file_review"].clone();
layers["execution_strategy"]["data_risk_focus_distribution"] =
    data_risk_guidance["layers"]["execution_strategy"]["data_risk_focus_distribution"].clone();
layers["execution_strategy"]["projects_requiring_hardcoded_review"] =
    data_risk_guidance["layers"]["execution_strategy"]["projects_requiring_hardcoded_review"].clone();
layers["execution_strategy"]["projects_requiring_mock_review"] =
    data_risk_guidance["layers"]["execution_strategy"]["projects_requiring_mock_review"].clone();
layers["execution_strategy"]["projects_requiring_mixed_file_review"] =
    data_risk_guidance["layers"]["execution_strategy"]["projects_requiring_mixed_file_review"].clone();
```

```rust
// src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs
"mock_data_summary": {
    "hardcoded_candidate_count": 1,
    "mock_candidate_count": 2,
    "mixed_review_file_count": 1,
    "data_risk_focus": {
        "primary_focus": "hardcoded",
        "priority_order": ["hardcoded", "mixed", "mock"],
        "basis": [
            "hardcoded_candidates_present",
            "mixed_review_files_present",
            "high_severity_content_hits_present"
        ]
    }
}
```

- [ ] **Step 4: Run the focused projection tests and nearby regressions**

Run: `cargo test workspace_data_risk_overview_payload_ --lib`
Expected: PASS for the workspace aggregation tests.

Run: `cargo test agent_guidance_summarizes_data_risk_focus_distribution --lib`
Expected: PASS.

Run: `cargo test decision_brief_payload_projects_selected_data_risk_focus --lib`
Expected: PASS.

Run: `cargo test data_risk_cases --lib`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add \
  src/mcp/data_risk/workspace.rs \
  src/mcp/guidance_payload.rs \
  src/mcp/workspace_decision.rs \
  src/mcp/tests/data_risk_cases/workspace_aggregation.rs \
  src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs \
  src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs \
  src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope/fixtures.rs
git commit -m "feat: project data risk focus through guidance"
```

### Task 4: Contract Docs And Final Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`

- [ ] **Step 1: Update JSON contract documentation**

```markdown
<!-- docs/json-contracts.md -->

- `data_risk_focus.primary_focus`
- `data_risk_focus.priority_order`
- `data_risk_focus.basis`

- `projects[*].data_risk_focus`

- `guidance.layers.workspace_observation.data_risk_focus_distribution`
- `guidance.layers.workspace_observation.projects_requiring_hardcoded_review`
- `guidance.layers.workspace_observation.projects_requiring_mock_review`
- `guidance.layers.workspace_observation.projects_requiring_mixed_file_review`

- `guidance.layers.execution_strategy.data_risk_focus_distribution`
- `guidance.layers.execution_strategy.projects_requiring_hardcoded_review`
- `guidance.layers.execution_strategy.projects_requiring_mock_review`
- `guidance.layers.execution_strategy.projects_requiring_mixed_file_review`

- `decision.data_risk_focus`
- `decision.signals.mixed_review_file_count`
```

- [ ] **Step 2: Update MCP tool reference documentation**

```markdown
<!-- docs/mcp-tool-reference.md -->

`get_data_risk_candidates`
- `data_risk_focus.primary_focus`: `none | mock | hardcoded | mixed`
- `data_risk_focus.priority_order`: stable review-family order for this project
- `data_risk_focus.basis`: stable machine-readable reasons for the selected focus

`get_workspace_data_risk_overview`
- `projects[*].data_risk_focus`
- `guidance.layers.workspace_observation.data_risk_focus_distribution`
- `guidance.layers.workspace_observation.projects_requiring_hardcoded_review`
- `guidance.layers.workspace_observation.projects_requiring_mock_review`
- `guidance.layers.workspace_observation.projects_requiring_mixed_file_review`

`get_agent_guidance` / `get_decision_brief`
- execution-strategy and decision payloads now surface the same focus interpretation without changing candidate counts
```

- [ ] **Step 3: Run formatting, full tests, and governance validation**

Run: `cargo fmt --check`
Expected: PASS.

Run: `cargo check`
Expected: PASS.

Run: `cargo test`
Expected: PASS with the full unit and integration suite green.

Run: `python3 scripts/validate_planning_governance.py`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add \
  docs/json-contracts.md \
  docs/mcp-tool-reference.md
git commit -m "docs: document data risk focus"
```

- [ ] **Step 5: Final branch verification**

Run: `git status --short`
Expected: no tracked-file changes remaining before branch completion.
