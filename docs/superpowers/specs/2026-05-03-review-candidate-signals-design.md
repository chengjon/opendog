# Review Candidate Signals Design

Date: 2026-05-03
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.05.01` so OPENDOG exposes stable machine-readable reasons for cleanup and refactor review candidates instead of relying mostly on freeform `reason` prose inside `stats` and `unused` guidance.

The target is intentionally narrow:

- keep the current `recommended_next_action` enum unchanged
- keep current action-level scoring private
- add only a small candidate-level machine-readable surface
- make `stats_guidance(...)` and `unused_guidance(...)` consume shared candidate-signal vocabulary instead of hand-written local explanations

This is candidate-signal hardening, not a new ranking engine and not a schema expansion across every guidance surface.

## Capability Scope

FT IDs touched:

- `FT-03.01.01` Explain readiness and evidence gaps
- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.05.01` Surface cleanup and refactor candidates
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `OBS-01..04`
- `STRAT-01..04`
- `CLEAN-01..04`
- `BOUND-01..04`

`FT-03.05.01` remains the owner of file-level cleanup and refactor review candidates. This batch consumes existing readiness, freshness, and repository-risk facts rather than widening evidence collection.

## Current Problem

OPENDOG already decides whether the next safer review action is `review_unused_files` or `inspect_hot_files`, but candidate-level reasoning is still too ad hoc.

Current weaknesses:

- `stats_guidance(...)` and `unused_guidance(...)` each hand-build `file_recommendations[*]` with broad prose and no shared machine-readable reason fields
- hotspot candidates can still look too aggressive under stale activity or caution-level refactor gates because the payload does not make those caveats explicit per candidate
- unused candidates still look too thin because "no recorded access" is exposed as prose instead of a stable candidate basis
- recommendation payloads know which review family won, but they do not yet expose a small machine-readable summary of what kind of candidate should be reviewed first

This is most visible when the action choice is reasonable, but downstream consumers still cannot reliably answer two follow-up questions:

- why is this file category worth reviewing first
- what makes it advisory-only instead of a direct cleanup or refactor permission

## Design

### 1. Keep Action Selection Stable

This work does not change:

- `recommended_next_action`
- existing recommendation-level eligibility and scoring ownership
- existing `reason`, `recommended_flow`, and `confidence`
- `decision_brief` and `agent_guidance` schemas in this batch

The public change is narrower:

- recommendation payloads gain a small action-level `review_focus`
- `stats_guidance(...)` and `unused_guidance(...)` gain richer candidate objects

### 2. Add A Small Recommendation-Level Review Focus

When the selected action is a cleanup/refactor review action, recommendation payloads should expose:

- `review_focus`

Preferred shape:

```json
{
  "review_focus": {
    "candidate_family": "hot_file",
    "candidate_basis": ["highest_access_activity", "activity_present"],
    "candidate_risk_hints": ["repo_risk_elevated"]
  }
}
```

Field meaning:

- `candidate_family`
  - names the file candidate class the consumer should inspect first
- `candidate_basis`
  - positive reasons the candidate family surfaced first
- `candidate_risk_hints`
  - advisory caveats explaining why review is still bounded or cautious

This batch should support exactly two families:

- `hot_file`
- `unused_candidate`

Non-review actions should keep `review_focus = null`.

Under the current recommendation cascade, stale snapshot and stale activity states are observation-first:

- stale snapshot selects `take_snapshot` before any review action
- stale activity selects `generate_activity_then_stats` before any review action

That means recommendation-level `review_focus` should only describe reachable review actions. Stale-evidence hints belong on the observation-first action or later candidate-level payloads, not on `review_focus`.

### 3. Add Three Candidate-Level Machine-Readable Fields

Existing candidate objects remain:

- `kind`
- `file_path`
- `reason`
- `suggested_commands`

Add only these extra fields:

- `candidate_basis: string[]`
- `candidate_risk_hints: string[]`
- `candidate_priority: string`

Preferred `candidate_priority` values:

- `primary`
- `secondary`

Preferred `candidate_basis` vocabulary:

- `highest_access_activity`
- `zero_recorded_access`
- `snapshot_present`
- `activity_present`
- `mock_data_overlap`
- `hardcoded_data_overlap`

Preferred `candidate_risk_hints` vocabulary:

- `repo_risk_elevated`
- `activity_evidence_stale`
- `snapshot_evidence_stale`

Rules:

- `candidate_basis` contains positive "why inspect this first" signals only
- `candidate_risk_hints` contains advisory environment caveats only
- `candidate_priority` reflects queue position, not safety level
- exact cleanup/refactor gate state remains on the parent recommendation or guidance layer through existing gate-level fields rather than being re-encoded inside each candidate

A `primary` candidate may still carry multiple risk hints.

### 4. Use A Shared Candidate Helper Instead Of Local Hand Assembly

This batch should not make `recommend_project_action(...)` construct concrete file lists. Recommendation still chooses the review family, while `stats` and `unused` guidance still own concrete file instantiation.

Recommended helper split:

- `review_focus_for_action(...)`
  - input: selected action, repo risk
  - output: recommendation-level `review_focus`

- `build_review_candidate(...)`
  - input: candidate kind, file path, priority, readiness snapshot, mock-data report, freshness flags, repo risk, suggested commands
  - output: normalized candidate object with the new machine-readable fields

