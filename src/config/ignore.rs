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
