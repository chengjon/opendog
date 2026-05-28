#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY_FILE = ROOT / ".planning" / "structural_hygiene_rules.json"
MCP_TOOL_INVENTORY_FILE = ROOT / "src" / "mcp" / "tool_inventory.rs"
MCP_RESOURCE_HANDLERS_FILE = ROOT / "src" / "mcp" / "resource_handlers.rs"
MCP_FULL_REFERENCE_DOCS = [
    "docs/mcp-tool-reference.md",
    "README.md",
    "QUICKSTART.md",
    "FUNCTION_TREE.md",
    "CLAUDE.md",
]
MCP_TOOL_COUNT_DOCS = [
    "docs/mcp-tool-reference.md",
    "QUICKSTART.md",
    "FUNCTION_TREE.md",
    "CLAUDE.md",
]
MCP_CURRENT_GUIDANCE_DOCS = [
    "docs/opendog-feature-introduction.md",
    *MCP_FULL_REFERENCE_DOCS,
]
REMOVED_PUBLIC_MCP_TOOL_NAMES = [
    "get_agent_guidance",
    "get_decision_brief",
]


def load_rules(policy_path: Path = POLICY_FILE) -> list[dict[str, object]]:
    payload = json.loads(policy_path.read_text(encoding="utf-8"))
    raw_rules = payload.get("rules")
    if not isinstance(raw_rules, list):
        raise ValueError(f"{policy_path}: missing 'rules' list")

    rules: list[dict[str, object]] = []
    for raw_rule in raw_rules:
        if not isinstance(raw_rule, dict):
            raise ValueError(f"{policy_path}: each rule must be an object")

        name = raw_rule.get("name")
        include = raw_rule.get("include")
        exclude = raw_rule.get("exclude", [])
        max_lines = raw_rule.get("max_lines")
        max_bytes = raw_rule.get("max_bytes")

        if not isinstance(name, str) or not name.strip():
            raise ValueError(f"{policy_path}: rule is missing a non-empty 'name'")
        if not isinstance(include, list) or not include or not all(isinstance(item, str) for item in include):
            raise ValueError(f"{policy_path}: rule '{name}' must include a non-empty string 'include' list")
        if not isinstance(exclude, list) or not all(isinstance(item, str) for item in exclude):
            raise ValueError(f"{policy_path}: rule '{name}' must use a string 'exclude' list")
        if max_lines is None and max_bytes is None:
            raise ValueError(f"{policy_path}: rule '{name}' must define max_lines and/or max_bytes")
        if max_lines is not None and (not isinstance(max_lines, int) or max_lines <= 0):
            raise ValueError(f"{policy_path}: rule '{name}' has invalid max_lines={max_lines!r}")
        if max_bytes is not None and (not isinstance(max_bytes, int) or max_bytes <= 0):
            raise ValueError(f"{policy_path}: rule '{name}' has invalid max_bytes={max_bytes!r}")

        rules.append(
            {
                "name": name,
                "include": include,
                "exclude": exclude,
                "max_lines": max_lines,
                "max_bytes": max_bytes,
            }
        )

    return rules


def measure_file(path: Path) -> tuple[int, int]:
    text = path.read_text(encoding="utf-8", errors="ignore")
    lines = text.count("\n") + 1
    return lines, path.stat().st_size


def match_files(root: Path, include_patterns: list[str], exclude_patterns: list[str]) -> list[Path]:
    included: set[Path] = set()
    for pattern in include_patterns:
        included.update(path for path in root.glob(pattern) if path.is_file())

    excluded: set[Path] = set()
    for pattern in exclude_patterns:
        excluded.update(path for path in root.glob(pattern) if path.is_file())

    return sorted(path for path in included if path not in excluded)


