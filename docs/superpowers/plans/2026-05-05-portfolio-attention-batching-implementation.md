# Portfolio Attention Batching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a thin, read-only `attention_batches` projection above the existing workspace attention queue so AI consumers can process the current top portfolio window in `immediate / next / later` batches.

**Architecture:** Keep batching entirely inside `workspace_portfolio_layer(...)` by slicing the already sorted and truncated `attention_queue`. Reuse existing `attention_score`, `attention_band`, and `recommended_next_action` fields without changing any ranking, recommendation, or orchestration logic. `agent_guidance_payload(...)` should pick up the new field through the existing portfolio-layer handoff without new batching logic of its own.

**Tech Stack:** Rust 2021, serde_json, `cargo test`, OPENDOG Phase 6 portfolio guidance tests

---

### Task 1: Write The Failing Attention-Batching Tests

**Files:**
- Modify: `src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs`
- Modify: `src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs`
- Test: `src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs`
- Test: `src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs`

- [x] **Step 1: Add a reusable overview helper and the golden-path batching test**

Create a small helper at the top of `src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs`:

```rust
fn portfolio_overview(
    project_id: &str,
    action: &str,
    snapshot_status: &str,
    verification_status: &str,
    has_failing_verification: bool,
) -> serde_json::Value {
    json!({
        "project_id": project_id,
        "unused_files": 1,
        "recommended_next_action": action,
        "observation": {
            "coverage_state": if snapshot_status == "missing" {
                "missing_snapshot"
            } else {
                "ready"
            },
            "freshness": {
                "snapshot": {"status": snapshot_status},
                "activity": {"status": "fresh"},
                "verification": {"status": if verification_status == "not_recorded" { "missing" } else { "fresh" }}
            }
        },
        "mock_data_summary": {
            "hardcoded_candidate_count": 0,
            "mock_candidate_count": 0
        },
        "verification_evidence": {
            "status": verification_status,
            "failing_runs": if has_failing_verification {
                json!([{"kind": "test"}])
            } else {
                json!([])
            }
        },
        "repo_status_risk": {
            "status": "available",
            "risk_level": "low",
            "is_dirty": false,
            "operation_states": []
        },
        "safe_for_cleanup": true,
        "safe_for_refactor": true
    })
}
```

Then add a golden-path test in the same file:

```rust
#[test]
fn workspace_portfolio_layer_batches_attention_queue_into_immediate_next_and_later() {
    let value = workspace_portfolio_layer(&[
        portfolio_overview("alpha", "review_failing_verification", "fresh", "available", true),
        portfolio_overview("beta", "run_verification_before_high_risk_changes", "fresh", "not_recorded", false),
        portfolio_overview("gamma", "take_snapshot", "missing", "available", false),
        portfolio_overview("delta", "start_monitor", "fresh", "available", false),
        portfolio_overview("epsilon", "review_unused_files", "fresh", "available", false),
        portfolio_overview("zeta", "inspect_hot_files", "fresh", "available", false),
    ]);

    assert_eq!(value["attention_batches"]["status"], json!("available"));
    assert_eq!(value["attention_batches"]["source"], json!("attention_queue"));
    assert_eq!(value["attention_batches"]["batched_project_count"], json!(5));
    assert_eq!(value["attention_batches"]["unbatched_project_count"], json!(1));
    assert_eq!(value["attention_batches"]["immediate"][0]["project_id"], json!("alpha"));
    assert_eq!(value["attention_batches"]["next"][0]["project_id"], json!("beta"));
    assert_eq!(value["attention_batches"]["next"][1]["project_id"], json!("gamma"));
    assert_eq!(value["attention_batches"]["later"][0]["project_id"], json!("delta"));
    assert_eq!(value["attention_batches"]["later"][1]["project_id"], json!("epsilon"));
}
```

- [x] **Step 2: Add the four short-queue edge tests**

In the same file, add four explicit edge tests:

