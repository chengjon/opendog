---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-05-14T07:30:55.462Z"
---

# State: OPENDOG

**Updated:** 2026-04-28

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-28)

**Core value:** Accurately identify which project files AI tools actually use and which are dead weight, then expose that activity as reusable intelligence for AI workflows
**Current focus:** Phase 6 hardening and selective deepening — keep the shipped guidance, freshness/coverage metadata, attention scoring, structured repository risk findings, verification, data-risk, retained-evidence, and local-control-plane capabilities aligned with the function tree, requirement mappings, task cards, and operator-facing docs, while prioritizing `FT-03` trustworthiness over opening unrelated new capability families

## Current Design Judgment

- No direction drift: the project still centers on multi-project AI observation plus decision support
- Broad but bounded scope: the capability surface is wider than a simple file-monitor, but it remains constrained by explicit observation, evidence, and non-destructive advisory boundaries
- Current priority is depth over breadth: strengthen existing Phase 6 families, especially evidence quality, strategy clarity, and boundary messaging, before expanding the function map further
- FD attribution credibility is now a governed baseline: `fix-fd-attribution` is accepted, verified, and future scanner attribution changes must use OpenSpec governance before shipping

## Phase Status

| Phase | Name | Status | Progress |
|-------|------|--------|----------|
| 1 | Foundation — Storage, Project & Snapshot | ✅ | 100% |
| 2 | Monitoring Engine — /proc Scanner + inotify | ✅ | 100% |
| 3 | Statistics Engine — Usage Analytics | ✅ | 100% |
| 4 | Service Interfaces — MCP Server & CLI | ✅ | 100% |
| 5 | Daemon & Deployment | ✅ | 100% |
| 6 | AI Guidance & Reusable Intelligence | In Progress | Core shipped; hardening active |

## Completed Phases

### Phase 1: Foundation — Storage, Project & Snapshot

**Requirements covered:** PROJ-01..05, SNAP-01..05 (10 requirements)

- SQLite storage layer with WAL mode, per-project database isolation
- Project manager with CRUD operations and configurable data directory
- Snapshot engine with recursive directory scanning and smart filtering
- 17 integration tests

### Phase 2: Monitoring Engine — /proc Scanner + inotify

**Requirements covered:** MON-01..06, PROC-01..06 (12 requirements)

- /proc scanner with process name whitelist and fd symlink resolution (procfs crate)
- inotify change detection via notify crate (recursive watches)
- Approximate attribution: timestamp cross-reference between sightings and events
- Monitor threads with start/stop lifecycle, open-state tracking, duration accumulation

### Phase 3: Statistics Engine — Usage Analytics

**Requirements covered:** STAT-01..08 (8 requirements)

- Per-file access count, estimated duration, modification count, last access tracking
- Unused file detection (snapshot LEFT JOIN file_stats where never accessed)
- Core file identification (high access_count threshold query)
- Project summary (total/accessed/unused counts)
- 8 new integration tests

### Phase 4: Service Interfaces — MCP Server & CLI

**Requirements covered:** MCP-01..09, CLI-01..09 (18 requirements)

- MCP server via rmcp 1.5 with stdio transport and shared tool-routing over the core services
- CLI via clap 4 with shared reporting, JSON output, and operator-friendly summaries
- Shared core logic between both interfaces, with later hardening extending beyond the original CRUD/stat surface
- Automatic mode detection: stdin pipe -> MCP, terminal -> CLI

### Phase 5: Daemon & Deployment

**Requirements covered:** DAEM-01..05 (5 requirements)

- Daemon mode with `opendog daemon` command
- SIGTERM graceful shutdown (tokio::select! with ctrl_c)
- sd_notify integration for systemd Type=notify
- Structured logging: journald (when JOURNAL_STREAM set) or JSON to stderr fallback
- WSL2 detection with WSL1 deprecation warning
- systemd unit file with resource limits (10MB RAM, 1% CPU), security hardening
- PID file management

## Key Metrics

- **Requirements:** 114 total | 114 phase-mapped | 0 backlog / unscheduled
- **Function tree:** 3 L1 domains | 26 L3 leaf capabilities | 0 orphan requirement sections
- **Governance artifacts:** 9 validated task cards | inline FT ownership across 22 requirement sections
- **Tests in suite:** 106 unit tests + 22 integration tests
- **Warnings:** 0
- **Overall progress:** v1 complete; Phase 6 core capabilities shipped and now in refinement/hardening

## Next Milestone Queue

