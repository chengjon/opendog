# Review: Runtime Weak-Token Data-Risk Design

Spec: `docs/superpowers/specs/2026-05-04-runtime-weak-token-data-risk-design.md`
Reviewer: Claude (GLM-5.1)
Date: 2026-05-04
Verdict: **Approve with minor revisions**

---

## Summary

The spec targets a real, confirmed false-positive path:

```
runtime_shared + weak path token (seed/demo/sample)
  -> mock candidate (path.mock_token)
  -> mixed_review_file (when hardcoded content also present)
  -> data_risk_focus flips to "mixed" or overstates urgency
```

The fix — split path tokens into strong/weak tiers, gate weak tokens by path classification — is the right level of complexity. It does not overreach into semantic analysis and preserves existing recall for genuine test/mock assets.

---

## Confirmed Against Code

| Spec claim | Code reality | Status |
|---|---|---|
| Flat `mock_path_tokens` array causes the problem | `mock_detection.rs:138-154` — single array, no tiering | Confirmed |
| `src/customer_seed.rs` becomes mock candidate via `path.mock_token` | `mock_detection.rs:204-212` — `path_lower.contains(token)` with no classification gate | Confirmed |
| `mixed_review_files` is same-file overlap of mock + hardcoded | `mock_detection.rs:328-359` — direct `any()` intersection check | Confirmed |
| Existing test asserts the false positive as correct behavior | `report_detection.rs:61` — `mock_candidates.len() == 3`; `report_detection.rs:110-113` — asserts `src/customer_seed.rs` IS in `mixed_review_files` | Confirmed — will break |

---

## Strengths

1. **Narrow scope, clear boundary.** The spec explicitly lists what it does NOT change (hardcoded detection, content tokens, schemas, CLI). This makes review and regression testing straightforward.

2. **Correct tiering logic.** Strong tokens (`mock`, `fixture`, `stub`, `fake`, `testdata`, `__fixtures__`) are genuinely test-intent markers. Weak tokens (`seed`, `demo`, `sample`) are commonly descriptive in production code. The split is defensible.

3. **Layered cleanup through input tightening.** Section 6 correctly avoids adding special `mixed_review_files` logic, instead letting it clean up naturally from upstream candidate reduction. This preserves the existing separation between detection and report assembly.

4. **No schema changes.** Downstream `data_risk_focus`, workspace aggregation, and guidance payloads only shift in values, not structure. This eliminates migration risk.

5. **Test strategy covers the right four scenarios.** The spec identifies the exact test matrix needed, and the existing test file (`report_detection.rs`) is the right place to add coverage.

---

## Issues and Suggestions

### 1. Gating condition for "additional stronger evidence" needs explicit definition

**Section 4** says weak tokens may contribute when "combined with stronger evidence already available in the current scan" but does not name the variable or check point.

In the current code flow (`mock_detection.rs:195-212`), `content_mock_keywords` from content matching is computed **before** the path-token block. The natural gating condition is:

```rust
// proposed: weak token only contributes when content mock tokens already matched
if has_weak_mock_path_token && path_classification != "runtime_shared" {
    // weak token counts
} else if has_weak_mock_path_token && !content_mock_keywords.is_empty() {
    // weak token counts because content evidence already present
}
```

**Recommendation:** Add one sentence to Section 4 specifying that `content_mock_keywords` (or equivalent existing content-evidence variable) is the gating signal. This removes ambiguity during implementation.

### 2. `unknown` path behavior may be status quo, not a change

**Section 3, point 3** says for `unknown` paths, "weak tokens alone are insufficient for mock classification." But this is already effectively true: unknown paths already get no path-classification boost, and without strong content evidence, they are marginal candidates anyway.

**Recommendation:** Clarify whether this is a behavioral change or an explicit statement of preserved behavior. If it is preserved, mark it as such. If it is a change, describe what currently happens for `unknown + weak token only` that needs to stop.

### 3. Existing test breakage should be called out more prominently

The spec's test strategy describes what to add, but does not explicitly call out that the existing test `detect_mock_data_report_distinguishes_mock_and_hardcoded_candidates` (report_detection.rs:4) **will break** in specific ways:

- Line 61: `assert_eq!(report.mock_candidates.len(), 3)` → will become `2`
- Lines 110-113: assertion that `src/customer_seed.rs` IS in `mixed_review_files` → must be removed or inverted

**Recommendation:** Add a "breaking test changes" subsection listing the specific assertions in `report_detection.rs` that must change, with before/after expected values. This prevents implementation surprise.

### 4. Token boundary matching is a latent weakness (out of scope, but note it)

Current code uses `path_lower.contains(token)`. This means `seed` matches `seedless`, `unseeded`, `overseeds`. Same for `demo` matching `demographic`, `stubs` matching `stubbornly`.

The spec correctly does not fix this (out of scope), but the weak/strong split is a good moment to acknowledge it.

**Recommendation:** Add a one-line note under "Out Of Scope" or "Future Consideration": word-boundary matching for path tokens. No action now.

### 5. Helper function placement is fine but could be more specific

The spec proposes three helpers in `src/mcp/mock_detection.rs`:

- `path_has_strong_mock_token`
- `path_has_weak_mock_token`
- `allow_weak_path_token_as_mock_signal`

These are private functions in the same module, which is correct. One minor point: the third helper takes `path_classification: &str` but the classification is already computed by `classify_path_kind()` at the call site. This is fine (dependency injection of classification), but the spec should note that the caller must use the same classification source consistently.

**Recommendation:** Minor — add a note that the caller passes the already-computed `path_classification` from `classify_path_kind()` to avoid double-classification.

### 6. Consider whether `generated_artifact` weak-token behavior is intentional

Section 3 says weak tokens "still count as valid path-based mock signals" for `test_only`, `generated_artifact`, and example-like paths. For `test_only` and examples, this is clearly correct.

For `generated_artifact` (e.g., `dist/demo_seed.json`), the current behavior is: file becomes a mock candidate with `confidence: "low"`, `priority: "low"`. After the change, this behavior is preserved. This seems reasonable — generated artifacts with demo/sample tokens are likely build outputs of test data and not worth suppressing.

**Recommendation:** No change needed. The existing test at `report_detection.rs:72-74` already validates this case.

---

## Minor Nits

1. **Spec line 169**: `src/mcp/mock_detection.rs` is correct as the primary implementation file. The existing `mock_path_tokens` array at lines 138-154 is exactly where the split should happen.

2. **Spec Section 7**: The list of preserved schemas is complete. Verified against `data_risk/report.rs` (`data_risk_focus`), `data_risk/guidance.rs` (guidance payload), and `data_risk/workspace.rs` (workspace aggregation).

3. **Spec mentions `FT-03.08.01` and `FT-03.08.02`**: Cross-referenced against `FUNCTION_TREE.md` — these FT IDs are the correct owners for mock/hardcoded candidate detection.

---

## Verdict

The spec is well-designed, correctly scoped, and grounded in the actual code structure. The four issues above are clarifications, not blockers. After addressing items 1-3 (explicit gating variable, unknown-path clarification, test breakage callout), this is ready for implementation.

Recommended priority: **item 3 (test breakage)** first, since it affects the first thing an implementer will encounter.
