# Runtime Weak-Token Data-Risk Design

Date: 2026-05-04
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.08.01` and `FT-03.08.02` so runtime/shared source files are not promoted into mock or mixed-review risk mainly because their path contains weak tokens such as `seed`, `demo`, or `sample`.

The target is intentionally narrow:

- reduce false positives for runtime/shared source files
- preserve existing detection for clear test-only or example-style mock assets
- keep the current scan-cost model unchanged
- keep `data_risk_focus`, workspace aggregation, and decision projection schemas unchanged

This is heuristic tightening, not a new scanner and not a semantic-analysis subsystem.

## Capability Scope

FT IDs touched:

- `FT-03.08.01` Detect mock and test-only data artifacts
- `FT-03.08.02` Detect and prioritize hardcoded pseudo-business data

Consumer-side effects only:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.04.01` Rank projects by attention and evidence quality

Primary requirement families:

- `MOCK-01..10`
- consumed downstream benefit for `STRAT-01..04` and `PORT-01..04`

`FT-03.08.*` remains the owner of mock and hardcoded candidate detection. This batch changes token weighting and path-aware gating only; it does not broaden evidence collection.

## Current Problem

`detect_mock_data_report(...)` currently treats all path tokens in one bucket. That creates a recurring false-positive pattern:

- a file sits under `src/`, `app/`, `lib/`, or another `runtime_shared` path
- the path contains a weak naming token such as `seed`, `demo`, or `sample`
- the file becomes a mock candidate even though the path token is only descriptive
- if the same file also hits hardcoded-content rules, it becomes a `mixed_review_file`
- project-level `data_risk_focus` and workspace aggregation then overstate `mixed` or `hardcoded` urgency

This is most visible in normal source files like `src/customer_seed.rs`, where `seed` may describe bootstrap or initializer behavior rather than test-only data.

## Design

### 1. Split Path Tokens Into Strong And Weak Tiers

Path tokens should be divided into:

- strong tokens
  - `mock`
  - `mocks`
  - `fixture`
  - `fixtures`
  - `stub`
  - `stubs`
  - `fake`
  - `fakes`
  - `testdata`
  - `__fixtures__`
- weak tokens
  - `seed`
  - `seeds`
  - `demo`
  - `sample`
  - `samples`

Rationale:

- strong tokens already carry high test/mock intent
- weak tokens are common in normal runtime code and should not upgrade files on their own

This tiering applies only to path-token handling. Existing content tokens remain unchanged in this batch.

### 2. Keep Strong Tokens As Direct Mock Signals

Files whose paths hit a strong token should keep the current path-based mock behavior:

- they may still enter `mock_candidates`
- they may still contribute to `mixed_review_files` if hardcoded conditions also match
- current confidence and review-priority logic remains unchanged unless later rules lower it through existing path classification

This preserves recall for clearly test-oriented assets.

### 3. Make Weak Tokens Path-Aware

Weak tokens should stop acting as direct mock-upgrade signals in `runtime_shared` paths.

Preferred behavior:

1. `test_only`, `generated_artifact`, and example-like paths
- weak tokens still count as valid path-based mock signals
- examples: `tests/fixtures/demo.json`, `examples/sample.json`

2. `runtime_shared` paths
- weak tokens become hint-only
- they do not create a mock candidate on their own
- they do not push a file into `mixed_review_files` on their own

3. `unknown` paths
- weak tokens alone are insufficient for mock classification
- they need additional mock-oriented content evidence to upgrade

This keeps the detection lightweight while aligning path intent with actual risk.

This is a real behavior change for `unknown` paths. Current code still upgrades `unknown + weak token only` because all path tokens are treated uniformly through the flat `mock_path_tokens` array.

### 4. Require Additional Evidence Before Weak Runtime Paths Become Mock Candidates

For `runtime_shared` or `unknown` paths, a weak token should only contribute to mock classification when combined with stronger evidence already available in the current scan:

- content mock tokens such as `mock`, `fixture`, `fake`, or `sample data`
- other existing path/context signals that clearly indicate test-oriented use

The batch does not add new evidence sources. It only changes when existing weak path hits are allowed to matter.

Implementation note: the gating signal should reuse the existing content-token result already computed in `detect_mock_data_report(...)`, such as `content_mock_keywords` or an equivalent derived boolean like `has_content_mock_signal`.

### 5. Keep Hardcoded Detection Stable

The hardcoded-data path should stay unchanged in this batch:

