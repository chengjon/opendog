from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import structural_hygiene_test_support as support


class StructuralHygieneTestSupportTests(unittest.TestCase):
    def test_write_file_creates_parent_directories(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            path = support.write_file(root, support.RUST_EXAMPLE_FILE, "fn main() {}\n")

            self.assertEqual(root / support.RUST_EXAMPLE_FILE, path)
            self.assertEqual("fn main() {}\n", path.read_text(encoding="utf-8"))

    def test_write_policy_uses_structural_hygiene_policy_path(self) -> None:
        rules = [{"name": "rust", "include": [support.RUST_INCLUDE_GLOB], "max_lines": 100}]
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            path = support.write_policy(root, rules)

            self.assertEqual(root / support.POLICY_RELATIVE_PATH, path)
            self.assertEqual({"rules": rules}, json.loads(path.read_text(encoding="utf-8")))


if __name__ == "__main__":
    unittest.main()
