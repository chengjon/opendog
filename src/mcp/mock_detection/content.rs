use std::fs;
use std::path::Path;

pub(super) fn read_file_sample(root: &Path, relative_path: &str) -> Option<String> {
    let full_path = root.join(relative_path);
    let metadata = fs::metadata(&full_path).ok()?;
    if metadata.len() > 128 * 1024 {
        return None;
    }
    let bytes = fs::read(full_path).ok()?;
    let sample_len = bytes.len().min(16 * 1024);
    String::from_utf8(bytes[..sample_len].to_vec()).ok()
}

pub(super) fn count_keyword_hits(haystack: &str, keywords: &[&str]) -> usize {
    keywords.iter().filter(|kw| haystack.contains(**kw)).count()
}

pub(super) fn content_has_template_placeholder(content_lower: &str) -> bool {
    content_lower.contains("${")
        || content_lower.contains("{{")
        || content_lower.contains("<your_")
        || content_lower.contains("<insert_")
        || content_lower.contains("example.com")
}

pub(super) fn matched_keywords(haystack: &str, keywords: &[&str], limit: usize) -> Vec<String> {
    keywords
        .iter()
        .filter(|kw| haystack.contains(**kw))
        .take(limit)
        .map(|kw| (*kw).to_string())
        .collect()
}

pub(super) fn content_preview_snippet(content: &str, keywords: &[String]) -> Option<String> {
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

pub(super) fn previous_char_boundary(value: &str, mut index: usize) -> usize {
    index = index.min(value.len());
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

pub(super) fn next_char_boundary(value: &str, mut index: usize) -> usize {
    index = index.min(value.len());
    while index < value.len() && !value.is_char_boundary(index) {
        index += 1;
    }
    index
}
