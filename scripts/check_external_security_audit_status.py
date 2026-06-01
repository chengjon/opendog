#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import repo_paths

DEFAULT_BRANCH = "master"
DEFAULT_MAX_AGE_HOURS = 168
DEFAULT_WORKFLOW = "external-security-audit.yml"
ROOT = repo_paths.ROOT


@dataclass(frozen=True)
class AuditStatus:
    passed: bool
    summary: str
    run_url: str | None = None


@dataclass(frozen=True)
class WorkflowRun:
    conclusion: str | None
    head_sha: str
    html_url: str
    status: str
    updated_at: datetime


def parse_github_repo(remote_url: str) -> str:
    cleaned = remote_url.strip()
    if cleaned.startswith("git@github.com:"):
        cleaned = cleaned.removeprefix("git@github.com:")
    elif "github.com/" in cleaned:
        cleaned = cleaned.split("github.com/", 1)[1]
    else:
        raise ValueError(f"unsupported GitHub remote URL: {remote_url}")
    cleaned = cleaned.removesuffix(".git").strip("/")
    if cleaned.count("/") != 1:
        raise ValueError(f"unsupported GitHub remote URL: {remote_url}")
    return cleaned


def discover_repo(root: Path) -> str:
    try:
        completed = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError as error:
        raise RuntimeError("git CLI is required to discover repository origin") from error
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "unable to read git origin remote")
    return parse_github_repo(completed.stdout)


def current_head_sha(root: Path) -> str:
    try:
        completed = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError as error:
        raise RuntimeError("git CLI is required to read current HEAD") from error
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "unable to read current git HEAD")
    return completed.stdout.strip()


def parse_timestamp(value: str) -> datetime:
    normalized = value.replace("Z", "+00:00")
    return datetime.fromisoformat(normalized).astimezone(timezone.utc)


def workflow_run_from_json(value: dict[str, Any]) -> WorkflowRun:
    return WorkflowRun(
        conclusion=value.get("conclusion"),
        head_sha=str(value.get("head_sha", "")),
        html_url=str(value.get("html_url", "")),
        status=str(value.get("status", "")),
        updated_at=parse_timestamp(str(value["updated_at"])),
    )


def latest_completed_run(payload: dict[str, Any]) -> WorkflowRun | None:
    runs = [
        workflow_run_from_json(value)
        for value in payload.get("workflow_runs", [])
        if value.get("status") == "completed" and "updated_at" in value
    ]
    return max(runs, key=lambda run: run.updated_at) if runs else None


def evaluate_workflow_runs(
    payload: dict[str, Any],
    *,
    now: datetime,
    max_age_hours: int,
    expected_head_sha: str | None = None,
) -> AuditStatus:
    latest = latest_completed_run(payload)
    if latest is None:
        return AuditStatus(False, "external security audit: FAIL - no completed workflow runs found")
    short_sha = latest.head_sha[:7] if latest.head_sha else "unknown"
    age_hours = (now.astimezone(timezone.utc) - latest.updated_at).total_seconds() / 3600
    if latest.conclusion != "success":
        return AuditStatus(
            False,
            f"external security audit: FAIL - latest completed run for {short_sha} ended with {latest.conclusion}",
            latest.html_url,
        )
    if age_hours > max_age_hours:
        return AuditStatus(
            False,
            (
                "external security audit: FAIL - latest successful run is stale "
                f"({age_hours:.1f}h old, max {max_age_hours}h) for {short_sha}"
            ),
            latest.html_url,
        )
    if expected_head_sha and latest.head_sha != expected_head_sha:
        return AuditStatus(
            False,
            (
                "external security audit: FAIL - latest successful run "
                f"{short_sha} does not match required HEAD {expected_head_sha[:7]}"
            ),
            latest.html_url,
        )
    return AuditStatus(
        True,
        f"external security audit: PASS - latest successful run for {short_sha} completed {age_hours:.1f}h ago",
        latest.html_url,
    )


def build_workflow_runs_command(repo: str, workflow: str, branch: str) -> list[str]:
    return [
        "gh",
        "api",
        "-X",
        "GET",
        f"repos/{repo}/actions/workflows/{workflow}/runs",
        "-f",
        f"branch={branch}",
        "-f",
        "per_page=10",
    ]


def fetch_workflow_runs(repo: str, workflow: str, branch: str) -> dict[str, Any]:
    if shutil.which("gh") is None:
        raise RuntimeError("gh CLI is required to check external security audit status")
    try:
        completed = subprocess.run(
            build_workflow_runs_command(repo, workflow, branch),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError as error:
        raise RuntimeError("gh CLI is required to check external security audit status") from error
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "gh api workflow runs query failed")
    return json.loads(completed.stdout)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Check the latest External Security Audit workflow result.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root path.")
    parser.add_argument("--repo", help="GitHub repository in owner/name form. Defaults to origin remote.")
    parser.add_argument("--branch", default=DEFAULT_BRANCH, help="Branch to check.")
    parser.add_argument("--workflow", default=DEFAULT_WORKFLOW, help="Workflow file name or id.")
    parser.add_argument(
        "--max-age-hours",
        type=int,
        default=DEFAULT_MAX_AGE_HOURS,
        help="Maximum acceptable age for the latest successful run.",
    )
    parser.add_argument(
        "--require-head",
        action="store_true",
        help="Require the latest successful workflow run to match the current git HEAD.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve()
    try:
        repo = args.repo or discover_repo(root)
        expected_head_sha = current_head_sha(root) if args.require_head else None
        payload = fetch_workflow_runs(repo, args.workflow, args.branch)
        status = evaluate_workflow_runs(
            payload,
            now=datetime.now(timezone.utc),
            max_age_hours=args.max_age_hours,
            expected_head_sha=expected_head_sha,
        )
    except (RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"external security audit: FAIL - {error}")
        return 2
    print(status.summary)
    if status.run_url:
        print(status.run_url)
    return 0 if status.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
