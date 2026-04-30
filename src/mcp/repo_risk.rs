mod collection;
mod findings;

use serde_json::Value;

#[cfg(test)]
pub(super) use self::collection::{detect_lockfile_anomalies, parse_status_porcelain};
#[cfg(test)]
pub(super) use self::findings::repo_risk_findings;
pub(super) use self::findings::repo_status_risk_layer;

pub(super) struct GitStatusEntry {
    pub(super) staged: char,
    pub(super) unstaged: char,
    pub(super) path: String,
}

pub(super) struct RepoRiskSnapshot {
    pub(super) status: &'static str,
    pub(super) branch: Option<String>,
    pub(super) is_dirty: bool,
    pub(super) changed_file_count: usize,
    pub(super) staged_count: usize,
    pub(super) unstaged_count: usize,
    pub(super) untracked_count: usize,
    pub(super) conflicted_count: usize,
    pub(super) operation_states: Vec<String>,
    pub(super) top_changed_directories: Vec<(String, usize)>,
    pub(super) large_diff: bool,
    pub(super) lockfile_anomalies: Vec<Value>,
    pub(super) evidence: Vec<String>,
    pub(super) risk_reasons: Vec<String>,
    pub(super) risk_level: &'static str,
}
