#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BRANCH = "master"
DEFAULT_MAX_AGE_HOURS = 168


@dataclass(frozen=True)
class ReleaseCommand:
    name: str
    argv: list[str]


def release_readiness_commands(args: argparse.Namespace) -> list[ReleaseCommand]:
    security_audit = [
        "python3",
        "scripts/check_external_security_audit_status.py",
        "--branch",
        args.branch,
        "--max-age-hours",
        str(args.max_age_hours),
        "--require-head",
    ]
    if args.repo:
        security_audit.extend(["--repo", args.repo])

    return [
        ReleaseCommand("repository-gate", ["python3", "scripts/validate_repository_gate.py"]),
        ReleaseCommand("external-security-audit", security_audit),
    ]


def run_command(command: ReleaseCommand, root: Path) -> int:
    print(f"==> {command.name}: {' '.join(command.argv)}")
    try:
        return subprocess.run(command.argv, cwd=root, check=False).returncode
    except FileNotFoundError:
        print(f"ERROR: missing executable for {command.name}: {command.argv[0]}", file=sys.stderr)
        return 127


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run release readiness checks for this repository.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root path.")
    parser.add_argument("--repo", help="GitHub repository in owner/name form. Defaults to origin remote.")
    parser.add_argument("--branch", default=DEFAULT_BRANCH, help="Branch to check.")
    parser.add_argument(
        "--max-age-hours",
        type=int,
        default=DEFAULT_MAX_AGE_HOURS,
        help="Maximum acceptable age for the latest successful External Security Audit run.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve()
    for command in release_readiness_commands(args):
        status = run_command(command, root)
        if status != 0:
            print(f"release readiness failed at {command.name}")
            return status
    print("release readiness PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
