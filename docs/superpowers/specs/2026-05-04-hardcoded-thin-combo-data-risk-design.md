# Hardcoded Thin-Combo Data-Risk Design

Date: 2026-05-04
Status: implemented and verified (2026-05-05)
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.08.02` so OPENDOG stops promoting thin business/literal combinations into `hardcoded pseudo-business data` when those files are more likely to be normal config defaults, example constants, or lightweight runtime metadata.

The target is intentionally narrow:

- reduce false positives from `business keyword + literal marker` combinations
- reduce recurring false positives from `runtime_shared` paths whose names include weak tokens such as `seed`, `demo`, or `sample`
- reduce recurring false positives from broad literal markers such as `city`, `postal`, `zip`, `usd`, `cny`, and `phone`
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
- runtime source files with a small number of broad terms such as `customer_id` and `email`
- lightweight unknown-path metadata files with only a small business/literal footprint

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
- downgrade overly broad literal markers instead of removing them outright

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

This is intentionally conservative for non-runtime paths:

- `unknown` paths still require the denser universal threshold
- `runtime_shared` paths get the narrower `business >= 2 && literal >= 2` entry because those files sit closer to production execution paths

These cases preserve current recall for clearly suspicious runtime literals such as:

- `customer + invoice + amount + usd`
- `customer + address + zip + email`

### 4. Stop Thin Combos From Upgrading On Their Own

The following cases should no longer directly become hardcoded candidates:

1. `runtime_shared` paths with only a thin combo:
- `business_hits >= 2`
- `literal_hits == 1`

This explicitly includes `config/` because current code already classifies `config/` as `runtime_shared`.

2. `unknown` paths with only a thin combo:
- `business_hits >= 2`
- `literal_hits == 1`

These files may still be reviewed through other signals, but they should not be promoted into high-confidence hardcoded-data review by this batch alone.

`example/` and `examples/` are already treated as `test_only` by the current detector, so they remain outside the hardcoded entry path in this batch as well.

### 5. Make `path.runtime_shared` A Pure Amplifier

The existing `path.runtime_shared` rule is too broad because it triggers on runtime/shared files with only `business_hits >= 2`.

This batch should change its role, not just its threshold.

New rule:

- `path.runtime_shared` may only be added after the same file has already qualified as a strong hardcoded combo
- it must not remain an independent path into `hardcoded_candidates`

That means:

- thin runtime/shared combos do not get `content.business_literal_combo`
- thin runtime/shared combos also do not get `path.runtime_shared`
- dense runtime/shared combos may receive both rule hits

This keeps `path.runtime_shared` useful as a confidence and focus amplifier while preventing it from reintroducing the thin-combo false-positive path by itself.

### 6. Downgrade Broad Literal Markers Instead Of Removing Them

This batch should keep the current literal vocabulary but stop treating every marker as equally strong evidence.

Recommended split:

- **strong literal markers**
  - `@`
  - `street`
  - `road`
  - `avenue`
  - `$`
  - `customer_id`
  - `tenant_id`
  - `invoice_no`
- **weak literal markers**
  - `city`
  - `postal`
  - `zip`
  - `usd`
  - `cny`
  - `phone`

Weak markers should stay observable, but they should contribute less to `literal_hits`.

Preferred rule:

- strong markers continue to count directly
- weak markers are discounted so that two weak hits together contribute one effective literal hit

This keeps the strong-combo thresholds unchanged while making ordinary config/default files much less likely to accumulate accidental literal density from generic labels alone.

Examples:

- `customer + order + phone + city` in a runtime file should no longer be enough for hardcoded promotion
- `customer + order + price + phone + city + postal + zip + usd` may still become hardcoded because the weak markers accumulate into denser evidence

This is a weighting change, not a rule deletion. The existing marker list stays visible in evidence, and the signal remains reversible if later tuning shows recall dropped too far.

### 7. Let Focus And Mixed-Review Clean Up Through Input Tightening

`MockDataReport::data_risk_focus()` should not change its rules in this batch.

`mixed_review_files` should also keep the current derivation.

Instead, those consumers should get cleaner automatically because:

- thin combos stop entering `hardcoded_candidates`
- fewer runtime files overlap with mock-oriented candidates
- project and workspace hardcoded focus becomes more reflective of dense pseudo-business literals

This can change project focus outcomes:

- some projects that currently land on `primary_focus = "hardcoded"` only because thin runtime/shared files emit `path.runtime_shared` should drop to `mock`, `mixed`, or `none`
- that shift is desirable, not a regression

This preserves the current layering:

- detection owns candidate generation
- report/focus logic consumes candidate sets

### 8. Keep This Change Commutative With Runtime Weak-Token Tightening

This spec and `2026-05-04-runtime-weak-token-data-risk-design.md` both modify `detect_mock_data_report(...)`, but they tighten different entry paths:

- the weak-token spec removes incidental mock promotion from weak runtime path tokens
- this spec removes incidental hardcoded promotion from thin business/literal combos

They should remain implementable in either order.

Their combined downstream effect should be verified as a unit after both land because both reduce accidental inputs into:

- `mixed_review_files`
- `data_risk_focus.primary_focus`
- workspace hardcoded/mock prioritization

## Implementation Shape

Primary implementation file:

- `src/mcp/mock_detection.rs`

Preferred helper structure:

- `is_strong_hardcoded_combo(path_classification: &str, business_hits: usize, literal_hits: usize) -> bool`
- `allow_runtime_shared_hardcoded_amplification(path_classification: &str, combo_is_strong: bool) -> bool`
- `discounted_weak_literal_hits(raw_weak_hits: usize) -> usize`

Rules:

- reuse the existing `classify_path_kind()` / `path_is_runtime_shared()` results rather than reclassifying inside helpers
- do not re-scan content
- do not duplicate `data_risk_focus` logic here
- do not change `DATA_RISK_RULES` metadata; only when the existing rules fire changes

The implementation should keep one clear split:

- dense combos may still create `content.business_literal_combo`
- thin combos may not
- `path.runtime_shared` may only be attached after a strong combo has already qualified the file as hardcoded

This is intentionally implemented through threshold tightening and path-aware amplification only.
It does not introduce new scanning passes, semantic parsing, configuration flags, or MCP/CLI contract changes.

## Test Strategy

Primary test file:

- `src/mcp/tests/data_risk_cases/report_detection.rs`

Add or tighten coverage for these cases:

1. `config/ + thin combo`
- example: `config/defaults.toml`
- expected:
  - not in `hardcoded_candidates`

2. `unknown + thin combo`
- example: `metadata/defaults.json`
- expected:
  - not in `hardcoded_candidates`

3. `runtime_shared + dense combo`
- example: `src/customer_invoice_seed.rs`
- expected:
  - still in `hardcoded_candidates`
  - still hits `content.business_literal_combo`
  - still hits `path.runtime_shared`

4. `unknown + dense combo`
- example: `customer_manifest.json`
- expected:
  - still in `hardcoded_candidates`
  - hits `content.business_literal_combo`
  - does not hit `path.runtime_shared`

Optional contract check:

- `src/mcp/tests/data_risk_cases/single_project_guidance.rs`
- ensure `data_risk_focus` does not elevate a thin-combo project into hardcoded focus unless a denser candidate remains present

Fixture construction guidance:

- prefer clearly distinct keywords across the two lists
- for example, use business terms like `customer` and `order`, and literal markers like `@` and `usd`
- avoid ambiguous overlaps where one token can inflate both counters unexpectedly

## Success Criteria

This batch is successful when:

- config/runtime thin combos stop appearing in `hardcoded_candidates`
- unknown thin combos stop appearing in `hardcoded_candidates`
- runtime/shared dense combos still appear in `hardcoded_candidates`
- `path.runtime_shared` no longer creates a hardcoded candidate by itself
- downstream `data_risk_focus` becomes cleaner without any schema changes

## Non-Goals

This batch does not:

- add AST or language-aware semantic analysis
- change CLI wording
- redesign `mock` detection
- alter `data_risk_focus` fields
- add automatic remediation or deletion behavior
