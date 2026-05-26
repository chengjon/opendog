use crate::core::report::ReportWindow;

use super::{ControlRequest, ControlResponse, MonitorController};

fn respond<T>(
    result: crate::error::Result<T>,
    ok: impl FnOnce(T) -> ControlResponse,
) -> ControlResponse {
    match result {
        Ok(value) => ok(value),
        Err(e) => ControlResponse::Error {
            message: e.to_string(),
        },
    }
}

impl MonitorController {
    pub fn handle_request(&mut self, request: ControlRequest) -> ControlResponse {
        match request {
            ControlRequest::Ping => ControlResponse::Pong,
            ControlRequest::CreateProject { id, path } => {
                respond(self.create_project(&id, &path), |info| {
                    ControlResponse::ProjectCreated { info }
                })
            }
            ControlRequest::DeleteProject { id } => respond(self.delete_project(&id), |deleted| {
                ControlResponse::ProjectDeleted { id, deleted }
            }),
            ControlRequest::ListProjects => respond(self.list_projects(), |projects| {
                ControlResponse::Projects { projects }
            }),
            ControlRequest::GetGlobalConfig => respond(self.global_config(), |config| {
                ControlResponse::GlobalConfig { config }
            }),
            ControlRequest::GetProjectConfig { id } => {
                respond(self.project_config_view(&id), |view| {
                    ControlResponse::ProjectConfig { view }
                })
            }
            ControlRequest::UpdateGlobalConfig(patch) => {
                respond(self.update_global_config(patch), |result| {
                    ControlResponse::GlobalConfigUpdated { result }
                })
            }
            ControlRequest::UpdateProjectConfig(fields) => respond(
                self.update_project_config(&fields.id, fields.patch),
                |result| ControlResponse::ProjectConfigUpdated { result },
            ),
            ControlRequest::ReloadProjectConfig { id } => respond(
                self.reload_project_config(&id).and_then(|reload| {
                    self.pm
                        .effective_project_config(&id)
                        .map(|effective| (reload, effective))
                }),
                |(reload, effective)| ControlResponse::ProjectConfigReloaded {
                    id,
                    reload,
                    effective,
                },
            ),
            ControlRequest::ListMonitors => ControlResponse::Monitors {
                ids: self.monitor_ids(),
            },
            ControlRequest::GetStats { id } => {
                respond(self.get_stats(&id), |(summary, entries)| {
                    ControlResponse::Stats {
                        id,
                        summary,
                        entries,
                    }
                })
            }
            ControlRequest::GetUnusedFiles { id } => {
                respond(self.get_unused_files(&id), |entries| {
                    ControlResponse::UnusedFiles { id, entries }
                })
            }
            ControlRequest::GetTimeWindowReport { id, window, limit } => {
                let window = match ReportWindow::parse(&window) {
                    Ok(w) => w,
                    Err(e) => {
                        return ControlResponse::Error {
                            message: e.to_string(),
                        };
                    }
                };
                respond(self.get_time_window_report(&id, window, limit), |report| {
                    ControlResponse::TimeWindowReport { id, report }
                })
            }
            ControlRequest::CompareSnapshots {
                id,
                base_run_id,
                head_run_id,
                limit,
            } => respond(
                self.compare_snapshots(&id, base_run_id, head_run_id, limit),
                |comparison| ControlResponse::SnapshotComparison { id, comparison },
            ),
            ControlRequest::GetUsageTrends { id, window, limit } => {
                let window = match ReportWindow::parse(&window) {
                    Ok(w) => w,
                    Err(e) => {
                        return ControlResponse::Error {
                            message: e.to_string(),
                        };
                    }
                };
                respond(self.get_usage_trends(&id, window, limit), |report| {
                    ControlResponse::UsageTrends { id, report }
                })
            }
            ControlRequest::GetDataRiskCandidates {
                id,
                candidate_type,
                min_review_priority,
                limit,
                schema_version,
            } => respond(
                self.get_data_risk_candidates(
                    &schema_version,
                    &id,
                    &candidate_type,
                    &min_review_priority,
                    limit,
                ),
                |payload| ControlResponse::DataRisk { payload },
            ),
            ControlRequest::GetWorkspaceDataRiskOverview {
                candidate_type,
                min_review_priority,
                project_limit,
                schema_version,
            } => respond(
                self.get_workspace_data_risk_overview(
                    &schema_version,
                    &candidate_type,
                    &min_review_priority,
                    project_limit,
                ),
                |payload| ControlResponse::WorkspaceDataRisk { payload },
            ),
            ControlRequest::GetAgentGuidance { project, top } => respond(
                self.get_agent_guidance(project.as_deref(), top),
                |payload| ControlResponse::AgentGuidance { payload },
            ),
            ControlRequest::GetDecisionBrief {
                project,
                top,
                schema_version,
            } => respond(
                self.get_decision_brief(&schema_version, project.as_deref(), top),
                |payload| ControlResponse::DecisionBrief { payload },
            ),
            ControlRequest::GetVerificationStatus { id } => {
                respond(self.get_verification_status(&id), |runs| {
                    ControlResponse::VerificationStatus { id, runs }
                })
            }
            ControlRequest::CleanupProjectData(fields) => respond(
                self.cleanup_project_data(&fields.id, fields.request),
                |result| ControlResponse::CleanupProjectData {
                    id: fields.id,
                    result,
                },
            ),
            ControlRequest::RecordVerificationResult(fields) => respond(
                self.record_verification_result(&fields.id, fields.input),
                |run| ControlResponse::VerificationRecorded { id: fields.id, run },
            ),
            ControlRequest::ExecuteVerification(fields) => respond(
                self.execute_verification(&fields.id, fields.input),
                |result| ControlResponse::VerificationExecuted {
                    id: fields.id,
                    result,
                },
            ),
            ControlRequest::StartMonitor { id } => respond(self.start_monitor(&id), |outcome| {
                ControlResponse::Started {
                    id,
                    already_running: outcome.already_running,
                    snapshot_taken: outcome.snapshot_taken,
                }
            }),
            ControlRequest::StopMonitor { id } => ControlResponse::Stopped {
                was_running: self.stop_monitor(&id),
                id,
            },
            ControlRequest::TakeSnapshot { id } => respond(self.take_snapshot(&id), |result| {
                ControlResponse::Snapshot { id, result }
            }),
            ControlRequest::CreateGovernanceLane { id, input } => {
                respond(self.create_governance_lane(&id, input), |lane| {
                    ControlResponse::GovernanceLaneCreated { id, lane }
                })
            }
            ControlRequest::UpsertGovernanceNode { id, input } => {
                respond(self.upsert_governance_node(&id, input), |result| {
                    ControlResponse::GovernanceNodeUpserted { id, result }
                })
            }
            ControlRequest::GetGovernanceState { id, input } => {
                respond(self.get_governance_state(&id, input), |state| {
                    ControlResponse::GovernanceState { id, state }
                })
            }
            ControlRequest::CloseGovernanceLane { id, input } => {
                let lane_id = input.lane_id.clone();
                let action = input.action.clone();
                respond(
                    self.close_governance_lane(&id, input),
                    move |(status, nodes_affected)| ControlResponse::GovernanceLaneClosed {
                        id,
                        lane_id,
                        action_taken: action,
                        status,
                        nodes_affected,
                    },
                )
            }
            ControlRequest::ScanOrphans { id, input } => {
                respond(self.scan_orphans(&id, input), |result| {
                    ControlResponse::OrphansScanned { id, result }
                })
            }
            ControlRequest::VerifyDeletionPlan { id, input } => {
                respond(self.verify_deletion_plan(&id, input), |result| {
                    ControlResponse::DeletionPlanVerified { id, result }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respond_ok_maps_to_custom_response() {
        let result: crate::error::Result<i32> = Ok(42);
        let response = respond(result, |_| ControlResponse::Pong);
        // The ok closure is called, producing the mapped response
        assert!(matches!(response, ControlResponse::Pong));
    }

    #[test]
    fn respond_err_maps_to_error_response() {
        let result: crate::error::Result<i32> = Err(crate::error::OpenDogError::InvalidInput(
            "bad input".to_string(),
        ));
        let response = respond(result, |_| ControlResponse::Pong);
        match response {
            ControlResponse::Error { message } => {
                assert!(message.contains("bad input"));
            }
            other => panic!("expected Error variant, got {:?}", other),
        }
    }

    #[test]
    fn respond_ok_with_complex_value() {
        let result: crate::error::Result<String> = Ok("hello".to_string());
        let response = respond(result, |val| ControlResponse::Error { message: val });
        // The ok closure receives the value and maps it
        match response {
            ControlResponse::Error { message } => {
                assert_eq!(message, "hello");
            }
            other => panic!(
                "expected Error variant (used as test wrapper), got {:?}",
                other
            ),
        }
    }
}
