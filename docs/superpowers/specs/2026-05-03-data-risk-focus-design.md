# Data-Risk Focus Design

Date: 2026-05-03
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.08.01` and `FT-03.08.02` so OPENDOG exposes a stable machine-readable explanation of mock, hardcoded, and mixed-review risk instead of relying mostly on counts and prose.

The target is intentionally narrow:

- keep the current detection heuristics and scan-cost model unchanged
- keep current CLI text output unchanged
- add only a small machine-readable explanation layer on top of existing project and workspace data-risk payloads
- project the same explanation into `agent_guidance` and `decision_brief`

This is explanation and aggregation hardening, not a new scanner and not a semantic-analysis subsystem.

## Capability Scope

FT IDs touched:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.07.01` State blind spots and authority boundaries
- `FT-03.08.01` Detect mock and test-only data artifacts
- `FT-03.08.02` Detect and prioritize hardcoded pseudo-business data

Primary requirement families:

- `STRAT-01..04`
- `BOUND-01..04`
- `MOCK-01..10`

`FT-03.08.*` remains the owner of candidate detection and data-risk explanation. This batch consumes existing counts, candidate arrays, and rule hits rather than widening evidence collection.

## Current Problem

OPENDOG already detects:

- `mock_data_candidates`
- `hardcoded_data_candidates`
- `mixed_review_files`
- `rule_groups_summary`
- `rule_hits_summary`

Current weaknesses:

- the project-level payload does not expose one stable answer to "what should I review first"
- workspace aggregation mainly exposes counts, not a compact focus summary
- `agent_guidance` and `decision_brief` can show hardcoded/mock counts, but not the dominant data-risk interpretation for the current project or workspace
- AI consumers still need to infer order from prose such as "review high-priority hardcoded-data candidates first"

This is most visible when counts are present but the consumer still cannot answer two follow-up questions reliably:

- is the current risk mainly `mock`, `hardcoded`, or `mixed`
- what machine-readable basis caused that conclusion

## Design

### 1. Keep Detection Stable

This work does not change:

- `detect_mock_data_report(...)` scan limits or file-reading behavior
- path-token and content-token heuristics
- `mock_candidate_count`, `hardcoded_candidate_count`, and `mixed_review_file_count` semantics
- current CLI rendering

The public change is narrower:

- add a small `data_risk_focus` object
- propagate it through single-project data-risk payloads, workspace aggregation, `agent_guidance`, and `decision_brief`

### 2. Add A Small Project-Level `data_risk_focus`

Single-project data-risk output should gain:

- `data_risk_focus`

Preferred shape:

```json
{
  "data_risk_focus": {
    "primary_focus": "none",
    "priority_order": [],
    "basis": ["no_candidates_detected"]
  }
}
```

Supported `primary_focus` values:

- `none`
- `mock`
- `hardcoded`
- `mixed`

Field meaning:

- `primary_focus`
  - the dominant data-risk category that should be reviewed first
- `priority_order`
  - the recommended review family order for the current project
- `basis`
  - stable machine-readable reasons that justify the selected focus

This object is a decision-compression layer. It does not replace counts or candidate arrays.

### 3. Add The Same Focus Object To Workspace Project Summaries

`get_workspace_data_risk_overview` already returns per-project summaries. Each summary should also carry:

- `data_risk_focus`

That keeps data-risk as its own source of truth and avoids forcing `agent_guidance` to reconstruct project-level focus from raw arrays or prose.

### 4. Use A Small Stable `basis` Vocabulary

This batch should keep `basis` intentionally small:

- `no_candidates_detected`
- `mock_candidates_present`
- `hardcoded_candidates_present`
- `mixed_review_files_present`
- `runtime_shared_candidates_present`
- `high_severity_content_hits_present`

Rules:

- `basis` should contain only stable keys, never prose
- `basis` should describe why the selected focus won, not every fact in the report
- `basis` should be derived only from existing counts and existing candidate rule hits

