# Repository Risk Strategy Coupling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a thin, read-only workspace repository-risk-to-strategy coupling layer so AI consumers can see which repository risk is reinforcing the current strategy posture.

**Architecture:** Reuse the already sorted top workspace recommendation, match it to the existing project overview, project its `repo_status_risk.highest_priority_finding` into a new `execution_strategy.risk_strategy_coupling` object, and tighten only the shared first recommended-flow sentence when a coupling exists. Do not touch project-level action logic or repository-risk detection.

**Tech Stack:** Rust 2021, serde_json, `cargo test`, OPENDOG Phase 6 guidance payload tests

---

### Task 1: Write The Failing Coupling Tests

**Files:**
- Modify: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Modify: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [x] **Step 1: Add a workspace guidance assertion block for `risk_strategy_coupling`**

Extend the existing workspace advice test so the inline `repo_status_risk` fixture includes:

```rust
"risk_findings": [{
    "kind": "working_tree_conflicted",
    "severity": "high",
    "priority": "immediate",
    "confidence": "high",
    "summary": "2 conflicted paths detected in the working tree."
}],
"highest_priority_finding": {
    "kind": "working_tree_conflicted",
    "severity": "high",
    "priority": "immediate",
    "confidence": "high",
    "summary": "2 conflicted paths detected in the working tree."
}
```

Then assert:

```rust
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["status"],
    json!("coupled")
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["source"],
    json!("primary_repo_risk_finding")
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["source_project_id"],
    json!("demo")
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["strategy_mode"],
    json!("verify_before_modify")
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["preferred_primary_tool"],
    json!("shell")
);
assert_eq!(
    value["guidance"]["layers"]["execution_strategy"]["risk_strategy_coupling"]["primary_repo_risk_finding"]["kind"],
    json!("working_tree_conflicted")
);
assert!(
    value["guidance"]["recommended_flow"][0]
        .as_str()
        .unwrap()
        .contains("2 conflicted paths detected in the working tree.")
);
```

- [x] **Step 2: Add a decision summary assertion using an inline overview with a primary repository-risk finding**

Inside `decision_brief_payload_exposes_unified_entry_envelope()`, mutate the local `project_overview` before payload construction:

```rust
project_overview["repo_status_risk"]["risk_findings"] = json!([{
    "kind": "working_tree_conflicted",
    "severity": "high",
    "priority": "immediate",
    "confidence": "high",
    "summary": "2 conflicted paths detected in the working tree."
}]);
project_overview["repo_status_risk"]["highest_priority_finding"] = json!({
    "kind": "working_tree_conflicted",
    "severity": "high",
    "priority": "immediate",
    "confidence": "high",
    "summary": "2 conflicted paths detected in the working tree."
});
```

Then assert:

```rust
assert!(
    brief["decision"]["summary"]
        .as_str()
        .unwrap()
        .contains("2 conflicted paths detected in the working tree.")
);
```

- [x] **Step 3: Run the focused tests to verify they fail**

Run:

```bash
cargo test workspace_advice decision_brief_payload_exposes_unified_entry_envelope -- --nocapture
```

Expected:

- FAIL because `execution_strategy.risk_strategy_coupling` does not exist yet
- FAIL because the shared first-step summary does not yet include the primary repository-risk finding

### Task 2: Implement Minimal Workspace Coupling

