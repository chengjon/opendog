use crate::core::stats::ProjectSummary;
use crate::error::{OpenDogError, Result};
use crate::storage::queries::StatsEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportView {
    Stats,
    Unused,
    Core,
}

impl ExportView {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "stats" => Ok(Self::Stats),
            "unused" => Ok(Self::Unused),
            "core" => Ok(Self::Core),
            _ => Err(OpenDogError::InvalidInput(format!(
                "view must be one of: stats, unused, core; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stats => "stats",
            Self::Unused => "unused",
            Self::Core => "core",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(OpenDogError::InvalidInput(format!(
                "format must be one of: json, csv; got '{}'",
                value
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortableProjectExport {
    pub schema_version: String,
    pub project_id: String,
    pub format: String,
    pub view: String,
    pub generated_at: String,
    pub summary: ProjectSummary,
    pub row_count: usize,
    pub rows: Vec<StatsEntry>,
}
