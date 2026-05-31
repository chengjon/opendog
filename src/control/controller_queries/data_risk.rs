use super::*;

impl MonitorController {
    pub fn get_data_risk_candidates(
        &self,
        schema_version: &str,
        id: &str,
        candidate_type: &str,
        min_review_priority: &str,
        limit: usize,
    ) -> Result<Value> {
        let candidate_type =
            normalize_candidate_type(Some(candidate_type.to_string())).map_err(|error| {
                OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
            })?;
        let min_review_priority = normalize_min_review_priority(Some(
            min_review_priority.to_string(),
        ))
        .map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
        })?;

        self.with_project_info_db(id, |info, db| {
            let entries = stats::get_stats(db)?;
            Ok(project_data_risk_payload(ProjectDataRiskPayloadInput {
                schema_version,
                id,
                candidate_type: &candidate_type,
                min_review_priority: &min_review_priority,
                limit: limit.max(1),
                root_path: &info.root_path,
                entries: &entries,
                db: Some(db),
            }))
        })
    }

    pub fn get_workspace_data_risk_overview(
        &self,
        schema_version: &str,
        candidate_type: &str,
        min_review_priority: &str,
        project_limit: usize,
    ) -> Result<Value> {
        let candidate_type =
            normalize_candidate_type(Some(candidate_type.to_string())).map_err(|error| {
                OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
            })?;
        let min_review_priority = normalize_min_review_priority(Some(
            min_review_priority.to_string(),
        ))
        .map_err(|error| {
            OpenDogError::InvalidInput(error["error"].as_str().unwrap_or("").to_string())
        })?;

        let projects = self.list_projects()?;
        Ok(workspace_data_risk_payload(
            schema_version,
            &projects,
            &candidate_type,
            &min_review_priority,
            project_limit.max(1),
            |item| {
                self.pm
                    .open_project_db(&item.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
            |project_id: &str| self.pm.open_project_db(project_id).ok(),
        ))
    }
}
