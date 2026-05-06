# Review: Data-Risk Focus Design

**Spec:** `docs/superpowers/specs/2026-05-03-data-risk-focus-design.md`
**Reviewer:** Claude (GLM-5.1)
**Date:** 2026-05-03
**Verdict:** Approved with 4 medium issues to resolve before implementation

---

## Accuracy: Confirmed

All claims about current codebase state are correct:

- `data_risk_focus` does not exist yet (zero matches in `src/`)
- `workspace_priority_reason` returns prose strings, not structured focus (`workspace.rs:141-172`)
- All referenced function signatures, field names, and module paths match the source
- FT IDs map to the correct `FUNCTION_TREE.md` nodes
- Requirement families `STRAT-01..04`, `BOUND-01..04`, `MOCK-01..10` are correctly attributed

---

## Issues

### M1. `basis` vocabulary incomplete for focus derivation rules

**Severity:** Medium
**Sections:** 4 and 5

Section 5 defines three independent conditions that make `hardcoded` win:

1. `mixed_review_file_count > 0`
2. any hardcoded candidate hits `path.runtime_shared`
3. any hardcoded candidate hits `content.business_literal_combo`

But section 4's vocabulary has only one composite key `runtime_shared_high_severity_present`. If hardcoded candidates exist with `path.runtime_shared` but without `content.business_literal_combo`, no single basis key covers that case.

**Fix:** Split `runtime_shared_high_severity_present` into two independent keys:

- `runtime_shared_candidates_present`
- `high_severity_content_hits_present`

Or explicitly state that the composite key fires when either or both conditions hold.

---

### M2. `data_risk_focus_distribution` shape unspecified

**Severity:** Medium
**Section:** 8

The spec says workspace guidance should add `data_risk_focus_distribution` but never defines its type. Consumers need this to parse the field.

**Fix:** Add an explicit shape, e.g.:

```json
{
  "data_risk_focus_distribution": {
    "hardcoded": 2,
    "mixed": 0,
    "mock": 1,
    "none": 3
  }
}
```

---

### M3. "Can mirror" in section 7 is ambiguous

**Severity:** Medium
**Section:** 7

> `guidance.layers.cleanup_refactor_candidates` **can** mirror `data_risk_focus`

"Can" is not testable. For a spec, every surface should have a clear "add this field" or "do not add this field."

**Fix:** Replace with a concrete instruction:

> `guidance.layers.cleanup_refactor_candidates` **must** include the already-derived `data_risk_focus` object.

---

### M4. Canonical derivation site unspecified

**Severity:** Medium
**Sections:** 2, 5, 7

The spec defines what to derive and from what, but not which function owns the derivation logic. Without this, implementers may inline the logic in multiple places.

**Fix:** Specify a single canonical function, e.g.:

> A new function `derive_data_risk_focus(report: &MockDataReport) -> Value` in `src/mcp/data_risk/guidance.rs` is the sole derivation site. `project_data_risk_payload` and `data_risk_guidance` both call it; they must not recompute focus independently.

---

### L1. `mixed` focus `priority_order` spans overlapping categories

**Severity:** Low
**Section:** 6

For `mixed` focus, the order is `["mixed", "hardcoded", "mock"]`. Mixed files already appear in both hardcoded and mock candidate sets. Telling a consumer "review mixed files first, then hardcoded" may double-count the same files.

**Fix:** Add a note clarifying these are review-attention domains, not disjoint file sets. Or clarify that mixed files should be excluded from the subsequent hardcoded/mock buckets in the priority-order interpretation.

---

### L2. Contract version impact not mentioned

**Severity:** Low
**Sections:** 2, 3, 9

The spec adds new fields to four versioned payloads. `contracts.rs` tracks these shapes. The spec should note whether a contract version bump is required or whether the additions are backward-compatible (extra fields under existing schema version).

---

### L3. Test placement guidance missing

**Severity:** Low
**Section:** Testing

Existing test structure is:

```
src/mcp/tests/data_risk_cases/
  report_detection.rs
  parameter_normalization.rs
  single_project_guidance.rs
  workspace_aggregation.rs
```

The spec lists three test layers but does not map them to files. Adding a placement hint would save implementation judgment.

**Suggested mapping:**

| Test layer | File |
|---|---|
| Focus derivation | New `focus_derivation.rs` or extend `report_detection.rs` |
| Workspace aggregation | Extend `workspace_aggregation.rs` |
| Guidance/decision projection | Extend `single_project_guidance.rs` + relevant decision-brief fixture |

---

## Non-issues (confirmed sound)

- "Keep detection stable" claim (section 1) is credible -- design only reads from `MockDataReport`, never rescans
- Focus priority ordering (section 5) matches existing product logic in `workspace_priority_reason`: runtime-shared > mixed > mock
- Static `priority_order` per focus (section 6) is a correct simplification
- "Out of scope" boundaries are clear and defensive
- No overlap with `detect_mock_data_report` internals; no second heuristic engine introduced

---

## Summary

The spec is precise, well-scoped, and faithful to the codebase. Resolving the 4 medium issues (basis vocabulary completeness, distribution shape, explicit mirror semantics, canonical derivation function) will bring it to implementation-ready state without requiring major rework.
