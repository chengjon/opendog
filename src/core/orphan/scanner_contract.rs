use crate::error::{OpenDogError, Result};
use std::collections::BTreeSet;

use super::types::{ExternalScannerReport, DEFAULT_REQUIRED_SCANNERS, INTERNAL_SCANNERS};

pub fn validate_required_scanners(
    required_scanners: Option<&[String]>,
    external_reports: &[ExternalScannerReport],
) -> Result<Vec<String>> {
    let Some(required_scanners) = required_scanners else {
        return Ok(DEFAULT_REQUIRED_SCANNERS
            .iter()
            .map(|scanner| (*scanner).to_string())
            .collect());
    };

    if required_scanners.is_empty() {
        return Err(OpenDogError::InvalidInput(
            "required_scanners cannot be empty".to_string(),
        ));
    }

    let external_names: BTreeSet<&str> = external_reports
        .iter()
        .map(|report| report.scanner.as_str())
        .collect();
    let mut validated = BTreeSet::new();
    for scanner in required_scanners {
        let known_internal = INTERNAL_SCANNERS.contains(&scanner.as_str());
        if !known_internal && !external_names.contains(scanner.as_str()) {
            return Err(OpenDogError::InvalidInput(format!(
                "unknown required scanner '{}'",
                scanner
            )));
        }
        validated.insert(scanner.clone());
    }

    Ok(validated.into_iter().collect())
}
