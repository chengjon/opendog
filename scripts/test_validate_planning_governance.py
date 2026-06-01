from __future__ import annotations

import json
import importlib
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_planning_governance as planning_governance
import validate_requirement_mappings as requirement_mappings
import validate_task_cards as task_cards


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

    def dependency_baseline(self) -> dict[str, object]:
        baseline = self.baseline()
        baseline.update(
            {
                "dependency_audit_issue_count": 0,
                "gated_metrics": [
                    "rust_check_errors",
                    "production_unwrap_count",
                    "dependency_audit_issue_count",
                ],
            }
        )
        return baseline

    def write_cargo_inventory(self, root: Path, *, with_lockfile: bool = True) -> None:
        self.write_file(
            root,
            "Cargo.toml",
            "\n".join(
                [
                    "[package]",
                    'name = "demo"',
                    'version = "0.1.0"',
                    'edition = "2021"',
                    "",
                    "[dependencies]",
                    'serde = "1"',
                ]
            ),
        )
        if with_lockfile:
            self.write_file(
                root,
                "Cargo.lock",
                "\n".join(
                    [
                        "version = 3",
                        "",
                        "[[package]]",
                        'name = "demo"',
                        'version = "0.1.0"',
                    ]
                ),
            )

    def test_planning_paths_reuse_task_card_module_constants(self) -> None:
        source = Path(planning_governance.__file__).read_text(encoding="utf-8")

        self.assertIs(task_cards.TREE_FILE, planning_governance.FUNCTION_TREE_FILE)
        self.assertIs(task_cards.REQUIREMENTS_FILE, planning_governance.REQUIREMENTS_FILE)
        self.assertIs(task_cards.TASK_DIR, planning_governance.TASK_CARD_DIR)
        self.assertNotIn('ROOT / "FUNCTION_TREE.md"', source)
        self.assertNotIn('ROOT / ".planning" / "REQUIREMENTS.md"', source)
        self.assertNotIn('ROOT / ".planning" / "task-cards"', source)

    def test_planning_modules_share_planning_path_constants(self) -> None:
        planning_paths = importlib.import_module("planning_paths")

        self.assertIs(planning_paths.FUNCTION_TREE_FILE, task_cards.TREE_FILE)
        self.assertIs(planning_paths.REQUIREMENTS_FILE, task_cards.REQUIREMENTS_FILE)
        self.assertIs(planning_paths.TASK_CARD_DIR, task_cards.TASK_DIR)
        self.assertIs(planning_paths.FUNCTION_TREE_FILE, requirement_mappings.TREE_FILE)
        self.assertIs(planning_paths.REQUIREMENTS_FILE, requirement_mappings.REQUIREMENTS_FILE)
        self.assertIs(planning_paths.FUNCTION_TREE_FILE, planning_governance.FUNCTION_TREE_FILE)
        self.assertIs(planning_paths.REQUIREMENTS_FILE, planning_governance.REQUIREMENTS_FILE)
        self.assertIs(planning_paths.TASK_CARD_DIR, planning_governance.TASK_CARD_DIR)

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

    def test_lightweight_tech_debt_gate_includes_dependency_audit_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            baseline_path = self.write_file(
                root,
                "reports/analysis/tech-debt-baseline.json",
                json.dumps(self.dependency_baseline()),
            )
            self.write_cargo_inventory(root)

            errors, warnings = planning_governance.validate_tech_debt_baseline(
                root,
                baseline_path,
            )

            self.assertEqual([], errors)
            self.assertEqual([], warnings)


if __name__ == "__main__":
    unittest.main()
