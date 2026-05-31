from __future__ import annotations

from tech_debt_baseline import (
    ValidationResult,
    compare_to_baseline,
    load_baseline,
    main,
    measure_current_metrics,
)

__all__ = [
    "ValidationResult",
    "compare_to_baseline",
    "load_baseline",
    "main",
    "measure_current_metrics",
]


if __name__ == "__main__":
    raise SystemExit(main())
