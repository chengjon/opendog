from __future__ import annotations

import contextlib
import io
import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from tech_debt_baseline.cli import print_result
from tech_debt_baseline.validation import ValidationResult


class TechDebtBaselineCliTests(unittest.TestCase):
    def test_print_result_shows_clean_gate_and_observation_statuses(self) -> None:
        stream = io.StringIO()
        result = ValidationResult(passed=True, errors=[], warnings=[])

        with contextlib.redirect_stdout(stream):
            print_result(result)

        lines = stream.getvalue().splitlines()
        self.assertIn("validated tech debt baseline: PASS", lines)
        self.assertIn("tech debt gate drift: PASS", lines)
        self.assertIn("observed tech debt drift: PASS", lines)

    def test_print_result_separates_observed_warning_status(self) -> None:
        stream = io.StringIO()
        result = ValidationResult(
            passed=True,
            errors=[],
            warnings=["duplicate_dependency_crate_count regressed: 5 > 4"],
        )

        with contextlib.redirect_stdout(stream):
            print_result(result)

        lines = stream.getvalue().splitlines()
        self.assertIn("validated tech debt baseline: PASS", lines)
        self.assertIn("tech debt gate drift: PASS", lines)
        self.assertIn("observed tech debt drift: WARN", lines)
        self.assertIn("WARN: duplicate_dependency_crate_count regressed: 5 > 4", lines)


if __name__ == "__main__":
    unittest.main()
