use super::ProjectConfig;

pub fn matches_ignore_pattern(rel_path: &str, pattern: &str) -> bool {
    rel_path == pattern
        || rel_path.starts_with(&format!("{}/", pattern))
        || rel_path.contains(&format!("/{}/", pattern))
        || rel_path.ends_with(&format!("/{}", pattern))
        || rel_path
            .split('/')
            .any(|segment| wildcard_matches(pattern, segment))
}

pub fn should_ignore_path(rel_path: &str, config: &ProjectConfig) -> bool {
    let normalized = rel_path.replace('\\', "/");
    config
        .ignore_patterns
        .iter()
        .any(|pattern| matches_ignore_pattern(&normalized, pattern))
}

fn wildcard_matches(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');
    let mut remainder = text;

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if index == 0 && anchored_start {
            let Some(stripped) = remainder.strip_prefix(part) else {
                return false;
            };
            remainder = stripped;
            continue;
        }

        if index == parts.len() - 1 && anchored_end {
            return remainder.ends_with(part);
        }

        let Some(found_at) = remainder.find(part) else {
            return false;
        };
        remainder = &remainder[found_at + part.len()..];
    }

    !anchored_end || parts.last().is_none_or(|part| remainder.ends_with(part))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- wildcard_matches ---

    #[test]
    fn wildcard_exact_match_no_wildcard() {
        assert!(super::wildcard_matches("hello", "hello"));
    }

    #[test]
    fn wildcard_no_match_no_wildcard() {
        assert!(!super::wildcard_matches("hello", "world"));
    }

    #[test]
    fn wildcard_star_prefix() {
        assert!(super::wildcard_matches("*.pyc", "test.pyc"));
        assert!(!super::wildcard_matches("*.pyc", "test.py"));
    }

    #[test]
    fn wildcard_star_suffix() {
        assert!(super::wildcard_matches("test*", "test.rs"));
        assert!(!super::wildcard_matches("test*", "prod.rs"));
    }

    #[test]
    fn wildcard_star_both_ends() {
        assert!(super::wildcard_matches("*cache*", "my_cache_dir"));
        assert!(!super::wildcard_matches("*cache*", "my_dir"));
    }

    #[test]
    fn wildcard_star_middle() {
        assert!(super::wildcard_matches("node*modules", "node_modules"));
        assert!(super::wildcard_matches("a*b", "axxb"));
        // * can match empty string, so "a*b" matches "ab"
        assert!(super::wildcard_matches("a*b", "ab"));
        assert!(!super::wildcard_matches("a*b", "xb"));
    }

    #[test]
    fn wildcard_question_mark_is_not_special() {
        // The current implementation only handles *, not ?
        // So ? is treated literally
        assert!(!super::wildcard_matches("file?.rs", "file1.rs"));
        assert!(super::wildcard_matches("file?.rs", "file?.rs"));
    }

    #[test]
    fn wildcard_empty_pattern_and_text() {
        assert!(super::wildcard_matches("", ""));
        assert!(!super::wildcard_matches("", "nonempty"));
        assert!(!super::wildcard_matches("nonempty", ""));
    }

    #[test]
    fn wildcard_star_only_matches_everything() {
        assert!(super::wildcard_matches("*", ""));
        assert!(super::wildcard_matches("*", "anything"));
    }

    // --- matches_ignore_pattern ---

    #[test]
    fn matches_exact_path() {
        assert!(matches_ignore_pattern("target", "target"));
        assert!(!matches_ignore_pattern("target", "dist"));
    }

    #[test]
    fn matches_path_prefix() {
        assert!(matches_ignore_pattern("target/debug/app", "target"));
    }

    #[test]
    fn matches_path_middle_segment() {
        assert!(matches_ignore_pattern("src/node_modules/pkg", "node_modules"));
    }

    #[test]
    fn matches_path_suffix_segment() {
        assert!(matches_ignore_pattern("src/main/node_modules", "node_modules"));
    }

    #[test]
    fn matches_wildcard_pattern_against_segment() {
        assert!(matches_ignore_pattern("build/main.pyc", "*.pyc"));
        assert!(!matches_ignore_pattern("src/main.rs", "*.pyc"));
    }

    #[test]
    fn no_match_on_partial_segment() {
        assert!(!matches_ignore_pattern("src/my_target_file.rs", "target"));
    }

    // --- should_ignore_path ---

    #[test]
    fn should_ignore_with_configured_patterns() {
        let config = ProjectConfig {
            ignore_patterns: vec!["node_modules".to_string(), "*.pyc".to_string()],
            process_whitelist: vec![],
        };
        assert!(should_ignore_path("src/node_modules/pkg/index.js", &config));
        assert!(should_ignore_path("build/app.pyc", &config));
        assert!(!should_ignore_path("src/main.rs", &config));
    }

    #[test]
    fn should_ignore_no_patterns_matches_nothing() {
        let config = ProjectConfig {
            ignore_patterns: vec![],
            process_whitelist: vec![],
        };
        assert!(!should_ignore_path("any/path.rs", &config));
    }

    #[test]
    fn should_ignore_with_default_config() {
        let config = ProjectConfig::default();
        // Default config includes "node_modules", ".git", "target", etc.
        assert!(should_ignore_path("node_modules/pkg/index.js", &config));
        assert!(should_ignore_path("target/debug/app", &config));
        assert!(!should_ignore_path("src/main.rs", &config));
    }

    #[test]
    fn should_ignore_normalizes_backslashes() {
        let config = ProjectConfig {
            ignore_patterns: vec!["node_modules".to_string()],
            process_whitelist: vec![],
        };
        assert!(should_ignore_path("src\\node_modules\\pkg", &config));
    }
}
