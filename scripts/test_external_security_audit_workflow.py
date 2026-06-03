from __future__ import annotations

import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import check_external_security_audit_status as audit_status
from tech_debt_baseline import metrics as tech_debt_metrics


class ExternalSecurityAuditWorkflowTests(unittest.TestCase):
    def test_workflow_runs_pinned_external_security_tools(self) -> None:
        workflow = audit_status.WORKFLOW_FILE
        self.assertEqual(audit_status.DEFAULT_WORKFLOW, workflow.name)
        content = workflow.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", content)
        self.assertIn("cron:", content)
        self.assertIn("actions/checkout@v6", content)
        self.assertIn("cargo install cargo-audit --version 0.22.1 --locked", content)
        self.assertIn("cargo audit --color never", content)
        self.assertIn("cargo install cargo-deny --version 0.19.8 --locked", content)
        self.assertIn("cargo deny check advisories bans licenses sources", content)
        self.assertIn("docker run --rm", content)
        self.assertIn("zricethezav/gitleaks:v8.30.1", content)
        self.assertIn("detect \\", content)
        self.assertIn("--redact", content)

    def test_cargo_deny_policy_covers_dependency_governance_dimensions(self) -> None:
        policy = REPO_ROOT / "deny.toml"
        content = policy.read_text(encoding="utf-8")

        self.assertIn("[advisories]", content)
        self.assertIn("[licenses]", content)
        self.assertIn("[licenses.private]", content)
        self.assertIn("[bans]", content)
        self.assertIn("[sources]", content)
        self.assertIn('ignore = true', content)
        self.assertIn('unknown-registry = "deny"', content)
        self.assertIn('unknown-git = "deny"', content)

    def test_cargo_deny_graph_matches_ci_target_scope(self) -> None:
        policy = REPO_ROOT / "deny.toml"
        content = policy.read_text(encoding="utf-8")

        self.assertIn("[graph]", content)
        self.assertIn('targets = ["x86_64-unknown-linux-gnu"]', content)

    def test_package_manifest_is_marked_unpublished_for_license_checks(self) -> None:
        manifest = REPO_ROOT / tech_debt_metrics.CARGO_MANIFEST_FILE
        content = manifest.read_text(encoding="utf-8")

        self.assertIn("publish = false", content)


if __name__ == "__main__":
    unittest.main()
