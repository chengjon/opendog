# Review: Portfolio Attention Batching Design

Reviewer: Claude (GLM-5.1)
Date: 2026-05-05
Spec: `2026-05-05-portfolio-attention-batching-design.md`

## Verdict: Approve with minor suggestions

The spec is well-scoped, honest about its narrow intent, and correctly grounded in the existing codebase. The core idea (project an already-computed queue into fixed handling buckets) is the right call. Below are specific observations.

---

## Strengths

**Scope discipline is exemplary.** The spec lists five explicit "keep unchanged" items, a clear non-goals section, and a tight acceptance checklist. This is exactly the right pattern for a thin projection slice.

**Derives from existing data, not a new engine.** The constraint that `attention_batches` must be a pure projection of `attention_queue` — no rescoring, no reordering, no new thresholds — prevents scope creep. This aligns with the current `workspace_portfolio_layer` implementation in `src/mcp/attention.rs:294-340`, which already sorts and truncates the queue.

**Fixed `1/2/rest` split is the right simplification.** Given the queue is truncated to 5 entries (line 340 in `attention.rs`), adding score-based thresholds would be over-engineering. The fixed split is predictable and debuggable.

**Truncation accounting is honest.** `unbatched_project_count` surfaces the fact that `later` does not represent the full workspace tail. This prevents a false sense of completeness.

**Edge case coverage is thorough.** The four edge cases (queue length 0, 1, 2, 3) plus the general >=4 case are all specified with deterministic outcomes.

## Suggestions

### 1. `status` field semantics need clarification

The proposed shape includes `"status": "available"`. The parent `multi_project_portfolio` layer already has its own `status` field (currently `"available"` when projects exist, `"unavailable"` otherwise). The spec should state:

- `attention_batches.status` mirrors the parent layer status
- When the parent is `"unavailable"`, `attention_batches` should either be absent or have all three buckets empty with `status: "unavailable"`
- This avoids a scenario where the parent says unavailable but the batch object says available

### 2. Implementation files are incomplete

The spec names `src/mcp/attention.rs` as the primary file, which is correct for the batching logic itself. However, the `multi_project_portfolio` layer is assembled in `src/mcp/guidance_payload.rs:531-542`, where `workspace_portfolio_layer` output gets additional fields spliced in. The implementation will need to:

1. Add the batching logic inside `workspace_portfolio_layer` in `attention.rs`
2. Ensure `guidance_payload.rs` carries the new field through (it currently replaces the layer output and re-splices fields — the batching field would need to survive this, or be added after)

This is a minor omission but could confuse an implementer who only looks at `attention.rs`.

### 3. Test strategy should enumerate edge-case tests explicitly

The spec lists 5 coverage goals but only names 2 existing test files. Given the 4 explicit edge behaviors (queue len 0, 1, 2, 3), the test plan should name at least:

- one test for the golden path (queue >= 4, all three buckets populated)
- one test for each short-queue edge (len 0, 1, 2, 3)

The existing test files (`layer_scores.rs`, `attention_priorities.rs`) each contain a single test function. Adding 4-5 more cases to the same files is reasonable, but the spec should make this explicit rather than saying "stay inside existing tests."

### 4. Consider whether `source` is worth its field

The `"source": "attention_queue"` field documents derivation intent, which is useful during development. However, it is a static string that never changes — it will always be `"attention_queue"`. Two options:

- Keep it: it costs one field and helps future readers understand the relationship
- Drop it: the spec already documents the derivation rule, and the field adds no runtime value

Minor point. Either choice is defensible.

### 5. Batch entry shape may want `project_name` or `project_path`

The batch entry includes `project_id` but not the human-readable project name or path. The parent `project_overviews` array carries these, and the spec correctly says not to duplicate them. However, for MCP consumers rendering a batch list, having `project_id` alone means a second lookup. Consider whether adding one human-readable field (e.g. `project_path`) is worth the marginal redundancy. If not, the current shape is fine — the spec's rationale for thinness is sound.

### 6. No discussion of CLI output

The spec explicitly says "do not add CLI text output changes in this batch," which is a reasonable scope choice. But the existing CLI `agent-guidance` command and `decision-brief` command both format portfolio output. A future spec or follow-up should acknowledge that batching data will eventually need CLI rendering. Not blocking, just worth noting for the roadmap.

---

## Consistency with Codebase

| Spec claim | Codebase reality | Aligned? |
|---|---|---|
| `attention_queue` is already truncated | Yes, `attention.rs:340` truncates to 5 | Yes |
| `workspace_portfolio_layer` computes the queue | Yes, `attention.rs:294-357` | Yes |
| Existing tests cover attention scoring | Yes, 3 test functions across 3 files | Yes |
| `src/mcp/attention.rs` is the right place | Yes, it already owns the portfolio layer | Yes |
| `attention_score`, `attention_band` are already computed | Yes, via `enrich_project_overview_with_attention` | Yes |

No factual misalignments found.

---

## Summary

The spec is ready for implementation with the suggestions above addressed. The most actionable items are: (1) clarify `status` field behavior when the parent layer is unavailable, (2) mention `guidance_payload.rs` in the implementation shape, and (3) enumerate specific edge-case test functions rather than relying on the existing test files to organically gain coverage.
