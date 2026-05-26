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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- read_resource_kind ---

    #[test]
    fn resource_kind_projects_uri() {
        assert_eq!(
            read_resource_kind("opendog://projects"),
            Some(ResourceKind::Projects)
        );
    }

    #[test]
    fn resource_kind_project_verification() {
        let kind = read_resource_kind("opendog://project/my-app/verification").unwrap();
        assert_eq!(
            kind,
            ResourceKind::ProjectVerification {
                id: "my-app".to_string()
            }
        );
    }

    #[test]
    fn resource_kind_project_verification_with_dashes() {
        let kind = read_resource_kind("opendog://project/my-cool-app/verification").unwrap();
        assert_eq!(
            kind,
            ResourceKind::ProjectVerification {
                id: "my-cool-app".to_string()
            }
        );
    }

    #[test]
    fn resource_kind_empty_id_rejected() {
        assert_eq!(read_resource_kind("opendog://project//verification"), None);
    }

    #[test]
    fn resource_kind_slash_in_id_rejected() {
        assert_eq!(
            read_resource_kind("opendog://project/a/b/verification"),
            None
        );
    }

    #[test]
    fn resource_kind_unknown_uri() {
        assert_eq!(read_resource_kind("opendog://unknown"), None);
        assert_eq!(read_resource_kind("http://example.com"), None);
        assert_eq!(read_resource_kind(""), None);
    }

    #[test]
    fn resource_kind_partial_verification_path() {
        assert_eq!(read_resource_kind("opendog://project/demo/verify"), None);
        assert_eq!(read_resource_kind("opendog://project/demo"), None);
    }

    // --- json_resource_result ---

    #[test]
    fn json_resource_result_ok() {
        let value = json!({"status": "ok", "count": 42});
        let result = json_resource_result("opendog://test", &value).unwrap();
        assert_eq!(result.contents.len(), 1);
        let content = &result.contents[0];
        // The text field should be pretty-printed JSON
        let text = match content {
            ResourceContents::TextResourceContents { text, .. } => text,
            other => panic!("expected TextResourceContents, got {:?}", other),
        };
        assert!(text.contains("\"status\": \"ok\""));
        assert!(text.contains("\"count\": 42"));
    }

    #[test]
    fn json_resource_result_preserves_uri() {
        let value = json!({});
        let result = json_resource_result("opendog://projects", &value).unwrap();
        let uri = match &result.contents[0] {
            ResourceContents::TextResourceContents { uri, .. } => uri.clone(),
            other => panic!("expected TextResourceContents, got {:?}", other),
        };
        assert_eq!(uri, "opendog://projects");
    }
}
