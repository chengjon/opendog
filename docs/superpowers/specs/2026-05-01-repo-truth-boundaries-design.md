# Repo Truth Boundaries Design

Date: 2026-05-01
Status: proposed
Scope: Phase 6 selective deepening

## Goal

Strengthen `FT-03.07.01` so OPENDOG exposes repository-truth blind spots as stable machine-readable fields instead of relying only on human-facing `reason`, `blind_spots`, or `when_to_use_shell` text.

The target is narrow and intentional:

- keep the current `recommended_next_action` enum unchanged
- keep existing `constraints`, `boundaries`, and `reason` fields intact
- add machine-readable repo-truth gaps that recommendation, guidance, and decision surfaces can reuse
- make it clearer when OPENDOG is advisory only and direct shell or repo-native truth is mandatory

This is a boundary hardening pass, not a new capability family.

## Capability Scope

FT IDs touched:

- `FT-03.02.02` Recommend next-step execution strategy
- `FT-03.04.01` Aggregate and prioritize across projects
- `FT-03.07.01` State blind spots and authority boundaries

Primary requirement families:

- `BOUND-01..04`
- `STRAT-01..04`
- `RISK-01..04`

`RISK-01..04` is a consumed dependency from `FT-03.02.01`, not a directly expanded ownership target in this batch. This design reuses existing repository-risk output rather than broadening risk collection scope.

## Current Problem

OPENDOG already tells users and AI consumers to use `git status`, `git diff`, and project-native verification when needed, but that guidance is still too text-heavy.

Current weaknesses:

- repository-truth gaps are embedded in prose instead of stable keys
- downstream AI consumers can see that shell is recommended, but not always why it is mandatory
- recommendation payloads and workspace strategy summaries do not currently expose a small reusable structure for repo-truth blind spots
- `constraints.boundaries.blind_spots` is useful but too broad to serve as the primary machine contract for repo-native truth gaps

This is most visible when OPENDOG sees repository instability or cannot collect trustworthy git metadata. The system has the signal, but the boundary is not explicit enough for deterministic AI behavior.

## Design

### 1. Keep The Existing Decision Contract Stable

This work keeps the public recommendation contract stable.

It does not change:

- `recommended_next_action`
- `strategy_mode`
- existing schema versions
- current `reason`, `blind_spots`, `requires_shell_verification`, `cleanup_blockers`, or `refactor_blockers`

It also does not introduce hard blocking. OPENDOG still returns recommendations in all normal cases. The change is that recommendations and decision surfaces become more explicit about when repo-native truth must come from shell commands instead of OPENDOG-derived inference.

### 2. Add Minimal Machine-Readable Boundary Fields

Add two new fields to single-project recommendation output:

- `repo_truth_gaps: string[]`
- `mandatory_shell_checks: string[]`

These fields are not a replacement for human-readable explanation.

Their roles are:

- `repo_truth_gaps`
  - stable keys describing which repository-truth blind spots are active
- `mandatory_shell_checks`
  - the smallest set of shell checks that must happen before trusting OPENDOG guidance for risky repository decisions

These fields should also flow into `decision_brief` through the existing recommendation path so the decision envelope exposes the same boundary facts without re-deriving them.

### 3. Use A Standalone Gap Projection Helper

The normalized boundary projection should not be embedded directly inside `recommend_project_action(...)`.

Instead, add a focused helper that can be reused by recommendation and later consumers:

- `repo_truth_gap_projection(repo_risk: &Value) -> (Vec<String>, Vec<String>)`

Responsibilities:

- derive normalized `repo_truth_gaps`
- derive normalized `mandatory_shell_checks`
- preserve stable ordering and de-duplication
- stay independent from action scoring, eligibility, and reason wording

`recommend_project_action(...)` becomes the first consumer of this helper, not the owner of the boundary derivation rules.

### 4. Restrict Gap Generation To Existing Repo-Risk Facts

The new fields must be derived from existing repository-risk and boundary facts, not from a second parallel rule engine.

Allowed source facts:

- `repo_status_risk.status`
- `repo_status_risk.operation_states`
- `repo_status_risk.conflicted_count`
- `repo_status_risk.lockfile_anomalies`
- existing structured repo-risk findings when they clearly imply direct repository review

This keeps the boundary model aligned with `FT-03.07.01` and avoids duplicating git-state interpretation in multiple layers.

This also keeps the projection consistent with the already modular recommendation stack in `src/mcp/project_recommendation/`, where eligibility, scoring, and reasoning are separated responsibilities. Repo-truth gap projection should stay parallel to those helpers rather than getting folded back into one large recommendation function.

### 5. Stable Gap Key Set

Start with a small key set that is broad enough to be useful and narrow enough to stay stable:

- `not_git_repository`
- `git_metadata_unavailable`
- `repository_mid_operation`
- `working_tree_conflicted`
- `dependency_state_requires_repo_review`

Meaning:

