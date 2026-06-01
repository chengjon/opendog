from __future__ import annotations

import json
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from .validation import ValidationResult, is_metric_regression


def build_drift_report(
    baseline: dict[str, Any],
    current: dict[str, Any],
    result: ValidationResult,
    *,
    excluded_metrics: set[str] | None = None,
) -> dict[str, Any]:
    excluded = excluded_metrics or set()
    return {
        "metric_version": baseline.get("metric_version"),
        "project": baseline.get("project"),
        "generated_at": datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "gate_status": "PASS" if result.passed else "FAIL",
        "status": "PASS" if result.passed else "FAIL",
        "observation_status": "WARN" if result.warnings else "PASS",
        "errors": result.errors,
        "warnings": result.warnings,
        "metrics": [
            metric_report(metric, baseline, current, "gated", excluded)
            for metric in baseline.get("gated_metrics", [])
        ]
        + [
            metric_report(metric, baseline, current, "observed", excluded)
            for metric in baseline.get("observed_metrics", [])
        ],
    }


def metric_report(
    metric: str,
    baseline: dict[str, Any],
    current: dict[str, Any],
    kind: str,
    excluded: set[str],
) -> dict[str, Any]:
    baseline_value = baseline.get(metric)
    current_value = current.get(metric)
    return {
        "name": metric,
        "kind": kind,
        "baseline": baseline_value,
        "current": current_value,
        "status": metric_status(metric, baseline_value, current_value, kind, excluded),
    }


def metric_status(
    metric: str,
    baseline_value: Any,
    current_value: Any,
    kind: str,
    excluded: set[str],
) -> str:
    if metric in excluded:
        return "excluded"
    if baseline_value is None or current_value is None:
        return "unavailable"
    if is_metric_regression(current_value, baseline_value):
        return "fail" if kind == "gated" else "warn"
    if current_value == baseline_value:
        return "unchanged"
    return "improved"


def write_drift_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
