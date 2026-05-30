use super::*;

fn make_template(id: &str) -> Value {
    json!({ "template_id": id })
}

#[path = "tests/follow_ups.rs"]
mod follow_ups;
#[path = "tests/plan_stage.rs"]
mod plan_stage;
#[path = "tests/policy_and_outputs.rs"]
mod policy_and_outputs;
#[path = "tests/run_conditions.rs"]
mod run_conditions;
#[path = "tests/terminality_and_evidence.rs"]
mod terminality_and_evidence;
