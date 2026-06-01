from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ValidationResult:
    passed: bool
    errors: list[str]
    warnings: list[str]


def is_metric_regression(current: Any, baseline: Any) -> bool:
    if isinstance(current, bool) and isinstance(baseline, bool):
        return baseline and not current
    if isinstance(current, (int, float)) and isinstance(baseline, (int, float)):
        return current > baseline
    return current != baseline


def metric_regression_message(metric: str, current: Any, baseline: Any) -> str:
    if isinstance(current, list) or isinstance(baseline, list):
        return f"{metric} changed: {current} != {baseline}"
    return f"{metric} regressed: {current} > {baseline}"


def compare_to_baseline(
    baseline: dict[str, Any],
    current: dict[str, Any],
    *,
    excluded_metrics: set[str] | None = None,
) -> ValidationResult:
    errors: list[str] = []
    warnings: list[str] = []
    excluded = excluded_metrics or set()
    for metric in baseline.get("gated_metrics", []):
        if metric in excluded:
            continue
        if metric not in current:
            errors.append(f"{metric} unavailable in current measurement")
            continue
        if metric not in baseline:
            errors.append(f"{metric} unavailable in baseline")
            continue
        if is_metric_regression(current[metric], baseline[metric]):
            errors.append(metric_regression_message(metric, current[metric], baseline[metric]))
    for metric in baseline.get("observed_metrics", []):
        if metric in excluded:
            continue
        if metric not in current or metric not in baseline:
            warnings.append(f"{metric} unavailable for comparison")
            continue
        if is_metric_regression(current[metric], baseline[metric]):
            warnings.append(metric_regression_message(metric, current[metric], baseline[metric]))
    return ValidationResult(passed=not errors, errors=errors, warnings=warnings)
