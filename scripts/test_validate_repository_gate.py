from __future__ import annotations

import contextlib
import io
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
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
                "python-ruff",
                "python-unit-tests",
                "tech-debt-baseline",
                "planning-governance",
                "structural-hygiene",
                "diff-check",
            ],
            command_names,
        )
        python_tests = next(command.argv for command in repository_gate.gate_commands() if command.name == "python-unit-tests")
        self.assertIn("scripts.test_check_external_security_audit_status", python_tests)
        self.assertIn("scripts.test_check_release_readiness", python_tests)
        self.assertIn("scripts.test_external_security_audit_workflow", python_tests)
        self.assertIn("scripts.test_tech_debt_dependency_security", python_tests)
        self.assertIn("scripts.test_validate_tech_debt_baseline_cli", python_tests)
        self.assertIn("scripts.test_validate_tech_debt_baseline_report", python_tests)

    def test_openspec_command_disables_telemetry(self) -> None:
        openspec = next(command for command in repository_gate.gate_commands() if command.name == "openspec")

        self.assertEqual({"OPENSPEC_TELEMETRY": "0"}, openspec.env)

    def test_run_command_passes_command_environment(self) -> None:
        result = type("CompletedProcess", (), {"returncode": 0})()
        command = repository_gate.GateCommand(
            "openspec",
            ["openspec", "validate", "--specs", "--strict"],
            {"OPENSPEC_TELEMETRY": "0"},
        )

        with (
            patch.object(repository_gate.subprocess, "run", return_value=result) as run,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = repository_gate.run_command(command, REPO_ROOT)

        self.assertEqual(0, status)
        self.assertEqual("0", run.call_args.kwargs["env"]["OPENSPEC_TELEMETRY"])

    def test_github_workflow_delegates_to_repository_gate(self) -> None:
        workflow = REPO_ROOT / ".github" / "workflows" / "repository-gate.yml"
        content = workflow.read_text()

        self.assertIn("pull_request:", content)
        self.assertIn("push:", content)
        self.assertIn("actions/checkout@v6", content)
        self.assertIn("actions/setup-python@v6", content)
        self.assertIn("actions/setup-node@v6", content)
        self.assertIn("@fission-ai/openspec@1.2.0", content)
        self.assertIn("python3 -m pip install ruff==0.15.15", content)
        self.assertIn("python3 scripts/validate_repository_gate.py", content)

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
