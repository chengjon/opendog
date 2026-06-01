from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import tech_debt_test_support as support


class TechDebtTestSupportTests(unittest.TestCase):
    def test_baseline_payload_supplies_standard_fields_and_overrides(self) -> None:
        payload = support.baseline_payload(
            gated_metrics=["rust_check_errors", "production_unwrap_count"],
            documentation_documents=[],
            rust_check_errors=0,
            duplicate_dependency_crate_count=9,
        )

        self.assertEqual("v1.0", payload["metric_version"])
        self.assertEqual("opendog-test", payload["project"])
        self.assertEqual(["rust_check_errors", "production_unwrap_count"], payload["gated_metrics"])
        self.assertEqual(["duplicate_dependency_crate_count"], payload["observed_metrics"])
        self.assertEqual(0, payload["rust_check_errors"])
        self.assertEqual(9, payload["duplicate_dependency_crate_count"])
        self.assertEqual({"documents": []}, payload["documentation_policy"])


if __name__ == "__main__":
    unittest.main()
