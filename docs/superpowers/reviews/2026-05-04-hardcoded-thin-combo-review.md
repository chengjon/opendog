# Review: Hardcoded Thin-Combo Data-Risk Design

Spec: `docs/superpowers/specs/2026-05-04-hardcoded-thin-combo-data-risk-design.md`
Reviewer: Claude (GLM-5.1)
Date: 2026-05-04
Verdict: **Request revisions — one blocking design gap**

---

## Summary

The spec targets a real false-positive path in `detect_mock_data_report()`: thin combinations of business keywords + literal markers (`business_hits == 2, literal_hits == 1`) promote ordinary config defaults and lightweight runtime constants into `hardcoded_candidates`. The dense/thin split concept is sound. However, there is a **blocking contradiction** between Sections 4 and 5 around the `path.runtime_shared` independent entry path that must be resolved before implementation.

---

## Confirmed Against Code

| Spec claim | Code reality | Status |
|---|---|---|
| Current threshold: `business_hits >= 2 && literal_hits >= 1` creates `content.business_literal_combo` | `mock_detection.rs:259-272` — exact match | Confirmed |
| `path.runtime_shared` triggers on `business_hits >= 2` with **no** literal requirement | `mock_detection.rs:281-288` — `runtime_path && business_hits >= 2`, no `literal_hits` check | Confirmed — more permissive than spec states |
| `path.runtime_shared` is an independent hardcoded entry path, not just an amplifier | `mock_detection.rs:281-288` — adds to `hardcoded_reasons` independently of the `content.business_literal_combo` block | Confirmed |
| `config/` paths are classified as `runtime_shared` | `mock_detection.rs:83-87` — `path_is_runtime_shared` includes `"config/"` | Confirmed |
| Existing test `src/customer_seed.rs` has dense content (3 business, 2 literal) | Content: `"Acme Corp"`, `"ops@corp.com"`, `"1 Market Street"` — hits: customer, email, address + @, street = dense combo | Confirmed — test won't break |

---

## Blocking Issue: Section 4 vs Section 5 Contradiction on `path.runtime_shared`

This is the most important finding in the review.

### The problem

The current code has **two independent paths** into `hardcoded_candidates`:

1. **`content.business_literal_combo`** (lines 259-272): triggers on `!test_only && !generated && business_hits >= 2 && literal_hits >= 1`
2. **`path.runtime_shared`** (lines 281-288): triggers on `!test_only && runtime_path && business_hits >= 2` — **no literal_hits check at all**

These are independent. A file can become a hardcoded candidate through `path.runtime_shared` alone, even if `content.business_literal_combo` never fires.

### The contradiction

**Section 4, case 3** says:

> `runtime_shared` paths with only a thin combo (`business_hits == 2, literal_hits == 1`) should NOT become hardcoded candidates.

**Section 5** says:

> Tighten `path.runtime_shared` so it requires `literal_hits >= 1`.

But a thin combo **by definition** has `literal_hits >= 1`. So after Section 5's tightening, a thin combo on a runtime path would still pass the `path.runtime_shared` gate (`runtime_path && business_hits >= 2 && literal_hits >= 1`), and the file would still become a hardcoded candidate through this independent path — exactly what Section 4 says should stop happening.

### Resolution options

**Option A: Make `path.runtime_shared` a pure amplifier.** Only fire it when `content.business_literal_combo` has already fired for the same file. This means:
- Thin combos: no `content.business_literal_combo` → no `path.runtime_shared` → no hardcoded candidate
- Dense combos: `content.business_literal_combo` fires → `path.runtime_shared` amplifies → hardcoded candidate with stronger confidence

This is the cleanest model but changes the semantics of `path.runtime_shared` from independent signal to dependent amplifier.

**Option B: Raise the `path.runtime_shared` literal bar.** Require `literal_hits >= 2` instead of `>= 1` for runtime paths. This means:
- Thin combos (`literal_hits == 1`): blocked
- Dense combos (`literal_hits >= 2`): pass

This keeps `path.runtime_shared` as an independent entry but raises its threshold to match the dense-combo bar for runtime paths.

**Option C: Add a unified thin-combo gate before either rule fires.** Compute `is_thin_combo = (business_hits < 3 && literal_hits < 2)` early, then skip both `content.business_literal_combo` and `path.runtime_shared` for thin combos on runtime/unknown paths.

**Recommendation:** Option A is the cleanest because it matches the spec's stated intent that `path.runtime_shared` is "amplification." It also aligns with the existing `data_risk_focus()` logic in `report.rs:24`, which treats `path.runtime_shared` as one of several amplifying basis keys rather than a standalone trigger. But the spec should explicitly choose one and document it.

---

## Non-Blocking Issues

### 1. `config/` is already `runtime_shared` — Section 4 cases 1 and 3 overlap

The spec lists these as separate cases:

- Case 1: `config/` or config-like paths with thin combo → no hardcoded
- Case 3: `runtime_shared` paths with thin combo → no hardcoded

But `path_is_runtime_shared()` (line 83) already includes `"config/"`. So `config/defaults.toml` is classified as `runtime_shared`, and cases 1 and 3 are the same code path.

