use super::*;

impl MonitorController {
    pub fn get_verification_status(&self, id: &str) -> Result<Vec<VerificationRun>> {
        self.with_project_db(id, verification::get_latest_verification_runs)
    }

    pub fn record_verification_result(
        &self,
        id: &str,
        input: RecordVerificationInput,
    ) -> Result<VerificationRun> {
        self.with_project_db(id, |db| verification::record_verification_result(db, input))
    }

    pub fn execute_verification(
        &self,
        id: &str,
        input: ExecuteVerificationInput,
    ) -> Result<ExecutedVerificationResult> {
        self.with_project_info_db(id, |info, db| {
            verification::execute_verification_command(db, &info.root_path, input)
        })
    }
}
