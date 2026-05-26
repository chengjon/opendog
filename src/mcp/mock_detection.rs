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

fn path_is_documentation(path_lower: &str) -> bool {
    [
        "docs/",
        "doc/",
        "documentation/",
        "operations/",
        "runbooks/",
        "readme",
        "changelog",
    ]
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

fn path_is_infrastructure(path_lower: &str) -> bool {
    let infra_dirs = [".claude/", ".cursor/", ".agents/", ".amazonq/", ".zread/", ".vscode/", ".idea/"];
    infra_dirs.iter().any(|dir| path_lower.contains(dir))
}

fn classify_path_kind(path_lower: &str) -> &'static str {
    if path_is_infrastructure(path_lower) {
        "infrastructure"
    } else if path_is_generated_artifact(path_lower) {
        "generated_artifact"
    } else if path_is_test_only(path_lower) {
        "test_only"
    } else if path_is_runtime_shared(path_lower) {
        "runtime_shared"
    } else if path_is_documentation(path_lower) {
        "documentation"
    } else {
        "unknown"
    }
}

fn content_has_template_placeholder(content_lower: &str) -> bool {
    content_lower.contains("${")
        || content_lower.contains("{{")
        || content_lower.contains("<your_")
        || content_lower.contains("<insert_")
        || content_lower.contains("example.com")
}

fn is_strong_hardcoded_combo(
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

fn allow_runtime_shared_hardcoded_amplification(
    path_classification: &str,
    combo_is_strong: bool,
) -> bool {
    path_classification == "runtime_shared" && combo_is_strong
}

fn hardcoded_review_priority(
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

fn hardcoded_confidence(path_classification: &str, has_template_placeholder: bool) -> &'static str {
    if path_classification == "runtime_shared" && !has_template_placeholder {
        "high"
    } else if path_classification == "documentation" || has_template_placeholder {
        "low"
    } else {
        "medium"
    }
}

fn discounted_weak_literal_hits(raw_weak_hits: usize) -> usize {
    raw_weak_hits / 2
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
            let start = previous_char_boundary(content, index.saturating_sub(24));
            let end = next_char_boundary(content, (index + keyword.len() + 40).min(content.len()));
            let snippet = content[start..end].replace(['\n', '\r'], " ");
            return Some(snippet);
        }
    }
    None
}