**Recommendation:** Either acknowledge they collapse into a single check, or propose a new path classification (e.g., `config_default`) if you genuinely want different behavior for `config/` vs `src/`. For this batch, collapsing is simpler and the spec's intent is preserved either way.

### 2. Dense universal threshold (`business_hits >= 3 && literal_hits >= 2`) is aggressive

The `literal_markers` list (lines 164-179) has 14 entries but many are narrow: `"customer_id"`, `"tenant_id"`, `"invoice_no"`, `"postal"`, `"cny"`. Hitting 2 distinct literal markers requires fairly specific content. Meanwhile, the `business_keywords` list has 15 entries, so hitting 3 is moderate.

A file at an `unknown` path with `customer`, `email`, `address` (3 business) and `@`, `street` (2 literal) would pass. But a file with `customer`, `email` (2 business) and `@`, `city` (2 literal) would NOT pass the universal gate — even though it has 2 literal hits, it only has 2 business hits.

**Recommendation:** Consider adding an alternative dense threshold for non-runtime paths:

```
universal dense: business_hits >= 3 && literal_hits >= 2
      OR: business_hits >= 2 && literal_hits >= 3
```

Or keep the current single threshold but document that the universal gate primarily catches files with high keyword density, and runtime paths catch the `business >= 2, literal >= 2` case. This is a judgment call — the spec's current threshold is defensible if the intent is to be conservative.

### 3. Interaction with the weak-token spec should be documented

Both specs modify `detect_mock_data_report()` in `src/mcp/mock_detection.rs`. They are mostly independent (one gates mock path tokens, the other gates hardcoded combos), but both affect `mixed_review_files` through different entry paths.

The combined effect could be larger than either spec alone:
- Weak-token spec removes mock candidates for runtime + weak path token
- Thin-combo spec removes hardcoded candidates for thin combos
- Together, they could eliminate most `mixed_review_files` for ordinary runtime source code

**Recommendation:** Add a brief note (one paragraph) stating that both specs are commutative (implementable in any order) but their combined downstream effect on `data_risk_focus` stability should be verified as a unit after both land.

### 4. `data_risk_focus` depends on `path.runtime_shared` as a basis key

`report.rs:24` checks `candidate_has_rule(&self.hardcoded_candidates, "path.runtime_shared")` as one of the amplifying conditions for `"hardcoded"` primary focus. If `path.runtime_shared` stops firing for thin combos, some projects may lose their `"hardcoded"` focus and drop to `"mock"` or `"none"`.

This is likely the correct outcome (the focus was inflated), but it's a behavioral change the spec should explicitly acknowledge.

**Recommendation:** Add to Section 6: note that `data_risk_focus.primary_focus` may shift from `"hardcoded"` to `"mixed"` or `"mock"` for projects where the only hardcoded amplification came from thin-combo `path.runtime_shared` hits. This is desirable, not a regression.

### 5. Existing test won't break, but new test fixtures need careful construction

The existing test file `src/customer_seed.rs` has dense content (3 business, 2 literal), so it passes both old and new thresholds. No assertions break.

But the new test cases need content that hits exactly `business_hits == 2, literal_hits == 1` to validate the thin-combo suppression. For example:

- `config/defaults.toml` with `"support_email"` and `"default_city"` — need to verify `business_keywords` actually match the intended terms
- `literal_markers` contains `"phone"` and `business_keywords` also contains `"phone"` — this overlap could cause unexpected double-counting

**Recommendation:** When writing tests, use keywords from `business_keywords` and `literal_markers` that are clearly distinct (e.g., business: `customer`, `order`; literal: `@`, `usd`) to avoid cross-list contamination in the count logic.

---

## Minor Nits

1. **Section 3, dense combo #2**: `runtime_shared && business_hits >= 2 && literal_hits >= 2` — this is the right threshold, but note it's tighter than the current universal bar (`business >= 2, literal >= 1`). The runtime-specific dense bar is `literal >= 2` vs the old `literal >= 1`. This is the core of the fix and is correct.

2. **Helper signatures**: `is_strong_hardcoded_combo(path_lower, business_hits, literal_hits)` — the `path_lower` parameter is only needed for path classification, which is already computed at the call site. Consider passing `path_classification: &str` instead to avoid re-classification. Same pattern as the weak-token spec's `allow_weak_path_token_as_mock_signal`.

3. **Spec doesn't mention changes to `DATA_RISK_RULES`**: No changes needed there (the rule metadata stays the same; only when it fires changes). But worth a one-liner confirming this.

---

## Verdict

The spec identifies a genuine false-positive path and the dense/thin conceptual split is the right approach. However, **the Section 4 vs Section 5 contradiction on `path.runtime_shared` must be resolved first** — without that, the implementation will either not achieve the spec's stated goal (thin runtime combos still enter via independent `path.runtime_shared`) or will need to make a design decision not documented in the spec.

**Recommended priority for revisions:**
1. Resolve the `path.runtime_shared` contradiction (blocking)
2. Clarify `config/` overlap with `runtime_shared` (minor)
3. Document combined effect with weak-token spec (minor)
4. Acknowledge `data_risk_focus` focus-shift effect (minor)
