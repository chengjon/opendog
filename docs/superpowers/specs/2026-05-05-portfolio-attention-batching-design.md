# Portfolio Attention Batching Design

Date: 2026-05-05
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.04.01` by adding a thin batching projection above the existing workspace portfolio attention queue so AI consumers can see not only which project deserves attention first, but also how to process the current top workspace window in small batches.

This slice is intentionally narrow:

- add only a read-only batching projection under `guidance.layers.multi_project_portfolio`
- keep `attention_score` calculation unchanged
- keep `priority_candidates` ordering unchanged
- keep project-level `recommended_next_action` unchanged
- keep existing MCP/CLI command lists unchanged
- keep existing payload fields stable and only add a backward-compatible field
- reuse the current truncated `attention_queue` as the only batching source

This is portfolio consumption hardening, not a new scheduler.

## Capability Scope

FT IDs touched:

- `FT-03.04.01` Multi-project Portfolio Aggregation & Batching

Primary requirement families:

- `PORT-01..04`
- `STRAT-04`
- `EVID-02..03`

## Current Problem

Current Phase 6 output already exposes the key cross-project portfolio fields:

- `priority_candidates`
- `attention_queue`
- `project_overviews`

This is enough to answer:

- which project deserves attention first
- why that project ranks above the others

But it still leaves a small consumption gap for AI batching behavior.

The AI currently has to infer:

- which single project belongs to the first pass
- which projects should be handled immediately after that
- which projects can be deferred to the current portfolio tail

That means the ranking exists, but the batching intent is still implicit.

## Design

### 1. Add A Thin Portfolio Batching Object

Add one read-only field:

- `guidance.layers.multi_project_portfolio.attention_batches`

This object exists only to project the existing `attention_queue` into small, stable handling batches.

`status` should mirror the parent `multi_project_portfolio.status`.

Under the current implementation this will normally be `"available"` because the parent layer already emits an available portfolio summary. If the parent layer ever becomes unavailable in a future batch, `attention_batches` should mirror that state and return empty buckets rather than claiming availability independently.

Proposed shape:

```json
{
  "status": "available",
  "source": "attention_queue",
  "batched_project_count": 5,
  "unbatched_project_count": 2,
  "immediate": [
    {
      "project_id": "alpha",
      "recommended_next_action": "review_failing_verification",
      "attention_score": 128,
      "attention_band": "critical"
    }
  ],
  "next": [
    {
      "project_id": "beta",
      "recommended_next_action": "run_verification_before_high_risk_changes",
      "attention_score": 92,
      "attention_band": "high"
    },
    {
      "project_id": "gamma",
      "recommended_next_action": "take_snapshot",
      "attention_score": 71,
      "attention_band": "medium"
    }
  ],
  "later": [
    {
      "project_id": "delta",
      "recommended_next_action": "inspect_hot_files",
      "attention_score": 48,
      "attention_band": "medium"
    },
    {
      "project_id": "epsilon",
      "recommended_next_action": "review_unused_files",
      "attention_score": 34,
      "attention_band": "low"
    }
  ]
}
```

Keep `source`.

Rationale:

- it makes the projection relationship explicit in-machine
- it documents that batching comes from the existing queue, not a second ranking engine
- it is a static field, but the payload cost is small and the derivation clarity is useful

### 2. Derive Batches Only From Existing `attention_queue`

This slice must not create a second ranking engine.

`attention_batches` must be derived from the already computed `attention_queue`:

- do not rescore projects
- do not reorder projects
- do not widen the queue to include more than the current top window
- do not consult additional detection or freshness rules

This keeps batching aligned with the existing portfolio ranking instead of inventing a parallel portfolio model.

### 3. Use Fixed Window Splits, Not New Thresholds

Batch splits should stay fixed and simple:

- `immediate`: first queued project
- `next`: second and third queued projects
- `later`: remaining queued projects

This is intentionally a `1 / 2 / rest` split.

Why this is the right ceiling:

- it does not add score thresholds
- it does not reinterpret `attention_band`
- it makes the first pass explicit without creating new policy logic
- it stays stable even if future attention weights change

### 4. Keep Batch Entries Thin

Each batch item should include only:

- `project_id`
- `recommended_next_action`
- `attention_score`
- `attention_band`

Do not duplicate:

- full `project_overviews`
- `attention_reasons`
- `priority_basis`
- repository risk payloads
- toolchain payloads

Those already exist elsewhere in the response. The batching object should point to them indirectly through the project id, not clone them.

### 5. Make Truncation Explicit

Because `attention_queue` is already truncated to the current top workspace window, `attention_batches` must surface that explicitly.

Rules:

- `batched_project_count` = `attention_queue.len()`
- `unbatched_project_count` = `project_count - batched_project_count`

This avoids a false impression that `later` represents the full workspace tail.

### 6. Edge Behavior Must Stay Deterministic

If `attention_queue` is empty:

- `immediate = []`
- `next = []`
- `later = []`
- `batched_project_count = 0`
- `unbatched_project_count = project_count`

If `attention_queue.len() == 1`:

- only `immediate` receives one item

If `attention_queue.len() == 2`:

- `immediate` receives one
- `next` receives one
- `later` stays empty

If `attention_queue.len() == 3`:

- `1 / 2 / 0`

If `attention_queue.len() >= 4`:

- `1 / 2 / rest`

## Implementation Shape

Primary implementation files:

- `src/mcp/attention.rs`
- `src/mcp/guidance_payload.rs`

Likely helper shape:

- a small helper that maps one attention item into the thin batch-entry form
- a small helper that slices the existing `attention_queue` into the three batch lists

Rules:

- do not change `workspace_portfolio_layer(...)` sorting logic
- do not change `sort_project_recommendations(...)`
- do not change `agent_guidance_payload(...)` orchestration except to preserve and expose the new portfolio field through the existing `workspace_portfolio_layer(...)` handoff
- do not add CLI text output changes in this batch

## Test Strategy

Stay inside the existing workspace portfolio tests.

Primary test files:

- `src/mcp/tests/portfolio_commands/workspace_portfolio/layer_scores.rs`
- `src/mcp/tests/portfolio_commands/workspace_portfolio/attention_priorities.rs`

Coverage goals:

1. `attention_batches` exists and is read-only
2. the first queued project appears in `immediate`
3. the second and third queued projects appear in `next`
4. remaining queued projects appear in `later`
5. short queues return stable empty buckets without missing fields

Required test cases:

1. one golden-path case where the queue length is at least 4 and all three buckets are populated
2. one edge case where the queue length is 0
3. one edge case where the queue length is 1
4. one edge case where the queue length is 2
5. one edge case where the queue length is 3

Do not add tests for new ranking behavior, because this slice does not change ranking behavior.

## Non-Goals

Do not:

- change `attention_score`
- change `priority_candidates`
- change `project_overviews`
- add serial-vs-parallel execution semantics
- add batch scoring thresholds based on `attention_band` or raw score
- add `project_path` or other human-readable duplicate fields to batch entries in this batch
- expand this slice into decision-brief, execution-strategy, or project-level recommendation changes

## Acceptance Criteria

This batch is complete when all of the following are true:

- `guidance.layers.multi_project_portfolio.attention_batches` exists
- it is entirely derived from `attention_queue`
- `priority_candidates[0]` and `attention_batches.immediate[0]` identify the same top project when a queue exists
- empty and short queues return stable structure
- docs describe the field as a batching projection rather than a new scheduling engine
