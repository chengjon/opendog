from __future__ import annotations

import contextlib
import importlib
import io
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import check_release_readiness as release_readiness
import check_external_security_audit_status as external_audit

TEST_REPOSITORY = "chengjon/opendog"
TEST_BRANCH = "master"
TEST_MAX_AGE_HOURS = "72"


class ReleaseReadinessTests(unittest.TestCase):
    def test_defaults_follow_external_security_audit_defaults(self) -> None:
        original_branch = external_audit.DEFAULT_BRANCH
        original_max_age_hours = external_audit.DEFAULT_MAX_AGE_HOURS
        try:
            external_audit.DEFAULT_BRANCH = "release/audit-default"
            external_audit.DEFAULT_MAX_AGE_HOURS = 24
            importlib.reload(release_readiness)

            args = release_readiness.parse_args([])

            self.assertEqual("release/audit-default", args.branch)
            self.assertEqual(24, args.max_age_hours)
        finally:
            external_audit.DEFAULT_BRANCH = original_branch
            external_audit.DEFAULT_MAX_AGE_HOURS = original_max_age_hours
            importlib.reload(release_readiness)

    def test_commands_include_repository_gate_and_head_matched_security_audit(self) -> None:
        args = release_readiness.parse_args(
            [
                "--repo",
                TEST_REPOSITORY,
                "--branch",
                TEST_BRANCH,
                "--max-age-hours",
                TEST_MAX_AGE_HOURS,
            ]
        )

        commands = release_readiness.release_readiness_commands(args)

        self.assertEqual(
            [
                release_readiness.REPOSITORY_GATE_COMMAND_NAME,
                release_readiness.EXTERNAL_SECURITY_AUDIT_COMMAND_NAME,
            ],
            [command.name for command in commands],
        )
        self.assertEqual(
            [release_readiness.PYTHON_EXECUTABLE, release_readiness.REPOSITORY_GATE_SCRIPT],
            commands[0].argv,
        )
        self.assertEqual(
            [
                release_readiness.PYTHON_EXECUTABLE,
                release_readiness.EXTERNAL_SECURITY_AUDIT_SCRIPT,
                "--branch",
                TEST_BRANCH,
                "--max-age-hours",
                TEST_MAX_AGE_HOURS,
                "--require-head",
                "--repo",
                TEST_REPOSITORY,
            ],
            commands[1].argv,
        )

    def test_repo_argument_is_optional(self) -> None:
        args = release_readiness.parse_args([])

        commands = release_readiness.release_readiness_commands(args)

        self.assertNotIn("--repo", commands[1].argv)
        self.assertIn("--require-head", commands[1].argv)

    def test_run_command_reports_missing_executable(self) -> None:
        command = release_readiness.ReleaseCommand(
            release_readiness.REPOSITORY_GATE_COMMAND_NAME,
            [release_readiness.PYTHON_EXECUTABLE, release_readiness.REPOSITORY_GATE_SCRIPT],
        )
        stderr = io.StringIO()

        with (
            patch.object(release_readiness.subprocess, "run", side_effect=FileNotFoundError()),
            contextlib.redirect_stdout(io.StringIO()),
            contextlib.redirect_stderr(stderr),
        ):
            status = release_readiness.run_command(command, SCRIPT_DIR.parent)

        self.assertEqual(127, status)
        self.assertIn(
            f"missing executable for {release_readiness.REPOSITORY_GATE_COMMAND_NAME}: {release_readiness.PYTHON_EXECUTABLE}",
            stderr.getvalue(),
        )

    def test_main_stops_at_first_failure(self) -> None:
        calls: list[str] = []

        def fake_run(command: release_readiness.ReleaseCommand, root: Path) -> int:
            calls.append(command.name)
            return 1 if command.name == release_readiness.REPOSITORY_GATE_COMMAND_NAME else 0

        with (
            patch.object(release_readiness, "run_command", side_effect=fake_run),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            status = release_readiness.main([])

        self.assertEqual(1, status)
        self.assertEqual([release_readiness.REPOSITORY_GATE_COMMAND_NAME], calls)

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
        self.assertEqual(
            [
                release_readiness.REPOSITORY_GATE_COMMAND_NAME,
                release_readiness.EXTERNAL_SECURITY_AUDIT_COMMAND_NAME,
            ],
            calls,
        )


if __name__ == "__main__":
    unittest.main()
