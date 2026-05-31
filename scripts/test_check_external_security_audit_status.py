from __future__ import annotations

import sys
import unittest
from datetime import datetime, timezone
from pathlib import Path

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


if __name__ == "__main__":
    unittest.main()
