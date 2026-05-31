from __future__ import annotations

from tech_debt_baseline import (
    ValidationResult,
    build_drift_report,
    compare_to_baseline,
    load_baseline,
    main,
    measure_current_metrics,
    write_drift_report,
)

__all__ = [
    "ValidationResult",
    "build_drift_report",
    "compare_to_baseline",
    "load_baseline",
    "main",
    "measure_current_metrics",
    "write_drift_report",
]


if __name__ == "__main__":
    raise SystemExit(main())
