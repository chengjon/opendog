from __future__ import annotations

import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


class ExternalSecurityAuditWorkflowTests(unittest.TestCase):
    def test_workflow_runs_pinned_external_security_tools(self) -> None:
        workflow = REPO_ROOT / ".github" / "workflows" / "external-security-audit.yml"
        content = workflow.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", content)
        self.assertIn("cron:", content)
        self.assertIn("cargo install cargo-audit --version 0.22.1 --locked", content)
        self.assertIn("cargo audit --color never", content)
        self.assertIn("docker run --rm", content)
        self.assertIn("zricethezav/gitleaks:v8.30.1", content)
        self.assertIn("detect \\", content)
        self.assertIn("--redact", content)


if __name__ == "__main__":
    unittest.main()
