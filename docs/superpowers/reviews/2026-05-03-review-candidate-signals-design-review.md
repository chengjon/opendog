# Review: Review Candidate Signals Design

Spec: `docs/superpowers/specs/2026-05-03-review-candidate-signals-design.md`
Date: 2026-05-03
Verdict: Solid design, two structural issues to resolve before implementation

---

## Strengths

### 1. High alignment with actual codebase state

The spec accurately describes the current weakness in `stats.rs:103-124` and `unused.rs:53-68`. Both files hand-build `file_recommendations[*]` with broad prose `reason` strings and no shared machine-readable vocabulary. The proposed `candidate_basis`, `candidate_risk_hints`, and `candidate_priority` fields are exactly what downstream consumers need to parse "why this file?" and "is it safe to act on?" reliably.

### 2. Correct action scope

The two families — `inspect_hot_files` and `review_unused_files` — are the only review-facing actions in `project_recommendation.rs:389,426`. The spec does not invent new actions or redefine existing ones. The rule that `review_focus = null` for all non-review actions is correct.

### 3. Target layer already exists

The `cleanup_refactor_candidates` layer already exists in both `stats.rs:137` and `unused.rs:78`. The spec proposes enriching the candidates inside that layer, not creating a parallel surface. This avoids surface duplication.

### 4. Well-separated concerns

The split between recommendation-level `review_focus` (which family) and candidate-level `candidate_basis`/`candidate_risk_hints` (per-file detail) is clean. Recommendation stays stateless — it picks the family, guidance owns the files.

### 5. Vocabulary maps to existing signals

The `candidate_basis` vocabulary items (`highest_access_activity`, `zero_recorded_access`, `activity_present`, `snapshot_present`) map directly to fields already present in `RecommendationSignals` from `eligibility.rs:21-33`. No new data collection required.

---

## Issues

### Critical: `candidate_risk_hints` vocabulary overlaps with existing signal fields

The spec lists six risk hints:

```
cleanup_gate_caution, cleanup_gate_blocked,
refactor_gate_caution, refactor_gate_blocked,
repo_risk_elevated, activity_evidence_stale, snapshot_evidence_stale
```

But `cleanup_gate_level` and `refactor_gate_level` already exist as structured values (`allow`/`caution`/`blocked`) at:

- Recommendation payloads (`verification_gate_levels`, `cleanup_blockers`, `refactor_blockers`)
- Guidance layers (`execution_strategy.cleanup_gate_level`, `execution_strategy.refactor_gate_level`)
- `project_readiness_snapshot(...)` output

Encoding gate state as separate string-array hints (`cleanup_gate_caution` + `cleanup_gate_blocked`) rebuilds the same decision in a less structured form. Consumers would have to scan the array to infer the gate level instead of reading the single field that already exists.

**Suggested fix:** Either:

- **Drop the gate hints entirely** from `candidate_risk_hints` since gate levels are already in the parent layer. Keep only the observation-freshness and repo-risk hints (`repo_risk_elevated`, `activity_evidence_stale`, `snapshot_evidence_stale`).
- Or use a single `gate_state` hint derived from the existing `GateLevel`, e.g. `"candidate_risk_hints": ["gate_caution", "repo_risk_elevated"]`.

This avoids semantic duplication and stays consistent with the spec's own principle of "not duplicating what is already exposed."

### Medium: Spec does not explain how `mock_data_overlap` / `hardcoded_data_overlap` relates to the existing report

Section 6 states:

> If mock or hardcoded-data overlap already exists for a candidate file, that overlap may appear in `candidate_basis`

But `detect_mock_data_report(root_path, entries)` already produces `mock_data_candidates` and `hardcoded_data_candidates` per guidance layer (`stats.rs:101`, `unused.rs:52`). The spec does not explain how per-candidate overlap detection works:

- Is it matching file paths against the existing `mock_data_candidates` array?
- Or is it a separate re-scan of file content?

Since `detect_mock_data_report` already produces per-file candidates, the spec should state whether `build_review_candidate` receives that report as input, or whether overlap detection is new logic. This affects the helper signature and performance (avoiding double scans).

**Suggested fix:** Add one sentence to Section 4 or 6 clarifying that `build_review_candidate` receives the existing `detect_mock_data_report` output and matches by file path. Something like:

> `build_review_candidate` should receive the already-computed mock-data report and match candidate file paths against `mock_data_candidates` and `hardcoded_data_candidates` arrays. This avoids re-scanning file content inside the helper.

### Minor: Helper signature is underspecified

Section 4 describes `build_review_candidate` as taking:

> input: candidate kind, file path, priority, local evidence hints, readiness/freshness signals, repo risk, suggested commands

But "local evidence hints" and "readiness/freshness signals" are abstract. The implementation needs to know whether these are:

- Raw `RecommendationSignals` (currently only the recommendation layer has these; guidance does not)
- Pre-computed values from `project_readiness_snapshot(...)` (already available in guidance)
- A simpler subset

The spec should state which type the helper accepts so implementors know whether to thread `RecommendationSignals` into the guidance code path (where they are currently unavailable) or derive hints independently.

**Suggested fix:** State that the helper accepts `&Value` from `project_readiness_snapshot`, since guidance paths already produce this. Something like:

> `build_review_candidate` receives the readiness snapshot (`&Value` from `project_readiness_snapshot(...)`) and derives risk hints from its `safe_for_cleanup`, `safe_for_refactor`, `cleanup_gate_level`, and `refactor_gate_level` fields. This avoids requiring raw `RecommendationSignals` in guidance code paths.

---

## Suggestions

1. **Simplify the risk-hints vocabulary.** Remove `cleanup_gate_caution`/`cleanup_gate_blocked` and `refactor_gate_caution`/`refactor_gate_blocked` since gate levels are already in the parent layer. Keep only observation hints (`repo_risk_elevated`, `activity_evidence_stale`, `snapshot_evidence_stale`).

2. **Clarify per-candidate mock/hardcoded overlap.** State whether `build_review_candidate` receives the existing `detect_mock_data_report` output and matches by file path, or whether overlap detection is new logic.

3. **Specify helper input types.** State that `build_review_candidate` accepts `&Value` from `project_readiness_snapshot`, since guidance paths already produce this.

4. **Consider whether `advisory` priority should also convey gate state.** Section 3 says `candidate_priority` reflects queue position, not safety. But if a candidate has `advisory` priority under `cleanup_gate_level = "blocked"`, consumers may still treat it as actionable. Consider clarifying whether `advisory` also implies "gate-blocked, informational only" (not just "lower in the list"), or whether priority is strictly independent of gate state.

---

## Summary

The spec is precise, well-scoped, and correctly identifies real weaknesses in `stats.rs` and `unused.rs`. The candidate-signal vocabulary is well-designed. The two structural issues — risk-hint overlap with existing gate-level fields, and unspecified mock/hardcoded overlap mechanism — should be resolved before implementation planning. Everything else is implementable as-is.
