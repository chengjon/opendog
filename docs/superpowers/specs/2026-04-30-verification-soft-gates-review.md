# Verification Soft Gates Design — Review

**Reviewed spec:** `docs/superpowers/specs/2026-04-30-verification-soft-gates-design.md`
**Reviewer:** Claude (GLM-5.1)
**Date:** 2026-04-30
**Verdict:** Sound design direction, needs 6 specific fixes before implementation

---

## 1. Named Functions — All Verified

All 7 named functions exist with the expected signatures:

| Function | Location | Match |
|----------|----------|-------|
| `verification_status_layer` | `src/mcp/verification_evidence.rs:13` | ✅ |
| `project_overview` | `src/mcp/project_recommendation.rs:18` | ✅ |
| `recommend_project_action` | `src/mcp/project_recommendation.rs:122` | ✅ |
| `stats_guidance` | `src/mcp/project_guidance/stats_unused/stats.rs:13` | ✅ |
| `unused_guidance` | `src/mcp/project_guidance/stats_unused/unused.rs:12` | ✅ |
| `data_risk_guidance` | `src/mcp/data_risk/guidance.rs:11` | ✅ |
| `workspace_verification_evidence_layer` | `src/mcp/verification_evidence.rs:115` | ✅ |

**Unmentioned function:** `project_readiness_reasons()` in `src/mcp/constraints.rs:38` — this is the key combiner that merges repo_risk and verification blockers. It is called from 5 locations and is central to the duplication problem. See Finding #1.

---

## 2. FT IDs and Requirement Families — Minor Discrepancy

FT IDs match `.planning/FUNCTION_TREE.md`:

| Spec Reference | FUNCTION_TREE.md Mapping | Match |
|---|---|---|
| FT-03.03.01 → EVID-01..04 | `requirement_ranges: [EVID-01..04]` | ✅ |
| FT-03.02.02 → STRAT-02..04 | `requirement_ranges: [STRAT-01..04]` | ⚠️ |
| FT-03.01.01 → OBS-02 | `requirement_ranges: [OBS-01..04]` | ✅ (valid subset) |
| FT-03.07.01 → BOUND-01..04 | `requirement_ranges: [BOUND-01..04]` | ✅ |

**Discrepancy:** The spec claims STRAT-02..04 but FT-03.02.02 maps to STRAT-01..04. Since the work touches `recommend_project_action()` which implements STRAT-01 (tool vs shell choice), the scope should include STRAT-01 or explain why it is excluded. See Finding #5.

---

## 3. Gate Rules — Matches Current Code, Caution Is New

Current `verification_status_layer()` logic at `verification_evidence.rs:23-25`:

```rust
safe_for_cleanup = all_passed && recorded_kinds.contains(&"test");
safe_for_refactor = all_passed && recorded_kinds.contains(&"test") && recorded_kinds.contains(&"build");
```

Current `verification_gate_reasons()` at `verification_evidence.rs:347-380` already blocks on:

- missing test → blocks both cleanup and refactor
- missing build → blocks refactor only
- any failure → blocks both
- stale evidence → blocks both

**This matches the spec's proposed gate rules exactly.** The spec accurately describes what already exists.

**The genuinely new part is the `caution` level** (advisory kinds missing/stale while required kinds pass). This does not exist in the current code — the system is binary safe/blocked today.

---

## 4. Implementation Order — Dependency Gap in Steps 3–4

The 5-step order is mostly sound, but has a concrete problem at Step 4.

### Current state of `stats_guidance()` and `unused_guidance()`

Both functions:

- Call `project_readiness_reasons(repo_risk, verification_layer, "cleanup"/"refactor")` directly
- Independently recompute `safe_for_cleanup`, `safe_for_refactor`, `cleanup_blockers`, `refactor_blockers`
- Neither receives a `project_overview` Value as input

### Problem

The spec says they should "read project-level readiness instead of inventing local gate logic" but does not address the signature changes needed. These functions have no path to receive `gate_assessment` data without changing their signatures or restructuring how they get called.

### Also unmentioned

`build_constraints_boundaries_layer()` in `constraints.rs:110` also calls `project_readiness_reasons()` internally and emits its own `cleanup_blockers`/`refactor_blockers`. This is another site of duplication the spec does not address.

---

## 5. Findings and Gaps

### FINDING 1 — MEDIUM: `project_readiness_reasons()` is the real duplication hub

**Spec says:** only `verification_status_layer` computes verification gate state.

**Reality:** `project_readiness_reasons()` in `constraints.rs:38` is the function that actually combines repo_risk + verification blockers into unified readiness reasons. It is called from:

