from __future__ import annotations

import re


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
            errors.append(
                f"requirement '{requirement_id}' exists in REQUIREMENTS.md but not in FUNCTION_TREE.md"
            )

    for requirement_id, leaves in sorted(ft_requirement_to_leaves.items()):
        if requirement_id not in requirement_ids:
            errors.append(
                f"requirement '{requirement_id}' exists in FUNCTION_TREE.md but not in REQUIREMENTS.md"
            )
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


def compute_counts(
    nodes: dict[str, dict[str, object]],
    sections: list[dict[str, object]],
) -> dict[str, int]:
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
