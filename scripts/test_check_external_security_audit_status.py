from __future__ import annotations

import sys
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest.mock import patch

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import check_external_security_audit_status as audit_status


class ExternalSecurityAuditStatusTests(unittest.TestCase):
    def run_payload(self, *, conclusion: str = "success", updated_at: str = "2026-05-31T10:00:00Z") -> dict:
        return {
            "workflow_runs": [
                {
                    "id": 123,
                    "status": "completed",
                    "conclusion": conclusion,
                    "head_sha": "abc123def456",
                    "html_url": "https://github.com/chengjon/opendog/actions/runs/123",
                    "updated_at": updated_at,
                }
            ]
        }

    def test_evaluates_recent_successful_run_as_pass(self) -> None:
        result = audit_status.evaluate_workflow_runs(
            self.run_payload(),
            now=datetime(2026, 5, 31, 12, 0, tzinfo=timezone.utc),
            max_age_hours=24,
        )

        self.assertTrue(result.passed)
        self.assertIn("PASS", result.summary)
        self.assertIn("abc123d", result.summary)

    def test_failed_run_does_not_pass(self) -> None:
        result = audit_status.evaluate_workflow_runs(
            self.run_payload(conclusion="failure"),
            now=datetime(2026, 5, 31, 12, 0, tzinfo=timezone.utc),
            max_age_hours=24,
        )

        self.assertFalse(result.passed)
        self.assertIn("failure", result.summary)

    def test_required_head_sha_must_match_latest_successful_run(self) -> None:
        result = audit_status.evaluate_workflow_runs(
            self.run_payload(),
            now=datetime(2026, 5, 31, 12, 0, tzinfo=timezone.utc),
            max_age_hours=24,
            expected_head_sha="abc123def456",
        )

        self.assertTrue(result.passed)

    def test_required_head_sha_mismatch_does_not_pass(self) -> None:
        result = audit_status.evaluate_workflow_runs(
            self.run_payload(),
            now=datetime(2026, 5, 31, 12, 0, tzinfo=timezone.utc),
            max_age_hours=24,
            expected_head_sha="fffffff00000",
        )

        self.assertFalse(result.passed)
        self.assertIn("does not match required HEAD", result.summary)

    def test_stale_successful_run_does_not_pass(self) -> None:
        result = audit_status.evaluate_workflow_runs(
            self.run_payload(updated_at="2026-05-29T10:00:00Z"),
            now=datetime(2026, 5, 31, 12, 0, tzinfo=timezone.utc),
            max_age_hours=24,
        )

        self.assertFalse(result.passed)
        self.assertIn("stale", result.summary)

    def test_parses_github_remote_urls(self) -> None:
        self.assertEqual(
            "chengjon/opendog",
            audit_status.parse_github_repo("git@github.com:chengjon/opendog.git"),
        )
        self.assertEqual(
            "chengjon/opendog",
            audit_status.parse_github_repo("https://github.com/chengjon/opendog.git"),
        )

    def test_builds_gh_api_command_for_workflow_runs(self) -> None:
        command = audit_status.build_workflow_runs_command(
            "chengjon/opendog",
            "external-security-audit.yml",
            "master",
        )

        self.assertEqual("gh", command[0])
        self.assertIn("repos/chengjon/opendog/actions/workflows/external-security-audit.yml/runs", command)
        self.assertIn("branch=master", command)

    def test_fetch_workflow_runs_reports_missing_gh_executable(self) -> None:
        with patch.object(audit_status.shutil, "which", return_value=None):
            with self.assertRaisesRegex(RuntimeError, "gh CLI is required to check external security audit status"):
                audit_status.fetch_workflow_runs("chengjon/opendog", "external-security-audit.yml", "master")

    def test_fetch_workflow_runs_reports_gh_disappearing_after_lookup(self) -> None:
        with (
            patch.object(audit_status.shutil, "which", return_value="/usr/bin/gh"),
            patch.object(audit_status.subprocess, "run", side_effect=FileNotFoundError()),
        ):
            with self.assertRaisesRegex(RuntimeError, "gh CLI is required to check external security audit status"):
                audit_status.fetch_workflow_runs("chengjon/opendog", "external-security-audit.yml", "master")

    def test_parse_args_supports_require_head(self) -> None:
        args = audit_status.parse_args(["--require-head"])

        self.assertTrue(args.require_head)

    def test_discover_repo_reports_missing_git_executable(self) -> None:
        with patch.object(audit_status.subprocess, "run", side_effect=FileNotFoundError()):
            with self.assertRaisesRegex(RuntimeError, "git CLI is required to discover repository origin"):
                audit_status.discover_repo(Path.cwd())

    def test_current_head_sha_reports_missing_git_executable(self) -> None:
        with patch.object(audit_status.subprocess, "run", side_effect=FileNotFoundError()):
            with self.assertRaisesRegex(RuntimeError, "git CLI is required to read current HEAD"):
                audit_status.current_head_sha(Path.cwd())


if __name__ == "__main__":
    unittest.main()