- `project_overview()` (`project_recommendation.rs:26-27`)
- `recommend_project_action()` (`project_recommendation.rs:133-134`)
- `stats_guidance()` (`stats.rs:101-104`)
- `unused_guidance()` (`unused.rs:52-55`)
- `build_constraints_boundaries_layer()` (`constraints.rs:119-129`)

The spec's consumer order section should reference this function explicitly and explain what happens to it after the refactor. Without this, the "single source of truth" claim cannot be achieved — callers will still bypass `verification_status_layer` by calling `project_readiness_reasons` directly.

### FINDING 2 — LOW: `data_risk_guidance()` does not have gate logic to remove

**Spec says:** data_risk_guidance should "read project-level readiness instead of inventing local gate logic."

**Reality:** `data_risk_guidance()` (`guidance.rs:11`) takes only `(root_path, report)` — it does not compute `safe_for_cleanup` or `safe_for_refactor` at all. There is no local gate logic to refactor away. This line in the spec describes a non-existent problem.

### FINDING 3 — MEDIUM: `caution` → boolean derivation is undefined

The spec proposes a 3-level gate (`blocked` / `caution` / `allow`) but the existing boolean fields `safe_for_cleanup` / `safe_for_refactor` can only express 2 states. The spec says "old fields stay semantically stable because they are derived from the new gate model" but never specifies the derivation rule:

| `gate_assessment.level` | `safe_for_cleanup` / `safe_for_refactor` | Reasoning |
|---|---|---|
| `blocked` | `false` | Clear — required evidence missing, failing, or stale |
| `allow` | `true` | Clear — all required and advisory evidence fresh and passing |
| `caution` | ??? | Required passes, but advisory is missing/stale |

At `caution` level, the boolean should be `true` (because the required gate passed), but this needs to be stated explicitly in the spec. Otherwise implementers may disagree.

### FINDING 4 — LOW: Staleness threshold not referenced

The spec says "stale evidence blocks" but does not define or reference the staleness threshold. Current code uses `verification_is_stale()` from `observation.rs` with a specific time window. The spec should either:

- Reference the existing threshold explicitly, or
- State that the threshold is outside the scope of this design

### FINDING 5 — LOW: STRAT-01 scope exclusion unexplained

FT-03.02.02 maps to STRAT-01..04 but the spec claims only STRAT-02..04. Since `recommend_project_action` implements STRAT-01 behavior (choosing between OPENDOG tools vs shell commands), either:

- Include STRAT-01 in the scope, or
- Add a note explaining why STRAT-01 is excluded from this change

### FINDING 6 — MEDIUM: `build_constraints_boundaries_layer()` is not mentioned

This function (`constraints.rs:110`) independently computes `cleanup_blockers` and `refactor_blockers` by calling `project_readiness_reasons()`. It is used by both `stats_guidance()` and `unused_guidance()` to build the `constraints_boundaries` layer in their output payloads.

If the spec's goal is to eliminate duplicated gate logic, this function is part of the duplication chain and should be addressed in the implementation order. Otherwise Step 4 ("update stats / unused / data-risk guidance to consume project-level readiness") cannot be completed without also refactoring how these functions build their constraints layer.

---

## 6. Summary

| Aspect | Verdict |
|--------|---------|
| Function existence | ✅ All 7 verified |
| FT ID mapping | ⚠️ STRAT-01 exclusion unexplained |
| Gate rules accuracy | ✅ Matches current code exactly |
| `caution` level concept | ✅ Genuinely new, sound addition |
| Implementation order | ⚠️ Step 4 has signature-change gap |
| Compatibility claim | ⚠️ `caution` → boolean derivation undefined |
| Completeness | ⚠️ Missing `project_readiness_reasons`, `build_constraints_boundaries_layer`, `data_risk_guidance` has no gate logic |

### Recommendation

**Approve with revisions.** Address Findings 1, 3, and 6 before implementation begins — they affect the actual refactoring scope and backward compatibility contract.

Specifically:

1. Add `project_readiness_reasons()` and `build_constraints_boundaries_layer()` to the spec's consumer order section with explicit "before/after" behavior
2. State the derivation rule: `caution → safe = true` (or `false` with justification)
3. Remove or correct the `data_risk_guidance` claim (it has no gate logic to refactor)
4. Consider adding a Step 3.5: refactor `build_constraints_boundaries_layer()` to accept pre-computed gate data instead of recomputing it

---

*Review completed 2026-04-30.*
