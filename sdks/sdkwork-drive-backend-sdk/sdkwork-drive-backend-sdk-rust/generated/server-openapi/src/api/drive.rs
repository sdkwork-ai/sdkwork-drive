use std::sync::Arc;

use crate::api::paths::backend_path;
use crate::api::paths::append_query_string;
use crate::http::{SdkworkError, SdkworkHttpClient};
use crate::models::{CreateLabelRequest, CreateSandboxGrantRequest, CreateSandboxVolumeRequest, DriveLabel, SweepObjectStoreRequest, SweepUploadSessionsRequest, UpdateLabelRequest, UpdateQuotaPolicyRequest, UpdateSandboxGrantRequest, UpdateSandboxVolumeRequest};

#[derive(Clone)]
pub struct DriveApi {
    client: Arc<SdkworkHttpClient>,
}

impl DriveApi {
    pub fn new(client: Arc<SdkworkHttpClient>) -> Self {
        Self { client }
    }

    pub async fn audit_events_list(&self, action: Option<&str>, resource_type: Option<&str>, resource_id: Option<&str>, correlation_id: Option<&str>, trace_id: Option<&str>, page: Option<i64>, page_size: Option<i64>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("action", action, "form", true, false, None),
            QueryParameterSpec::new("resourceType", resource_type, "form", true, false, None),
            QueryParameterSpec::new("resourceId", resource_id, "form", true, false, None),
            QueryParameterSpec::new("correlationId", correlation_id, "form", true, false, None),
            QueryParameterSpec::new("traceId", trace_id, "form", true, false, None),
            QueryParameterSpec::new("page", page, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/audit_events".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// List Drive label definitions
    pub async fn labels_list(&self, lifecycle_status: Option<&str>, page_size: Option<i64>, cursor: Option<&str>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("lifecycleStatus", lifecycle_status, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/labels".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Create a Drive label definition
    pub async fn labels_create(&self, body: &CreateLabelRequest) -> Result<DriveLabel, SdkworkError> {
        let path = backend_path(&"/drive/labels".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Get a Drive label definition
    pub async fn labels_retrieve(&self, label_id: &str) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/labels/{}", serialize_path_parameter(label_id, PathParameterSpec::new("labelId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Update a Drive label definition
    pub async fn labels_update(&self, label_id: &str, body: &UpdateLabelRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/labels/{}", serialize_path_parameter(label_id, PathParameterSpec::new("labelId", "simple", false))));
        self.client.patch(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Delete a Drive label definition
    pub async fn labels_delete(&self, label_id: &str) -> Result<(), SdkworkError> {
        let path = backend_path(&format!("/drive/labels/{}", serialize_path_parameter(label_id, PathParameterSpec::new("labelId", "simple", false))));
        self.client.delete(&path, None, None).await
    }

    pub async fn maintenance_jobs_list(&self, job_type: Option<&str>, status: Option<&str>, operator_id: Option<&str>, page: Option<i64>, page_size: Option<i64>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("jobType", job_type, "form", true, false, None),
            QueryParameterSpec::new("status", status, "form", true, false, None),
            QueryParameterSpec::new("operatorId", operator_id, "form", true, false, None),
            QueryParameterSpec::new("page", page, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/maintenance/jobs".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    pub async fn maintenance_object_sweep(&self, body: &SweepObjectStoreRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/maintenance/object_sweep".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    pub async fn maintenance_upload_session_sweep(&self, body: &SweepUploadSessionsRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/maintenance/upload_session_sweep".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    pub async fn maintenance_expired_upload_content_sweep(&self, body: &SweepUploadSessionsRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/maintenance/expired_upload_content_sweep".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    pub async fn maintenance_abandoned_upload_task_sweep(&self, body: &SweepUploadSessionsRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/maintenance/abandoned_upload_task_sweep".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    pub async fn quotas_retrieve(&self) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/quotas".to_string());
        self.client.get(&path, None, None).await
    }

    /// Update tenant quota policy
    pub async fn quotas_update(&self, body: &UpdateQuotaPolicyRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/quotas".to_string());
        self.client.put(&path, Some(body), None, None, Some("application/json")).await
    }

    pub async fn spaces_admin_list(&self, owner_subject_type: Option<&str>, owner_subject_id: Option<&str>, page_size: Option<i64>, cursor: Option<&str>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("ownerSubjectType", owner_subject_type, "form", true, false, None),
            QueryParameterSpec::new("ownerSubjectId", owner_subject_id, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
            QueryParameterSpec::new("cursor", cursor, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/spaces".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    pub async fn download_packages_list(&self, state: Option<&str>, page: Option<i64>, page_size: Option<i64>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("state", state, "form", true, false, None),
            QueryParameterSpec::new("page", page, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/download_packages".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// List server sandbox volumes
    pub async fn sandbox_volumes_list(&self, lifecycle_status: Option<&str>, provider_kind: Option<&str>, page: Option<i64>, page_size: Option<i64>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("lifecycle_status", lifecycle_status, "form", true, false, None),
            QueryParameterSpec::new("provider_kind", provider_kind, "form", true, false, None),
            QueryParameterSpec::new("page", page, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&"/drive/sandbox_volumes".to_string()), &query);
        self.client.get(&path, None, None).await
    }

    /// Create a server sandbox volume
    pub async fn sandbox_volumes_create(&self, body: &CreateSandboxVolumeRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&"/drive/sandbox_volumes".to_string());
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Retrieve a server sandbox volume
    pub async fn sandbox_volumes_retrieve(&self, sandbox_id: &str) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Update a server sandbox volume
    pub async fn sandbox_volumes_update(&self, sandbox_id: &str, body: &UpdateSandboxVolumeRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false))));
        self.client.patch(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Delete a server sandbox volume
    pub async fn sandbox_volumes_delete(&self, sandbox_id: &str) -> Result<(), SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false))));
        self.client.delete(&path, None, None).await
    }

    /// List explicit sandbox grants
    pub async fn sandbox_grants_list(&self, sandbox_id: &str, page: Option<i64>, page_size: Option<i64>) -> Result<serde_json::Value, SdkworkError> {
        let query = build_query_string(&[
            QueryParameterSpec::new("page", page, "form", true, false, None),
            QueryParameterSpec::new("page_size", page_size, "form", true, false, None),
        ]);
        let path = append_query_string(backend_path(&format!("/drive/sandbox_volumes/{}/grants", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false)))), &query);
        self.client.get(&path, None, None).await
    }

    /// Create an explicit sandbox grant
    pub async fn sandbox_grants_create(&self, sandbox_id: &str, body: &CreateSandboxGrantRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}/grants", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false))));
        self.client.post(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Retrieve a sandbox grant
    pub async fn sandbox_grants_retrieve(&self, sandbox_id: &str, grant_id: &str) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}/grants/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false)), serialize_path_parameter(grant_id, PathParameterSpec::new("grantId", "simple", false))));
        self.client.get(&path, None, None).await
    }

    /// Update a sandbox grant
    pub async fn sandbox_grants_update(&self, sandbox_id: &str, grant_id: &str, body: &UpdateSandboxGrantRequest) -> Result<serde_json::Value, SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}/grants/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false)), serialize_path_parameter(grant_id, PathParameterSpec::new("grantId", "simple", false))));
        self.client.patch(&path, Some(body), None, None, Some("application/json")).await
    }

    /// Delete a sandbox grant
    pub async fn sandbox_grants_delete(&self, sandbox_id: &str, grant_id: &str) -> Result<(), SdkworkError> {
        let path = backend_path(&format!("/drive/sandbox_volumes/{}/grants/{}", serialize_path_parameter(sandbox_id, PathParameterSpec::new("sandboxId", "simple", false)), serialize_path_parameter(grant_id, PathParameterSpec::new("grantId", "simple", false))));
        self.client.delete(&path, None, None).await
    }

}