### 5. Canonical Derivation Site

`data_risk_focus` derivation should have exactly one owner:

- `MockDataReport::data_risk_focus(&self) -> Value`

Preferred implementation location:

- `src/mcp/data_risk/report.rs`

Rules:

- `project_data_risk_payload(...)` must consume this canonical method
- `data_risk_guidance(...)` must consume this canonical method
- workspace aggregation must consume this canonical method or already-rendered values derived from it
- no sibling module should reimplement the focus rules inline

### 6. Derive `primary_focus` From Existing Results Only

`data_risk_focus` must consume existing results:

- `hardcoded_candidate_count`
- `mock_candidate_count`
- `mixed_review_file_count`
- `hardcoded_data_candidates[*].path_classification`
- `hardcoded_data_candidates[*].rule_hits`

It must not rescan file content and must not add a second heuristic engine.

Preferred focus rules:

1. `hardcoded`
- choose when hardcoded candidates exist and any of the following is true:
  - `mixed_review_file_count > 0`
  - any hardcoded candidate hits `path.runtime_shared`
  - any hardcoded candidate hits `content.business_literal_combo`

2. `mixed`
- choose when the `hardcoded` rule above does not win and `mixed_review_file_count > 0`

3. `mock`
- choose when no `hardcoded` or `mixed` rule wins and `mock_candidate_count > 0`

4. `none`
- choose when all three counts are zero

This preserves the current product posture:

- runtime-shared business-like literals remain the highest-scrutiny case
- mixed files stay ahead of plain mock-only review
- mock-only findings remain review signals, not high-risk governance events

Preferred basis mapping:

- `hardcoded_candidates_present`
  - include when `primary_focus = "hardcoded"`
- `mixed_review_files_present`
  - include when `mixed_review_file_count > 0`
- `runtime_shared_candidates_present`
  - include when any hardcoded candidate hits `path.runtime_shared`
- `high_severity_content_hits_present`
  - include when any hardcoded candidate hits `content.business_literal_combo`
- `mock_candidates_present`
  - include when `primary_focus = "mock"`
- `no_candidates_detected`
  - include only when `primary_focus = "none"`

### 7. Keep `priority_order` Stable Instead Of Dynamic

This batch should not introduce a separate ranking engine. A fixed order per focus is enough:

- `hardcoded` focus:
  - `["hardcoded", "mixed", "mock"]`
- `mixed` focus:
  - `["mixed", "hardcoded", "mock"]`
- `mock` focus:
  - `["mock", "hardcoded", "mixed"]`
- `none` focus:
  - `[]`

This gives AI consumers a direct machine-readable order instead of forcing them to reinterpret counts or prose.

These order entries are review-attention domains, not guaranteed-disjoint file sets. In particular, `mixed` review files may also appear in the broader `hardcoded` or `mock` candidate arrays.

### 8. Project Data-Risk Guidance Should Mirror The Same Focus

`data_risk_guidance(...)` should consume the same already-derived `data_risk_focus` and expose it directly rather than recomputing a parallel explanation.

Rules:

- `recommended_flow` prose stays as-is in spirit
- `guidance.layers.cleanup_refactor_candidates` must include the already-derived `data_risk_focus`
- no second inference path should be created inside guidance

This keeps a single data-risk explanation source for:

- `project_data_risk_payload(...)`
- `data_risk_guidance(...)`

### 9. Workspace Guidance Should Add Small Focus Aggregates

Workspace guidance should add compact machine-readable summary fields:

- `data_risk_focus_distribution`
- `projects_requiring_hardcoded_review`
- `projects_requiring_mock_review`
- `projects_requiring_mixed_file_review`

These fields should be derived from per-project `data_risk_focus.primary_focus`, not from fresh rescans.

They answer:

- what dominant data-risk class is most common in this workspace
- how many projects currently need each type of data-risk review

Existing count-based fields remain:

