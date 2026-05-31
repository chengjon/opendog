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

import check_release_readiness as release_readiness


class ReleaseReadinessTests(unittest.TestCase):
    def test_commands_include_repository_gate_and_head_matched_security_audit(self) -> None:
        args = release_readiness.parse_args(
            [
                "--repo",
                "chengjon/opendog",
                "--branch",
                "master",
                "--max-age-hours",
                "72",
            ]
        )

        commands = release_readiness.release_readiness_commands(args)

        self.assertEqual(["repository-gate", "external-security-audit"], [command.name for command in commands])
        self.assertEqual(["python3", "scripts/validate_repository_gate.py"], commands[0].argv)
        self.assertEqual(
            [
                "python3",
                "scripts/check_external_security_audit_status.py",
                "--branch",
                "master",
                "--max-age-hours",
                "72",
                "--require-head",
                "--repo",
                "chengjon/opendog",
            ],
            commands[1].argv,
        )

    def test_repo_argument_is_optional(self) -> None:
        args = release_readiness.parse_args([])

        commands = release_readiness.release_readiness_commands(args)

        self.assertNotIn("--repo", commands[1].argv)
        self.assertIn("--require-head", commands[1].argv)

    def test_main_stops_at_first_failure(self) -> None:
        calls: list[str] = []

        def fake_run(command: release_readiness.ReleaseCommand, root: Path) -> int:
            calls.append(command.name)
            return 1 if command.name == "repository-gate" else 0

        with (
            patch.object(release_readiness, "run_command", side_effect=fake_run),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = release_readiness.main([])

        self.assertEqual(1, status)
        self.assertEqual(["repository-gate"], calls)

    def test_main_runs_all_commands_when_successful(self) -> None:
        calls: list[str] = []

        def fake_run(command: release_readiness.ReleaseCommand, root: Path) -> int:
            calls.append(command.name)
            return 0

        with (
            patch.object(release_readiness, "run_command", side_effect=fake_run),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = release_readiness.main([])

        self.assertEqual(0, status)
        self.assertEqual(["repository-gate", "external-security-audit"], calls)


if __name__ == "__main__":
    unittest.main()

