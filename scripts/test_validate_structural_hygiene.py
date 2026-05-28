from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_structural_hygiene as structural_hygiene


class StructuralHygieneValidationTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> Path:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def write_policy(self, root: Path, rules: list[dict[str, object]]) -> Path:
        path = root / ".planning" / "structural_hygiene_rules.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps({"rules": rules}, indent=2), encoding="utf-8")
        return path

    def test_validate_limits_reports_line_and_byte_violations(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(root, "src/example.rs", "a\nb\nc\nd\n")

            rules = [
                {
                    "name": "rust",
                    "include": ["src/**/*.rs"],
                    "max_lines": 3,
                    "max_bytes": 6,
                }
            ]

            errors = structural_hygiene.validate_limits(root, rules)

            self.assertIn(
                "src/example.rs exceeds max_lines for rule 'rust': 5 > 3",
                errors,
            )
            self.assertIn(
                "src/example.rs exceeds max_bytes for rule 'rust': 8 > 6",
                errors,
            )

    def test_validate_limits_honors_exclude_patterns(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(root, "src/generated/big.rs", "a\nb\nc\nd\ne\n")

            rules = [
                {
                    "name": "rust",
                    "include": ["src/**/*.rs"],
                    "exclude": ["src/generated/**/*.rs"],
                    "max_lines": 3,
                }
            ]

            errors = structural_hygiene.validate_limits(root, rules)

            self.assertEqual([], errors)

    def test_load_rules_reads_machine_readable_policy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            policy_path = self.write_policy(
                root,
                [
                    {
                        "name": "docs",
                        "include": ["docs/**/*.md"],
                        "max_lines": 900,
                        "max_bytes": 24000,
                    }
                ],
            )

            rules = structural_hygiene.load_rules(policy_path)

            self.assertEqual(
                [
                    {
                        "name": "docs",
                        "include": ["docs/**/*.md"],
                        "exclude": [],
                        "max_lines": 900,
                        "max_bytes": 24000,
                    }
                ],
                rules,
            )


if __name__ == "__main__":
    unittest.main()
