#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

import validate_requirement_mappings as requirement_mappings
import validate_structural_hygiene as structural_hygiene
import validate_task_cards as task_cards


ROOT = Path(__file__).resolve().parents[1]
FUNCTION_TREE_FILE = ROOT / ".planning" / "FUNCTION_TREE.md"
REQUIREMENTS_FILE = ROOT / ".planning" / "REQUIREMENTS.md"
ROADMAP_FILE = ROOT / ".planning" / "ROADMAP.md"
TASK_CARD_DIR = ROOT / ".planning" / "task-cards"


def parse_list_field(line: str, field_name: str) -> list[str]:
    match = re.match(rf"^\s*{re.escape(field_name)}:\s*\[(.*)\]\s*$", line)
    if not match:
        return []
    inner = match.group(1).strip()
    if not inner:
        return []
    return [item.strip() for item in inner.split(",") if item.strip()]


def expand_requirement_token(token: str) -> list[str]:
    token = token.strip()
    range_match = re.match(r"^([A-Z0-9]+-)(\d+)\.\.(\d+)$", token)
    if range_match:
        prefix, start, end = range_match.groups()
        width = len(start)
        return [f"{prefix}{value:0{width}d}" for value in range(int(start), int(end) + 1)]
    return [token]


def parse_function_tree_nodes(text: str) -> dict[str, dict[str, object]]:
    nodes: dict[str, dict[str, object]] = {}
    current_id: str | None = None

    for line in text.splitlines():
        id_match = re.match(r"^\s*-\s+id:\s*(FT-[0-9.]+)\s*$", line)
        if id_match:
            current_id = id_match.group(1)
            nodes[current_id] = {
                "level": "",
                "requirement_ids": [],
                "roadmap_phases": [],
            }
            continue

        if current_id is None:
            continue

        level_match = re.match(r"^\s*level:\s*(L\d)\s*$", line)
        if level_match:
            nodes[current_id]["level"] = level_match.group(1)
            continue

        req_tokens = parse_list_field(line, "requirement_ranges")
        if req_tokens:
            requirement_ids: list[str] = []
            for token in req_tokens:
                requirement_ids.extend(expand_requirement_token(token))
            nodes[current_id]["requirement_ids"] = requirement_ids
            continue

        if re.match(r"^\s*roadmap_phases:\s*\[.*\]\s*$", line):
            phase_tokens = parse_list_field(line, "roadmap_phases")
            nodes[current_id]["roadmap_phases"] = [int(token) for token in phase_tokens]

    return nodes


def validate_task_cards(ft_levels: dict[str, str]) -> list[str]:
    requirement_sections = requirement_mappings.parse_requirement_sections(
        REQUIREMENTS_FILE.read_text(encoding="utf-8")
    )
    valid_requirement_ids = {
        req_id
        for section in requirement_sections
        for req_id in section["requirement_ids"]
    }
    errors: list[str] = []
    for path in sorted(TASK_CARD_DIR.glob("TASK-*.md")):
        errors.extend(task_cards.validate_card(path, ft_levels, valid_requirement_ids))
    return errors


def validate_requirement_sections(ft_levels: dict[str, str]) -> tuple[list[str], list[dict[str, object]]]:
    sections = requirement_mappings.parse_requirement_sections(REQUIREMENTS_FILE.read_text(encoding="utf-8"))
    errors = requirement_mappings.validate_sections(sections, ft_levels)
    return errors, sections


