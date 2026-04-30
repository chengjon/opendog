# OPENDOG

## What This Is

A multi-project observation and decision-support system for AI development workflows. Running on WSL with a background daemon, a shared local control plane, a CLI operator surface, and an MCP AI surface, it tracks which files AI tools (Claude Code, Codex, GPT, GLM) access across multiple projects — recording access frequency, duration, and modifications. Per-project SQLite databases store snapshots, usage evidence, verification runs, and retained operational evidence so OPENDOG can identify unused/stale files, highlight core files, and help AI agents decide what to inspect, verify, or clean up next.

This direction has not drifted from the original goal. Later additions such as repository-risk summaries, verification evidence, workspace prioritization, constraints/boundaries, and mock-data review deepen the same multi-project observation + AI decision-support mission rather than opening unrelated product lines.

## Core Value

Accurately identify which project files AI tools actually use and which are dead weight, then expose that activity as reusable project intelligence for AI agents — enabling better cleanup, review, and next-step decisions across multiple concurrent projects.

## Requirements

### Validated

- v1 baseline shipped: multi-project isolation, snapshot, monitoring, statistics, MCP/CLI interfaces, daemon deployment
- Local control-plane coordination shipped: CLI / MCP can reuse daemon-owned project operations instead of silently creating divergent monitor state
- Verification evidence shipped: latest test/lint/build results can be recorded, queried, and executed through OPENDOG
- Data-risk layer shipped: mock/hardcoded pseudo-data detection available at project and workspace scope
- AI guidance skeleton shipped: reusable observation, risk, strategy, verification, portfolio, cleanup, toolchain, and boundary layers are now part of the MCP guidance shape
- Observation freshness and evidence coverage shipped: guidance and decision payloads now distinguish missing versus stale snapshot, activity, and verification evidence
- Workspace portfolio attention scoring shipped: cross-project ranking now exposes explicit attention score, band, and reasons instead of relying on opaque sorting
- Structured repository risk findings shipped: repo-state summaries now expose machine-readable findings, top risk item, and severity counts for decision support
- Function-tree governance shipped: `FUNCTION_TREE.md`, the task-card template, and a lightweight validator now define capability-first execution cards
- Configuration management shipped: `CONF-01..03` now exists in code with global defaults, project overrides, CLI/MCP surfaces, and daemon-safe live reload
- Portable export shipped: `EXPORT-01..02` now exists in code with stable JSON/CSV artifact generation for project evidence rows
- Comparative reporting shipped: `RPT-01..03` now exists in code with time-window summaries, snapshot comparison, usage trends, and CLI/MCP/daemon-control access
- Retained-evidence lifecycle shipped: users and AI can selectively prune retained OPENDOG evidence per project, inspect storage-maintenance signals, and keep source files untouched

### Active

Current active work is intentionally focused on hardening and selectively deepening already shipped Phase 6 / `FT-03` capability families. The surface area is broad by design because OPENDOG sits above multiple projects, but future work should stay bounded by the project's observation-first and decision-support-only principles.

- [ ] Continue strengthening workspace observation layer — improve freshness, readiness, and project-level state summaries
- [ ] Continue strengthening repository status and risk summaries — build richer strategy coupling and workspace aggregation on top of the new structured findings
- [ ] Continue strengthening AI execution strategy suggestions — make tool-vs-shell choice and sequencing more explicit
- [ ] Continue strengthening verification and evidence layer — broaden how evidence is attached to recommendations and safety gates
- [ ] Continue strengthening multi-project portfolio views — extend attention scoring into richer aggregation and review batching
- [ ] Continue strengthening cleanup and refactor candidate layer — improve file-level prioritization and review ordering
- [ ] Continue strengthening project type and toolchain identification — improve confidence and command recommendations
- [ ] Continue strengthening constraint and boundary layer — make blind spots and non-authoritative areas even more explicit
- [ ] Continue tuning MOCK/hardcoded data detection — reduce false positives and improve review signals
- [ ] Continue improving AI/operator documentation so new CLI/MCP capabilities are actually discoverable and used correctly
- [ ] Keep `Maps to FT:` ownership current whenever requirement sections are added or revised
- [ ] Keep adding concrete task cards under `.planning/task-cards/` instead of ad hoc execution notes
- [ ] Keep `.planning/GOVERNANCE.md` and `scripts/validate_planning_governance.py` aligned with the real planning workflow
- [ ] Continue refining the new comparative reporting surfaces so AI clients can combine report outputs with guidance, verification, and data-risk layers without ambiguity
- [ ] Keep future retention and coordination work mapped to `FT-01.04.05` and `FT-02.03.02` instead of reintroducing interface-first capability ownership
- [ ] Prefer deepening `FT-03` trustworthiness, evidence quality, and operator clarity over opening unrelated new capability families