Suggested module location:

- `src/mcp/review_candidates.rs`

This keeps shared candidate vocabulary out of:

- `src/mcp/project_guidance/stats_unused/stats.rs`
- `src/mcp/project_guidance/stats_unused/unused.rs`

and avoids leaking full recommendation scoring into file-level payloads.

For clarity:

- `review_focus_for_action(...)` stays limited to reachable review-action metadata
- `build_review_candidate(...)` should consume the already-available `&Value` returned by `project_readiness_snapshot(...)` plus simple freshness/repository-risk inputs

That keeps guidance code paths from depending on raw recommendation-only structs.

### 5. Recommendation-Level Review Focus Must Follow The Existing Action Choice

`review_focus` is a structured explanation of an already-selected action. It is not a second action selector.

Rules:

- if recommendation selects `inspect_hot_files`, emit `review_focus.candidate_family = "hot_file"`
- if recommendation selects `review_unused_files`, emit `review_focus.candidate_family = "unused_candidate"`
- all other actions emit `review_focus = null`

Preferred basis rules:

- `inspect_hot_files`
  - include `highest_access_activity`
  - include `activity_present`

- `review_unused_files`
  - include `zero_recorded_access`
  - include `snapshot_present`

Preferred risk-hint rules:

- hotspot review
  - include `repo_risk_elevated` when repo risk is not low or `large_diff = true`

Stale freshness does not appear in recommendation-level `review_focus` under the current cascade because:

- `take_snapshot` preempts `review_unused_files` when snapshot evidence is stale
- `generate_activity_then_stats` preempts `inspect_hot_files` when activity evidence is stale

This preserves the existing action cascade while making the selected review family easier to consume.

### 6. Stats And Unused Guidance Should Reuse The Same Candidate Vocabulary

`stats_guidance(...)` and `unused_guidance(...)` remain concrete candidate builders, but both should normalize through the shared helper.

Preferred behavior:

- `stats_guidance(...)`
  - hottest file candidate becomes `primary`
  - optional unused companion candidate becomes `secondary`

- `unused_guidance(...)`
  - first unused file candidate becomes `primary`
  - later unused file candidates become `secondary`

Reasoning rules:

- hotspot candidates should say they are high-interest review targets, not safe-to-refactor approvals
- unused candidates should say they lack recorded access in the current snapshot/activity window, not that they are proven safe to delete

If mock or hardcoded-data overlap already exists for a candidate file, that overlap may appear in `candidate_basis`, but this batch does not redesign mock-data ranking itself.

Overlap detection should reuse the existing `detect_mock_data_report(...)` output that `stats_guidance(...)` and `unused_guidance(...)` already compute. `build_review_candidate(...)` should receive that report and match candidate `file_path` values against `mock_data_candidates` and `hardcoded_data_candidates` rather than re-scanning file content.

### 7. Output Surfaces

This batch should update exactly three surfaces.

#### Single-project recommendation

`recommend_project_action(...)` becomes the source of truth for:

- `review_focus`

#### Stats guidance

`stats_guidance(...)` should emit enriched `file_recommendations[*]` and mirror them into:

- `layers.cleanup_refactor_candidates.candidates`

#### Unused guidance

`unused_guidance(...)` should emit the same normalized candidate fields in:

- `file_recommendations[*]`
- `layers.cleanup_refactor_candidates.candidates[*]`

This batch does not expand:

- `decision_brief`
- `agent_guidance`
- workspace portfolio payloads

### 8. Testing Strategy

Testing should stay focused on the new candidate-signal contract.

#### Recommendation tests

Add or extend focused tests proving:

- `inspect_hot_files` emits `review_focus.candidate_family = "hot_file"`
- reachable hotspot review carries `repo_risk_elevated` when that fact is present
- `review_unused_files` emits `review_focus.candidate_family = "unused_candidate"`
- stale snapshot selects `take_snapshot` and keeps `review_focus = null`
- stale activity selects `generate_activity_then_stats` and keeps `review_focus = null`
- non-review actions keep `review_focus = null`

#### Stats guidance tests

Add focused tests proving:

- hottest file becomes `candidate_priority = "primary"`
- hottest file includes `highest_access_activity`
- companion unused candidate includes `zero_recorded_access`
- stale activity or elevated repo risk appears in hotspot `candidate_risk_hints`

#### Unused guidance tests

Add focused tests proving:

- first unused candidate is `primary`
- later unused candidates are `secondary`
- basis and risk-hint arrays are present and stable
- existing cleanup/refactor gate fields remain unchanged

## Non-Goals

This batch does not:

- expose internal numeric action scores
- redesign `decision_brief` or `agent_guidance`
- change the recommendation action enum
- change CLI text rendering
- redesign mock or hardcoded-data prioritization
- introduce a generic candidate-ranking framework for every payload surface

## Expected Outcome

After this work:

- cleanup and refactor review candidates stay advisory-first but become easier to interpret programmatically
- hotspot candidates surface why they are interesting without overstating safety under stale or caution-level conditions
- unused candidates explain their basis more concretely than "no observed access"
- `stats` and `unused` guidance stop drifting into separate candidate vocabularies
