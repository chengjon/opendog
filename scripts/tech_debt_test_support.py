from __future__ import annotations

from collections.abc import Iterable


DEFAULT_GATED_METRICS = [
    "production_unwrap_count",
    "should_panic_test_count",
    "policy_document_over_1000_count",
]
DEFAULT_OBSERVED_METRICS = ["duplicate_dependency_crate_count"]
DEFAULT_DOCUMENTATION_DOCUMENTS: list[dict[str, object]] = [
    {
        "file": "docs/mcp-tool-reference.md",
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