**Files:**
- Modify: `src/mcp/guidance_payload.rs`
- Modify: `src/mcp/strategy.rs`
- Test: `src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs`
- Test: `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

- [x] **Step 1: Add a helper that projects the top workspace repository risk into a coupling object**

In `src/mcp/guidance_payload.rs`, add a small helper that:

- reads the first sorted project recommendation
- finds the matching project overview by `project_id`
- reads `repo_status_risk.highest_priority_finding`
- returns:

```rust
json!({
    "status": "coupled",
    "source": "primary_repo_risk_finding",
    "source_project_id": project_id,
    "recommended_next_action": recommendation["recommended_next_action"].clone(),
    "strategy_mode": workspace_strategy["global_strategy_mode"].clone(),
    "preferred_primary_tool": workspace_strategy["preferred_primary_tool"].clone(),
    "primary_repo_risk_finding": primary_finding.clone(),
    "summary": format!(
        "Top repository risk keeps the workspace in {} mode and {}-first handling.",
        workspace_strategy["global_strategy_mode"].as_str().unwrap_or("current"),
        workspace_strategy["preferred_primary_tool"].as_str().unwrap_or("current")
    )
})
```

Fallback shape:

```rust
json!({
    "status": "no_repo_risk_signal",
    "source": Value::Null,
    "source_project_id": Value::Null,
    "recommended_next_action": recommendation["recommended_next_action"].clone(),
    "strategy_mode": workspace_strategy["global_strategy_mode"].clone(),
    "preferred_primary_tool": workspace_strategy["preferred_primary_tool"].clone(),
    "primary_repo_risk_finding": Value::Null,
    "summary": Value::Null,
})
```

- [x] **Step 2: Attach that helper output under `execution_strategy`**

In `agent_guidance_payload(...)`, compute:

```rust
let risk_strategy_coupling = execution_strategy_repo_risk_coupling(
    &sorted_project_recommendations,
    project_overviews,
    &workspace_strategy,
);
```

Then add:

```rust
"risk_strategy_coupling": risk_strategy_coupling.clone(),
```

to `guidance.layers.execution_strategy`.

- [x] **Step 3: Tighten only the first recommended-flow step when coupling exists**

In `src/mcp/strategy.rs`, add a helper like:

```rust
fn apply_repo_risk_context(first_step: String, risk_strategy_coupling: Option<&Value>) -> String
```

Behavior:

- if no coupled primary repository-risk finding exists, return `first_step` unchanged
- otherwise append:

```text
; top repository risk: <finding summary>
```

Use it only on the first step of the existing action-specific flow. Keep all other steps unchanged.

- [x] **Step 4: Thread the coupling object into `agent_guidance_recommended_flow(...)`**

Update the function signature to accept:

```rust
risk_strategy_coupling: Option<&Value>
```

and apply the new helper only to the first step before serializing the flow.

- [x] **Step 5: Run the focused tests to verify they pass**

Run:

```bash
cargo test workspace_advice decision_brief_payload_exposes_unified_entry_envelope -- --nocapture
```

Expected:

- PASS

### Task 3: Documentation And Full Verification

**Files:**
- Modify: `docs/json-contracts.md`
- Modify: `docs/mcp-tool-reference.md`
- Modify: `docs/superpowers/specs/2026-05-05-repo-risk-strategy-coupling-design.md`
- Modify: `docs/superpowers/plans/2026-05-05-repo-risk-strategy-coupling-implementation.md`

- [x] **Step 1: Add the new execution-strategy field to payload docs**

Add:

```text
guidance.layers.execution_strategy.risk_strategy_coupling
```

to the relevant field lists in:

- `docs/json-contracts.md`
- `docs/mcp-tool-reference.md`

Document that it is read-only explanatory metadata derived from the top workspace repository risk finding and does not override action selection.

- [x] **Step 2: Run formatting**

Run:

```bash
cargo fmt --check
```

Expected: PASS

- [x] **Step 3: Run full Rust regression**

Run:

```bash
cargo test
```

Expected: PASS

- [x] **Step 4: Run lint gate**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS

- [x] **Step 5: Run governance validation**

Run:

```bash
python3 scripts/validate_planning_governance.py
```

Expected: PASS

- [x] **Step 6: Mark spec/plan as implemented and summarize evidence**

Changed files should stay limited to:

```text
docs/json-contracts.md
docs/mcp-tool-reference.md
docs/superpowers/specs/2026-05-05-repo-risk-strategy-coupling-design.md
docs/superpowers/plans/2026-05-05-repo-risk-strategy-coupling-implementation.md
src/mcp/guidance_payload.rs
src/mcp/strategy.rs
src/mcp/tests/guidance_basics/workspace_guidance/workspace_advice.rs
src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs
```