struct PathParameterSpec<'a> {
    name: &'a str,
    style: &'a str,
    explode: bool,
}

impl<'a> PathParameterSpec<'a> {
    fn new(name: &'a str, style: &'a str, explode: bool) -> Self {
        Self { name, style, explode }
    }
}

fn serialize_path_parameter<T: serde::Serialize>(value: T, spec: PathParameterSpec<'_>) -> String {
    let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    if value.is_null() {
        return String::new();
    }
    let style = if spec.style.is_empty() { "simple" } else { spec.style };
    match value {
        serde_json::Value::Array(values) => serialize_path_array(spec.name, &values, style, spec.explode),
        serde_json::Value::Object(values) => serialize_path_object(spec.name, &values, style, spec.explode),
        value => format!("{}{}", path_primitive_prefix(spec.name, style), percent_encode(&primitive_to_string(&value))),
    }
}

fn serialize_path_array(name: &str, values: &[serde_json::Value], style: &str, explode: bool) -> String {
    let serialized = values
        .iter()
        .filter(|value| !value.is_null())
        .map(|value| percent_encode(&primitive_to_string(value)))
        .collect::<Vec<_>>();
    if serialized.is_empty() {
        return path_prefix(name, style);
    }
    if style == "matrix" {
        if explode {
            return serialized.iter().map(|item| format!(";{}={}", name, item)).collect::<Vec<_>>().join("");
        }
        return format!(";{}={}", name, serialized.join(","));
    }
    let separator = if explode { "." } else { "," };
    format!("{}{}", path_prefix(name, style), serialized.join(separator))
}

fn serialize_path_object(
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    style: &str,
    explode: bool,
) -> String {
    let mut entries = Vec::new();
    let mut exploded = Vec::new();
    for (key, value) in values {
        if value.is_null() {
            continue;
        }
        let escaped_key = percent_encode(key);
        let escaped_value = percent_encode(&primitive_to_string(value));
        if explode {
            if style == "matrix" {
                exploded.push(format!(";{}={}", escaped_key, escaped_value));
            } else {
                exploded.push(format!("{}={}", escaped_key, escaped_value));
            }
        } else {
            entries.push(escaped_key);
            entries.push(escaped_value);
        }
    }
    if style == "matrix" {
        if explode {
            return exploded.join("");
        }
        return format!(";{}={}", name, entries.join(","));
    }
    if explode {
        let separator = if style == "label" { "." } else { "," };
        return format!("{}{}", path_prefix(name, style), exploded.join(separator));
    }
    format!("{}{}", path_prefix(name, style), entries.join(","))
}

