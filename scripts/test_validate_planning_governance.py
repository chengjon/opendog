from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_planning_governance as planning_governance


class PlanningGovernanceTechDebtTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> Path:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def baseline(self) -> dict[str, object]:
        return {
            "metric_version": "v1.0",
            "generated_at": "2026-05-31T02:27:49Z",
            "project": "opendog-test",
            "rust_check_errors": 0,
            "production_unwrap_count": 0,
            "gated_metrics": ["rust_check_errors", "production_unwrap_count"],
            "observed_metrics": ["duplicate_dependency_crate_count"],
            "duplicate_dependency_crate_count": 4,
            "documentation_policy": {"documents": []},
        }

    def test_lightweight_tech_debt_gate_skips_command_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            baseline_path = self.write_file(
                root,
                "reports/analysis/tech-debt-baseline.json",
                json.dumps(self.baseline()),
            )

            errors, warnings = planning_governance.validate_tech_debt_baseline(
                root,
                baseline_path,
            )

            self.assertEqual([], errors)
            self.assertEqual([], warnings)

    def test_lightweight_tech_debt_gate_reports_static_regression(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            baseline_path = self.write_file(
                root,
                "reports/analysis/tech-debt-baseline.json",
                json.dumps(self.baseline()),
            )
            self.write_file(root, "src/main.rs", "fn demo(value: Option<u8>) { value.unwrap(); }\n")

            errors, warnings = planning_governance.validate_tech_debt_baseline(
                root,
                baseline_path,
            )

            self.assertEqual([], warnings)
            self.assertIn(
                "technical debt baseline: production_unwrap_count regressed: 1 > 0",
                errors,
            )


if __name__ == "__main__":
    unittest.main()
