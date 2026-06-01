from __future__ import annotations

import re
from pathlib import Path

import repo_paths


ROOT = repo_paths.ROOT
MCP_TOOL_INVENTORY_FILE = ROOT / "src" / "mcp" / "tool_inventory.rs"
MCP_RESOURCE_HANDLERS_FILE = ROOT / "src" / "mcp" / "resource_handlers.rs"
MCP_FULL_REFERENCE_DOCS = [
    "docs/mcp-tool-reference.md",
    "README.md",
    "QUICKSTART.md",
    "FUNCTION_TREE.md",
    "CLAUDE.md",
]
MCP_TOOL_COUNT_DOC_EXCLUSIONS = {"README.md"}
MCP_TOOL_COUNT_DOCS = [
    relative_path for relative_path in MCP_FULL_REFERENCE_DOCS if relative_path not in MCP_TOOL_COUNT_DOC_EXCLUSIONS
]
MCP_CURRENT_GUIDANCE_DOCS = [
    "docs/opendog-feature-introduction.md",
    *MCP_FULL_REFERENCE_DOCS,
]
REMOVED_PUBLIC_MCP_TOOL_NAMES = [
    "get_agent_guidance",
    "get_decision_brief",
]


def mcp_tool_names(root: Path = ROOT) -> list[str]:
    inventory_path = root / MCP_TOOL_INVENTORY_FILE.relative_to(ROOT)
    if not inventory_path.exists():
        return []
    inventory = inventory_path.read_text(encoding="utf-8")
    start = inventory.find("pub(crate) const MCP_TOOL_INVENTORY")
    end = inventory.find("pub(crate) fn mcp_tool_inventory", start)
    if start < 0 or end < 0:
        return []
    return re.findall(r'name:\s*"([a-z][a-z0-9_]+)"', inventory[start:end])


def mcp_resource_uris(root: Path = ROOT) -> list[str]:
    resource_handlers_path = root / MCP_RESOURCE_HANDLERS_FILE.relative_to(ROOT)
    if not resource_handlers_path.exists():
        return []
    return re.findall(
        r'const\s+[A-Z0-9_]+:\s*&str\s*=\s*"(opendog://[^"]+)"',
        resource_handlers_path.read_text(encoding="utf-8"),
    )


def mcp_tool_reference_headings(root: Path = ROOT) -> list[str]:
    reference_path = root / "docs" / "mcp-tool-reference.md"
    if not reference_path.exists():
        return []
    reference = reference_path.read_text(encoding="utf-8")
    return re.findall(r"^## `([a-z][a-z0-9_]+)`\s*$", reference, re.MULTILINE)


def validate_mcp_surface_docs(root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    tool_names = mcp_tool_names(root)
    if not tool_names:
        return ["src/mcp/tool_inventory.rs does not expose MCP_TOOL_INVENTORY tool names"]
    expected_count = len(tool_names)
    expected_tool_names = set(tool_names)

    for relative_path in MCP_FULL_REFERENCE_DOCS:
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path} is missing")
            continue
        text = path.read_text(encoding="utf-8")
        if relative_path in MCP_TOOL_COUNT_DOCS:
            if f"{expected_count} MCP tools" not in text and f"{expected_count} tools" not in text:
                declared_counts = re.findall(r"(\d+)\s*(?:MCP\s*)?tools", text)
                if declared_counts:
                    errors.append(f"{relative_path} declares {declared_counts[0]} MCP tools, expected {expected_count}")
                else:
                    errors.append(f"{relative_path} does not declare current MCP tool count: {expected_count}")
        for tool_name in tool_names:
            if tool_name not in text:
                errors.append(f"{relative_path} is missing MCP tool name: {tool_name}")

    reference_headings = mcp_tool_reference_headings(root)
    if reference_headings:
        documented_tool_names = set(reference_headings)
        for tool_name in sorted(expected_tool_names - documented_tool_names):
            errors.append(f"docs/mcp-tool-reference.md is missing MCP tool heading: {tool_name}")
        for tool_name in sorted(documented_tool_names - expected_tool_names):
            errors.append(f"docs/mcp-tool-reference.md documents unknown MCP tool heading: {tool_name}")
        for tool_name in sorted(documented_tool_names):
            if reference_headings.count(tool_name) > 1:
                errors.append(f"docs/mcp-tool-reference.md documents duplicate MCP tool heading: {tool_name}")

    for relative_path in MCP_CURRENT_GUIDANCE_DOCS:
        path = root / relative_path
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for tool_name in REMOVED_PUBLIC_MCP_TOOL_NAMES:
            if tool_name in text:
                errors.append(f"{relative_path} mentions removed MCP tool name: {tool_name}")

    resource_doc = root / "docs" / "mcp-tool-reference.md"
    if resource_doc.exists():
        text = resource_doc.read_text(encoding="utf-8")
        for uri in mcp_resource_uris(root):
            if uri not in text:
                errors.append(f"docs/mcp-tool-reference.md is missing read-only MCP resource URI: {uri}")
    return errors


def validate_openspec_purpose_placeholders(root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    for path in sorted((root / "openspec" / "specs").glob("*/spec.md")):
        if "TBD - created by archiving" in path.read_text(encoding="utf-8"):
            errors.append(f"{path.relative_to(root).as_posix()} has archived OpenSpec Purpose placeholder")
    return errors
