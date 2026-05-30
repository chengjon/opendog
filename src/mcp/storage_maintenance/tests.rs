use super::*;
use crate::config::RetentionPolicy;
use crate::core::retention::{StorageEvidenceCounts, StorageMetrics};

#[path = "tests/augmentation.rs"]
mod augmentation;
#[path = "tests/execution_templates.rs"]
mod execution_templates;
#[path = "tests/layer_aggregation.rs"]
mod layer_aggregation;
#[path = "tests/project_assessment.rs"]
mod project_assessment;
#[path = "tests/reclaim_ratio.rs"]
mod reclaim_ratio;