- Keep function-tree-based capability governance active, but apply it proportionally to project scale and change risk
- Treat `/proc/<pid>/fd` scanner attribution as a high-trust-boundary path: future semantic changes require OpenSpec proposal, review plan, task mapping, and verification evidence
- Treat `TASK-20260509-agent-guidance-utf8-panic` as the completed reference for separating guidance-layer defects from scanner attribution governance
- Treat `TASK-20260510-mcp-observation-payload-bounds` as the completed reference for bounding MCP row-heavy observation payloads without removing full CLI visibility
- Treat `TASK-20260510-daemon-ipc-response-integrity` as the completed reference for separating daemon transport-integrity failures from business-logic serialization errors
- Treat `TASK-20260510-infrastructure-file-classification` as the completed reference for separating source-file signal from AI/tool infrastructure noise without changing scanner attribution or default ignore semantics
- Treat `TASK-20260510-data-risk-context-aware-noise-reduction` as the completed reference for lowering documentation/template data-risk false positives without hiding audit evidence
- Treat `TASK-20260510-mcp-regression-coverage-expansion` as the completed reference for adding coverage around high-value MCP/daemon paths without changing contracts
- Treat `TASK-20260510-verification-evidence-ttl-policy` as the completed reference for exposing default verification freshness TTL policy without hiding stale evidence
- Treat `TASK-20260510-mcp-readonly-resources` as the completed reference for adding low-parameter read-only MCP Resources while keeping mutations on tools/CLI
- Treat `TASK-20260511-source-signal-observation-calibration` as the completed reference for proving transient Claude Code source reads are a sampling boundary, not a scanner-attribution regression
- Treat `TASK-20260511-source-first-observation-views` as the completed reference for resolving infrastructure-dominated observation outputs with source-first filters and guidance boundaries
- Treat `TASK-20260511-manual-self-update-workflow` as the completed reference for CLI-only release-binary maintenance without letting MCP mutate host configuration or restart itself
- Next governed improvement candidates should be created from fresh project-exchange evidence rather than continuing the closed 2026-05-10 hardening batch
- Use `.planning/TASK_CARD_TEMPLATE.md` as the default execution card format for substantial capability work
- Use `.planning/GOVERNANCE.md` as the canonical operator/AI workflow for planning artifacts without forcing heavyweight ceremony on every small iteration
- Prefer `scripts/validate_planning_governance.py` as the single governance check when planning artifacts change materially
- Use `.planning/task-cards/` for concrete execution cards and validate them with `scripts/validate_task_cards.py` when the work introduces or materially reshapes capability scope
- Validate and preserve requirement-section ownership with `scripts/validate_requirement_mappings.py`
- Keep local control-plane coordination mapped to `FT-02.03.02` instead of drifting back into CLI/MCP-specific ownership
- Keep retained-evidence cleanup and storage hygiene mapped to `FT-01.04.05` instead of framing it as source cleanup
- Keep backlog-only requirement families explicit if any future requirement families are intentionally left unscheduled
- Treat `TASK-20260427-comparative-time-window-analytics` as the completed reference pattern for promoting a governed backlog family into shipped code
- Treat `TASK-20260428-observation-freshness-and-evidence-coverage` as the reference pattern for tightening machine-readable guidance metadata without inventing a new capability family
- Treat `TASK-20260428-workspace-portfolio-attention-scoring` as the reference pattern for replacing opaque ordering logic with shared machine-readable prioritization
- Treat `TASK-20260428-repository-risk-findings-structure` as the reference pattern for converting text-only risk summaries into structured decision-support fields
- Keep retention cleanup discoverable so long-lived multi-project deployments can prune OPENDOG evidence without deleting project source files
- Treat the current design assessment as stable guidance: no direction drift, broad but bounded scope, and selective deepening before new families
- Continue tightening workspace observation and readiness signals
- Continue tightening repository status and risk summaries above the new structured-finding baseline
- Continue tightening AI execution strategy suggestions
- Expand verification and evidence workflows
- Continue tightening multi-project portfolio aggregation above the new attention-scoring baseline
- Improve cleanup/refactor prioritization and review ordering
- Improve stack/toolchain confidence and recommended commands
- Tighten constraints and boundary messaging for AI consumers
- Keep tuning MOCK / hardcoded pseudo-data detection and review evidence

---
*State updated: 2026-04-28 after aligning Phase 6 hardening work with bounded-scope framing, selective deepening priorities, and proportional governance guidance*
