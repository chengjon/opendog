use super::*;

impl MonitorController {
    pub fn get_agent_guidance(&self, project: Option<&str>, top: usize) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_agent_guidance_for_projects(
            &self.pm,
            &projects,
            top.max(1),
            |item| self.guidance_project_state(item),
        ))
    }

    pub fn get_decision_brief(
        &self,
        schema_version: &str,
        project: Option<&str>,
        top: usize,
    ) -> Result<Value> {
        let projects = self.guidance_projects(project)?;
        Ok(build_decision_brief_for_projects(
            &self.pm,
            schema_version,
            if project.is_some() {
                "project"
            } else {
                "workspace"
            },
            project,
            &projects,
            top.max(1),
            |item| self.guidance_project_state(item),
            |item| {
                self.pm
                    .open_project_db(&item.id)
                    .ok()
                    .and_then(|db| stats::get_stats(&db).ok())
                    .unwrap_or_default()
            },
        ))
    }
}
