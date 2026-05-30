pub(super) fn is_strong_hardcoded_combo(
    path_classification: &str,
    business_hits: usize,
    literal_hits: usize,
) -> bool {
    match path_classification {
        "runtime_shared" => business_hits >= 2 && literal_hits >= 2,
        "test_only" | "generated_artifact" => false,
        _ => business_hits >= 3 && literal_hits >= 2,
    }
}

pub(super) fn allow_runtime_shared_hardcoded_amplification(
    path_classification: &str,
    combo_is_strong: bool,
) -> bool {
    path_classification == "runtime_shared" && combo_is_strong
}

pub(super) fn hardcoded_review_priority(
    path_classification: &str,
    has_template_placeholder: bool,
) -> &'static str {
    if path_classification == "runtime_shared" && !has_template_placeholder {
        "high"
    } else if path_classification == "documentation" || has_template_placeholder {
        "low"
    } else {
        "medium"
    }
}

pub(super) fn hardcoded_confidence(
    path_classification: &str,
    has_template_placeholder: bool,
) -> &'static str {
    if path_classification == "runtime_shared" && !has_template_placeholder {
        "high"
    } else if path_classification == "documentation" || has_template_placeholder {
        "low"
    } else {
        "medium"
    }
}

pub(super) fn discounted_weak_literal_hits(raw_weak_hits: usize) -> usize {
    raw_weak_hits / 2
}
