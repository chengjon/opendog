# Review: 2026-05-06-config-incremental-editing-design.md

**Type**: .md / spec | **Perspective**: completeness + consistency + feasibility (auto) | **Date**: 2026-05-06 | **Reviewer**: Claude

---

## Executive Summary

This is a well-scoped, internally consistent design spec for adding incremental list-editing flags to `opendog config set-global` and `opendog config set-project`. The document correctly describes current codebase behavior, accurately identifies all relevant modules, and proposes a design that aligns with existing patterns. The main gap is an incomplete treatment of the control protocol layer: the `ControlRequest` enum inlines patch fields rather than using the patch structs, so the spec needs to explicitly call out what changes in both the protocol enum variants and the client-side reconstruction in `config_ops.rs`.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | docs/superpowers/specs/2026-05-06-config-incremental-editing-design.md |
| File Type | .md |
| Doc Type | spec |
| Sections | 9 (Goal through Non-Goals) |
| Referenced Files | 10 found / 0 missing |
| Referenced Symbols | 6 found / 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/cli/config_commands.rs` | yes | /opt/claude/opendog/src/cli/config_commands.rs |
| `src/config.rs` | yes | /opt/claude/opendog/src/config.rs |
| `src/config/patching.rs` | yes | /opt/claude/opendog/src/config/patching.rs |
| `src/core/project.rs` | yes | /opt/claude/opendog/src/core/project.rs |
| `src/control/protocol.rs` | yes | /opt/claude/opendog/src/control/protocol.rs |
| `src/control/request_handler.rs` | yes | /opt/claude/opendog/src/control/request_handler.rs |
| `src/control/client/config_ops.rs` | yes | /opt/claude/opendog/src/control/client/config_ops.rs |
| `src/control/tests.rs` | yes | /opt/claude/opendog/src/control/tests.rs |
| `tests/integration_test/storage_project_snapshot.rs` | yes | /opt/claude/opendog/tests/integration_test/storage_project_snapshot.rs |
| `tests/integration_test/daemon_process_cli.rs` | yes | /opt/claude/opendog/tests/integration_test/daemon_process_cli.rs |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `ConfigPatch` | yes | src/config.rs:65 |
| `ProjectConfigPatch` | yes | src/config.rs:73 |
| `inherit_ignore_patterns` | yes | src/config.rs:79 |
| `inherit_process_whitelist` | yes | src/config.rs:81 |
| `apply_global_config_patch` | yes | src/config/patching.rs:71 |
| `apply_project_config_patch` | yes | src/config/patching.rs:83 |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| Current flags treat repeated list flags as complete replacement | confirmed | config_commands.rs:74-82 converts non-empty Vec to Some(Vec), patching.rs:73-80 applies as full replacement via `unwrap_or_else` |
| `ConfigPatch` has `ignore_patterns: Option<Vec<String>>` and `process_whitelist: Option<Vec<String>>` | confirmed | src/config.rs:65-70 |
| `ProjectConfigPatch` carries `inherit_ignore_patterns: bool` and `inherit_process_whitelist: bool` | confirmed | src/config.rs:78-81 |
| Empty-patch protection exists and fails with error | confirmed | patching.rs:21-23, 37-42 define `is_empty()`; project.rs:103-105 and :144-146 reject empty patches |
| CLI and daemon paths share overwrite-only semantics | confirmed | config_commands.rs:155-164 (daemon-first path) and :176-189 (daemon-first path) both construct the same patch structs; request_handler.rs:45-48 and :60-65 reconstruct them on the daemon side |
| No incremental flags currently exist | confirmed | grep for `add_ignore_pattern|remove_ignore_pattern|add_process|remove_process` returned zero matches across src/ |
| No conflict detection currently exists | confirmed | grep for `conflict|mutually_exclusive|conflicts_with|arg_conflict` in config_commands.rs returned zero matches |

## Checklist Results

### Completeness

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Has Goal, Capability Scope, Current Problem, Design (7 subsections), Implementation Shape, Test Strategy, Non-Goals |
| C2 | Edge cases | PASS | Covers: empty-patch protection (section 7), conflict rules (section 5), inheritance materialization (section 4), de-duplication (section 2), remove of nonexistent value (section 2) |
| C3 | Implicit assumptions | PASS | Single-writer SQLite model limits race risk; `resolve_project_config` already exists for materialization |
| C4 | Acceptance criteria | FAIL | Test Strategy lists coverage areas but no formal acceptance criteria (see finding M2) |
| C5 | Missing roles/stakeholders | N/A | Internal design spec, no multi-party stakeholder concerns |

### Consistency

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | "overwrite" used consistently; "incremental" and "add/remove" used consistently throughout |
| N2 | Naming conventions | PASS | CLI flags use kebab-case (`--add-ignore-pattern`) consistent with existing `--ignore-pattern`; struct fields use snake_case matching project convention |
| N3 | Formatting | PASS | Uniform heading hierarchy, consistent code-block usage |
| N4 | Cross-references | PASS | All module paths resolve to existing files |
| N5 | Style consistency | PASS | Uniform technical prose throughout |

### Feasibility

| # | Check | Result | Notes |
|---|-------|--------|-------|
| F1 | Technical risk | PASS | Hardest part (inheritance materialization) already has `resolve_project_config`; clap `conflicts_with` is well-documented for conflict rules |
| F2 | Dependency availability | PASS | No new dependencies; all referenced crates (clap, serde) already in use |
| F3 | Timeline realism | N/A | No timeline specified |
| F4 | Resource constraints | N/A | Not specified |
| F5 | Rollback plan | PASS | Feature is purely additive; existing overwrite flags and behavior preserved; no on-disk schema change |

## Findings

### Critical Issues

None.

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | Implementation Shape (line 200) | Control protocol serialization gap: the spec says the control protocol layer should "serialize new patch fields" but does not explicitly state that `ControlRequest::UpdateGlobalConfig` and `ControlRequest::UpdateProjectConfig` inline patch fields as separate struct members rather than embedding the patch structs. The protocol enum variants (protocol.rs:33-43) and the client-side reconstruction in `config_ops.rs:43-62` both need new inline fields added to mirror the incremental patch fields. | Implementor may miss updating one side of the daemon path, causing silent field loss in daemon-backed mode. | Codebase: protocol.rs:33-43 shows `UpdateGlobalConfig` has `ignore_patterns: Option<Vec<String>>` and `process_whitelist: Option<Vec<String>>` as inline fields, not a `ConfigPatch` struct. config_ops.rs:43-47 reconstructs `ConfigPatch` from these inline fields on the client side, and request_handler.rs:45-48 reconstructs it on the server side. Doc: Implementation Shape section mentions "control protocol layer - serializes new patch fields" but does not call out the inline-field pattern or the need to update both protocol.rs variants and config_ops.rs reconstruction. | Add a subsection or note under Implementation Shape explicitly listing: (1) new inline fields needed in `ControlRequest::UpdateGlobalConfig` and `ControlRequest::UpdateProjectConfig` variants in protocol.rs, (2) updated client-side reconstruction in config_ops.rs, (3) updated server-side reconstruction in request_handler.rs. |
| M2 | Test Strategy (lines 206-242) | No formal acceptance criteria: the Test Strategy section lists coverage areas and likely test files but does not state concrete pass/fail conditions for the feature as a whole. | Without explicit acceptance criteria, "done" is subjective. Could lead to premature merge or endless refinement. | Codebase: existing tests in src/config.rs:152-220 cover the current overwrite-only behavior with concrete assertions. Doc: Test Strategy lists 5 coverage areas with descriptive bullet points but no measurable criteria like "all 5 test groups pass" or "CLI help text shows new flags." | Add a brief Acceptance Criteria section: (1) all new unit and integration tests pass, (2) `opendog config set-global --help` shows new flags, (3) incremental CLI invocations produce correct persisted config, (4) daemon-backed path produces identical results to direct mode. |
| M3 | Capability Scope (line 32) | Struct definitions live in `src/config.rs` (the module root at lines 65-82), not in `src/config/patching.rs`. The spec lists `src/config/patching.rs` as a primary module, which is correct for the impl blocks and patch-application functions, but adding new fields to the structs requires editing `src/config.rs` first. | An implementor following the module list may not realize two files need coordinated edits for the struct definition change. | Codebase: `pub struct ConfigPatch` at src/config.rs:65 and `pub struct ProjectConfigPatch` at src/config.rs:73. The impl blocks (`is_empty`, `normalized`) are in src/config/patching.rs:20-53. Doc: Capability Scope lists both `src/config.rs` and `src/config/patching.rs` in the module list but does not distinguish their roles. | Clarify in the module list or Implementation Shape that `src/config.rs` holds the struct definitions (add fields here) while `src/config/patching.rs` holds the impl blocks and patch-application functions (add incremental logic here). |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | Non-Goals (line 252) | "Update operator-facing docs in this batch" is listed as a Non-Goal, but clap help strings (`/// doc comments` on the clap variants and `#[arg(help = "...")]` attributes) are code-level concerns that should be in scope. Without them, the new flags will have no user-visible descriptions. | Codebase: config_commands.rs:28 uses `/// Update per-project override fields` doc comment on SetProject variant. Doc: Non-Goals excludes "operator-facing docs" which could be interpreted as including in-code clap help. | Add a note to Implementation Shape or remove "operator-facing docs" ambiguity: "clap help strings for new flags are in scope; external documentation is out of scope for this batch." |
| L2 | Design section 7 (line 175) | The `is_empty()` update is implicitly required by the empty-patch protection section ("no add/remove values" listed as no-op condition) but not explicitly called out as a code change. | Codebase: patching.rs:21-23 (`ConfigPatch::is_empty`) and :37-42 (`ProjectConfigPatch::is_empty`) only check overwrite fields. Doc: section 7 says "no add/remove values" should be treated as empty, requiring `is_empty()` to check the new incremental fields too. | Add one sentence to Implementation Shape or section 7: "Both `is_empty()` implementations must be updated to treat empty incremental fields as no-op." |
| L3 | Test Strategy (line 237) | Likely test files list `src/config.rs` but the existing unit tests for patch structs are embedded in `src/config.rs:152-220` as a `#[cfg(test)] mod tests`. New tests for incremental behavior could live in either `src/config.rs` (alongside existing patch tests) or `src/config/patching.rs`. The spec does not state a preference. | Codebase: src/config.rs:152-220 contains the existing `config_patch_empty_detection_is_precise` and related tests. src/config/patching.rs has no test module. | State that new patch-logic tests should be added alongside the existing ones in `src/config.rs` (or alternatively in `src/config/patching.rs` if splitting is preferred). |

