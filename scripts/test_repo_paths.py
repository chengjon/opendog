from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import check_external_security_audit_status as external_audit
import check_release_readiness as release_readiness
import planning_paths
import repo_paths
import structural_contract_guards
import structural_rust_guards
import validate_repository_gate
import validate_structural_hygiene


class RepoPathsTests(unittest.TestCase):
    def test_root_points_to_repository_root(self) -> None:
        self.assertEqual(SCRIPT_DIR.parent, repo_paths.ROOT)

    def test_scripts_share_repo_root_constant(self) -> None:
        self.assertIs(repo_paths.ROOT, external_audit.ROOT)
        self.assertIs(repo_paths.ROOT, release_readiness.ROOT)
        self.assertIs(repo_paths.ROOT, planning_paths.ROOT)
        self.assertIs(repo_paths.ROOT, structural_contract_guards.ROOT)
        self.assertIs(repo_paths.ROOT, structural_rust_guards.ROOT)
        self.assertIs(repo_paths.ROOT, validate_repository_gate.ROOT)
        self.assertIs(repo_paths.ROOT, validate_structural_hygiene.ROOT)


if __name__ == "__main__":
    unittest.main()
