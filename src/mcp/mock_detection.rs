use std::fs;
use std::path::Path;

use crate::storage::queries::StatsEntry;

use super::{path_kind_score, review_priority_score, DataCandidate, MockDataReport};

fn is_text_like_file(file_path: &str, file_type: &str) -> bool {
    let normalized = if file_type.is_empty() {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_lowercase()
    } else {
        file_type.to_lowercase()
    };
    matches!(
        normalized.as_str(),
        "rs" | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "md"
            | "txt"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "rb"
            | "php"
            | "sh"
            | "env"
            | "ini"
            | "cfg"
            | "conf"
            | "sql"
    )
}

fn read_file_sample(root: &Path, relative_path: &str) -> Option<String> {
    let full_path = root.join(relative_path);
    let metadata = fs::metadata(&full_path).ok()?;
    if metadata.len() > 128 * 1024 {
        return None;
    }
    let bytes = fs::read(full_path).ok()?;
    let sample_len = bytes.len().min(16 * 1024);
    String::from_utf8(bytes[..sample_len].to_vec()).ok()
}

fn count_keyword_hits(haystack: &str, keywords: &[&str]) -> usize {
    keywords.iter().filter(|kw| haystack.contains(**kw)).count()
}

fn path_is_test_only(path_lower: &str) -> bool {
    [
        "tests/",
        "test/",
        "__tests__/",
        "spec/",
        "specs/",
        "fixtures/",
        "__fixtures__/",
        "testdata/",
        "examples/",
        "example/",
    ]
    .iter()
    .any(|token| path_lower.contains(token))
}

fn path_is_runtime_shared(path_lower: &str) -> bool {
    ["src/", "app/", "config/", "internal/", "lib/", "server/"]
        .iter()
        .any(|token| path_lower.contains(token))
}

fn path_is_generated_artifact(path_lower: &str) -> bool {
    [
        "target/",
        "node_modules/",
        "dist/",
        "build/",
        ".next/",
        "coverage/",
        ".turbo/",
    ]
    .iter()
    .any(|token| path_lower.contains(token))
}

fn classify_path_kind(path_lower: &str) -> &'static str {
    if path_is_generated_artifact(path_lower) {
        "generated_artifact"
    } else if path_is_test_only(path_lower) {
        "test_only"
    } else if path_is_runtime_shared(path_lower) {
        "runtime_shared"
    } else {
        "unknown"
    }
}

fn matched_keywords(haystack: &str, keywords: &[&str], limit: usize) -> Vec<String> {
    keywords
        .iter()
        .filter(|kw| haystack.contains(**kw))
        .take(limit)
        .map(|kw| (*kw).to_string())
        .collect()
}

fn content_preview_snippet(content: &str, keywords: &[String]) -> Option<String> {
    let lower = content.to_lowercase();
    for keyword in keywords {
        if let Some(index) = lower.find(keyword) {
            let start = index.saturating_sub(24);
            let end = (index + keyword.len() + 40).min(content.len());
            let snippet = content[start..end].replace(['\n', '\r'], " ");
            return Some(snippet);
        }
    }
    None
}

