from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path


DEFAULT_GATED_METRICS = [
    "production_unwrap_count",
    "should_panic_test_count",
    "policy_document_over_1000_count",
]
DEFAULT_OBSERVED_METRICS = ["duplicate_dependency_crate_count"]
DEFAULT_DOCUMENTATION_POLICY_FILE = "docs/mcp-tool-reference.md"
DEFAULT_DOCUMENTATION_DOCUMENTS: list[dict[str, object]] = [
    {
        "file": DEFAULT_DOCUMENTATION_POLICY_FILE,
        "line_limit": 1000,
    }
]


def baseline_payload(
    *,
    gated_metrics: Iterable[str] | None = None,
    observed_metrics: Iterable[str] | None = None,
    documentation_documents: list[dict[str, object]] | None = None,
    **overrides: object,
) -> dict[str, object]:
    documents = DEFAULT_DOCUMENTATION_DOCUMENTS if documentation_documents is None else documentation_documents
    data: dict[str, object] = {
        "metric_version": "v1.0",
        "generated_at": "2026-05-31T02:27:49Z",
        "project": "opendog-test",
        "gated_metrics": list(gated_metrics or DEFAULT_GATED_METRICS),
        "observed_metrics": list(observed_metrics or DEFAULT_OBSERVED_METRICS),
        "production_unwrap_count": 0,
        "should_panic_test_count": 0,
        "policy_document_over_1000_count": 0,
        "duplicate_dependency_crate_count": 4,
        "documentation_policy": {"documents": [dict(document) for document in documents]},
    }
    data.update(overrides)
    return data


def write_file(root: Path, relative_path: str, content: str) -> Path:
    path = root / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return path


def write_cargo_inventory(
    root: Path,
    *,
    dependencies: Iterable[str] | None = None,
    dev_dependencies: Iterable[str] | None = None,
    lock_packages: Iterable[tuple[str, str]] | None = None,
    with_lockfile: bool = True,
) -> None:
    manifest_lines = [
        "[package]",
        'name = "demo"',
        'version = "0.1.0"',
        'edition = "2021"',
        "",
        "[dependencies]",
        *(dependencies or ['serde = "1"']),
    ]
    dev_dependency_lines = list(dev_dependencies or [])
    if dev_dependency_lines:
        manifest_lines.extend(["", "[dev-dependencies]", *dev_dependency_lines])
    write_file(root, "Cargo.toml", "\n".join(manifest_lines))

    if not with_lockfile:
        return

    packages = list(lock_packages or [("demo", "0.1.0")])
    lock_lines = ["version = 3"]
    for name, version in packages:
        lock_lines.extend(["", "[[package]]", f'name = "{name}"', f'version = "{version}"'])
    write_file(root, "Cargo.lock", "\n".join(lock_lines))
