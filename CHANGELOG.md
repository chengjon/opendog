# Changelog

All notable changes to OPENDOG are documented here.

## 2026-05-02

### Added

- Verification-driven soft gates for cleanup and refactor decisions, including machine-readable `gate_assessment.cleanup` and `gate_assessment.refactor` outputs.
- Repository-truth boundary projection fields such as `repo_truth_gaps` and `mandatory_shell_checks` in project recommendations, decision briefs, and guidance summaries.
- Machine-readable `execution_sequence` payloads for repository stabilization, verification-first, and observation-first workflows, including resume conditions and suggested follow-up commands.

### Changed

- Project action selection now uses shared priority gating, scoring, and stable reasoning while preserving the existing `recommended_next_action` enum.
- Guidance and decision payloads now summarize which projects require repository stabilization, fresh verification runs, failing-verification repair, monitor start, snapshot refresh, or activity generation before broader cleanup or refactor review.
- MCP contract docs now describe the new sequencing and boundary fields so AI consumers can follow the same recommendation chain consistently.

### Scope

- This hardening line completed the current selective-deepening slice across `FT-03.01.01`, `FT-03.02.02`, `FT-03.03.01`, `FT-03.06.01`, and `FT-03.07.01`.