fn path_prefix(name: &str, style: &str) -> String {
    match style {
        "label" => ".".to_string(),
        "matrix" => format!(";{}", name),
        _ => String::new(),
    }
}

fn path_primitive_prefix(name: &str, style: &str) -> String {
    if style == "matrix" {
        format!(";{}=", name)
    } else {
        path_prefix(name, style)
    }
}


struct QueryParameterSpec<'a> {
    name: &'a str,
    value: serde_json::Value,
    style: &'a str,
    explode: bool,
    allow_reserved: bool,
    content_type: Option<&'a str>,
}

impl<'a> QueryParameterSpec<'a> {
    fn new<T: serde::Serialize>(
        name: &'a str,
        value: T,
        style: &'a str,
        explode: bool,
        allow_reserved: bool,
        content_type: Option<&'a str>,
    ) -> Self {
        Self {
            name,
            value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
            style,
            explode,
            allow_reserved,
            content_type,
        }
    }
}

fn build_query_string(parameters: &[QueryParameterSpec<'_>]) -> String {
    let mut pairs = Vec::new();
    for parameter in parameters {
        append_serialized_parameter(&mut pairs, parameter);
    }
    pairs.join("&")
}

fn append_serialized_parameter(pairs: &mut Vec<String>, parameter: &QueryParameterSpec<'_>) {
    if parameter.value.is_null() {
        return;
    }
    if parameter.content_type.is_some() {
        pairs.push(format!(
            "{}={}",
            percent_encode(parameter.name),
            encode_query_value(&parameter.value.to_string(), parameter.allow_reserved)
        ));
        return;
    }

    let style = if parameter.style.is_empty() { "form" } else { parameter.style };
    match &parameter.value {
        serde_json::Value::Array(values) => append_array_parameter(pairs, parameter.name, values, style, parameter.explode, parameter.allow_reserved),
        serde_json::Value::Object(values) if style == "deepObject" => append_deep_object_parameter(pairs, parameter.name, values, parameter.allow_reserved),
        serde_json::Value::Object(values) => append_object_parameter(pairs, parameter.name, values, style, parameter.explode, parameter.allow_reserved),
        value => pairs.push(format!("{}={}", percent_encode(parameter.name), encode_query_value(&primitive_to_string(value), parameter.allow_reserved))),
    }
}

fn append_array_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &[serde_json::Value],
    style: &str,
    explode: bool,
    allow_reserved: bool,
) {
    let serialized = values.iter().filter(|value| !value.is_null()).map(primitive_to_string).collect::<Vec<_>>();
    if serialized.is_empty() {
        return;
    }
    if style == "form" && explode {
        for item in serialized {
            pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&item, allow_reserved)));
        }
        return;
    }
    pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&serialized.join(","), allow_reserved)));
}

fn append_object_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    style: &str,
    explode: bool,
    allow_reserved: bool,
) {
    let mut serialized = Vec::new();
    for (key, value) in values {
        if value.is_null() {
            continue;
        }
        if style == "form" && explode {
            pairs.push(format!("{}={}", percent_encode(key), encode_query_value(&primitive_to_string(value), allow_reserved)));
        } else {
            serialized.push(key.clone());
            serialized.push(primitive_to_string(value));
        }
    }
    if !serialized.is_empty() {
        pairs.push(format!("{}={}", percent_encode(name), encode_query_value(&serialized.join(","), allow_reserved)));
    }
}

fn append_deep_object_parameter(
    pairs: &mut Vec<String>,
    name: &str,
    values: &serde_json::Map<String, serde_json::Value>,
    allow_reserved: bool,
) {
    for (key, value) in values {
        if !value.is_null() {
            pairs.push(format!("{}={}", percent_encode(&format!("{}[{}]", name, key)), encode_query_value(&primitive_to_string(value), allow_reserved)));
        }
    }
}

fn encode_query_value(value: &str, allow_reserved: bool) -> String {
    let mut encoded = percent_encode(value);
    if !allow_reserved {
        return encoded;
    }
    for (escaped, reserved) in [
        ("%3A", ":"), ("%2F", "/"), ("%3F", "?"), ("%23", "#"),
        ("%5B", "["), ("%5D", "]"), ("%40", "@"), ("%21", "!"),
        ("%24", "$"), ("%26", "&"), ("%27", "'"), ("%28", "("),
        ("%29", ")"), ("%2A", "*"), ("%2B", "+"), ("%2C", ","),
        ("%3B", ";"), ("%3D", "="),
    ] {
        encoded = encoded.replace(escaped, reserved);
    }
    encoded
}

fn primitive_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        other => other.to_string(),
    }
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{:02X}", byte).chars().collect(),
        })
        .collect()
}
