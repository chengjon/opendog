from __future__ import annotations

from pathlib import Path
from typing import Any

from .debt_exceptions import count_debt_exception_annotations
from .metrics import (
    iter_project_files,
    measure_command_metrics,
    measure_dependency_metrics,
    measure_document_policy_metrics,
    measure_production_rust_metrics,
    measure_secret_scan_metrics,
    measure_size_metrics,
    measure_test_metrics,
    measure_tool_availability,
)


def measure_current_metrics(
    root: Path,
    baseline: dict[str, Any] | None = None,
    *,
    include_command_metrics: bool = True,
    include_dependency_metrics: bool = True,
) -> dict[str, Any]:
    files = iter_project_files(root)
    metrics: dict[str, Any] = {}
    metrics.update(measure_production_rust_metrics(root, files))
    metrics.update(measure_test_metrics(root, files))
    metrics.update(measure_size_metrics(root, files))
    metrics.update(measure_document_policy_metrics(root, baseline))
    metrics.update(measure_tool_availability())
    metrics.update(measure_secret_scan_metrics(root, files))
    metrics["debt_exception_count"] = sum(
        count_debt_exception_annotations(path)
        for path in files
        if path.suffix in {".rs", ".py", ".md", ".json", ".toml"}
    )
    if include_dependency_metrics:
        metrics.update(measure_dependency_metrics(root))
    if include_command_metrics:
        metrics.update(measure_command_metrics(root))
    return metrics
