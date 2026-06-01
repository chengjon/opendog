from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_tech_debt_baseline as tech_debt


class TechDebtBaselineReportTests(unittest.TestCase):
    def test_drift_report_marks_gate_failure_without_observation_warning(self) -> None:
        baseline = {
            "metric_version": "v1.0",
            "project": "opendog-test",
            "gated_metrics": ["production_unwrap_count"],
            "observed_metrics": ["duplicate_dependency_crate_count"],
            "production_unwrap_count": 0,
            "duplicate_dependency_crate_count": 4,
        }
        current = {
            "production_unwrap_count": 1,
            "duplicate_dependency_crate_count": 4,
        }
        result = tech_debt.compare_to_baseline(baseline, current)

        report = tech_debt.build_drift_report(baseline, current, result)

        self.assertEqual("FAIL", report["gate_status"])
        self.assertEqual("FAIL", report["status"])
        self.assertEqual("PASS", report["observation_status"])
        self.assertEqual(["production_unwrap_count regressed: 1 > 0"], report["errors"])

    def test_drift_report_separates_observation_status_from_gate_status(self) -> None:
        baseline = {
            "metric_version": "v1.0",
            "project": "opendog-test",
            "gated_metrics": [
                "production_unwrap_count",
                "should_panic_test_count",
                "policy_document_over_1000_count",
            ],
            "observed_metrics": ["duplicate_dependency_crate_count"],
            "production_unwrap_count": 0,
            "should_panic_test_count": 0,
            "policy_document_over_1000_count": 0,
            "duplicate_dependency_crate_count": 4,
        }
        current = {
            "production_unwrap_count": 0,
            "should_panic_test_count": 0,
            "policy_document_over_1000_count": 0,
            "duplicate_dependency_crate_count": 5,
        }
        result = tech_debt.compare_to_baseline(baseline, current)

        report = tech_debt.build_drift_report(baseline, current, result)

        self.assertEqual("PASS", report["status"])
        self.assertEqual("PASS", report["gate_status"])
        self.assertEqual("WARN", report["observation_status"])
        self.assertEqual(["duplicate_dependency_crate_count regressed: 5 > 4"], report["warnings"])


if __name__ == "__main__":
    unittest.main()
