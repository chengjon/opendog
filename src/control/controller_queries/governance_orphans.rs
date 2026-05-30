use super::*;

impl MonitorController {
    pub fn create_governance_lane(
        &self,
        id: &str,
        input: CreateLaneInput,
    ) -> Result<GovernanceLane> {
        self.with_project_db(id, |db| governance::create_lane(db, input))
    }

    pub fn upsert_governance_node(
        &self,
        id: &str,
        input: UpsertNodeInput,
    ) -> Result<UpsertNodeResult> {
        self.with_project_db(id, |db| governance::upsert_node(db, input))
    }

    pub fn get_governance_state(
        &self,
        id: &str,
        input: GetGovernanceStateInput,
    ) -> Result<GovernanceState> {
        self.with_project_db(id, |db| governance::get_governance_state(db, input))
    }

    pub fn close_governance_lane(
        &self,
        id: &str,
        input: CloseLaneInput,
    ) -> Result<(String, usize)> {
        self.with_project_db(id, |db| governance::close_lane(db, input))
    }

    pub fn scan_orphans(&self, id: &str, input: ScanOrphansInput) -> Result<ScanOrphansResult> {
        self.with_project_info_db(id, |info, _| {
            let config = self.pm.effective_project_config(id)?;
            orphan::scan_project_orphans(&info.root_path, &config, input)
        })
    }

    pub fn verify_deletion_plan(
        &self,
        id: &str,
        input: DeletionPlanInput,
    ) -> Result<DeletionPlanVerification> {
        self.with_project_info_db(id, |info, _| {
            let config = self.pm.effective_project_config(id)?;
            orphan::verify_deletion_plan(&info.root_path, &config, input)
        })
    }
}
