from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from tech_debt_baseline import metrics as debt_metrics


class TechDebtMissingToolsTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> Path:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_command_metrics_missing_cargo_marks_failures(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            with mock.patch.object(debt_metrics.subprocess, "run", side_effect=FileNotFoundError()):
                metrics = debt_metrics.measure_command_metrics(root)

        self.assertEqual(1, metrics["rust_check_errors"])
        self.assertEqual(1, metrics["rust_clippy_errors"])
        self.assertEqual(1, metrics["rust_clippy_warnings"])
        self.assertEqual(1, metrics["backend_lint_errors"])

    def test_dependency_metrics_reports_missing_cargo_tree_as_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
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

            with (
                mock.patch.object(debt_metrics.subprocess, "run", side_effect=FileNotFoundError()),
                mock.patch.object(debt_metrics, "dependency_audit_tool", return_value=None),
            ):
                metrics = debt_metrics.measure_dependency_metrics(root)

        self.assertEqual(0, metrics["duplicate_dependency_crate_count"])
        self.assertEqual([], metrics["duplicate_dependency_crates"])
        self.assertFalse(metrics["dependency_audit"]["cargo_tree_available"])
        self.assertEqual(127, metrics["dependency_audit"]["cargo_tree_status"])


if __name__ == "__main__":
    unittest.main()
