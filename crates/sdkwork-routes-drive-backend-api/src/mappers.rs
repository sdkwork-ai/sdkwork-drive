use crate::dto::{DownloadPackageItemResponse, LabelResponse, SpaceResponse};
use sdkwork_drive_workspace_service::domain::space::DriveSpace;
use sqlx::Row;

pub(crate) fn map_label_row(row: &sqlx::postgres::PgRow) -> LabelResponse {
    LabelResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        label_key: row.get("label_key"),
        display_name: row.get("display_name"),
        color: row.get("color"),
        description: row.get("description"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    }
}

pub(crate) fn map_download_package_row(row: &sqlx::postgres::PgRow) -> DownloadPackageItemResponse {
    DownloadPackageItemResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        package_name: row.get("package_name"),
        state: row.get("state"),
        storage_provider_id: row.get("storage_provider_id"),
        bucket: row.get("bucket"),
        archive_object_key: row.get("archive_object_key"),
        content_type: row.get("content_type"),
        file_count: row.get("file_count"),
        total_bytes: row.get("total_bytes"),
        archive_size_bytes: row.get("archive_size_bytes"),
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        error_message: row.get("error_message"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn map_space(space: DriveSpace) -> SpaceResponse {
    SpaceResponse {
        id: space.id,
        tenant_id: space.tenant_id,
        owner_subject_type: space.owner_subject_type,
        owner_subject_id: space.owner_subject_id,
        display_name: space.display_name,
        space_type: space.space_type.as_str().to_string(),
        lifecycle_status: space.lifecycle_status,
        version: space.version,
    }
}
