from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from tech_debt_baseline import metrics as debt_metrics


class TechDebtDependencySecurityTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> Path:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def test_duplicate_dependency_parser_distinguishes_version_splits(self) -> None:
        duplicate_tree = "\n".join(
            [
                "hashbrown v0.16.1",
                "└── hashlink v0.11.0",
                "hashbrown v0.17.0",
                "└── indexmap v2.14.0",
                "",
                "serde_json v1.0.149 (*)",
                "",
                "serde_json v1.0.149 (*)",
            ]
        )

        versions = debt_metrics.parse_duplicate_crate_versions(duplicate_tree)

        self.assertEqual(
            {
                "hashbrown": ["0.16.1", "0.17.0"],
                "serde_json": ["1.0.149"],
            },
            versions,
        )

    def test_dependency_audit_reports_internal_cargo_inventory(self) -> None:
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
                        "",
                        "[dev-dependencies]",
                        'tempfile = "3"',
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
                        "",
                        "[[package]]",
                        'name = "serde"',
                        'version = "1.0.0"',
                    ]
                ),
            )

            metrics = debt_metrics.measure_dependency_metrics(root)

            self.assertIn("dependency_audit_issue_count", metrics)
            self.assertEqual(0, metrics["dependency_audit_issue_count"])
            self.assertEqual(0, metrics["dependency_lockfile_missing_count"])
            self.assertEqual(2, metrics["manifest_dependency_count"])
            self.assertEqual(2, metrics["locked_dependency_package_count"])
            self.assertEqual([], metrics["duplicate_dependency_crates"])
            self.assertEqual({}, metrics["duplicate_dependency_crate_versions"])
            self.assertEqual(0, metrics["duplicate_dependency_version_split_count"])
            self.assertEqual([], metrics["duplicate_dependency_version_splits"])
            self.assertEqual(0, metrics["dependency_audit"]["version_split_count"])
            self.assertEqual("internal-cargo-inventory", metrics["dependency_audit"]["scanner"])

    def test_tool_availability_marks_internal_audits_available(self) -> None:
        availability = debt_metrics.measure_tool_availability()

        self.assertTrue(availability["dependency_audit_available"])
        self.assertTrue(availability["secret_scan_available"])

    def test_tool_availability_detects_external_security_audit_workflow(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                ".github/workflows/external-security-audit.yml",
                "\n".join(
                    [
                        "name: External Security Audit",
                        "jobs:",
                        "  cargo-audit:",
                        "    steps:",
                        "      - run: cargo audit --color never",
                        "  cargo-deny:",
                        "    steps:",
                        "      - run: cargo deny check advisories",
                        "  gitleaks:",
                        "    steps:",
                        "      - run: gitleaks detect --source=.",
                    ]
                ),
            )

            availability = debt_metrics.measure_tool_availability(root)

        self.assertTrue(availability["external_dependency_audit_available"])
        self.assertTrue(availability["external_secret_scan_available"])

    def test_dependency_audit_marks_external_workflow_vulnerability_scan_available(self) -> None:
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
                        "",
                        "[[package]]",
                        'name = "serde"',
                        'version = "1.0.0"',
                    ]
                ),
            )
            self.write_file(
                root,
                ".github/workflows/external-security-audit.yml",
                "\n".join(
                    [
                        "name: External Security Audit",
                        "jobs:",
                        "  cargo-audit:",
                        "    steps:",
                        "      - run: cargo audit --color never",
                    ]
                ),
            )

            metrics = debt_metrics.measure_dependency_metrics(root)

        self.assertFalse(metrics["dependency_audit"]["external_tool_available"])
        self.assertTrue(metrics["dependency_audit"]["external_workflow_available"])
        self.assertTrue(metrics["dependency_audit"]["vulnerability_scan_available"])

    def test_secret_scan_marks_external_workflow_available(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                ".github/workflows/external-security-audit.yml",
                "\n".join(
                    [
                        "name: External Security Audit",
                        "jobs:",
                        "  gitleaks:",
                        "    steps:",
                        "      - run: gitleaks detect --source=.",
                    ]
                ),
            )
            metrics = debt_metrics.measure_secret_scan_metrics(root, [])

        self.assertFalse(metrics["secret_scan"]["external_tool_available"])
        self.assertTrue(metrics["secret_scan"]["external_workflow_available"])

    def test_secret_scan_counts_high_confidence_tokens_without_storing_values(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            token = "ghp_" + ("a" * 36)
            path = self.write_file(root, "src/example.rs", f'const TOKEN: &str = "{token}";\n')

            self.assertTrue(hasattr(debt_metrics, "measure_secret_scan_metrics"))
            metrics = debt_metrics.measure_secret_scan_metrics(root, [path])

            self.assertEqual(1, metrics["high_confidence_secret_count"])
            self.assertEqual(1, metrics["secret_scan"]["finding_count"])
            self.assertEqual("github_token", metrics["secret_scan"]["findings"][0]["pattern"])
            self.assertNotIn(token, json.dumps(metrics["secret_scan"]))


if __name__ == "__main__":
    unittest.main()