- `not_git_repository`
  - OPENDOG cannot rely on git truth because the registered root is not a git repository
- `git_metadata_unavailable`
  - git-backed risk collection failed or returned an unavailable status
- `repository_mid_operation`
  - merge, rebase, cherry-pick, or bisect state is active
- `working_tree_conflicted`
  - conflicted paths exist and direct repository inspection is required
- `dependency_state_requires_repo_review`
  - dependency manifest or lockfile mismatch signals require direct repo-level review

These keys are intentionally normalized. The goal is not to mirror every raw repo-risk finding but to expose the minimum reusable boundary contract.

`large_diff` is intentionally excluded from this key set. It already behaves as a repository risk severity signal and influences readiness and recommendation logic, but it does not mean OPENDOG lost access to repository truth. It should remain a risk or caution signal, not become a repo-truth boundary key in this first pass.

### 6. Mandatory Shell Check Rules

`mandatory_shell_checks` is not a general suggestion list. It is a bounded list of checks that should happen before treating OPENDOG guidance as sufficient for risky repo decisions.

Recommended mapping:

- `not_git_repository`
  - no mandatory git check
  - optional project-native verification remains outside this field
- `git_metadata_unavailable`
  - `git status`
- `repository_mid_operation`
  - `git status`
  - `git diff`
- `working_tree_conflicted`
  - `git status`
  - `git diff`
- `dependency_state_requires_repo_review`
  - `git status`
  - `git diff`

Rules:

- preserve stable ordering
- de-duplicate repeated commands
- do not inflate this field into a full suggested-command catalog
- do not insert project-native test commands unless the gap itself is specifically about repository truth

### 7. Output Surfaces

This batch should update three surfaces.

#### Single-project recommendation

`recommend_project_action(...)` becomes the source of truth for:

- `repo_truth_gaps`
- `mandatory_shell_checks`

This is where the normalized fields belong because recommendation already fuses repository risk, verification, and observation state.

#### Decision brief

`decision_brief` should consume and expose the same recommendation-derived fields so downstream AI consumers do not need to reconstruct boundary facts from prose.

Preferred shape:

- `decision.repo_truth_gaps`
- `decision.mandatory_shell_checks`

Do not duplicate the same fields again under `risk_profile`. One copy is enough if it is kept close to the final decision payload.

#### Workspace execution strategy summary

`agent_guidance.layers.execution_strategy` should expose a compact aggregation:

- `projects_with_repo_truth_gaps`
- `repo_truth_gap_distribution`
- `mandatory_shell_check_examples`

This workspace layer is summary-only. It should not expand every project's full gap array again.

### 8. Compatibility With Existing Boundary Fields

Existing fields remain valid and required:

- `constraints.boundaries.direct_observations`
- `constraints.boundaries.inferences`
- `constraints.boundaries.blind_spots`
- `constraints.boundaries.requires_shell_verification`
- recommendation `reason`

Compatibility rule:

- old fields stay human-oriented and broad
- new fields become the machine-first projection for repo-truth blind spots

This means consumers can continue reading legacy fields unchanged, while newer AI logic can switch to `repo_truth_gaps` and `mandatory_shell_checks` for deterministic boundary handling.

Boundary projection can influence recommendation explanation and decision payload shape, but it does not replace existing eligibility logic. For example, `operation_states` can still force `stabilize_repository_state` through the recommendation eligibility helper, while the new gap projection simply makes the authority boundary explicit and reusable.

### 9. Non-Goals

This batch does not:

- change CLI text output
- change project action ordering or scoring rules beyond boundary projection
- add new git probes
- claim OPENDOG is authoritative for repository truth
- add hard blocking to MCP or CLI execution
- expand every `project_overviews[*]` payload with the new fields in the first pass

## Testing

Add or update tests at three levels.

### 1. Repo and readiness unit coverage

Cover:

- `not_git_repository` projection
- unavailable git metadata projection
- mid-operation projection
- conflicted working tree projection
- lockfile anomaly projection
- stable ordering and de-duplication for `mandatory_shell_checks`

### 2. Recommendation behavior coverage

Verify:

- recommendation actions remain unchanged for existing scenarios
- `repo_truth_gaps` and `mandatory_shell_checks` appear when expected
- human-readable `reason` remains present and compatible

### 3. Guidance and decision integration coverage

Verify:

- `decision_brief` mirrors recommendation repo-truth fields
- `agent_guidance.layers.execution_strategy` aggregates repo-truth gap counts correctly
- existing `blind_spots` and `requires_shell_verification` fields remain present and structurally compatible

## Expected Outcome

After this work, OPENDOG still behaves as a bounded advisory system, but AI consumers can distinguish repository-truth blind spots without parsing prose.

The practical outcome is:

- more explicit handoff from OPENDOG to shell truth
- stronger authority-boundary signaling for AI consumers
- better consistency between recommendation, guidance, and decision surfaces
- no schema break for existing consumers
