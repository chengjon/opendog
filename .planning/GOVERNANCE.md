# Planning Governance

This file describes the minimum governance loop for OPENDOG planning artifacts.

## Canonical Chain

Use this order when changing product scope:

1. `.planning/PROJECT.md`
2. `.planning/FUNCTION_TREE.md`
3. `.planning/REQUIREMENTS.md`
4. `.planning/ROADMAP.md`
5. `.planning/task-cards/`

Interpretation:

- `PROJECT.md` defines intent and active priorities
- `FUNCTION_TREE.md` defines durable capability ownership
- `REQUIREMENTS.md` defines requirement statements and section ownership
- `ROADMAP.md` defines scheduled vs backlog scope
- task cards define bounded execution slices

## Required Rules

- New requirement sections must include `Maps to FT:`
- New task cards must include `ft_ids_touched`
- Task cards should only target `L3` function-tree leaves
- Backlog requirements must stay visible instead of being silently dropped
- Capability changes should update `FUNCTION_TREE.md` before or alongside execution cards

## Validation Commands

Preferred single entrypoint:

```bash
python3 scripts/validate_planning_governance.py
```

Lower-level validators:

```bash
python3 scripts/validate_task_cards.py
python3 scripts/validate_requirement_mappings.py
python3 scripts/validate_structural_hygiene.py
```

## Structural Hygiene Gate

In addition to ownership and mapping checks, OPENDOG now enforces a lightweight structural hygiene gate for oversized source and documentation files.

- Machine-readable policy: `.planning/structural_hygiene_rules.json`
- Validator: `scripts/validate_structural_hygiene.py`
- Current posture: default line-count and byte-size budgets for key Rust/Python/Markdown surfaces, a strict dedicated budget for `src/mcp/mod.rs`, and a separate budget for embedded Rust test modules such as `src/**/tests.rs`; temporary legacy exceptions should be removed once files fall back under the default budget

This gate is meant to stop new oversized files from entering the repo silently while still allowing gradual reduction of existing large-file debt.

## When To Update What

### Add or revise requirements

- update the relevant section in `.planning/REQUIREMENTS.md`
- keep or refine its `Maps to FT:` ownership
- update `.planning/FUNCTION_TREE.md` if the capability boundary changed
- update `.planning/ROADMAP.md` if scheduling or backlog state changed

### Add execution work

- create a task card in `.planning/task-cards/`
- map it to `FT-*` leaf nodes
- explain why the chosen leaves are affected
- include verification steps

### Add a new capability

- add or refine a function-tree node first
- then map requirements
- then schedule it in roadmap or backlog
- then create task cards

## Backlog Policy

If a requirement family is valid but not yet phase-scheduled:

- keep it in `REQUIREMENTS.md`
- map it to an `FT-*` leaf
- show it in the roadmap backlog section
- do not count it as phase-mapped work

## Backlog Maturity Ladder

Use this progression instead of jumping directly from idea to phase:

1. requirement is written and mapped in `.planning/REQUIREMENTS.md`
2. capability ownership exists in `.planning/FUNCTION_TREE.md`
3. backlog visibility exists in `.planning/ROADMAP.md`
4. concrete task card exists in `.planning/task-cards/`
5. numbered roadmap phase is assigned only when scheduling commitment is real

This keeps backlog work visible, reviewable, and execution-ready without pretending it is already phase-committed.
