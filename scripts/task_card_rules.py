from __future__ import annotations

import re
from pathlib import Path


ALLOWED_STATUSES = {
    "proposed",
    "in_progress",
    "completed",
    "blocked",
    "cancelled",
}


def read_frontmatter(path: Path) -> str:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0].strip() != "---":
        raise ValueError("missing frontmatter start fence")

    end = None
    for idx in range(1, len(lines)):
        if lines[idx].strip() == "---":
            end = idx
            break

    if end is None:
        raise ValueError("missing frontmatter end fence")

    return "\n".join(lines[1:end])


def field_value(block: str, key: str) -> str | None:
    match = re.search(rf"(?m)^{re.escape(key)}:\s*(.+)$", block)
    if not match:
        return None
    return match.group(1).strip().strip('"')


def list_values(block: str, key: str) -> list[str]:
    match = re.search(
        rf"(?m)^{re.escape(key)}:\s*\n((?:^[ \t]+- .*(?:\n|$))+)",
        block,
    )
    if not match:
        return []

    items: list[str] = []
    for line in match.group(1).splitlines():
        if line.lstrip().startswith("- "):
            items.append(line.split("- ", 1)[1].strip().strip('"'))
    return items


def validate_card(
    path: Path,
    ft_levels: dict[str, str],
    valid_requirement_ids: set[str] | None = None,
) -> list[str]:
    errors: list[str] = []
    try:
        block = read_frontmatter(path)
    except Exception as exc:  # noqa: BLE001
        return [f"{path}: {exc}"]

    card_id = field_value(block, "id")
    if not card_id:
        errors.append("missing id")
    elif path.stem != card_id:
        errors.append(f"id '{card_id}' does not match filename stem '{path.stem}'")

    status = field_value(block, "status")
    if not status:
        errors.append("missing status")
    elif status not in ALLOWED_STATUSES:
        allowed = ", ".join(sorted(ALLOWED_STATUSES))
        errors.append(f"invalid status '{status}' (expected one of: {allowed})")

    required_lists = {
        "ft_ids_touched": True,
        "why_these_ft_ids": True,
        "verification_plan": True,
    }

    for key in required_lists:
        values = list_values(block, key)
        if not values:
            errors.append(f"missing or empty {key}")

    for ft_id in list_values(block, "ft_ids_touched"):
        if ft_id not in ft_levels:
            errors.append(f"unknown FT id '{ft_id}'")
        elif ft_levels[ft_id] != "L3":
            errors.append(f"{ft_id} is {ft_levels[ft_id] or 'untyped'}, expected L3 leaf")

    if valid_requirement_ids is not None:
        for requirement_id in list_values(block, "requirement_ids"):
            if requirement_id not in valid_requirement_ids:
                errors.append(f"unknown requirement id '{requirement_id}'")

    return errors


def card_status(path: Path) -> str:
    block = read_frontmatter(path)
    status = field_value(block, "status")
    if not status:
        return "missing"
    if status not in ALLOWED_STATUSES:
        return "invalid"
    return status


def collect_status_counts(paths: list[Path]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for path in paths:
        status = card_status(path)
        counts[status] = counts.get(status, 0) + 1
    return counts
