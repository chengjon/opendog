from __future__ import annotations

import re
from pathlib import Path

import repo_paths

ROOT = repo_paths.ROOT
PANIC_LIKE_PATTERN = re.compile(
    r"(\.unwrap\s*\(|\.expect\s*\(|\bpanic!\s*\(|\bunreachable!\s*\(|\btodo!\s*\(|\bunimplemented!\s*\()"
)
TEST_ATTRIBUTE_PATTERN = re.compile(
    r"#\[(cfg\(test\)|test|tokio::test|rstest|case\b|should_panic)"
)
ITEM_PATTERN = re.compile(r"\b(mod|fn)\s+[A-Za-z_][A-Za-z0-9_]*")


def rust_production_files(root: Path = ROOT) -> list[Path]:
    src_root = root / "src"
    if not src_root.exists():
        return []
    files = []
    for path in sorted(src_root.rglob("*.rs")):
        relative = path.relative_to(root)
        if path.name in {"tests.rs"}:
            continue
        if "/tests/" in relative.as_posix():
            continue
        if path.name.endswith("_tests.rs") or path.name.startswith("test_"):
            continue
        files.append(path)
    return files


def validate_production_rust_panic_like_calls(root: Path = ROOT) -> list[str]:
    errors = []
    for path in rust_production_files(root):
        relative = path.relative_to(root)
        for line_number, line in production_rust_lines(path):
            if PANIC_LIKE_PATTERN.search(strip_comment_and_strings(line)):
                errors.append(
                    f"{relative}:{line_number} uses panic-like production call: {line.strip()}"
                )
    return errors


def production_rust_lines(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    output: list[tuple[int, str]] = []
    skip_next_item = False
    in_test_item = False
    test_depth = 0

    for index, line in enumerate(lines, start=1):
        stripped = line.strip()
        if TEST_ATTRIBUTE_PATTERN.match(stripped):
            skip_next_item = True
            continue

        if skip_next_item and ITEM_PATTERN.search(stripped):
            in_test_item = True
            test_depth = max(1, brace_delta(line))
            skip_next_item = False
            continue

        if skip_next_item and stripped and not stripped.startswith("#["):
            skip_next_item = False

        if in_test_item:
            test_depth += brace_delta(line)
            if test_depth <= 0:
                in_test_item = False
            continue

        output.append((index, line))

    return output


def strip_comment_and_strings(line: str) -> str:
    without_comment = line.split("//", 1)[0]
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', without_comment)


def brace_delta(line: str) -> int:
    stripped = strip_comment_and_strings(line)
    return stripped.count("{") - stripped.count("}")
