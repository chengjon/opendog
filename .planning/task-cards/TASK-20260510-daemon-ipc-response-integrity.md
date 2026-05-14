---
title: "Harden daemon IPC response integrity for guidance payloads"
id: "TASK-20260510-daemon-ipc-response-integrity"
status: completed
owner: "codex"
priority: high
phase_hint: "Phase 6 daemon and MCP reliability hardening"
ft_ids_touched:
  - FT-02.03.02
  - FT-02.02.01
  - FT-03.02.02
  - FT-03.07.01
why_these_ft_ids:
  - "FT-02.03.02 owns daemon-local control-plane reuse; socket clients must detect empty or truncated responses precisely."
  - "FT-02.02.01 owns MCP reporting; MCP guidance must not degrade into ambiguous serialization errors when daemon IPC fails."
  - "FT-03.02.02 owns decision guidance; `get_guidance(detail=decision)` must be reliable for AI next-step planning."
  - "FT-03.07.01 owns authority boundaries; transport failure must be distinguishable from valid but incomplete observation evidence."
requirement_ids:
  - CTRL-01
  - CTRL-02
  - MCP-01
  - STRAT-02
  - STRAT-04
  - BOUND-03
  - BOUND-04
interface_surfaces:
  - daemon
  - mcp
  - cli
non_goals:
  - "Do not change scanner attribution semantics."
  - "Do not solve large stats/unused payload shaping in this task; that is owned by TASK-20260510-mcp-observation-payload-bounds."
  - "Do not replace the local Unix socket control plane."
verification_plan:
  - "Reproduce or simulate an empty/truncated daemon socket response and assert it returns a specific transport/integrity error, not a generic serialization error."
  - "Verify `get_guidance(detail=decision, top=3)` over daemon-backed MCP returns valid JSON for the `mystocks` scale case after the fix."
  - "Add tests around `DaemonClient::send()` response framing or completeness checks."
  - "Run `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py`."
evidence_outputs:
  - "Regression test for empty/truncated daemon response handling."
  - "Large guidance payload validation against `mystocks` or an equivalent fixture."
  - "Updated error contract notes if a new transport-integrity error code is exposed."
---

## Goal

Make daemon-backed MCP guidance calls robust against empty or truncated Unix socket responses and expose precise errors when transport integrity fails.

## Evidence Source

`/opt/claude/mystocks_spec/docs/operations/monitoring/OPENDOG_USAGE_FEEDBACK.md` records that:

- MCP `get_guidance {project_id: "mystocks", detail: "decision", top: 3}` returned `serialization_error`.
- CLI `opendog decision-brief --project mystocks --top 3 --json` produced valid JSON around 101KB.
- The daemon was running and the socket existed, so the failure likely sits in daemon IPC response handling or response-size interaction, not core decision-brief generation.

## Change Plan

1. Trace daemon-backed MCP guidance request and response framing.
2. Add explicit empty-response and truncated-response detection before JSON deserialization.
3. Keep business logic errors separate from daemon IPC integrity failures.
4. Verify large decision guidance through daemon-backed MCP on `mystocks` scale.

## Guardrails

- Preserve successful CLI behavior.
- Preserve daemon reuse semantics.
- Keep this task independent from stats/unused payload bounding and fd attribution governance.

## Completion Criteria

- Empty/truncated daemon responses produce a precise error contract.
- Valid decision guidance payloads return through daemon-backed MCP.
- CLI and MCP guidance behavior are aligned for normal payload sizes.

## Completion Note

Daemon IPC response decoding now detects empty daemon responses and EOF/truncated JSON before falling back to generic serialization errors. MCP error contracts expose these failures as `daemon_response_integrity_error` with remediation guidance.

Verification evidence:

- `cargo test decode_control_response_reports` passes for empty and truncated daemon response cases.
- `cargo test error_contracts` passes with the `daemon_response_integrity_error` contract.
- `env OPENDOG_HOME=/root/.opendog target/debug/opendog decision-brief --project mystocks --top 3 --json` exits 0 against the existing daemon-backed `mystocks` state.
- `docs/json-contracts.md` documents the daemon response integrity error boundary.
- `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `python3 scripts/validate_planning_governance.py` pass.
- `cargo build --release` completed so MCP hosts pointing at `/opt/claude/opendog/target/release/opendog` can use the response-integrity error classification.
