#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

import planning_governance_rules as governance_rules
import tech_debt_baseline
import validate_requirement_mappings as requirement_mappings
import validate_structural_hygiene as structural_hygiene
import validate_task_cards as task_cards


ROOT = Path(__file__).resolve().parents[1]
FUNCTION_TREE_FILE = task_cards.TREE_FILE
REQUIREMENTS_FILE = task_cards.REQUIREMENTS_FILE
ROADMAP_FILE = ROOT / ".planning" / "ROADMAP.md"
TASK_CARD_DIR = task_cards.TASK_DIR
TECH_DEBT_BASELINE_FILE = ROOT / "reports" / "analysis" / "tech-debt-baseline.json"
TECH_DEBT_LIGHTWEIGHT_EXCLUDED_METRICS = {
    "duplicate_dependency_crate_count",
    "rust_check_errors",
    "rust_clippy_errors",
    "rust_clippy_warnings",
    "backend_lint_errors",
    "backend_lint_warnings",
}


def validate_task_cards(ft_levels: dict[str, str]) -> list[str]:
    requirement_sections = requirement_mappings.parse_requirement_sections(
        REQUIREMENTS_FILE.read_text(encoding="utf-8")
    )
    valid_requirement_ids = {
        req_id
        for section in requirement_sections
        for req_id in section["requirement_ids"]
    }
    errors: list[str] = []
    for path in sorted(TASK_CARD_DIR.glob("TASK-*.md")):
        errors.extend(task_cards.validate_card(path, ft_levels, valid_requirement_ids))
    return errors


def validate_requirement_sections(ft_levels: dict[str, str]) -> tuple[list[str], list[dict[str, object]]]:
    sections = requirement_mappings.parse_requirement_sections(REQUIREMENTS_FILE.read_text(encoding="utf-8"))
    errors = requirement_mappings.validate_sections(sections, ft_levels)
    return errors, sections


def validate_roadmap_counts(counts: dict[str, int]) -> list[str]:
    errors: list[str] = []
    text = ROADMAP_FILE.read_text(encoding="utf-8")
    match = re.search(r"^\*\*Requirements:\*\*\s*(\d+)\s+total\s+\|\s+(\d+)\s+phase-mapped\s+\|\s+(\d+)\s+backlog\s*$", text, re.M)
    if not match:
        return [f"{ROADMAP_FILE}: missing or malformed requirements headline"]

    expected_total, expected_phase_mapped, expected_backlog = map(int, match.groups())
    if expected_total != counts["total"]:
        errors.append(f"{ROADMAP_FILE}: header total={expected_total} but computed total={counts['total']}")
    if expected_phase_mapped != counts["phase_mapped"]:
        errors.append(
            f"{ROADMAP_FILE}: header phase-mapped={expected_phase_mapped} but computed phase-mapped={counts['phase_mapped']}"
        )
    if expected_backlog != counts["backlog"]:
        errors.append(f"{ROADMAP_FILE}: header backlog={expected_backlog} but computed backlog={counts['backlog']}")
    return errors


def validate_tech_debt_baseline(
    root: Path = ROOT,
    baseline_path: Path = TECH_DEBT_BASELINE_FILE,
) -> tuple[list[str], list[str]]:
    baseline = tech_debt_baseline.load_baseline(baseline_path)
    current = tech_debt_baseline.measure_current_metrics(
        root,
        baseline=baseline,
        include_command_metrics=False,
        include_dependency_metrics=True,
    )
    result = tech_debt_baseline.compare_to_baseline(
        baseline,
        current,
        excluded_metrics=TECH_DEBT_LIGHTWEIGHT_EXCLUDED_METRICS,
    )
    return (
        [f"technical debt baseline: {error}" for error in result.errors],
        [f"technical debt baseline: {warning}" for warning in result.warnings],
    )


def main() -> int:
    ft_text = FUNCTION_TREE_FILE.read_text(encoding="utf-8")
    ft_levels = requirement_mappings.parse_function_tree_levels(ft_text)
    ft_nodes = governance_rules.parse_function_tree_nodes(ft_text)
    task_files = sorted(TASK_CARD_DIR.glob("TASK-*.md"))

    errors: list[str] = []
    errors.extend(validate_task_cards(ft_levels))
    requirement_errors, sections = validate_requirement_sections(ft_levels)
    errors.extend(requirement_errors)
    errors.extend(governance_rules.validate_function_tree_coverage(ft_nodes, sections))
    counts = governance_rules.compute_counts(ft_nodes, sections)
    errors.extend(validate_roadmap_counts(counts))
    structural_errors, structural_rule_count, structural_file_count = structural_hygiene.validate_repository(ROOT)
    errors.extend(structural_errors)
    tech_debt_errors, tech_debt_warnings = validate_tech_debt_baseline()
    errors.extend(tech_debt_errors)

    if errors:
        print("planning governance validation failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    for warning in tech_debt_warnings:
        print(f"planning governance warning: {warning}")

    status_counts = task_cards.collect_status_counts(task_files)
    status_summary = ", ".join(
        f"{count} {status}" for status, count in sorted(status_counts.items())
    )
    print(
        "validated planning governance: "
        f"{counts['total']} requirements, {counts['phase_mapped']} phase-mapped, "
        f"{counts['backlog']} backlog, {len(task_files)} task card(s) [{status_summary}], "
        f"structural hygiene {structural_rule_count} rule(s) / {structural_file_count} file(s), "
        "technical debt baseline static gate PASS"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