## Strengths

- The problem statement is precise and grounded: the spec correctly identifies that existing overwrite-only semantics create operational risk, and the solution is the minimum viable change.
- The inheritance materialization rule (section 4) is a well-reasoned design decision that avoids ambiguous "add to inherited list" semantics by making divergence explicit and durable.
- The conflict rules (section 5) are exhaustive and cover all combinations: overwrite-vs-incremental per field, and inherit-vs-any for project config.
- Accurate codebase alignment: all 10 referenced files exist, all 6 referenced symbols exist, and the described current behavior matches the actual implementation.
- Non-Goals section is well-scoped and prevents scope creep (no schema changes, no new config fields, no inference).

## Detailed Recommendations

1. **Protocol gap is the most actionable fix**: Add a paragraph under Implementation Shape (after the control protocol layer bullet) that explicitly lists the three files needing coordinated changes for daemon path parity: `src/control/protocol.rs` (add inline fields to enum variants), `src/control/client/config_ops.rs` (update client-side patch reconstruction), and `src/control/request_handler.rs` (update server-side patch reconstruction). This is the one place where the spec's "serialize new patch fields" is too abstract.

2. **Add a short Acceptance Criteria section** between Test Strategy and Non-Goals: four bullet points covering test passage, CLI help visibility, correct persisted output, and daemon parity. This makes the feature objectively completable.

3. **Clarify the config.rs vs config/patching.rs split** in the Capability Scope module list. A parenthetical like "struct definitions" and "impl blocks + patch-application logic" would eliminate ambiguity.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 5 | All codebase claims verified correct; struct shapes, function names, and behavior descriptions match live code |
| Completeness | 4 | Thorough design sections and test strategy; missing formal acceptance criteria and explicit protocol-layer detail |
| Codebase Alignment | 5 | 10/10 files found, 6/6 symbols found, all behavioral claims confirmed |
| Actionability | 4 | Clear implementation ownership split and test areas; protocol-layer gap and struct-location ambiguity reduce direct implementability |
| Terminology Consistency | 5 | Consistent use of "overwrite", "incremental", "add/remove", "inherit", "effective config" throughout |
| **Overall** | **4.6** | |

## Verdict

**APPROVE_WITH_NOTES** -- The spec is technically accurate, well-scoped, and closely aligned with the codebase. The three medium findings (protocol serialization gap, missing acceptance criteria, struct-definition location) should be addressed before implementation begins, but none block design approval.
