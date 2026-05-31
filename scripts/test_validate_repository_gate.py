from __future__ import annotations

import contextlib
import io
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import validate_repository_gate as repository_gate


class RepositoryGateTests(unittest.TestCase):
    def test_gate_commands_include_core_validation_sequence(self) -> None:
        command_names = [command.name for command in repository_gate.gate_commands()]

        self.assertEqual(
            [
                "openspec",
                "cargo-fmt",
                "cargo-test",
                "cargo-clippy",
                "python-unit-tests",
                "tech-debt-baseline",
                "planning-governance",
                "structural-hygiene",
                "diff-check",
            ],
            command_names,
        )

    def test_main_stops_at_first_failure(self) -> None:
        calls: list[str] = []

        def fake_run(command: repository_gate.GateCommand, root: Path) -> int:
            calls.append(command.name)
            return 1 if command.name == "cargo-test" else 0

        with (
            patch.object(repository_gate, "run_command", side_effect=fake_run),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = repository_gate.main([])

        self.assertEqual(1, status)
        self.assertEqual(["openspec", "cargo-fmt", "cargo-test"], calls)

    def test_main_runs_all_commands_when_successful(self) -> None:
        calls: list[str] = []

        def fake_run(command: repository_gate.GateCommand, root: Path) -> int:
            calls.append(command.name)
            return 0

        with (
            patch.object(repository_gate, "run_command", side_effect=fake_run),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = repository_gate.main([])

        self.assertEqual(0, status)
        self.assertEqual(len(repository_gate.gate_commands()), len(calls))


if __name__ == "__main__":
    unittest.main()
