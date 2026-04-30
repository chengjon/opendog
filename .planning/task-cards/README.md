# Task Cards

Use this directory for concrete execution cards.

## Rules

- One file per task card
- File name pattern: `TASK-YYYYMMDD-<slug>.md`
- Each card must declare a valid `status`
- Each card must declare `ft_ids_touched`
- `ft_ids_touched` should point to capability leaves, not interface labels; use `interface_surfaces` separately for `cli` / `mcp` / `daemon`
- Each card must explain why those `FT-*` leaves change
- Each card must include a verification plan

## Validator

Run:

```bash
python3 scripts/validate_planning_governance.py
```

Task-card-only check:

```bash
python3 scripts/validate_task_cards.py
```

## Examples

- [`TASK-20260426-phase6-guidance-hardening.md`](./TASK-20260426-phase6-guidance-hardening.md)
- [`TASK-20260427-configuration-policy-and-live-reload.md`](./TASK-20260427-configuration-policy-and-live-reload.md)
- [`TASK-20260427-portable-usage-export.md`](./TASK-20260427-portable-usage-export.md)
- [`TASK-20260427-comparative-time-window-analytics.md`](./TASK-20260427-comparative-time-window-analytics.md)
- [`TASK-20260427-local-control-plane-coordination-hardening.md`](./TASK-20260427-local-control-plane-coordination-hardening.md)
- [`TASK-20260427-retained-evidence-lifecycle-hardening.md`](./TASK-20260427-retained-evidence-lifecycle-hardening.md)
- [`TASK-20260428-observation-freshness-and-evidence-coverage.md`](./TASK-20260428-observation-freshness-and-evidence-coverage.md)
- [`TASK-20260428-workspace-portfolio-attention-scoring.md`](./TASK-20260428-workspace-portfolio-attention-scoring.md)
- [`TASK-20260428-repository-risk-findings-structure.md`](./TASK-20260428-repository-risk-findings-structure.md)
