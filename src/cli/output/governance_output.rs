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
            println!("  {:8} {:22} {:32} {}", "Node", "State", "Summary", "Suggested Next");
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
