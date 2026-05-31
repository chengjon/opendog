from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_tech_debt_baseline as tech_debt


class TechDebtBaselineValidationTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> Path:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def baseline(self, **overrides: object) -> dict[str, object]:
        data: dict[str, object] = {
            "metric_version": "v1.0",
            "generated_at": "2026-05-31T02:27:49Z",
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
            "documentation_policy": {
                "documents": [
                    {
                        "file": "docs/mcp-tool-reference.md",
                        "line_limit": 1000,
                    }
                ]
            },
        }
        data.update(overrides)
        return data

    def test_gated_metric_regression_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(root, "src/main.rs", "fn demo(value: Option<u8>) { value.unwrap(); }\n")

            current = tech_debt.measure_current_metrics(
                root,
                include_command_metrics=False,
                include_dependency_metrics=False,
            )
            result = tech_debt.compare_to_baseline(self.baseline(), current)

            self.assertFalse(result.passed)
            self.assertIn("production_unwrap_count regressed: 1 > 0", result.errors)

    def test_observed_metric_regression_warns_without_failing(self) -> None:
        result = tech_debt.compare_to_baseline(
            self.baseline(duplicate_dependency_crate_count=4),
            {
                "production_unwrap_count": 0,
                "should_panic_test_count": 0,
                "policy_document_over_1000_count": 0,
                "duplicate_dependency_crate_count": 5,
            },
        )

        self.assertTrue(result.passed)
        self.assertIn("duplicate_dependency_crate_count regressed: 5 > 4", result.warnings)

    def test_drift_report_classifies_gated_and_observed_regressions(self) -> None:
        baseline = self.baseline(duplicate_dependency_crate_count=4)
        current = {
            "production_unwrap_count": 1,
            "should_panic_test_count": 0,
            "policy_document_over_1000_count": 0,
            "duplicate_dependency_crate_count": 5,
        }
        result = tech_debt.compare_to_baseline(baseline, current)

        report = tech_debt.build_drift_report(baseline, current, result)

        self.assertEqual("FAIL", report["status"])
        metrics = {metric["name"]: metric for metric in report["metrics"]}
        self.assertEqual("fail", metrics["production_unwrap_count"]["status"])
        self.assertEqual("warn", metrics["duplicate_dependency_crate_count"]["status"])

    def test_write_drift_report_creates_parent_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / "reports" / "analysis" / "drift.json"

            tech_debt.write_drift_report(path, {"status": "PASS", "metrics": []})

            self.assertEqual(
                {"status": "PASS", "metrics": []},
                json.loads(path.read_text(encoding="utf-8")),
            )

    def test_excluded_gated_metric_is_not_required(self) -> None:
        result = tech_debt.compare_to_baseline(
            self.baseline(gated_metrics=["rust_check_errors", "production_unwrap_count"]),
            {"production_unwrap_count": 0},
            excluded_metrics={"rust_check_errors"},
        )

        self.assertTrue(result.passed)
        self.assertEqual([], result.errors)

    def test_cfg_test_unwrap_is_not_counted_as_production_debt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "src/lib.rs",
                "\n".join(
                    [
                        "fn runtime() {}",
                        "#[cfg(test)]",
                        "mod tests {",
                        "    #[test]",
                        "    fn uses_unwrap_in_test() { Some(1).unwrap(); }",
                        "}",
                    ]
                ),
            )

            current = tech_debt.measure_current_metrics(
                root,
                include_command_metrics=False,
                include_dependency_metrics=False,
            )

            self.assertEqual(0, current["production_unwrap_count"])

    def test_nested_tests_directory_is_not_counted_as_production_debt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "src/mcp/tests/example.rs",
                "fn test_helper(value: Option<u8>) { value.unwrap(); }\n",
            )

            current = tech_debt.measure_current_metrics(
                root,
                include_command_metrics=False,
                include_dependency_metrics=False,
            )

            self.assertEqual(0, current["production_unwrap_count"])

    def test_scanner_string_literals_do_not_make_python_files_test_bearing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "scripts/validate_tech_debt_baseline.py",
                'MARKERS = ["#[ignore]", "#[should_panic", "#[cfg(test)]"]\n',
            )

            current = tech_debt.measure_current_metrics(
                root,
                include_command_metrics=False,
                include_dependency_metrics=False,
            )

            self.assertEqual(0, current["ignored_test_count"])
            self.assertEqual(0, current["should_panic_test_count"])

    def test_debt_exception_string_literal_is_not_counted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "scripts/validate_tech_debt_baseline.py",
                'TOKEN = "debt-exception"\n',
            )

            current = tech_debt.measure_current_metrics(
                root,
                include_command_metrics=False,
                include_dependency_metrics=False,
            )

            self.assertEqual(0, current["debt_exception_count"])

    def test_policy_document_over_limit_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(root, "docs/mcp-tool-reference.md", "\n".join(["x"] * 1001))

            current = tech_debt.measure_current_metrics(
                root,
                baseline=self.baseline(),
                include_command_metrics=False,
                include_dependency_metrics=False,
            )
            result = tech_debt.compare_to_baseline(self.baseline(), current)

            self.assertFalse(result.passed)
            self.assertIn("policy_document_over_1000_count regressed: 1 > 0", result.errors)

    def test_load_baseline_reads_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = self.write_file(
                root,
                "reports/analysis/tech-debt-baseline.json",
                json.dumps(self.baseline()),
            )

            self.assertEqual("opendog-test", tech_debt.load_baseline(path)["project"])


if __name__ == "__main__":
    unittest.main()
