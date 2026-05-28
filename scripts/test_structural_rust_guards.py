from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import structural_rust_guards as rust_guards


class StructuralRustGuardTests(unittest.TestCase):
    def write_file(self, root: Path, relative_path: str, content: str) -> None:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def test_validate_production_rust_panic_like_calls_reports_expect(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "src/daemon.rs",
                """
pub fn run() {
    make_runtime().expect("runtime should start");
}
""",
            )

            errors = rust_guards.validate_production_rust_panic_like_calls(root)

        self.assertEqual(len(errors), 1)
        self.assertIn("src/daemon.rs:3", errors[0])
        self.assertIn(".expect", errors[0])

    def test_validate_production_rust_panic_like_calls_ignores_tests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            self.write_file(
                root,
                "src/daemon.rs",
                """
pub fn run() {
    let _value = maybe_value();
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_test_expect() {
        maybe_value().expect("test fixture should exist");
    }
}
""",
            )
            self.write_file(
                root,
                "src/control/tests.rs",
                """
#[test]
fn accepts_test_panic() {
    panic!("assertion helper");
}
""",
            )

            errors = rust_guards.validate_production_rust_panic_like_calls(root)

        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
