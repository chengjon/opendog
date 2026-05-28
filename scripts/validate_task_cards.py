#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from task_card_rules import collect_status_counts, validate_card
import validate_requirement_mappings as requirement_mappings


ROOT = Path(__file__).resolve().parents[1]
TASK_DIR = ROOT / ".planning" / "task-cards"
TREE_FILE = ROOT / "FUNCTION_TREE.md"
REQUIREMENTS_FILE = ROOT / ".planning" / "REQUIREMENTS.md"


def main() -> int:
    if not TASK_DIR.exists():
        print(f"task-card directory not found: {TASK_DIR}")
        return 1

    ft_levels = requirement_mappings.parse_function_tree_levels(
        TREE_FILE.read_text(encoding="utf-8")
    )
    requirement_sections = requirement_mappings.parse_requirement_sections(
        REQUIREMENTS_FILE.read_text(encoding="utf-8")
    )
    valid_requirement_ids = {
        req_id
        for section in requirement_sections
        for req_id in section["requirement_ids"]
    }
    task_files = sorted(TASK_DIR.glob("TASK-*.md"))

    if not task_files:
        print("no task cards found")
        return 0

    failures: list[str] = []
    for path in task_files:
        failures.extend(validate_card(path, ft_levels, valid_requirement_ids))

    if failures:
        print("task-card validation failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1

    status_counts = collect_status_counts(task_files)
    status_summary = ", ".join(
        f"{count} {status}" for status, count in sorted(status_counts.items())
    )
    print(f"validated {len(task_files)} task card(s) [{status_summary}]")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
