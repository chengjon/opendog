#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class GateCommand:
    name: str
    argv: list[str]


def gate_commands() -> list[GateCommand]:
    return [
        GateCommand("openspec", ["openspec", "validate", "--specs", "--strict"]),
        GateCommand("cargo-fmt", ["cargo", "fmt", "--check"]),
        GateCommand("cargo-test", ["cargo", "test", "--quiet"]),
        GateCommand(
            "cargo-clippy",
            ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
        ),
        GateCommand(
            "python-unit-tests",
            [
                "python3",
                "-m",
                "unittest",
                "scripts.test_validate_structural_hygiene",
                "scripts.test_structural_contract_guards",
                "scripts.test_structural_rust_guards",
                "scripts.test_tech_debt_dependency_security",
                "scripts.test_validate_tech_debt_baseline",
                "scripts.test_validate_planning_governance",
                "scripts.test_validate_repository_gate",
            ],
        ),
        GateCommand("tech-debt-baseline", ["python3", "scripts/validate_tech_debt_baseline.py"]),
        GateCommand("planning-governance", ["python3", "scripts/validate_planning_governance.py"]),
        GateCommand("structural-hygiene", ["python3", "scripts/validate_structural_hygiene.py"]),
        GateCommand("diff-check", ["git", "diff", "--check"]),
    ]


def run_command(command: GateCommand, root: Path) -> int:
    print(f"==> {command.name}: {' '.join(command.argv)}")
    return subprocess.run(command.argv, cwd=root, check=False).returncode


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the repository validation gate.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root path.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve()
    for command in gate_commands():
        status = run_command(command, root)
        if status != 0:
            print(f"repository validation gate failed at {command.name}")
            return status
    print("repository validation gate PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
