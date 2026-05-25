use crate::core::governance::{GovernanceState, UpsertNodeResult};
use crate::storage::queries::GovernanceLane;

pub fn print_lane_created(id: &str, lane: &GovernanceLane) {
    println!("Governance lane created for project '{}'.", id);
    println!("  Lane: {} ({})", lane.lane_id, lane.title);
    println!("  Status: {}", lane.status);
}

pub fn print_node_upserted(id: &str, result: &UpsertNodeResult) {
    let action = if result.created { "Created" } else { "Updated" };
    println!("{} governance node for project '{}'.", action, id);
    println!("  Node: {} in lane {}", result.node_id, result.lane_id);
    println!("  State: {}", result.state);
}

pub fn print_governance_state(id: &str, state: &GovernanceState) {
    println!("Governance: {}", id);
    println!("  Observation: snapshot={} | verification={} | unused={} | data_risk={}",
        state.observation_hints.snapshot_freshness,
        state.observation_hints.verification_status,
        state.observation_hints.unused_files,
        state.observation_hints.data_risk_candidates,
    );
    println!();
    for lane in &state.lanes {
        println!("Lane: {}", lane.lane_id);
        println!("  Title: {} | Status: {}", lane.title, lane.status);
        println!();

        let lane_nodes: Vec<_> = state.nodes.iter().filter(|n| n.lane_id == lane.lane_id).collect();
        if !lane_nodes.is_empty() {
            println!("  {:8} {:22} {:32} Suggested Next", "Node", "State", "Summary");
            for node in lane_nodes {
                let summary = node.summary.as_deref().unwrap_or("").chars().take(30).collect::<String>();
                let suggested = node.suggested_next.as_deref().unwrap_or("").chars().take(30).collect::<String>();
                println!("  {:8} {:22} {:32} {}",
                    truncate_str(&node.node_id, 8),
                    truncate_str(&node.state, 22),
                    truncate_str(&summary, 32),
                    truncate_str(&suggested, 30),
                );
            }
            println!();
        }
    }
}

pub fn print_lane_closed(id: &str, lane_id: &str, action: &str, status: &str, nodes: usize) {
    println!("Governance lane '{}' for project '{}'.", lane_id, id);
    println!("  Action: {} | Result: {} | Nodes affected: {}", action, status, nodes);
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max - 3).collect::<String>() + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_str_short_string_unchanged() {
        assert_eq!(truncate_str("abc", 10), "abc");
    }

    #[test]
    fn truncate_str_exact_length_unchanged() {
        assert_eq!(truncate_str("abcde", 5), "abcde");
    }

    #[test]
    fn truncate_str_long_string_truncated() {
        assert_eq!(truncate_str("abcdefghij", 5), "ab...");
    }

    #[test]
    fn truncate_str_empty_string() {
        assert_eq!(truncate_str("", 5), "");
    }

    #[test]
    #[should_panic(expected = "subtract with overflow")]
    fn truncate_str_max_zero_panics() {
        // max=0 causes underflow in max-3; documents current behavior
        let _ = truncate_str("abc", 0);
    }

    #[test]
    fn truncate_str_one_char_over() {
        assert_eq!(truncate_str("abcdef", 5), "ab...");
    }

    #[test]
    fn truncate_str_max_three() {
        assert_eq!(truncate_str("abcde", 3), "...");
    }

    #[test]
    fn truncate_str_unicode_chars() {
        // 5 chars: a 😀 c 😁 e, max=4 => first (4-3=1) chars + "..."
        assert_eq!(truncate_str("a\u{1F600}c\u{1F601}e", 4), "a...");
    }

    #[test]
    fn truncate_str_unicode_exact_fit() {
        assert_eq!(truncate_str("a\u{1F600}c", 3), "a\u{1F600}c");
    }

    #[test]
    #[should_panic(expected = "subtract with overflow")]
    fn truncate_str_max_two_panics() {
        // max=2 causes underflow in max-3; documents current behavior
        let _ = truncate_str("hello", 2);
    }

    #[test]
    fn truncate_str_single_char_string() {
        assert_eq!(truncate_str("x", 1), "x");
    }
}