def validate_function_tree_coverage(
    nodes: dict[str, dict[str, object]],
    sections: list[dict[str, object]],
) -> list[str]:
    errors: list[str] = []
    requirement_ids = {req_id for section in sections for req_id in section["requirement_ids"]}
    ft_requirement_to_leaves: dict[str, set[str]] = {}

    for ft_id, node in nodes.items():
        for requirement_id in node["requirement_ids"]:
            ft_requirement_to_leaves.setdefault(requirement_id, set()).add(ft_id)

    for requirement_id in sorted(requirement_ids):
        if requirement_id not in ft_requirement_to_leaves:
            errors.append(f"requirement '{requirement_id}' exists in REQUIREMENTS.md but not in FUNCTION_TREE.md")

    for requirement_id, leaves in sorted(ft_requirement_to_leaves.items()):
        if requirement_id not in requirement_ids:
            errors.append(f"requirement '{requirement_id}' exists in FUNCTION_TREE.md but not in REQUIREMENTS.md")
        for leaf in leaves:
            if nodes[leaf]["level"] != "L3":
                errors.append(f"{leaf} maps requirement '{requirement_id}' but is not an L3 leaf")

    for section in sections:
        if not section["requirement_ids"]:
            continue
        ft_ids = re.findall(r"FT-[0-9.]+", str(section["maps_line"]))
        covered: set[str] = set()
        for ft_id in ft_ids:
            node = nodes.get(ft_id)
            if node:
                covered.update(node["requirement_ids"])
        for requirement_id in section["requirement_ids"]:
            if requirement_id not in covered:
                errors.append(
                    f"section '{section['title']}' maps to {', '.join(ft_ids)} but requirement '{requirement_id}' is not covered by those FT leaves"
                )

    return errors


def compute_counts(nodes: dict[str, dict[str, object]], sections: list[dict[str, object]]) -> dict[str, int]:
    total_requirement_ids = {req_id for section in sections for req_id in section["requirement_ids"]}
    phase_mapped: set[str] = set()
    backlog: set[str] = set()

    for node in nodes.values():
        req_ids = set(node["requirement_ids"])
        if node["roadmap_phases"]:
            phase_mapped.update(req_ids)
        else:
            backlog.update(req_ids)

    backlog -= phase_mapped

    return {
        "total": len(total_requirement_ids),
        "phase_mapped": len(phase_mapped),
        "backlog": len(backlog),
    }


def validate_roadmap_counts(counts: dict[str, int]) -> list[str]:
    errors: list[str] = []
    text = ROADMAP_FILE.read_text(encoding="utf-8")
    match = re.search(r"^\*\*Requirements:\*\*\s*(\d+)\s+total\s+\|\s+(\d+)\s+phase-mapped\s+\|\s+(\d+)\s+backlog\s*$", text, re.M)
    if not match:
        return [f"{ROADMAP_FILE}: missing or malformed requirements headline"]

    expected_total, expected_phase_mapped, expected_backlog = map(int, match.groups())
    if expected_total != counts["total"]:
        errors.append(f"{ROADMAP_FILE}: header total={expected_total} but computed total={counts['total']}")
    if expected_phase_mapped != counts["phase_mapped"]:
        errors.append(
            f"{ROADMAP_FILE}: header phase-mapped={expected_phase_mapped} but computed phase-mapped={counts['phase_mapped']}"
        )
    if expected_backlog != counts["backlog"]:
        errors.append(f"{ROADMAP_FILE}: header backlog={expected_backlog} but computed backlog={counts['backlog']}")
    return errors


def main() -> int:
    ft_text = FUNCTION_TREE_FILE.read_text(encoding="utf-8")
    ft_levels = requirement_mappings.parse_function_tree_levels(ft_text)
    ft_nodes = parse_function_tree_nodes(ft_text)
    task_files = sorted(TASK_CARD_DIR.glob("TASK-*.md"))

    errors: list[str] = []
    errors.extend(validate_task_cards(ft_levels))
    requirement_errors, sections = validate_requirement_sections(ft_levels)
    errors.extend(requirement_errors)
    errors.extend(validate_function_tree_coverage(ft_nodes, sections))
    counts = compute_counts(ft_nodes, sections)
    errors.extend(validate_roadmap_counts(counts))
    structural_errors, structural_rule_count, structural_file_count = structural_hygiene.validate_repository(ROOT)
    errors.extend(structural_errors)

    if errors:
        print("planning governance validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    status_counts = task_cards.collect_status_counts(task_files)
    status_summary = ", ".join(
        f"{count} {status}" for status, count in sorted(status_counts.items())
    )
    print(
        "validated planning governance: "
        f"{counts['total']} requirements, {counts['phase_mapped']} phase-mapped, "
        f"{counts['backlog']} backlog, {len(task_files)} task card(s) [{status_summary}], "
        f"structural hygiene {structural_rule_count} rule(s) / {structural_file_count} file(s)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