```rust
#[test]
fn workspace_portfolio_layer_batches_empty_queue_safely() {
    let value = workspace_portfolio_layer(&[]);

    assert_eq!(value["attention_batches"]["batched_project_count"], json!(0));
    assert_eq!(value["attention_batches"]["unbatched_project_count"], json!(0));
    assert_eq!(value["attention_batches"]["immediate"], json!([]));
    assert_eq!(value["attention_batches"]["next"], json!([]));
    assert_eq!(value["attention_batches"]["later"], json!([]));
}

#[test]
fn workspace_portfolio_layer_batches_single_project_into_immediate_only() {
    let value = workspace_portfolio_layer(&[
        portfolio_overview("alpha", "inspect_hot_files", "fresh", "available", false),
    ]);

    assert_eq!(value["attention_batches"]["immediate"][0]["project_id"], json!("alpha"));
    assert_eq!(value["attention_batches"]["next"], json!([]));
    assert_eq!(value["attention_batches"]["later"], json!([]));
}

#[test]
fn workspace_portfolio_layer_batches_two_projects_into_immediate_and_next() {
    let value = workspace_portfolio_layer(&[
        portfolio_overview("alpha", "review_unused_files", "fresh", "available", false),
        portfolio_overview("beta", "inspect_hot_files", "fresh", "available", false),
    ]);

    assert_eq!(value["attention_batches"]["immediate"][0]["project_id"], json!("alpha"));
    assert_eq!(value["attention_batches"]["next"][0]["project_id"], json!("beta"));
    assert_eq!(value["attention_batches"]["later"], json!([]));
}

#[test]
fn workspace_portfolio_layer_batches_three_projects_into_one_two_zero() {
    let value = workspace_portfolio_layer(&[
        portfolio_overview("alpha", "take_snapshot", "missing", "available", false),
        portfolio_overview("beta", "start_monitor", "fresh", "available", false),
        portfolio_overview("gamma", "inspect_hot_files", "fresh", "available", false),
    ]);

    assert_eq!(value["attention_batches"]["immediate"][0]["project_id"], json!("alpha"));
    assert_eq!(value["attention_batches"]["next"][0]["project_id"], json!("beta"));
    assert_eq!(value["attention_batches"]["next"][1]["project_id"], json!("gamma"));
    assert_eq!(value["attention_batches"]["later"], json!([]));
}
```

- [x] **Step 3: Extend the existing agent-guidance portfolio test to assert immediate matches the top recommendation**

In `src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs`, keep the current two-project setup and add:

```rust
assert_eq!(
    value["guidance"]["layers"]["multi_project_portfolio"]["attention_batches"]["immediate"][0]
        ["project_id"],
    json!("alpha")
);
assert_eq!(
    value["guidance"]["layers"]["multi_project_portfolio"]["attention_batches"]["immediate"][0]
        ["project_id"],
    value["guidance"]["layers"]["multi_project_portfolio"]["priority_candidates"][0]["project_id"]
);
```

- [x] **Step 4: Run the focused portfolio tests to verify they fail**

Run:

```bash
cargo test workspace_portfolio -- --nocapture
```

Expected:

- FAIL because `attention_batches` does not exist yet under `multi_project_portfolio`

### Task 2: Implement The Thin Batching Projection

**Files:**
- Modify: `src/mcp/attention.rs`
- Verify only: `src/mcp/guidance_payload.rs`
- Test: `src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs`
- Test: `src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs`

- [x] **Step 1: Add a helper that trims an attention item down to the batch-entry shape**

In `src/mcp/attention.rs`, above `workspace_portfolio_layer(...)`, add:

```rust
fn attention_batch_entry(item: &Value) -> Value {
    json!({
        "project_id": item["project_id"].clone(),
        "recommended_next_action": item["recommended_next_action"].clone(),
        "attention_score": item["attention_score"].clone(),
        "attention_band": item["attention_band"].clone(),
    })
}
```

- [x] **Step 2: Add a helper that slices the current queue into `immediate / next / later`**

In the same file, add:

```rust
fn attention_batches(attention_queue: &[Value], project_count: usize, status: &str) -> Value {
    let batched_project_count = attention_queue.len();
    let unbatched_project_count = project_count.saturating_sub(batched_project_count);

    json!({
        "status": status,
        "source": "attention_queue",
        "batched_project_count": batched_project_count,
        "unbatched_project_count": unbatched_project_count,
        "immediate": attention_queue
            .iter()
            .take(1)
            .map(attention_batch_entry)
            .collect::<Vec<_>>(),
        "next": attention_queue
            .iter()
            .skip(1)
            .take(2)
            .map(attention_batch_entry)
            .collect::<Vec<_>>(),
        "later": attention_queue
            .iter()
            .skip(3)
            .map(attention_batch_entry)
            .collect::<Vec<_>>(),
    })
}
```

