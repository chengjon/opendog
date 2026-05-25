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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orphan::types::{ScannerHealth, DEFAULT_REQUIRED_SCANNERS, INTERNAL_SCANNERS};

    fn make_external_report(scanner_name: &str) -> ExternalScannerReport {
        ExternalScannerReport {
            scanner: scanner_name.to_string(),
            version: "1.0.0".to_string(),
            health: ScannerHealth::Passed,
            started_at: None,
            finished_at: None,
            evidence: vec![],
            warnings: vec![],
            errors: vec![],
        }
    }

    // --- validate_required_scanners ---

    #[test]
    fn validate_required_scanners_none_returns_defaults() {
        let result = validate_required_scanners(None, &[]).unwrap();
        let expected: Vec<String> = DEFAULT_REQUIRED_SCANNERS
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn validate_required_scanners_with_known_internal_scanners() {
        let scanners: Vec<String> = INTERNAL_SCANNERS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let result = validate_required_scanners(Some(&scanners), &[]).unwrap();
        // Result should contain all internal scanners (sorted BTreeSet order)
        assert_eq!(result.len(), INTERNAL_SCANNERS.len());
        for scanner in &scanners {
            assert!(result.contains(scanner));
        }
    }

    #[test]
    fn validate_required_scanners_with_external_report() {
        let report = make_external_report("custom_scanner");
        let scanners = vec!["custom_scanner".to_string()];
        let result = validate_required_scanners(Some(&scanners), &[report]).unwrap();
        assert_eq!(result, vec!["custom_scanner".to_string()]);
    }

    #[test]
    fn validate_required_scanners_rejects_unknown_scanner() {
        let scanners = vec!["nonexistent_scanner".to_string()];
        let result = validate_required_scanners(Some(&scanners), &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            OpenDogError::InvalidInput(msg) => {
                assert!(msg.contains("nonexistent_scanner"));
            }
            other => panic!("Expected InvalidInput, got {:?}", other),
        }
    }

    #[test]
    fn validate_required_scanners_rejects_empty_list() {
        let scanners: Vec<String> = vec![];
        let result = validate_required_scanners(Some(&scanners), &[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            OpenDogError::InvalidInput(msg) => {
                assert!(msg.contains("cannot be empty"));
            }
            other => panic!("Expected InvalidInput, got {:?}", other),
        }
    }

    #[test]
    fn validate_required_scanners_deduplicates() {
        let scanners = vec![
            "candidate_collector".to_string(),
            "candidate_collector".to_string(),
        ];
        let result = validate_required_scanners(Some(&scanners), &[]).unwrap();
        assert_eq!(result, vec!["candidate_collector".to_string()]);
    }

    #[test]
    fn validate_required_scanners_mixed_internal_and_external() {
        let report = make_external_report("ext_scanner");
        let scanners = vec![
            "candidate_collector".to_string(),
            "ext_scanner".to_string(),
        ];
        let result = validate_required_scanners(Some(&scanners), &[report]).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"candidate_collector".to_string()));
        assert!(result.contains(&"ext_scanner".to_string()));
    }
}
