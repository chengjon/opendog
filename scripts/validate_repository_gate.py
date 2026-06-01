#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class GateCommand:
    name: str
    argv: list[str]
    env: dict[str, str] | None = None


def python_test_modules(root: Path) -> list[str]:
    scripts_dir = root / "scripts"
    return [f"scripts.{path.stem}" for path in sorted(scripts_dir.glob("test_*.py"))]


def gate_commands(root: Path = ROOT) -> list[GateCommand]:
    return [
        GateCommand(
            "openspec",
            ["openspec", "validate", "--specs", "--strict"],
            {"OPENSPEC_TELEMETRY": "0"},
        ),
        GateCommand("cargo-fmt", ["cargo", "fmt", "--check"]),
        GateCommand("cargo-test", ["cargo", "test", "--quiet"]),
        GateCommand(
            "cargo-clippy",
            ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
        ),
        GateCommand("python-ruff", ["ruff", "check", "scripts"]),
        GateCommand(
            "python-unit-tests",
            [
                "python3",
                "-m",
                "unittest",
                *python_test_modules(root),
            ],
        ),
        GateCommand("tech-debt-baseline", ["python3", "scripts/validate_tech_debt_baseline.py"]),
        GateCommand("planning-governance", ["python3", "scripts/validate_planning_governance.py"]),
        GateCommand("structural-hygiene", ["python3", "scripts/validate_structural_hygiene.py"]),
        GateCommand("diff-check", ["git", "diff", "--check"]),
    ]


def run_command(command: GateCommand, root: Path) -> int:
    print(f"==> {command.name}: {' '.join(command.argv)}")
    env = os.environ.copy()
    if command.env:
        env.update(command.env)
    try:
        return subprocess.run(command.argv, cwd=root, check=False, env=env).returncode
    except FileNotFoundError:
        print(f"ERROR: missing executable for {command.name}: {command.argv[0]}", file=sys.stderr)
        return 127


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the repository validation gate.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root path.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve()
    for command in gate_commands(root):
        status = run_command(command, root)
        if status != 0:
            print(f"repository validation gate failed at {command.name}")
            return status
    print("repository validation gate PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