- `projects_with_hardcoded_candidates`
- `projects_with_mock_candidates`
- `total_hardcoded_candidates`
- `total_mock_candidates`

The new fields are interpretation summaries, not replacements.

Preferred shape:

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

### 10. `agent_guidance` And `decision_brief` Should Only Project Existing Data-Risk Focus

`agent_guidance` should consume existing workspace/project data-risk results and add:

- `guidance.layers.execution_strategy.data_risk_focus_distribution`
- `guidance.layers.execution_strategy.projects_requiring_hardcoded_review`
- `guidance.layers.execution_strategy.projects_requiring_mock_review`
- `guidance.layers.execution_strategy.projects_requiring_mixed_file_review`

`decision_brief` should add:

- `decision.data_risk_focus`
- `decision.signals.mixed_review_file_count`

Rules:

- `agent_guidance_payload(...)` must not rescan candidates
- `decision_brief_payload(...)` must not invent its own focus logic
- the selected project should inherit the already-computed focus for that project

### 11. Output Surfaces

This batch should update exactly four surfaces.

#### Single-project data-risk payload

`project_data_risk_payload(...)` becomes the source of truth for:

- `data_risk_focus`

#### Project data-risk guidance

`data_risk_guidance(...)` should mirror the same:

- `data_risk_focus`

#### Workspace data-risk overview

`workspace_data_risk_overview_payload(...)` should expose:

- `projects[*].data_risk_focus`
- workspace-level focus-distribution summaries

#### Guidance and decision projection

`agent_guidance_payload(...)` and `decision_brief_payload(...)` should consume the existing focus object rather than recomputing it.

## Contract Compatibility

This batch is intended to be backward-compatible and additive under the current schema versions.

Rules:

- no contract version bump is required for these extra fields
- existing required fields keep their current meaning
- `data_risk_focus` and focus-aggregation fields are optional additions under existing versioned payloads

## Out Of Scope

This batch does not:

- change `detect_mock_data_report(...)` heuristics
- introduce AST or semantic analysis
- add automatic cleanup or rewriting
- redesign hardcoded versus mock candidate scoring
- change CLI text output
- add new shell commands beyond current candidate suggestions

## Testing

Testing should cover three layers.

### 1. Data-risk focus derivation

Add focused tests for:

- `hardcoded` focus with runtime/shared high-severity basis
- `mixed` focus when mixed files exist without runtime-shared hardcoded dominance
- `mock` focus when only mock candidates exist
- `none` focus when no candidates exist
- `priority_order` and `basis` stability

Suggested placement:

- extend `src/mcp/tests/data_risk_cases/report_detection.rs`
- add a dedicated `focus_derivation.rs` leaf only if `report_detection.rs` becomes structurally oversized

### 2. Workspace aggregation

Add tests for:

- `data_risk_focus_distribution`
- `projects_requiring_hardcoded_review`
- `projects_requiring_mock_review`
- `projects_requiring_mixed_file_review`

and confirm no regression in existing count-based aggregation.

Suggested placement:

- extend `src/mcp/tests/data_risk_cases/workspace_aggregation.rs`

### 3. Guidance and decision projection

Add tests for:

- `agent_guidance.layers.execution_strategy` focus aggregation
- `decision.data_risk_focus`
- `decision.signals.mixed_review_file_count`

and confirm no regression in:

- existing hardcoded/mock counts
- repo-truth fields
- sequencing fields
- verification gate fields

Suggested placement:

- extend `src/mcp/tests/data_risk_cases/single_project_guidance.rs`
- extend the relevant decision-brief contract tests under `src/mcp/tests/guidance_basics/basics_contracts/decision_brief_envelope.rs`

## Rationale

This design keeps `FT-03.08` inside OPENDOG's intended boundary:

- lightweight
- read-only
- explainable
- reusable across MCP decision surfaces

It improves data-risk review sequencing without turning OPENDOG into a deep repository semantic-analysis engine.
