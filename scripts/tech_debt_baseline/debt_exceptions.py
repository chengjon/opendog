from __future__ import annotations

from pathlib import Path


def count_debt_exception_annotations(path: Path) -> int:
    count = 0
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.lstrip()
        if (
            "debt-exception" in stripped
            and (
                stripped.startswith("#")
                or stripped.startswith("//")
                or stripped.startswith("<!--")
            )
        ):
            count += 1
    return count