- do not rewrite `business_hits + literal_hits + runtime_path`
- do not change `content.business_literal_combo`
- do not change current hardcoded severity mapping

The goal is to cut the false path:

`runtime_shared + weak token only -> mock candidate -> mixed review file`

not to redesign hardcoded detection.

### 6. Let Mixed-Review Become Cleaner Through Input Tightening

`mixed_review_files` should not receive a special new rule in this batch.

Instead, it should get cleaner automatically because:

- weak runtime path tokens no longer push ordinary source files into `mock_candidates`
- fewer ordinary runtime files will overlap with hardcoded candidates
- `mixed_review_files` will increasingly mean genuine semantic overlap rather than incidental naming

This preserves the current layering:

- detection owns candidate generation
- report/focus logic consumes the candidate sets as-is

### 7. Preserve Existing Downstream Schemas

Do not change:

- `mock_candidate_count`
- `hardcoded_candidate_count`
- `mixed_review_file_count`
- `data_risk_focus`
- `guidance`
- workspace data-risk overview fields
- `agent_guidance` and `decision_brief` data-risk projections

Only values should shift because upstream detection is more precise.

## Implementation Shape

Primary implementation file:

- `src/mcp/mock_detection.rs`

Preferred helper structure:

- `path_has_strong_mock_token(path_lower: &str) -> bool`
- `path_has_weak_mock_token(path_lower: &str) -> bool`
- `allow_weak_path_token_as_mock_signal(path_classification: &str, has_content_mock_signal: bool) -> bool`

Existing `mock_path_tokens` usage should be rewritten through these helpers rather than replaced with scattered inline conditionals.

`path_classification` should be passed through from the existing `classify_path_kind()` call site rather than recomputed inside the helper.

The implementation should keep one clear rule:

- strong tokens may still directly create a path-based mock signal
- weak tokens may only create a path-based mock signal when the path classification is not runtime-shared, or when additional stronger evidence is already present

## Test Strategy

Primary test file:

- `src/mcp/tests/data_risk_cases/report_detection.rs`

Add or tighten coverage for four scenarios:

1. `runtime_shared + weak token only`
- example: `src/customer_seed.rs`
- expected:
  - not in `mock_candidates`
  - may still be in `hardcoded_candidates`
  - not in `mixed_review_files` unless stronger mock evidence also exists

2. `test_only/example/generated + weak token`
- examples:
  - `tests/fixtures/demo.json`
  - `examples/sample.json`
- expected:
  - still in `mock_candidates`

3. `runtime_shared + weak token + strong mock content`
- example: `src/demo_seed.rs` with explicit `mock` or `fixture` content
- expected:
  - still in `mock_candidates`

4. `unknown path + weak token only`
- expected:
  - no direct mock upgrade without stronger content evidence

Regression checks should also confirm downstream effects indirectly:

- `mixed_review_files` count drops for weak-token-only runtime cases
- `data_risk_focus.primary_focus` remains `hardcoded` instead of flipping to `mixed` when the only mock evidence was a weak runtime path token

Breaking test updates to call out explicitly:

- in `detect_mock_data_report_distinguishes_mock_and_hardcoded_candidates`
  - `report.mock_candidates.len()` should drop from `3` to `2`
  - the assertion that `src/customer_seed.rs` appears in `mixed_review_files` should be removed or inverted

## Compatibility And Risk Control

Compatibility rules:

- no schema changes
- no CLI text changes required
- no scan-limit changes
- no new file types, readers, or path walks

Primary risk:

- accidentally lowering recall for real mock/demo assets

Risk controls:

- do not change strong-token behavior
- do not change content-token behavior
- do not change hardcoded detection in this batch
- keep weak-token behavior intact for `test_only`, `generated_artifact`, and example-like paths

## Success Criteria

This batch is successful when all of the following are true:

- `runtime_shared + weak token only` does not produce a mock candidate
- `runtime_shared + weak token only + hardcoded content` may still produce a hardcoded candidate
- `runtime_shared + weak token only` no longer produces `mixed_review_files`
- `test_only/example/generated + weak token` still produces mock candidates
- `runtime_shared + weak token + strong mock content` still produces mock candidates
- downstream `data_risk_focus` and workspace aggregation become more stable without any contract changes

## Out Of Scope

This batch does not do any of the following:

- AST or semantic analysis
- full-repository rescanning beyond current limits
- new content heuristics for pseudo-business data
- automatic remediation or file rewriting
- a broader redesign of `data_risk_focus`
- word-boundary-aware token matching for path fragments such as `seed` versus `seedless`
