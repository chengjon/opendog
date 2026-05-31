from __future__ import annotations

import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path
from typing import Any

CODE_FILE_LINE_LIMIT = 500
EXCLUDED_DIRS = {".git", ".zread", "node_modules", "reports", "target"}
SECRET_SCAN_SUFFIXES = {".env", ".json", ".md", ".py", ".rs", ".toml", ".yaml", ".yml"}
MAX_REPORTED_SECRET_FINDINGS = 25

PRODUCTION_RUST_PATTERNS: dict[str, re.Pattern[str]] = {
    "production_panic_count": re.compile(r"\bpanic!\s*\("),
    "production_unwrap_count": re.compile(r"\.unwrap\s*\("),
    "production_expect_count": re.compile(r"\.expect\s*\("),
    "production_allow_count": re.compile(r"#\[allow\s*\("),
    "production_todo_macro_count": re.compile(r"\btodo!\s*\("),
    "production_unimplemented_count": re.compile(r"\bunimplemented!\s*\("),
    "production_dbg_count": re.compile(r"\bdbg!\s*\("),
    "production_todo_comment_count": re.compile(r"TODO|FIXME|HACK|XXX"),
}

SECRET_PATTERNS: dict[str, re.Pattern[str]] = {
    "aws_access_key": re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    "github_token": re.compile(r"\bghp_[A-Za-z0-9_]{36}\b"),
    "google_api_key": re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b"),
    "openai_api_key": re.compile(r"\bsk-[A-Za-z0-9]{20,}\b"),
    "private_key": re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    "slack_token": re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
}


