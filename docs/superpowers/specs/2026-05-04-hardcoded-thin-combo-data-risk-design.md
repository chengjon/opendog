# Hardcoded Thin-Combo Data-Risk Design

Date: 2026-05-04
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.08.02` so OPENDOG stops promoting thin business/literal combinations into `hardcoded pseudo-business data` when those files are more likely to be normal config defaults, example constants, or lightweight runtime metadata.

The target is intentionally narrow:

- reduce false positives from `business keyword + literal marker` combinations
- preserve current detection for dense runtime/shared pseudo-business data
- keep the current scan-cost model unchanged
- keep `data_risk_focus`, workspace aggregation, and decision projection schemas unchanged

This is heuristic tightening, not a new scanner and not a semantic-analysis subsystem.

## Capability Scope

FT IDs touched:

- `FT-03.08.02` Detect and prioritize hardcoded pseudo-business data

Consumer-side effects only:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.04.01` Rank projects by attention and evidence quality

Primary requirement family:

- `MOCK-01..10`

`FT-03.08.*` remains the owner of hardcoded-data candidate detection. This batch changes combination gating only; it does not broaden evidence collection or add new projections.

## Current Problem

`detect_mock_data_report(...)` currently upgrades a file into `hardcoded_candidates` when all of the following are true:

- the path is not `test_only`
- the path is not `generated_artifact`
- `business_hits >= 2`
- `literal_hits >= 1`

It also adds `path.runtime_shared` whenever:

- the path is runtime/shared
- `business_hits >= 2`

This is too permissive for several common cases:

- `config/` defaults with fields like `support_email`, `city`, or `usd`
- example/demo files that describe usage rather than production-like seeded business data
- runtime source files with a small number of broad terms such as `customer_id` and `email`

Those files can be promoted into `hardcoded_candidates`, then influence:

- `mixed_review_files`
- project `data_risk_focus`
- workspace hardcoded-review ordering

The result is overstated hardcoded urgency for thin, low-context literal sets.

## Design

### 1. Keep The Existing Detection Skeleton

This batch does not redesign hardcoded detection from scratch.

Do not change:

- file scan limits
- file sample size
- business keyword vocabulary
- literal marker vocabulary
- `data_risk_focus` structure
- workspace or decision payload schemas

The change is narrower:

- add stronger combo gating before a file is promoted into `hardcoded_candidates`
- tighten when `path.runtime_shared` may amplify hardcoded confidence

### 2. Distinguish Dense Combos From Thin Combos

The current `business_hits >= 2 && literal_hits >= 1` threshold should stop acting as a universal hardcoded upgrade rule.

Preferred interpretation:

- **dense combo**
  - strong evidence of pseudo-business data
  - should continue to upgrade into `hardcoded_candidates`
- **thin combo**
  - weak evidence that often appears in config defaults, examples, or lightweight runtime constants
  - should no longer upgrade on its own

### 3. Keep Dense Runtime Combos As Hardcoded

The following patterns should still enter `hardcoded_candidates`:

1. `business_hits >= 3 && literal_hits >= 2`
- regardless of non-test, non-generated path family

2. `runtime_shared && business_hits >= 2 && literal_hits >= 2`
- runtime/shared code should remain sensitive to denser pseudo-business data patterns

These cases preserve current recall for clearly suspicious runtime literals such as:

- `customer + invoice + amount + usd`
- `customer + address + zip + email`

### 4. Stop Thin Combos From Upgrading On Their Own

The following cases should no longer directly become hardcoded candidates:

1. `config/` or config-like paths with:
- `business_hits >= 2`
- `literal_hits == 1`

2. `example/` or `examples/` paths with:
- `business_hits >= 2`
- `literal_hits == 1`

3. `runtime_shared` paths with only a thin combo:
- `business_hits == 2`
- `literal_hits == 1`

These files may still be reviewed through other signals, but they should not be promoted into high-confidence hardcoded-data review by this batch alone.

### 5. Tighten The `path.runtime_shared` Amplifier

The existing `path.runtime_shared` rule is too broad because it triggers on runtime/shared files with only `business_hits >= 2`.

It should be tightened so runtime/shared amplification requires at least one literal marker:

- `runtime_shared && business_hits >= 2 && literal_hits >= 1`

This is the minimum safe tightening for this batch.

It keeps `path.runtime_shared` useful for genuine pseudo-business data while reducing false amplification for generic source files that merely contain business-like identifiers.

### 6. Let Focus And Mixed-Review Clean Up Through Input Tightening

`MockDataReport::data_risk_focus()` should not change its rules in this batch.

`mixed_review_files` should also keep the current derivation.

Instead, those consumers should get cleaner automatically because:

- thin combos stop entering `hardcoded_candidates`
- fewer runtime files overlap with mock-oriented candidates
- project and workspace hardcoded focus becomes more reflective of dense pseudo-business literals

This preserves the current layering:

- detection owns candidate generation
- report/focus logic consumes candidate sets

## Implementation Shape

Primary implementation file:

- `src/mcp/mock_detection.rs`

Preferred helper structure:

- `is_strong_hardcoded_combo(path_lower: &str, business_hits: usize, literal_hits: usize) -> bool`
- `allow_runtime_shared_hardcoded_escalation(path_lower: &str, business_hits: usize, literal_hits: usize) -> bool`

Rules:

- reuse the existing `classify_path_kind()` / `path_is_runtime_shared()` results
- do not re-scan content
- do not duplicate `data_risk_focus` logic here

The implementation should keep one clear split:

- dense combos may still create `content.business_literal_combo`
- thin combos may not
- runtime/shared amplification should require at least one literal marker

## Test Strategy

Primary test file:

- `src/mcp/tests/data_risk_cases/report_detection.rs`

Add or tighten coverage for these cases:

1. `config/ + thin combo`
- example: `config/defaults.toml`
- expected:
  - not in `hardcoded_candidates`

2. `example-like + thin combo`
- example: `examples/customer_sample.json`
- expected:
  - not in `hardcoded_candidates`

3. `runtime_shared + dense combo`
- example: `src/customer_invoice_seed.rs`
- expected:
  - still in `hardcoded_candidates`
  - still hits `content.business_literal_combo`

4. `runtime_shared + thin combo`
- example: `src/config_defaults.rs`
- expected:
  - no `hardcoded_candidate`
  - no `path.runtime_shared` amplification from the thin combo alone

Optional contract check:

- `src/mcp/tests/data_risk_cases/single_project_guidance.rs`
- ensure `data_risk_focus` does not elevate a thin-combo project into hardcoded focus unless a denser candidate remains present

## Success Criteria

This batch is successful when:

- config/example thin combos stop appearing in `hardcoded_candidates`
- runtime/shared dense combos still appear in `hardcoded_candidates`
- runtime/shared thin combos stop receiving `path.runtime_shared` amplification
- downstream `data_risk_focus` becomes cleaner without any schema changes

## Non-Goals

This batch does not:

- add AST or language-aware semantic analysis
- change CLI wording
- redesign `mock` detection
- alter `data_risk_focus` fields
- add automatic remediation or deletion behavior
