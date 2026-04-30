use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::rules::{data_risk_rule_meta, review_priority_score};
use super::{DataCandidate, MockDataReport};

impl MockDataReport {
    pub(crate) fn to_value(&self, limit: usize) -> Value {
        json!({
            "mock_candidate_count": self.mock_candidates.len(),
            "hardcoded_candidate_count": self.hardcoded_candidates.len(),
            "mixed_review_file_count": self.mixed_review_files.len(),
            "rule_groups_summary": self.rule_groups_summary(),
            "rule_hits_summary": self.rule_hits_summary(),
            "mock_data_candidates": self.mock_candidates.iter().take(limit).map(data_candidate_value).collect::<Vec<_>>(),
            "hardcoded_data_candidates": self.hardcoded_candidates.iter().take(limit).map(data_candidate_value).collect::<Vec<_>>(),
            "mixed_review_files": self.mixed_review_files.iter().take(limit).cloned().collect::<Vec<_>>(),
        })
    }

    pub(crate) fn filtered(&self, candidate_type: &str, min_review_priority: Option<&str>) -> Self {
        let min_score = min_review_priority.map(review_priority_score).unwrap_or(0);
        let filter_candidates = |candidates: &[DataCandidate]| {
            candidates
                .iter()
                .filter(|candidate| review_priority_score(candidate.review_priority) >= min_score)
                .cloned()
                .collect::<Vec<_>>()
        };

        let mock_candidates = if candidate_type == "hardcoded" {
            Vec::new()
        } else {
            filter_candidates(&self.mock_candidates)
        };
        let hardcoded_candidates = if candidate_type == "mock" {
            Vec::new()
        } else {
            filter_candidates(&self.hardcoded_candidates)
        };
        let mixed_review_files = self
            .mixed_review_files
            .iter()
            .filter(|path| {
                let mock_match = mock_candidates
                    .iter()
                    .any(|candidate| &candidate.file_path == *path);
                let hardcoded_match = hardcoded_candidates
                    .iter()
                    .any(|candidate| &candidate.file_path == *path);
                match candidate_type {
                    "mock" => mock_match,
                    "hardcoded" => hardcoded_match,
                    _ => mock_match || hardcoded_match,
                }
            })
            .cloned()
            .collect();

        Self {
            mock_candidates,
            hardcoded_candidates,
            mixed_review_files,
        }
    }

    fn rule_hits_summary(&self) -> Value {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for candidate in self
            .mock_candidates
            .iter()
            .chain(self.hardcoded_candidates.iter())
        {
            for hit in &candidate.rule_hits {
                *counts.entry(hit.clone()).or_insert(0) += 1;
            }
        }

        json!(counts
            .into_iter()
            .map(|(rule, count)| {
                if let Some(meta) = data_risk_rule_meta(&rule) {
                    json!({
                        "rule": meta.rule,
                        "group": meta.group,
                        "severity": meta.severity,
                        "description": meta.description,
                        "count": count,
                    })
                } else {
                    json!({
                        "rule": rule,
                        "group": rule.split('.').next().unwrap_or("unknown"),
                        "severity": "unknown",
                        "description": "No rule metadata registered.",
                        "count": count,
                    })
                }
            })
            .collect::<Vec<_>>())
    }

    fn rule_groups_summary(&self) -> Value {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for candidate in self
            .mock_candidates
            .iter()
            .chain(self.hardcoded_candidates.iter())
        {
            for hit in &candidate.rule_hits {
                let group = data_risk_rule_meta(hit)
                    .map(|meta| meta.group)
                    .unwrap_or_else(|| hit.split('.').next().unwrap_or("unknown"))
                    .to_string();
                *counts.entry(group).or_insert(0) += 1;
            }
        }

        json!(counts
            .into_iter()
            .map(|(group, count)| json!({
                "group": group,
                "count": count,
                "severity": match group.as_str() {
                    "content" => "medium",
                    "classification" => "medium",
                    "path" => "low",
                    _ => "unknown",
                },
            }))
            .collect::<Vec<_>>())
    }
}

fn data_candidate_value(candidate: &DataCandidate) -> Value {
    let rule_hits = candidate
        .rule_hits
        .iter()
        .map(|rule| {
            if let Some(meta) = data_risk_rule_meta(rule) {
                json!({
                    "rule": meta.rule,
                    "group": meta.group,
                    "severity": meta.severity,
                    "description": meta.description,
                })
            } else {
                json!({
                    "rule": rule,
                    "group": rule.split('.').next().unwrap_or("unknown"),
                    "severity": "unknown",
                    "description": "No rule metadata registered.",
                })
            }
        })
        .collect::<Vec<_>>();
    json!({
        "file_path": candidate.file_path,
        "confidence": candidate.confidence,
        "review_priority": candidate.review_priority,
        "path_classification": candidate.path_classification,
        "rule_hits": rule_hits,
        "matched_keywords": candidate.matched_keywords,
        "reasons": candidate.reasons,
        "evidence": candidate.evidence,
        "access_count": candidate.access_count,
        "file_type": candidate.file_type,
        "suggested_commands": data_candidate_commands(candidate),
    })
}

fn data_candidate_commands(candidate: &DataCandidate) -> Vec<String> {
    let mut commands = vec![
        format!("rg \"{}\" .", candidate.file_path),
        format!(
            "rg \"mock|fixture|fake|stub|sample|demo|seed|customer|invoice|email|address\" \"{}\"",
            candidate.file_path
        ),
    ];
    if candidate.path_classification == "runtime_shared" || candidate.review_priority == "high" {
        commands.push("git diff".to_string());
    }
    commands
}
