from __future__ import annotations

import json
from pathlib import Path

import planning_paths

POLICY_RELATIVE_PATH = planning_paths.STRUCTURAL_HYGIENE_POLICY_FILE.relative_to(planning_paths.ROOT)
RUST_EXAMPLE_FILE = "src/example.rs"
RUST_INCLUDE_GLOB = "src/**/*.rs"


def write_file(root: Path, relative_path: str, content: str) -> Path:
    path = root / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return path


def write_policy(root: Path, rules: list[dict[str, object]] | None = None) -> Path:
    path = root / POLICY_RELATIVE_PATH
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"rules": rules or []}, indent=2), encoding="utf-8")
    return path