fn previous_char_boundary(value: &str, mut index: usize) -> usize {
    index = index.min(value.len());
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn next_char_boundary(value: &str, mut index: usize) -> usize {
    index = index.min(value.len());
    while index < value.len() && !value.is_char_boundary(index) {
        index += 1;
    }
    index
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
    let strong_literal_markers = [
        "@",
        "street",
        "road",
        "avenue",
        "$",
        "customer_id",
        "tenant_id",
        "invoice_no",
    ];
    let weak_literal_markers = ["city", "postal", "zip", "usd", "cny", "phone"];
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
            "runtime_shared" | "documentation" | "unknown" => has_content_mock_signal,
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
        let strong_literal_hits = count_keyword_hits(&content_lower, &strong_literal_markers);
        let weak_literal_hits = count_keyword_hits(&content_lower, &weak_literal_markers);
        let literal_hits = strong_literal_hits + discounted_weak_literal_hits(weak_literal_hits);
        let strong_hardcoded_combo =
            is_strong_hardcoded_combo(path_classification, business_hits, literal_hits);
        let has_template_placeholder = content_has_template_placeholder(&content_lower);
        let mut hardcoded_reasons = Vec::new();
        let mut hardcoded_evidence = Vec::new();
        let mut hardcoded_rule_hits = Vec::new();
        let business_matches = matched_keywords(&content_lower, &business_keywords, 5);
        let mut literal_matches = matched_keywords(&content_lower, &strong_literal_markers, 5);
        literal_matches.extend(matched_keywords(&content_lower, &weak_literal_markers, 5));
        literal_matches.sort();
        literal_matches.dedup();
        let mut hardcoded_keywords = business_matches.clone();
        hardcoded_keywords.extend(literal_matches.clone());
        hardcoded_keywords.sort();
        hardcoded_keywords.dedup();
        if strong_hardcoded_combo {
            hardcoded_reasons.push(
                "File contains business-like data keywords together with literal value markers."
                    .to_string(),
            );
            hardcoded_evidence.push(format!(
                "business_keyword_hits={}, literal_marker_hits={} (strong={}, weak_raw={})",
                business_hits, literal_hits, strong_literal_hits, weak_literal_hits
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
        if allow_runtime_shared_hardcoded_amplification(path_classification, strong_hardcoded_combo)
        {
            hardcoded_reasons.push(
                "Candidate appears in a shared runtime path rather than a test-only path."
                    .to_string(),
            );
            hardcoded_evidence.push("runtime/shared path".to_string());
            hardcoded_rule_hits.push("path.runtime_shared".to_string());
        }
        if path_classification == "documentation" && !hardcoded_reasons.is_empty() {
            hardcoded_reasons.push(
                "Candidate appears in documentation or operator notes, so treat literal-looking examples as lower-priority context."
                    .to_string(),
            );
            hardcoded_evidence.push("documentation/operator-note path".to_string());
            hardcoded_rule_hits.push("path.documentation".to_string());
        }
        if has_template_placeholder && !hardcoded_reasons.is_empty() {
            hardcoded_reasons.push(
                "Content includes template placeholders, so apparent values may be examples rather than runtime data."
                    .to_string(),
            );
            hardcoded_evidence.push("template placeholder pattern".to_string());
            hardcoded_rule_hits.push("content.template_placeholder".to_string());
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
                } else if path_classification == "generated_artifact"
                    || path_classification == "infrastructure"
                {
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
                confidence: hardcoded_confidence(path_classification, has_template_placeholder),
                review_priority: hardcoded_review_priority(
                    path_classification,
                    has_template_placeholder,
                ),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- is_text_like_file ----

    #[test]
    fn test_is_text_like_file_common_source_extensions() {
        assert!(is_text_like_file("main.rs", ""));
        assert!(is_text_like_file("app.py", ""));
        assert!(is_text_like_file("index.js", ""));
        assert!(is_text_like_file("component.tsx", ""));
        assert!(is_text_like_file("app.go", ""));
        assert!(is_text_like_file("main.java", ""));
        assert!(is_text_like_file("main.kt", ""));
        assert!(is_text_like_file("main.swift", ""));
        assert!(is_text_like_file("main.rb", ""));
        assert!(is_text_like_file("main.php", ""));
        assert!(is_text_like_file("main.c", ""));
        assert!(is_text_like_file("main.cc", ""));
        assert!(is_text_like_file("main.cpp", ""));
        assert!(is_text_like_file("main.h", ""));
        assert!(is_text_like_file("main.hpp", ""));
    }

    #[test]
    fn test_is_text_like_file_config_and_data() {
        assert!(is_text_like_file("Cargo.toml", ""));
        assert!(is_text_like_file("package.json", ""));
        assert!(is_text_like_file("config.yaml", ""));
        assert!(is_text_like_file("config.yml", ""));
        assert!(is_text_like_file("notes.txt", ""));
        assert!(is_text_like_file("README.md", ""));
        assert!(is_text_like_file("setup.sh", ""));
        assert!(is_text_like_file(".env", "env"));
        assert!(is_text_like_file("app.ini", ""));
        assert!(is_text_like_file("app.cfg", ""));
        assert!(is_text_like_file("app.conf", ""));
        assert!(is_text_like_file("query.sql", ""));
        assert!(is_text_like_file("app.jsx", ""));
    }

    #[test]
    fn test_is_text_like_file_non_text() {
        assert!(!is_text_like_file("image.png", ""));
        assert!(!is_text_like_file("image.jpg", ""));
        assert!(!is_text_like_file("image.jpeg", ""));
        assert!(!is_text_like_file("binary.exe", ""));
        assert!(!is_text_like_file("archive.zip", ""));
        assert!(!is_text_like_file("data.bin", ""));
        assert!(!is_text_like_file("font.woff", ""));
        assert!(!is_text_like_file("video.mp4", ""));
    }

    #[test]
    fn test_is_text_like_file_uses_file_type_when_provided() {
        // When file_type is non-empty, it takes precedence over the extension
        assert!(is_text_like_file("data.bin", "py"));
        assert!(!is_text_like_file("app.py", "bin"));
    }

    #[test]
    fn test_is_text_like_file_case_insensitive_file_type() {
        assert!(is_text_like_file("script", "PY"));
        assert!(is_text_like_file("script", "Rs"));
        assert!(is_text_like_file("script", "JSON"));
    }

    #[test]
    fn test_is_text_like_file_no_extension_empty_file_type() {
        assert!(!is_text_like_file("Makefile", ""));
        assert!(!is_text_like_file("Dockerfile", ""));
    }

    // ---- count_keyword_hits ----

    #[test]
    fn test_count_keyword_hits_multiple() {
        let hits = count_keyword_hits("customer invoice payment", &["customer", "invoice", "payment", "missing"]);
        assert_eq!(hits, 3);
    }

    #[test]
    fn test_count_keyword_hits_single() {
        let hits = count_keyword_hits("only one match here", &["match", "missing", "absent"]);
        assert_eq!(hits, 1);
    }

    #[test]
    fn test_count_keyword_hits_zero() {
        let hits = count_keyword_hits("nothing relevant", &["customer", "invoice", "payment"]);
        assert_eq!(hits, 0);
    }

    #[test]
    fn test_count_keyword_hits_empty_haystack() {
        let hits = count_keyword_hits("", &["customer", "invoice"]);
        assert_eq!(hits, 0);
    }

    #[test]
    fn test_count_keyword_hits_empty_keywords() {
        let hits = count_keyword_hits("customer invoice", &[]);
        assert_eq!(hits, 0);
    }

    // ---- path_is_test_only ----

    #[test]
    fn test_path_is_test_only_various_prefixes() {
        assert!(path_is_test_only("tests/unit/test_foo.rs"));
        assert!(path_is_test_only("test/bar.py"));
        assert!(path_is_test_only("__tests__/component.test.js"));
        assert!(path_is_test_only("spec/models/user_spec.rb"));
        assert!(path_is_test_only("specs/example.spec.ts"));
        assert!(path_is_test_only("fixtures/data.json"));
        assert!(path_is_test_only("__fixtures__/sample.json"));
        assert!(path_is_test_only("testdata/input.csv"));
        assert!(path_is_test_only("examples/demo.py"));
        assert!(path_is_test_only("example/sample.txt"));
    }

    #[test]
    fn test_path_is_test_only_negative() {
        assert!(!path_is_test_only("src/main.rs"));
        assert!(!path_is_test_only("lib/customer.py"));
        assert!(!path_is_test_only("config/app.yaml"));
        assert!(!path_is_test_only("README.md"));
    }

    // ---- path_is_runtime_shared ----

    #[test]
    fn test_path_is_runtime_shared_various() {
        assert!(path_is_runtime_shared("src/main.rs"));
        assert!(path_is_runtime_shared("app/controllers/user.py"));
        assert!(path_is_runtime_shared("config/settings.yaml"));
        assert!(path_is_runtime_shared("internal/service.go"));
        assert!(path_is_runtime_shared("lib/utils.py"));
        assert!(path_is_runtime_shared("server/handler.rs"));
    }

    #[test]
    fn test_path_is_runtime_shared_negative() {
        assert!(!path_is_runtime_shared("tests/test_main.rs"));
        assert!(!path_is_runtime_shared("docs/guide.md"));
        assert!(!path_is_runtime_shared("dist/bundle.js"));
        assert!(!path_is_runtime_shared("README.md"));
    }

    // ---- path_is_documentation ----

    #[test]
    fn test_path_is_documentation_various() {
        // Function expects already-lowercased input
        assert!(path_is_documentation("docs/api.md"));
        assert!(path_is_documentation("doc/guide.txt"));
        assert!(path_is_documentation("documentation/setup.md"));
        assert!(path_is_documentation("operations/runbook.md"));
        assert!(path_is_documentation("runbooks/incident.md"));
        assert!(path_is_documentation("readme.txt"));
        assert!(path_is_documentation("readme.md"));
        assert!(path_is_documentation("changelog.md"));
        assert!(path_is_documentation("changelog.txt"));
    }

    #[test]
    fn test_path_is_documentation_negative() {
        assert!(!path_is_documentation("src/main.rs"));
        assert!(!path_is_documentation("tests/test_foo.rs"));
        assert!(!path_is_documentation("config/app.yaml"));
    }

    // ---- path_is_generated_artifact ----

    #[test]
    fn test_path_is_generated_artifact_various() {
        assert!(path_is_generated_artifact("target/debug/opendog"));
        assert!(path_is_generated_artifact("node_modules/lodash/index.js"));
        assert!(path_is_generated_artifact("dist/bundle.js"));
        assert!(path_is_generated_artifact("build/output.o"));
        assert!(path_is_generated_artifact(".next/static/chunk.js"));
        assert!(path_is_generated_artifact("coverage/lcov.info"));
        assert!(path_is_generated_artifact(".turbo/cache.dat"));
    }

    #[test]
    fn test_path_is_generated_artifact_negative() {
        assert!(!path_is_generated_artifact("src/main.rs"));
        assert!(!path_is_generated_artifact("tests/test_foo.rs"));
        assert!(!path_is_generated_artifact("docs/api.md"));
    }

    // ---- classify_path_kind ----

    #[test]
    fn test_classify_path_kind_generated_artifact_takes_precedence() {
        assert_eq!(classify_path_kind("target/debug/test_main.rs"), "generated_artifact");
        assert_eq!(classify_path_kind("dist/tests/test_bundle.js"), "generated_artifact");
    }

    #[test]
    fn test_classify_path_kind_test_only() {
        assert_eq!(classify_path_kind("tests/unit/test_foo.rs"), "test_only");
        assert_eq!(classify_path_kind("spec/models/user_spec.rb"), "test_only");
    }

    #[test]
    fn test_classify_path_kind_runtime_shared() {
        assert_eq!(classify_path_kind("src/main.rs"), "runtime_shared");
        assert_eq!(classify_path_kind("app/handlers/user.py"), "runtime_shared");
        assert_eq!(classify_path_kind("lib/utils.js"), "runtime_shared");
    }

    #[test]
    fn test_classify_path_kind_documentation() {
        assert_eq!(classify_path_kind("docs/api.md"), "documentation");
        assert_eq!(classify_path_kind("readme.md"), "documentation");
    }

    #[test]
    fn test_classify_path_kind_unknown() {
        assert_eq!(classify_path_kind("Cargo.toml"), "unknown");
        assert_eq!(classify_path_kind("Makefile"), "unknown");
        assert_eq!(classify_path_kind("scripts/setup.sh"), "unknown");
    }

    #[test]
    fn test_classify_path_kind_precedence_generated_over_test() {
        // "build/" contains generated_artifact, "test/" contains test_only
        // generated_artifact has highest precedence
        assert_eq!(classify_path_kind("build/test/output"), "generated_artifact");
    }

    #[test]
    fn test_classify_path_kind_precedence_test_over_runtime() {
        // "tests/" is test_only, but not runtime_shared (no src/app/etc.)
        assert_eq!(classify_path_kind("tests/src/test_foo.rs"), "test_only");
    }

    // ---- content_has_template_placeholder ----

    #[test]
    fn test_content_has_template_placeholder_dollar_brace() {
        assert!(content_has_template_placeholder("value is ${name}"));
    }

    #[test]
    fn test_content_has_template_placeholder_mustache() {
        assert!(content_has_template_placeholder("hello {{name}}"));
    }

    #[test]
    fn test_content_has_template_placeholder_angle_your() {
        assert!(content_has_template_placeholder("enter <your_api_key> here"));
    }

    #[test]
    fn test_content_has_template_placeholder_angle_insert() {
        assert!(content_has_template_placeholder("fill in <insert_token>"));
    }

    #[test]
    fn test_content_has_template_placeholder_example_dot_com() {
        assert!(content_has_template_placeholder("email: user@example.com"));
    }

    #[test]
    fn test_content_has_template_placeholder_no_match() {
        assert!(!content_has_template_placeholder("normal text without placeholders"));
        assert!(!content_has_template_placeholder(""));
    }

    // ---- is_strong_hardcoded_combo ----

    #[test]
    fn test_is_strong_hardcoded_combo_runtime_shared_meets_threshold() {
        assert!(is_strong_hardcoded_combo("runtime_shared", 2, 2));
        assert!(is_strong_hardcoded_combo("runtime_shared", 5, 3));
    }

    #[test]
    fn test_is_strong_hardcoded_combo_runtime_shared_below_threshold() {
        assert!(!is_strong_hardcoded_combo("runtime_shared", 1, 2));
        assert!(!is_strong_hardcoded_combo("runtime_shared", 2, 1));
        assert!(!is_strong_hardcoded_combo("runtime_shared", 0, 0));
    }

    #[test]
    fn test_is_strong_hardcoded_combo_test_only_always_false() {
        assert!(!is_strong_hardcoded_combo("test_only", 10, 10));
    }

    #[test]
    fn test_is_strong_hardcoded_combo_generated_artifact_always_false() {
        assert!(!is_strong_hardcoded_combo("generated_artifact", 10, 10));
    }

    #[test]
    fn test_is_strong_hardcoded_combo_other_classification_higher_threshold() {
        // "unknown" or "documentation" requires business_hits >= 3
        assert!(is_strong_hardcoded_combo("unknown", 3, 2));
        assert!(is_strong_hardcoded_combo("documentation", 3, 2));
        assert!(!is_strong_hardcoded_combo("unknown", 2, 2));
        assert!(!is_strong_hardcoded_combo("documentation", 2, 2));
    }

    // ---- hardcoded_review_priority ----

    #[test]
    fn test_hardcoded_review_priority_runtime_shared_no_template() {
        assert_eq!(hardcoded_review_priority("runtime_shared", false), "high");
    }

    #[test]
    fn test_hardcoded_review_priority_runtime_shared_with_template() {
        // runtime_shared with template -> falls through to has_template_placeholder check -> "low"
        assert_eq!(hardcoded_review_priority("runtime_shared", true), "low");
    }

    #[test]
    fn test_hardcoded_review_priority_documentation() {
        assert_eq!(hardcoded_review_priority("documentation", false), "low");
        assert_eq!(hardcoded_review_priority("documentation", true), "low");
    }

    #[test]
    fn test_hardcoded_review_priority_has_template_placeholder() {
        assert_eq!(hardcoded_review_priority("unknown", true), "low");
    }

    #[test]
    fn test_hardcoded_review_priority_default_medium() {
        assert_eq!(hardcoded_review_priority("unknown", false), "medium");
        assert_eq!(hardcoded_review_priority("test_only", false), "medium");
    }

    // ---- hardcoded_confidence ----

    #[test]
    fn test_hardcoded_confidence_runtime_shared_no_template() {
        assert_eq!(hardcoded_confidence("runtime_shared", false), "high");
    }

    #[test]
    fn test_hardcoded_confidence_runtime_shared_with_template() {
        // runtime_shared with template -> falls through to has_template_placeholder check -> "low"
        assert_eq!(hardcoded_confidence("runtime_shared", true), "low");
    }

    #[test]
    fn test_hardcoded_confidence_documentation() {
        assert_eq!(hardcoded_confidence("documentation", false), "low");
        assert_eq!(hardcoded_confidence("documentation", true), "low");
    }

    #[test]
    fn test_hardcoded_confidence_has_template_placeholder() {
        assert_eq!(hardcoded_confidence("unknown", true), "low");
    }

    #[test]
    fn test_hardcoded_confidence_default_medium() {
        assert_eq!(hardcoded_confidence("unknown", false), "medium");
        assert_eq!(hardcoded_confidence("test_only", false), "medium");
    }

    // ---- matched_keywords ----

    #[test]
    fn test_matched_keywords_basic() {
        let result = matched_keywords("mock fixture stub", &["mock", "fixture", "stub", "fake"], 10);
        assert_eq!(result, vec!["mock", "fixture", "stub"]);
    }

    #[test]
    fn test_matched_keywords_respects_limit() {
        let result = matched_keywords("mock fixture stub fake", &["mock", "fixture", "stub", "fake"], 2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "mock");
        assert_eq!(result[1], "fixture");
    }

    #[test]
    fn test_matched_keywords_no_matches() {
        let result = matched_keywords("nothing here", &["mock", "fixture"], 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_matched_keywords_empty_haystack() {
        let result = matched_keywords("", &["mock", "fixture"], 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_matched_keywords_empty_keywords() {
        let result = matched_keywords("mock fixture", &[], 5);
        assert!(result.is_empty());
    }

    // ---- previous_char_boundary ----

    #[test]
    fn test_previous_char_boundary_ascii_within_bounds() {
        assert_eq!(previous_char_boundary("hello world", 3), 3);
    }

    #[test]
    fn test_previous_char_boundary_at_start() {
        assert_eq!(previous_char_boundary("hello", 0), 0);
    }

    #[test]
    fn test_previous_char_boundary_exceeds_length() {
        assert_eq!(previous_char_boundary("hello", 100), 5);
    }

    #[test]
    fn test_previous_char_boundary_zero() {
        assert_eq!(previous_char_boundary("hello", 0), 0);
    }

    #[test]
    fn test_previous_char_boundary_finds_boundary_in_multibyte() {
        // "cafe\u{301}" = "caf\u{e9}" = "cafe" with combining accent
        // Actually let's use a simpler multibyte case
        let s = "héllo"; // 'é' is 2 bytes in UTF-8
        // 'h' = 0..1, 'é' = 1..3, 'l' = 3..4, 'l' = 4..5, 'o' = 5..6
        // index 2 is mid-character in 'é'
        assert_eq!(previous_char_boundary(s, 2), 1);
        // index 1 is a valid boundary
        assert_eq!(previous_char_boundary(s, 1), 1);
        // index 3 is valid
        assert_eq!(previous_char_boundary(s, 3), 3);
    }

    #[test]
    fn test_previous_char_boundary_empty_string() {
        assert_eq!(previous_char_boundary("", 0), 0);
        assert_eq!(previous_char_boundary("", 5), 0);
    }

    // ---- next_char_boundary ----

    #[test]
    fn test_next_char_boundary_ascii_within_bounds() {
        assert_eq!(next_char_boundary("hello world", 3), 3);
    }

    #[test]
    fn test_next_char_boundary_at_end() {
        assert_eq!(next_char_boundary("hello", 5), 5);
    }

    #[test]
    fn test_next_char_boundary_exceeds_length() {
        assert_eq!(next_char_boundary("hello", 100), 5);
    }

    #[test]
    fn test_next_char_boundary_finds_boundary_in_multibyte() {
        let s = "héllo"; // 'é' is 2 bytes: positions 1..3
        // index 2 is mid-character, should advance to 3
        assert_eq!(next_char_boundary(s, 2), 3);
        // index 1 is a valid boundary
        assert_eq!(next_char_boundary(s, 1), 1);
    }

    #[test]
    fn test_next_char_boundary_empty_string() {
        assert_eq!(next_char_boundary("", 0), 0);
        assert_eq!(next_char_boundary("", 5), 0);
    }

    // ---- discounted_weak_literal_hits ----

    #[test]
    fn test_discounted_weak_literal_hits_even() {
        assert_eq!(discounted_weak_literal_hits(4), 2);
        assert_eq!(discounted_weak_literal_hits(0), 0);
        assert_eq!(discounted_weak_literal_hits(2), 1);
    }

    #[test]
    fn test_discounted_weak_literal_hits_odd_truncates() {
        assert_eq!(discounted_weak_literal_hits(5), 2);
        assert_eq!(discounted_weak_literal_hits(1), 0);
        assert_eq!(discounted_weak_literal_hits(3), 1);
    }

    #[test]
    fn test_discounted_weak_literal_hits_large() {
        assert_eq!(discounted_weak_literal_hits(100), 50);
    }

    // ---- path_is_infrastructure / infrastructure classification ----

    #[test]
    fn classify_path_kind_infrastructure_claude_paths() {
        assert_eq!(classify_path_kind(".claude/settings.json"), "infrastructure");
        assert_eq!(classify_path_kind(".claude/build-checker.json"), "infrastructure");
        assert_eq!(classify_path_kind(".claude/skills/playwright/references/guide.md"), "infrastructure");
        assert_eq!(classify_path_kind(".cursor/rules/project.mdc"), "infrastructure");
        assert_eq!(classify_path_kind(".agents/prompts/review.md"), "infrastructure");
    }

    #[test]
    fn infrastructure_paths_are_not_unknown() {
        assert_ne!(classify_path_kind(".claude/settings.json"), "unknown");
    }
}
