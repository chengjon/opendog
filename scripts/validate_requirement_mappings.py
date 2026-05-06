#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TREE_FILE = ROOT / "FUNCTION_TREE.md"
REQUIREMENTS_FILE = ROOT / ".planning" / "REQUIREMENTS.md"


def parse_function_tree_levels(text: str) -> dict[str, str]:
    levels: dict[str, str] = {}
    current_id: str | None = None

    for line in text.splitlines():
        id_match = re.match(r"^\s*-\s+id:\s*(FT-[0-9.]+)\s*$", line)
        if id_match:
            current_id = id_match.group(1)
            levels[current_id] = ""
            continue

        if current_id is None:
            continue

        level_match = re.match(r"^\s*level:\s*(L\d)\s*$", line)
        if level_match:
            levels[current_id] = level_match.group(1)

    return levels


def parse_requirement_sections(text: str) -> list[dict[str, object]]:
    sections: list[dict[str, object]] = []
    current: dict[str, object] | None = None

    for line_no, line in enumerate(text.splitlines(), start=1):
        heading = re.match(r"^###\s+(.+?)\s*$", line)
        if heading:
            if current is not None:
                sections.append(current)
            current = {
                "title": heading.group(1),
                "heading_line": line_no,
                "maps_line": None,
                "requirement_ids": [],
            }
            continue

        if current is None:
            continue

        maps = re.match(r"^Maps to FT:\s*(.+)$", line)
        if maps and current["maps_line"] is None:
            current["maps_line"] = maps.group(1)
            continue

        req = re.match(r"^\s*-\s*(?:\[[ xX]\]\s*)?\*\*([A-Z0-9-]+)\*\*:", line)
        if req:
            current["requirement_ids"].append(req.group(1))

    if current is not None:
        sections.append(current)

    return sections


def validate_sections(sections: list[dict[str, object]], ft_levels: dict[str, str]) -> list[str]:
    errors: list[str] = []

    for section in sections:
        requirement_ids = section["requirement_ids"]
        if not requirement_ids:
            continue

        maps_line = section["maps_line"]
        title = section["title"]
        heading_line = section["heading_line"]

        if not maps_line:
            errors.append(f"{REQUIREMENTS_FILE}:{heading_line}: section '{title}' is missing 'Maps to FT:'")
            continue

        ft_ids = re.findall(r"FT-[0-9.]+", str(maps_line))
        if not ft_ids:
            errors.append(f"{REQUIREMENTS_FILE}:{heading_line}: section '{title}' has no FT ids")
            continue

        for ft_id in ft_ids:
            if ft_id not in ft_levels:
                errors.append(f"{REQUIREMENTS_FILE}:{heading_line}: unknown FT id '{ft_id}' in section '{title}'")
            elif ft_levels[ft_id] != "L3":
                errors.append(
                    f"{REQUIREMENTS_FILE}:{heading_line}: {ft_id} in section '{title}' is {ft_levels[ft_id] or 'untyped'}, expected L3 leaf"
                )

    return errors


def main() -> int:
    ft_levels = parse_function_tree_levels(TREE_FILE.read_text(encoding="utf-8"))
    sections = parse_requirement_sections(REQUIREMENTS_FILE.read_text(encoding="utf-8"))
    errors = validate_sections(sections, ft_levels)

    if errors:
        print("requirement mapping validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    covered_sections = sum(1 for section in sections if section["requirement_ids"])
    print(f"validated requirement mappings for {covered_sections} section(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