- [x] **Step 3: Attach the new batching object inside `workspace_portfolio_layer(...)`**

Still in `src/mcp/attention.rs`, keep the current sort and truncate logic unchanged, then add:

```rust
let status = "available";
let attention_batches = attention_batches(&attention_queue, project_overviews.len(), status);
```

and extend the returned payload:

```rust
json!({
    "status": status,
    "project_count": project_overviews.len(),
    "priority_model": "action_urgency_plus_evidence_risk",
    "dirty_projects": dirty_projects,
    "high_risk_projects": high_risk_projects,
    "projects_with_failing_verification": projects_with_failing_verification,
    "projects_safe_for_cleanup": projects_safe_for_cleanup,
    "projects_safe_for_refactor": projects_safe_for_refactor,
    "projects_with_hardcoded_candidates": projects_with_hardcoded_candidates,
    "total_mock_candidates": total_mock_candidates,
    "total_hardcoded_candidates": total_hardcoded_candidates,
    "projects_in_operation": projects_in_operation,
    "attention_queue": attention_queue,
    "attention_batches": attention_batches,
})
```

- [x] **Step 4: Verify `agent_guidance_payload(...)` already carries the field through**

Read `src/mcp/guidance_payload.rs` and confirm that:

```rust
value["guidance"]["layers"]["multi_project_portfolio"] =
    workspace_portfolio_layer(project_overviews);
```

already preserves any new portfolio-layer fields before the monitoring and recommendation overlays are spliced back in.

If that is still true, do not modify `src/mcp/guidance_payload.rs`.

- [x] **Step 5: Run the focused portfolio tests to verify they pass**

Run:

```bash
cargo test workspace_portfolio -- --nocapture
```

Expected:

- PASS for the golden-path batching test
- PASS for all four short-queue edge tests
- PASS for the priority-candidate / immediate-batch alignment assertion

### Task 3: Document The New Portfolio Projection And Run Full Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-portfolio-attention-batching-design.md`
- Modify: `docs/superpowers/plans/2026-05-05-portfolio-attention-batching-implementation.md`

- [x] **Step 1: Add the new field to the payload docs**

In `docs/json-contracts.md`, add:

```text
guidance.layers.multi_project_portfolio.attention_batches
guidance.layers.multi_project_portfolio.attention_batches.{batched_project_count,unbatched_project_count}
guidance.layers.multi_project_portfolio.attention_batches.{immediate,next,later}[*].{project_id,recommended_next_action,attention_score,attention_band}
```

In `docs/mcp-tool-reference.md`, add the same field family to the useful response-field list for `get_guidance(detail=summary)`.

Document it as:

- read-only
- derived from `attention_queue`
- a batching projection, not a scheduling engine

- [x] **Step 2: Mark the spec as implemented after code and verification land**

Update:

```text
docs/superpowers/specs/2026-05-05-portfolio-attention-batching-design.md
```

from:

```text
Status: approved for implementation (2026-05-05)
```

to:

```text
Status: implemented and verified (2026-05-05)
```

- [x] **Step 3: Run formatting**

Run:

```bash
cargo fmt --check
```

Expected: PASS

- [x] **Step 4: Run full Rust regression**

Run:

```bash
cargo test
```

Expected: PASS

- [x] **Step 5: Run lint gate**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS

- [x] **Step 6: Run governance validation**

Run:

```bash
python3 scripts/validate_planning_governance.py
```

Expected: PASS

- [x] **Step 7: Summarize the final changed-file set**

The finished batch should stay limited to:

```text
docs/json-contracts.md
docs/mcp-tool-reference.md
docs/superpowers/specs/2026-05-05-portfolio-attention-batching-design.md
docs/superpowers/plans/2026-05-05-portfolio-attention-batching-implementation.md
src/mcp/attention.rs
src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs
src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs
```