def validate_limits(root: Path, rules: list[dict[str, object]]) -> list[str]:
    errors: list[str] = []

    for rule in rules:
        name = str(rule["name"])
        include_patterns = list(rule["include"])
        exclude_patterns = list(rule.get("exclude", []))
        max_lines = rule.get("max_lines")
        max_bytes = rule.get("max_bytes")

        for path in match_files(root, include_patterns, exclude_patterns):
            lines, byte_size = measure_file(path)
            relative_path = path.relative_to(root).as_posix()

            if isinstance(max_lines, int) and lines > max_lines:
                errors.append(
                    f"{relative_path} exceeds max_lines for rule '{name}': {lines} > {max_lines}"
                )
            if isinstance(max_bytes, int) and byte_size > max_bytes:
                errors.append(
                    f"{relative_path} exceeds max_bytes for rule '{name}': {byte_size} > {max_bytes}"
                )

    return errors


def count_checked_files(root: Path, rules: list[dict[str, object]]) -> int:
    files: set[Path] = set()
    for rule in rules:
        files.update(
            match_files(root, list(rule["include"]), list(rule.get("exclude", [])))
        )
    return len(files)


def mcp_tool_names(root: Path = ROOT) -> list[str]:
    inventory_path = root / MCP_TOOL_INVENTORY_FILE.relative_to(ROOT)
    if not inventory_path.exists():
        return []
    inventory = inventory_path.read_text(encoding="utf-8")
    start = inventory.find("pub(crate) const MCP_TOOL_INVENTORY")
    end = inventory.find("pub(crate) fn mcp_tool_inventory", start)
    if start < 0 or end < 0:
        return []
    inventory_block = inventory[start:end]
    return re.findall(r'name:\s*"([a-z][a-z0-9_]+)"', inventory_block)


def mcp_resource_uris(root: Path = ROOT) -> list[str]:
    resource_handlers_path = root / MCP_RESOURCE_HANDLERS_FILE.relative_to(ROOT)
    if not resource_handlers_path.exists():
        return []
    resource_handlers = resource_handlers_path.read_text(encoding="utf-8")
    return re.findall(
        r'const\s+[A-Z0-9_]+:\s*&str\s*=\s*"(opendog://[^"]+)"',
        resource_handlers,
    )


def validate_mcp_surface_docs(root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    tool_names = mcp_tool_names(root)
    if not tool_names:
        return ["src/mcp/tool_inventory.rs does not expose MCP_TOOL_INVENTORY tool names"]
    expected_count = len(tool_names)

    for relative_path in MCP_FULL_REFERENCE_DOCS:
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path} is missing")
            continue
        text = path.read_text(encoding="utf-8")
        if relative_path in MCP_TOOL_COUNT_DOCS:
            if (
                f"{expected_count} MCP tools" not in text
                and f"{expected_count} tools" not in text
            ):
                declared_counts = re.findall(r"(\d+)\s*(?:MCP\s*)?tools", text)
                if declared_counts:
                    errors.append(
                        f"{relative_path} declares {declared_counts[0]} MCP tools, expected {expected_count}"
                    )
                else:
                    errors.append(
                        f"{relative_path} does not declare current MCP tool count: {expected_count}"
                    )
        for tool_name in tool_names:
            if tool_name not in text:
                errors.append(f"{relative_path} is missing MCP tool name: {tool_name}")

    for relative_path in MCP_CURRENT_GUIDANCE_DOCS:
        path = root / relative_path
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for tool_name in REMOVED_PUBLIC_MCP_TOOL_NAMES:
            if tool_name in text:
                errors.append(
                    f"{relative_path} mentions removed MCP tool name: {tool_name}"
                )

    resource_doc = root / "docs" / "mcp-tool-reference.md"
    if resource_doc.exists():
        text = resource_doc.read_text(encoding="utf-8")
        for uri in mcp_resource_uris(root):
            if uri not in text:
                errors.append(
                    f"docs/mcp-tool-reference.md is missing read-only MCP resource URI: {uri}"
                )

    return errors


def validate_repository(
    root: Path = ROOT,
    policy_path: Path = POLICY_FILE,
) -> tuple[list[str], int, int]:
    rules = load_rules(policy_path)
    errors = validate_limits(root, rules)
    errors.extend(validate_mcp_surface_docs(root))
    return errors, len(rules), count_checked_files(root, rules)


def main() -> int:
    errors, rule_count, checked_files = validate_repository()
    if errors:
        print("structural hygiene validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print(
        "validated structural hygiene: "
        f"{rule_count} rule(s), {checked_files} file(s) within configured size budgets"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
