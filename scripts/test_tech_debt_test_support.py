from __future__ import annotations

import sys
import tempfile
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

    def test_write_file_creates_parent_directories(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            path = support.write_file(root, "nested/demo.txt", "demo")

            self.assertEqual(root / "nested" / "demo.txt", path)
            self.assertEqual("demo", path.read_text(encoding="utf-8"))

    def test_write_cargo_inventory_creates_manifest_and_optional_lockfile(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            support.write_cargo_inventory(
                root,
                dev_dependencies=['tempfile = "3"'],
            )

            manifest = (root / "Cargo.toml").read_text(encoding="utf-8")
            lockfile = (root / "Cargo.lock").read_text(encoding="utf-8")

        self.assertIn('[package]', manifest)
        self.assertIn('serde = "1"', manifest)
        self.assertIn('[dev-dependencies]', manifest)
        self.assertIn('tempfile = "3"', manifest)
        self.assertIn("version = 3", lockfile)
        self.assertIn('name = "demo"', lockfile)

    def test_write_cargo_inventory_can_skip_lockfile(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            support.write_cargo_inventory(root, with_lockfile=False)

            self.assertTrue((root / "Cargo.toml").exists())
            self.assertFalse((root / "Cargo.lock").exists())


if __name__ == "__main__":
    unittest.main()