pub(crate) fn detect_mock_data_report(root: &Path, entries: &[StatsEntry]) -> MockDataReport {
    let strong_mock_path_tokens = [
        "mock",
        "mocks",
        "fixture",
        "fixtures",
        "stub",
        "stubs",
        "fake",
        "fakes",
        "testdata",
        "__fixtures__",
    ];
    let weak_mock_path_tokens = ["sample", "samples", "demo", "seed", "seeds"];
    let mock_content_tokens = [
        "mock",
        "fixture",
        "stub",
        "fake",
        "sample data",
        "demo data",
        "seed data",
    ];
    let business_keywords = [
        "customer", "client", "tenant", "account", "order", "invoice", "payment", "amount",
        "price", "address", "phone", "email", "user", "member", "sku",
    ];
    let literal_markers = [
        "@",
        "street",
        "road",
        "avenue",
        "city",
        "postal",
        "zip",
        "usd",
        "cny",
        "$",
        "customer_id",
        "tenant_id",
        "invoice_no",
        "phone",
    ];
    let mut report = MockDataReport::default();

    for entry in entries.iter().take(200) {
        let path_lower = entry.file_path.to_lowercase();
        let is_test_only = path_is_test_only(&path_lower);
        let path_classification = classify_path_kind(&path_lower);
        let content = if is_text_like_file(&entry.file_path, &entry.file_type) {
            read_file_sample(root, &entry.file_path)
        } else {
            None
        };
        let content_lower = content
            .as_ref()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let mut mock_reasons = Vec::new();
        let mut mock_evidence = Vec::new();
        let mut mock_rule_hits = Vec::new();
        let content_mock_keywords = matched_keywords(&content_lower, &mock_content_tokens, 4);
        let has_content_mock_signal =
            !content_lower.is_empty() && !content_mock_keywords.is_empty();
        let strong_path_mock_keywords = matched_keywords(&path_lower, &strong_mock_path_tokens, 4);
        let weak_path_mock_keywords = matched_keywords(&path_lower, &weak_mock_path_tokens, 4);
        let has_strong_path_mock_signal = !strong_path_mock_keywords.is_empty();
        let has_weak_path_mock_signal = !weak_path_mock_keywords.is_empty();

        let allow_weak_path_mock_signal = match path_classification {
            "test_only" | "generated_artifact" => true,
            "runtime_shared" | "unknown" => has_content_mock_signal,
            _ => false,
        };

        let mut mock_keywords = strong_path_mock_keywords;
        mock_keywords.extend(weak_path_mock_keywords.clone());

        if has_strong_path_mock_signal || (has_weak_path_mock_signal && allow_weak_path_mock_signal)
        {
            mock_reasons
                .push("Path contains explicit mock/fixture/demo/test-data markers.".to_string());
            mock_evidence.push(entry.file_path.clone());
            mock_rule_hits.push("path.mock_token".to_string());
        }
        if has_content_mock_signal {
            mock_reasons
                .push("File content mentions mock/fixture/fake/sample data tokens.".to_string());
            mock_evidence.push(format!(
                "content token hit: {}",
                content_mock_keywords.join(", ")
            ));
            mock_rule_hits.push("content.mock_token".to_string());
        }
        mock_keywords.extend(content_mock_keywords);
        mock_keywords.sort();
        mock_keywords.dedup();
        if is_test_only && !mock_reasons.is_empty() {
            mock_reasons.push("File is under a test/example/fixture-oriented path.".to_string());
            mock_rule_hits.push("path.test_only".to_string());
        }
        if path_classification == "generated_artifact" && !mock_reasons.is_empty() {
            mock_reasons.push(
                "Candidate is inside a generated-artifact directory, so treat it as lower-confidence review context."
                    .to_string(),
            );
            mock_rule_hits.push("path.generated_artifact".to_string());
        }

        let business_hits = count_keyword_hits(&content_lower, &business_keywords);
        let literal_hits = count_keyword_hits(&content_lower, &literal_markers);
        let runtime_path = path_is_runtime_shared(&path_lower);
        let mut hardcoded_reasons = Vec::new();
        let mut hardcoded_evidence = Vec::new();
        let mut hardcoded_rule_hits = Vec::new();
        let business_matches = matched_keywords(&content_lower, &business_keywords, 5);
        let literal_matches = matched_keywords(&content_lower, &literal_markers, 5);
        let mut hardcoded_keywords = business_matches.clone();
        hardcoded_keywords.extend(literal_matches.clone());
        hardcoded_keywords.sort();
        hardcoded_keywords.dedup();
        if !is_test_only
            && path_classification != "generated_artifact"
            && business_hits >= 2
            && literal_hits >= 1
        {
            hardcoded_reasons.push(
                "File contains business-like data keywords together with literal value markers."
                    .to_string(),
            );
            hardcoded_evidence.push(format!(
                "business_keyword_hits={}, literal_marker_hits={}",
                business_hits, literal_hits
            ));
            hardcoded_rule_hits.push("content.business_literal_combo".to_string());
            if !business_matches.is_empty() || !literal_matches.is_empty() {
                hardcoded_evidence.push(format!(
                    "matched terms: business=[{}], literal=[{}]",
                    business_matches.join(", "),
                    literal_matches.join(", ")
                ));
            }
        }
        if !is_test_only && runtime_path && business_hits >= 2 {
            hardcoded_reasons.push(
                "Candidate appears in a shared runtime path rather than a test-only path."
                    .to_string(),
            );
            hardcoded_evidence.push("runtime/shared path".to_string());
            hardcoded_rule_hits.push("path.runtime_shared".to_string());
        }
        if !hardcoded_keywords.is_empty() {
            if let Some(snippet) =
                content_preview_snippet(&content.unwrap_or_default(), &hardcoded_keywords)
            {
                hardcoded_evidence.push(format!("content preview: {}", snippet));
            }
        }

        if !mock_reasons.is_empty() {
            report.mock_candidates.push(DataCandidate {
                file_path: entry.file_path.clone(),
                confidence: if is_test_only {
                    "high"
                } else if path_classification == "generated_artifact" {
                    "low"
                } else {
                    "medium"
                },
                review_priority: if is_test_only {
                    "medium"
                } else if path_classification == "generated_artifact" {
                    "low"
                } else {
                    "high"
                },
                path_classification,
                rule_hits: mock_rule_hits,
                matched_keywords: mock_keywords,
                reasons: mock_reasons,
                evidence: mock_evidence,
                access_count: entry.access_count,
                file_type: entry.file_type.clone(),
            });
        }
        if !hardcoded_reasons.is_empty() {
            report.hardcoded_candidates.push(DataCandidate {
                file_path: entry.file_path.clone(),
                confidence: if runtime_path { "high" } else { "medium" },
                review_priority: if runtime_path { "high" } else { "medium" },
                path_classification,
                rule_hits: hardcoded_rule_hits,
                matched_keywords: hardcoded_keywords,
                reasons: hardcoded_reasons,
                evidence: hardcoded_evidence,
                access_count: entry.access_count,
                file_type: entry.file_type.clone(),
            });
        }
        if !report.mock_candidates.is_empty()
            && !report.hardcoded_candidates.is_empty()
            && report
                .mock_candidates
                .iter()
                .any(|candidate| candidate.file_path == entry.file_path)
            && report
                .hardcoded_candidates
                .iter()
                .any(|candidate| candidate.file_path == entry.file_path)
        {
            report.mixed_review_files.push(entry.file_path.clone());
        }
    }

    report.mock_candidates.sort_by(|a, b| {
        review_priority_score(b.review_priority)
            .cmp(&review_priority_score(a.review_priority))
            .then_with(|| b.access_count.cmp(&a.access_count))
            .then_with(|| a.file_path.cmp(&b.file_path))
    });
    report.hardcoded_candidates.sort_by(|a, b| {
        review_priority_score(b.review_priority)
            .cmp(&review_priority_score(a.review_priority))
            .then_with(|| {
                path_kind_score(b.path_classification).cmp(&path_kind_score(a.path_classification))
            })
            .then_with(|| b.access_count.cmp(&a.access_count))
            .then_with(|| a.file_path.cmp(&b.file_path))
    });
    report.mixed_review_files.sort();
    report.mixed_review_files.dedup();

    report
}
