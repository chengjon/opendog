---
title: "Calibrate source signal visibility in observation outputs"
id: "TASK-20260511-source-signal-observation-calibration"
status: completed
owner: "unassigned"
priority: high
phase_hint: "Phase 6 observation quality hardening"
ft_ids_touched:
  - FT-01.03.01
  - FT-01.04.02
  - FT-03.01.01
  - FT-03.07.01
why_these_ft_ids:
  - "FT-01.03.01 owns AI-held file activity attribution; source files showing zero access after real work needs evidence review."
  - "FT-01.04.02 owns hotspot and unused views; `.claude/` dominance can make source-level stats hard to use."
  - "FT-03.01.01 owns observation quality and readiness; guidance should expose when source signal is absent or dominated by infrastructure."
  - "FT-03.07.01 owns authority boundaries; outputs must distinguish observed infrastructure activity from unsupported source-code conclusions."
requirement_ids:
  - STAT-05
  - STAT-06
  - RISK-01
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - cli
  - mcp
  - daemon
non_goals:
  - "Do not reopen Case H or Case I; both are fixed by mystocks 2026-05-11 retest."
  - "Do not change scanner attribution semantics without a new OpenSpec proposal if evidence points to scanner behavior."
  - "Do not globally ignore `.claude/` or other tool directories without an explicit compatibility decision."
  - "Do not hide infrastructure evidence silently."
verification_plan:
  - "Use mystocks retest evidence to quantify source, infrastructure, backup, and project classifications in stats and unused outputs."
  - "Run a source-code-heavy mystocks session and verify whether `.py` / `.vue` files receive access_count values when opened by real tools."
  - "Compare fd-level evidence, stats rows, and classification summaries before deciding whether this is real tool behavior, scanner behavior, or presentation/filtering debt."
  - "If scanner semantics must change, create an OpenSpec proposal before implementation."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `python3 scripts/validate_planning_governance.py`, and any new targeted MCP/CLI regression probes."
evidence_outputs:
  - "Mystocks sampling plan: docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md"
  - "Updated project-exchange issue entry with source-signal evidence and classification counts."
  - "Decision record stating whether the next change is scanner attribution, default filtering, MCP/CLI view filters, or guidance-only messaging."
  - "mystocks before/after evidence showing whether source files appear in stats under real source-code activity."
decision:
  date: "2026-05-11"
  outcome: "Expected observation-method behavior plus filtering/presentation debt."
  rationale: "mystocks source-heavy calibration showed Claude Code Read operations do not keep source file descriptors open long enough for fd sampling; Edit activity was visible through modification_count, while source access_count stayed unchanged."
  next_governed_path: "Prefer source-first views, classification filters, and guidance wording. Do not reopen scanner attribution unless new fd-level evidence contradicts this result."
---

## Goal

Determine why mystocks still reports `.claude/` infrastructure files as the hottest observed files while source files remain at `access_count=0`, then choose the smallest governed fix path.

## Evidence Source

`docs/project-exchange/reports/mystocks/opendog-mcp-retest-results-2026-05-11.md` records that Case H and Case I passed, but also notes:

- top 28 `.claude/` files share near-identical access counts and durations
- source files still show `access_count=0`
- this is not a payload-size or MCP resources problem

Sampling plan:

- `docs/project-exchange/reports/mystocks/source-signal-calibration-plan-2026-05-11.md`

Calibration result:

- `docs/project-exchange/reports/mystocks/OPENDOG_USAGE_FEEDBACK.md#source-signal-calibration-odx-20260511-source-signal-observation-calibration`
- Touched `.py` files remained at `access_count=0`.
- Touched `web/frontend/src/App.vue` retained `access_count=1` but changed `modification_count` from 0 to 2 after edit/revert activity.
- Top `.claude/` infrastructure files continued to share near-identical counts, showing sustained tool-infrastructure reads still dominate hot stats.
- Conclusion: this is not a `fix-fd-attribution` regression; it is a visibility limitation of fd sampling for transient Claude Code reads plus a presentation/filtering gap.

## Change Plan

1. Treat the residual C+D observation as a new shared issue, not as a reopening of fixed Case H/I. Completed.
2. Gather a source-code-heavy mystocks activity sample. Completed.
3. Compare scanner-level evidence with stats output and path classifications. Completed.
4. Decide whether the next governed change belongs to attribution semantics, default ignore/filter policy, source-first views, or guidance messaging. Completed: source-first views, classification filters, and guidance messaging.
5. If scanner attribution semantics need to change, require a new OpenSpec proposal and review plan before implementation. Not currently needed.

## Guardrails

- Keep the accepted `fix-fd-attribution` baseline intact until new evidence proves a different scanner issue.
- Keep infrastructure files observable unless a compatibility decision explicitly changes default ignore behavior.
- Do not infer that `access_count=0` means source files are safe to delete.

## Completion Criteria

- The residual mystocks source-signal problem is classified as expected tool behavior plus filtering/presentation debt.
- A follow-up implementation card or OpenSpec proposal exists if code behavior must change.
- Project-exchange evidence includes exact commands, counts, and affected file classes.
