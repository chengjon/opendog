use super::*;
use crate::config::{default_process_whitelist, DEFAULT_IGNORE_PATTERNS};

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: DEFAULT_IGNORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            process_whitelist: default_process_whitelist(),
            retention: Default::default(),
        }
    }
}
