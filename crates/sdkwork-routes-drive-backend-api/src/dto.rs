use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateLabelRequest {
    pub(crate) id: String,
    pub(crate) label_key: String,
    pub(crate) display_name: String,
    pub(crate) color: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateLabelRequest {
    pub(crate) display_name: Option<String>,
    pub(crate) color: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LabelListQuery {
    pub(crate) lifecycle_status: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LabelResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) label_key: String,
    pub(crate) display_name: String,
    pub(crate) color: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListAuditEventsQuery {
    pub(crate) action: Option<String>,
    pub(crate) resource_type: Option<String>,
    pub(crate) resource_id: Option<String>,
    #[serde(rename = "correlationId")]
    pub(crate) request_id: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) page: Option<u32>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuditEventItemResponse {
    pub(crate) id: i64,
    pub(crate) tenant_id: String,
    pub(crate) action: String,
    pub(crate) resource_type: String,
    pub(crate) resource_id: String,
    pub(crate) operator_id: String,
    #[serde(rename = "correlationId")]
    pub(crate) request_id: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SweepObjectStoreRequest {
    pub(crate) dry_run: bool,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SweepUploadSessionsRequest {
    pub(crate) now_epoch_ms: i64,
    pub(crate) dry_run: bool,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SweepResponse {
    pub(crate) scanned_count: i64,
    pub(crate) affected_count: i64,
    pub(crate) dry_run: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListMaintenanceJobsQuery {
    pub(crate) job_type: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) operator_id: Option<String>,
    pub(crate) page: Option<u32>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MaintenanceJobItemResponse {
    pub(crate) id: i64,
    pub(crate) job_type: String,
    pub(crate) status: String,
    pub(crate) dry_run: bool,
    pub(crate) scanned_count: i64,
    pub(crate) affected_count: i64,
    pub(crate) operator_id: String,
    #[serde(rename = "correlationId")]
    pub(crate) request_id: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListDownloadPackagesQuery {
    pub(crate) state: Option<String>,
    pub(crate) page: Option<u32>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadPackageItemResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) package_name: String,
    pub(crate) state: String,
    pub(crate) storage_provider_id: String,
    pub(crate) bucket: String,
    pub(crate) archive_object_key: String,
    pub(crate) content_type: String,
    pub(crate) file_count: i64,
    pub(crate) total_bytes: i64,
    pub(crate) archive_size_bytes: i64,
    pub(crate) expires_at_epoch_ms: i64,
    pub(crate) error_message: Option<String>,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListSpacesQuery {
    pub(crate) owner_subject_type: Option<String>,
    pub(crate) owner_subject_id: Option<String>,
    pub(crate) space_type: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpaceResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) owner_subject_type: String,
    pub(crate) owner_subject_id: String,
    pub(crate) display_name: String,
    pub(crate) space_type: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuotaQuery {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuotaResponse {
    pub(crate) tenant_id: String,
    pub(crate) total_bytes: i64,
    pub(crate) object_count: i64,
    pub(crate) quota_bytes: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateQuotaPolicyRequest {
    pub(crate) quota_bytes: Option<i64>,
    pub(crate) clear_tenant_policy: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OffsetPage {
    pub(crate) limit: i64,
    pub(crate) offset: i64,
}
