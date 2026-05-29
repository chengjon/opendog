mod gate;
mod status;
mod workspace;

#[cfg(test)]
pub(super) use gate::{
    blocker_reasons, failing_kinds, gate_assessment, gate_kinds, gate_next_steps, gate_reasons,
    kind_state_sets,
};
pub(super) use gate::{
    gate_blockers, pipeline_caution_kinds, project_gate_level, suspicious_summary_kinds,
    suspicious_summary_signals, VerificationGateAssessment, VerificationGateTarget,
};
pub(super) use status::{GateDistribution, VerificationStatusSummary};
pub(super) use workspace::VerificationEvidenceWorkspaceSummary;

const EXPECTED_KINDS: [&str; 3] = ["test", "lint", "build"];

#[cfg(test)]
mod tests;
