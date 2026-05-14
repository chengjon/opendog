use rmcp::model::{
    Annotated, RawResource, RawResourceTemplate, ReadResourceResult, Resource, ResourceContents,
    ResourceTemplate,
};
use serde_json::Value;

use super::{handle_get_verification_status, handle_list_projects, OpenDogServer};

const JSON_MIME_TYPE: &str = "application/json";
const PROJECTS_URI: &str = "opendog://projects";
const PROJECT_VERIFICATION_TEMPLATE: &str = "opendog://project/{id}/verification";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResourceKind {
    Projects,
    ProjectVerification { id: String },
}

pub(crate) fn mcp_resource_templates() -> Vec<ResourceTemplate> {
    vec![
        Annotated::new(
            RawResourceTemplate::new(PROJECTS_URI, "opendog.projects")
                .with_title("OpenDog Projects")
                .with_description(
                    "Registered OpenDog projects as a stable read-only JSON resource.",
                )
                .with_mime_type(JSON_MIME_TYPE),
            None,
        ),
        Annotated::new(
            RawResourceTemplate::new(
                PROJECT_VERIFICATION_TEMPLATE,
                "opendog.project.verification",
            )
            .with_title("OpenDog Project Verification")
            .with_description(
                "Latest recorded test/lint/build evidence for one project as read-only JSON.",
            )
            .with_mime_type(JSON_MIME_TYPE),
            None,
        ),
    ]
}

pub(crate) fn mcp_resources() -> Vec<Resource> {
    vec![Annotated::new(
        RawResource::new(PROJECTS_URI, "opendog.projects")
            .with_title("OpenDog Projects")
            .with_description("Registered OpenDog projects as a stable read-only JSON resource.")
            .with_mime_type(JSON_MIME_TYPE),
        None,
    )]
}

pub(crate) fn read_resource_kind(uri: &str) -> Option<ResourceKind> {
    if uri == PROJECTS_URI {
        return Some(ResourceKind::Projects);
    }

    uri.strip_prefix("opendog://project/")
        .and_then(|suffix| suffix.strip_suffix("/verification"))
        .filter(|id| !id.is_empty() && !id.contains('/'))
        .map(|id| ResourceKind::ProjectVerification { id: id.to_string() })
}

pub(super) fn read_mcp_resource(
    server: &OpenDogServer,
    uri: &str,
) -> Result<ReadResourceResult, rmcp::ErrorData> {
    let value = match read_resource_kind(uri) {
        Some(ResourceKind::Projects) => handle_list_projects(server).0,
        Some(ResourceKind::ProjectVerification { id }) => {
            handle_get_verification_status(server, &id).0
        }
        None => {
            return Err(rmcp::ErrorData::resource_not_found(
                format!("Unknown OpenDog resource URI: {uri}"),
                None,
            ));
        }
    };

    json_resource_result(uri, &value)
}

fn json_resource_result(uri: &str, value: &Value) -> Result<ReadResourceResult, rmcp::ErrorData> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| rmcp::ErrorData::internal_error(err.to_string(), None))?;
    Ok(ReadResourceResult::new(vec![ResourceContents::text(
        text, uri,
    )
    .with_mime_type(JSON_MIME_TYPE)]))
}