def load_baseline(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def iter_project_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        if any(part in EXCLUDED_DIRS for part in path.relative_to(root).parts):
            continue
        if path.is_file():
            files.append(path)
    return files


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def is_code_file(path: Path, root: Path) -> bool:
    rel = path.relative_to(root).as_posix()
    return (
        path.suffix in {".rs", ".py"}
        and (rel.startswith("src/") or rel.startswith("tests/") or rel.startswith("scripts/"))
    )


def is_rust_test_path(path: Path, root: Path) -> bool:
    rel_parts = path.relative_to(root).parts
    rel = path.relative_to(root).as_posix()
    return "tests" in rel_parts or rel.endswith("tests.rs")


def strip_cfg_test_sections(text: str) -> str:
    lines = text.splitlines()
    kept: list[str] = []
    index = 0
    while index < len(lines):
        line = lines[index]
        if re.search(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", line):
            index = skip_cfg_test_block(lines, index + 1)
            continue
        kept.append(line)
        index += 1
    return "\n".join(kept)


def skip_cfg_test_block(lines: list[str], index: int) -> int:
    if index >= len(lines):
        return index
    line = lines[index]
    if re.search(r"\bmod\s+\w+\s*;", line):
        return index + 1
    if not (re.search(r"\bmod\s+\w+\b", line) or re.search(r"\bfn\s+\w+\b", line)):
        return index + 1
    depth = 0
    started = False
    while index < len(lines):
        current = lines[index]
        depth += current.count("{") - current.count("}")
        started = started or "{" in current
        index += 1
        if started and depth <= 0:
            break
    return index


def count_pattern(text: str, pattern: re.Pattern[str]) -> int:
    return len(pattern.findall(text))


def measure_production_rust_metrics(root: Path, files: list[Path]) -> dict[str, int]:
    metrics = {name: 0 for name in PRODUCTION_RUST_PATTERNS}
    for path in files:
        if path.suffix != ".rs" or is_rust_test_path(path, root):
            continue
        text = strip_cfg_test_sections(path.read_text(encoding="utf-8"))
        for name, pattern in PRODUCTION_RUST_PATTERNS.items():
            metrics[name] += count_pattern(text, pattern)
    return metrics


def measure_test_metrics(root: Path, files: list[Path]) -> dict[str, int]:
    metrics = {
        "skip_xfail_count": 0,
        "ignored_test_count": 0,
        "should_panic_test_count": 0,
        "test_placeholder_assert_count": 0,
        "test_sleep_call_count": 0,
        "backend_todo_count": 0,
        "backend_placeholder_count": 0,
    }
    for path in files:
        rel = path.relative_to(root).as_posix()
        if path.suffix not in {".rs", ".py"}:
            continue
        text = path.read_text(encoding="utf-8")
        if not is_test_bearing_file(path, root, rel, text):
            continue
        update_test_metrics_for_file(metrics, path, text)
    return metrics


def is_test_bearing_file(path: Path, root: Path, rel: str, text: str) -> bool:
    if path.suffix == ".rs":
        return is_rust_test_path(path, root) or "#[cfg(test)]" in text
    return rel.startswith("tests/") or Path(rel).name.startswith("test_")


def update_test_metrics_for_file(metrics: dict[str, int], path: Path, text: str) -> None:
    if path.suffix == ".rs":
        metrics["ignored_test_count"] += text.count("#[ignore]")
        metrics["should_panic_test_count"] += text.count("#[should_panic")
        metrics["test_placeholder_assert_count"] += count_pattern(
            text, re.compile(r"assert!\s*\(\s*(true|false)\s*\)")
        )
    metrics["test_sleep_call_count"] += count_pattern(text, re.compile(r"sleep\s*\("))
    metrics["skip_xfail_count"] += count_pattern(
        text, re.compile(r"pytest\.mark\.(skip|xfail)|@unittest\.skip")
    )
    metrics["backend_todo_count"] += count_pattern(text, re.compile(r"TODO|FIXME|HACK|XXX"))
    metrics["backend_placeholder_count"] += count_pattern(
        text, re.compile(r"\bpass\b|todo!\s*\(|unimplemented!\s*\(")
    )


def measure_size_metrics(root: Path, files: list[Path]) -> dict[str, int]:
    code_files = [path for path in files if is_code_file(path, root)]
    return {
        "large_file_count_code": sum(1 for path in code_files if line_count(path) > CODE_FILE_LINE_LIMIT),
        "large_file_count_python": sum(
            1 for path in code_files if path.suffix == ".py" and line_count(path) > CODE_FILE_LINE_LIMIT
        ),
        "large_file_count_frontend": 0,
    }


def measure_document_policy_metrics(root: Path, baseline: dict[str, Any] | None) -> dict[str, int]:
    policy = (baseline or {}).get("documentation_policy", {})
    over_limit = 0
    for document in policy.get("documents", []):
        file_name = document.get("file")
        line_limit = document.get("line_limit")
        if not isinstance(file_name, str) or not isinstance(line_limit, int):
            continue
        path = root / file_name
        if path.exists() and line_count(path) > line_limit:
            over_limit += 1
    return {"policy_document_over_1000_count": over_limit}


def run_command(root: Path, command: list[str]) -> int:
    completed = subprocess.run(
        command,
        cwd=root,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return completed.returncode


def measure_command_metrics(root: Path) -> dict[str, int]:
    check_status = run_command(root, ["cargo", "check", "--all-targets", "--all-features", "--quiet"])
    clippy_status = run_command(
        root,
        ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
    )
    return {
        "rust_check_errors": 0 if check_status == 0 else 1,
        "rust_clippy_errors": 0 if clippy_status == 0 else 1,
        "rust_clippy_warnings": 0 if clippy_status == 0 else 1,
        "backend_lint_errors": 0 if clippy_status == 0 else 1,
        "backend_lint_warnings": 0,
    }


def load_toml_file(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError, UnicodeDecodeError):
        return {}


def count_manifest_dependencies(manifest: dict[str, Any]) -> int:
    sections = ("dependencies", "dev-dependencies", "build-dependencies")
    total = sum(len(manifest.get(section, {})) for section in sections if isinstance(manifest.get(section), dict))
    targets = manifest.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if not isinstance(target, dict):
                continue
            total += sum(len(target.get(section, {})) for section in sections if isinstance(target.get(section), dict))
    return total


def count_locked_packages(lockfile: dict[str, Any]) -> int:
    packages = lockfile.get("package", [])
    return len(packages) if isinstance(packages, list) else 0


def dependency_audit_tool() -> str | None:
    for tool in ("cargo-audit", "cargo-deny"):
        if shutil.which(tool):
            return tool
    return None


def secret_scan_tool() -> str | None:
    for tool in ("gitleaks", "trufflehog"):
        if shutil.which(tool):
            return tool
    return None


def measure_dependency_metrics(root: Path) -> dict[str, Any]:
    result = subprocess.run(
        ["cargo", "tree", "-d", "--depth", "3"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        check=False,
    )
    crates = {
        match.group(1)
        for line in result.stdout.splitlines()
        if (match := re.match(r"^\s*([A-Za-z0-9_-]+) v\d+\.\d+\.\d+", line))
    }
    manifest = load_toml_file(root / "Cargo.toml")
    lockfile_path = root / "Cargo.lock"
    locked_packages = count_locked_packages(load_toml_file(lockfile_path))
    lockfile_missing_count = 0 if lockfile_path.exists() else 1
    external_tool = dependency_audit_tool()
    dependency_audit = {
        "scanner": "internal-cargo-inventory",
        "external_tool": external_tool,
        "external_tool_available": external_tool is not None,
        "vulnerability_scan_available": external_tool is not None,
        "lockfile_present": lockfile_missing_count == 0,
        "duplicate_crate_count": len(crates),
        "manifest_dependency_count": count_manifest_dependencies(manifest),
        "locked_package_count": locked_packages,
    }
    return {
        "duplicate_dependency_crate_count": len(crates),
        "dependency_audit_issue_count": lockfile_missing_count,
        "dependency_lockfile_missing_count": lockfile_missing_count,
        "manifest_dependency_count": dependency_audit["manifest_dependency_count"],
        "locked_dependency_package_count": locked_packages,
        "dependency_audit": dependency_audit,
    }


def is_secret_scan_file(path: Path, root: Path) -> bool:
    rel = path.relative_to(root)
    if any(part in EXCLUDED_DIRS for part in rel.parts):
        return False
    return path.suffix in SECRET_SCAN_SUFFIXES or path.name.startswith(".env")


def secret_fingerprint(pattern_name: str, relative_path: str, line_number: int, value: str) -> str:
    return f"{pattern_name}:{relative_path}:{line_number}:len{len(value)}"


def measure_secret_scan_metrics(root: Path, files: list[Path]) -> dict[str, Any]:
    findings: list[dict[str, Any]] = []
    finding_count = 0
    for path in files:
        if not is_secret_scan_file(path, root):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        relative_path = path.relative_to(root).as_posix()
        for line_number, line in enumerate(text.splitlines(), start=1):
            for pattern_name, pattern in SECRET_PATTERNS.items():
                for match in pattern.finditer(line):
                    finding_count += 1
                    if len(findings) < MAX_REPORTED_SECRET_FINDINGS:
                        findings.append(
                            {
                                "file": relative_path,
                                "line": line_number,
                                "pattern": pattern_name,
                                "fingerprint": secret_fingerprint(
                                    pattern_name,
                                    relative_path,
                                    line_number,
                                    match.group(0),
                                ),
                            }
                        )
    external_tool = secret_scan_tool()
    return {
        "high_confidence_secret_count": finding_count,
        "secret_scan": {
            "scanner": "internal-high-confidence-patterns",
            "external_tool": external_tool,
            "external_tool_available": external_tool is not None,
            "finding_count": finding_count,
            "findings": findings,
        },
    }


def measure_tool_availability() -> dict[str, bool]:
    dependency_tool_available = dependency_audit_tool() is not None
    secret_tool_available = secret_scan_tool() is not None
    return {
        "dependency_audit_available": True,
        "secret_scan_available": True,
        "external_dependency_audit_available": dependency_tool_available,
        "external_secret_scan_available": secret_tool_available,
    }
