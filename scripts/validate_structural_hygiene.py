#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY_FILE = ROOT / ".planning" / "structural_hygiene_rules.json"


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


def validate_repository(
    root: Path = ROOT,
    policy_path: Path = POLICY_FILE,
) -> tuple[list[str], int, int]:
    rules = load_rules(policy_path)
    errors = validate_limits(root, rules)
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