### Out of Scope

- Visual dashboard / web UI — terminal-based for v1
- Auto-cleanup of unused files — only identify, never delete
- Windows native support — WSL only
- Remote/network monitoring — local filesystem only
- Real-time streaming to external services — local SQLite storage only

## Context

- Built for WSL2 environment — requires real Linux kernel for inotify and /proc filesystem
- Target users are developers running multiple AI-assisted projects concurrently (Codex + GPT-5.4, Claude Code + GLM-5.1)
- Current design judgment: no product-direction drift; the newer guidance, risk, verification, cleanup, and mock-review layers are depth extensions of the original mission
- Current scope is intentionally broad but bounded: OPENDOG observes, summarizes, prioritizes, and advises, but does not replace git truth, test truth, build truth, or destructive repo operations
- ⚠ **Derived design decision**: inotify does NOT provide process attribution (per inotify(7)). Actual approach: periodic /proc/<pid>/fd scanning (primary — what files AI processes have open) + inotify change detection via notify crate (secondary — what files changed). Cross-referenced by timestamp for approximate attribution. This is statistical sampling (2-5s intervals), not precise per-event auditing.
- Each project maps to one SQLite .db file that holds snapshot baseline, usage evidence, verification runs, and retained operational evidence
- Daemon, CLI, and MCP share a local control plane so AI tools do not silently diverge from daemon-owned monitoring state
- MCP stdio transport via rmcp crate — AI tools call OPENDOG directly
- Service delivery is now split conceptually into a CLI operator surface, an MCP AI surface, and daemon/runtime coordination
- MCP is not only a control surface; it is also a reusable decision-support surface for AI workflows, especially across multiple projects that should not each implement their own observation, review, and boundary metadata layers
- Retained OPENDOG evidence is managed separately from project source files so long-lived multi-project deployments can prune monitoring history safely
- Rust chosen for memory safety, low resource overhead, and 7x24 daemon stability

## Constraints

- **Platform**: WSL (Windows Subsystem for Linux) — relies on Linux inotify API
- **Language**: Rust (release-optimized builds)
- **Resource Budget**: CPU < 1% at idle, memory < 10MB
- **Storage**: SQLite per project — no external database dependencies
- **Transport**: MCP over stdio (no HTTP/SSE in v1)
- **Deployment**: Systemd service for daemon management

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as implementation language | Memory safety + low overhead for long-running daemon | Adopted |
| Per-project SQLite isolation | Zero cross-project data leakage, simple backup/deletion | Adopted |
| /proc/<pid>/fd scanning + process whitelist for approximate attribution | inotify cannot provide PID; /proc scanning is the viable non-intrusive alternative | Adopted |
| MCP stdio transport | Standard for CLI-integrated MCP servers, matches Claude Code integration | Adopted |
| MCP should provide reusable information layers, not only raw control operations | AI needs decision support, evidence, and explicit boundaries about what to inspect or run next | Adopted |
| CLI and MCP should route reusable project operations through the local control plane when the daemon is live | Avoid duplicate monitor ownership and state drift between interfaces | Adopted |
| Verification evidence should be stored, not inferred ad hoc from ephemeral shell output | AI needs durable evidence and safety gates across sessions | Adopted |
| Retained OPENDOG evidence should be manageable separately from source files | Long-lived multi-project deployments need safe cleanup and storage-hygiene operations without destructive repo actions | Adopted |
| `FUNCTION_TREE.md` should become the canonical business-capability anchor between `PROJECT.md`, `REQUIREMENTS.md`, and roadmap/task execution artifacts | Requirements alone are too flat to express durable capability ownership, change impact, and gating rules for AI-driven project evolution | Adopted |
| Function-tree governance should be applied proportionally to project scale and change risk | Capability ownership and validation are valuable anchors, but always-heavy ceremony would create more process than leverage for small iterations | Adopted |
| Current Phase 6 work should deepen existing `FT-03` capability quality before opening unrelated new families | The current surface is already broad enough; the next leverage comes from trustworthiness, clearer evidence, and tighter boundaries | Adopted |
| inotify for file watching | Kernel-level, non-intrusive change detection complements /proc-based attribution even though it cannot provide PID | Adopted |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-28 after aligning current-state terminology with the Phase 6 hardening posture, bounded-scope framing, and proportional governance guidance*
