from __future__ import annotations

import argparse
from pathlib import Path

from .collector import measure_current_metrics
from .metrics import load_baseline
from .validation import ValidationResult, compare_to_baseline

DEFAULT_BASELINE_PATH = Path("reports/analysis/tech-debt-baseline.json")


def print_result(result: ValidationResult) -> None:
    if result.passed:
        print("validated tech debt baseline: PASS")
    else:
        print("validated tech debt baseline: FAIL")
    for error in result.errors:
        print(f"ERROR: {error}")
    for warning in result.warnings:
        print(f"WARN: {warning}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate current technical debt metrics against baseline.")
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="Repository root path.")
    parser.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE_PATH,
        help="Baseline JSON path, relative to --root when not absolute.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    baseline_path = args.baseline if args.baseline.is_absolute() else root / args.baseline
    baseline = load_baseline(baseline_path)
    current = measure_current_metrics(root, baseline=baseline)
    result = compare_to_baseline(baseline, current)
    print_result(result)
    return 0 if result.passed else 1
