use crate::dto::*;
use chrono::{DateTime, SecondsFormat, Utc};
use sdkwork_drive_workspace_service::domain::space::DriveSpace;
use sqlx::Row;

pub(crate) fn map_node_row(row: &sqlx::postgres::PgRow) -> DriveNodeResponse {
    DriveNodeResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        space_type: row.get("space_type"),
        parent_node_id: row.get("parent_node_id"),
        shortcut_target_node_id: row.get("shortcut_target_node_id"),
        node_type: row.get("node_type"),
        node_name: row.get("node_name"),
        scene: row.get("scene"),
        source: row.get("source"),
        content_state: row.get("content_state"),
        file_extension: row.try_get("file_extension").ok().flatten(),
        content_type: row.try_get("head_content_type").ok().flatten(),
        content_type_group: row.try_get("head_content_type_group").ok().flatten(),
        content_length: row.try_get("head_content_length").ok().flatten(),
        folder_color: None,
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_at: normalize_timestamp_text(row.get("created_at")),
        updated_at: normalize_timestamp_text(row.get("updated_at")),
    }
}

fn normalize_timestamp_text(value: String) -> String {
    let trimmed = value.trim();
    let with_t = trimmed.replacen(' ', "T", 1);
    let with_offset = normalize_offset_suffix(&with_t);
    let candidate = if with_offset.ends_with('Z') || has_timezone_suffix(&with_offset) {
        with_offset
    } else {
        format!("{with_offset}Z")
    };

    DateTime::parse_from_rfc3339(&candidate)
        .map(|timestamp| {
            timestamp
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::AutoSi, true)
        })
        .unwrap_or_else(|_| trimmed.to_string())
}

fn has_timezone_suffix(value: &str) -> bool {
    let Some(time_start) = value.find('T') else {
        return false;
    };
    value[time_start..]
        .rfind('+')
        .or_else(|| value[time_start..].rfind('-'))
        .is_some_and(|index| index > 0)
}

fn normalize_offset_suffix(value: &str) -> String {
    let Some(time_start) = value.find('T') else {
        return value.to_string();
    };
    let Some(relative_index) = value[time_start..]
        .rfind('+')
        .or_else(|| value[time_start..].rfind('-'))
    else {
        return value.to_string();
    };
    let offset_index = time_start + relative_index;
    let offset = &value[offset_index..];
    match offset.len() {
        3 => format!("{}{}:00", &value[..offset_index], offset),
        5 if !offset.contains(':') => {
            format!(
                "{}{}:{}",
                &value[..offset_index],
                &offset[..3],
                &offset[3..]
            )
        }
        _ => value.to_string(),
    }
}

pub(crate) fn map_space_response(space: DriveSpace) -> CreateSpaceResponse {
    CreateSpaceResponse {
        id: space.id,
        tenant_id: space.tenant_id,
        owner_subject_type: space.owner_subject_type,
        owner_subject_id: space.owner_subject_id,
        display_name: space.display_name,
        space_type: space.space_type.as_str().to_string(),
        presentation_icon: space.presentation_icon,
        presentation_color: space.presentation_color,
        description: space.description,
        lifecycle_status: space.lifecycle_status,
        version: space.version,
        created_by: space.created_by,
    }
}

pub(crate) fn map_permission_row(row: &sqlx::postgres::PgRow) -> PermissionResponse {
    let inherited: i64 = row.get("inherited");
    PermissionResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        subject_type: row.get("subject_type"),
        subject_id: row.get("subject_id"),
        role: row.get("role"),
        inherited: inherited != 0,
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    }
}

pub(crate) fn map_file_version_row(row: &sqlx::postgres::PgRow) -> FileVersionResponse {
    FileVersionResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        storage_object_id: row.try_get("storage_object_id").ok().flatten(),
        version_no: row.get("version_no"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        lifecycle_status: row.get("lifecycle_status"),
        created_at: row.get("created_at"),
    }
}

pub(crate) fn map_share_link_row(row: &sqlx::postgres::PgRow) -> ShareLinkResponse {
    ShareLinkResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        role: row.get("role"),
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        download_limit: row.get("download_limit"),
        download_count: row.get("download_count"),
        access_code_required: row
            .try_get::<Option<String>, _>("access_code_hash")
            .ok()
            .flatten()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    }
}

pub(crate) fn map_share_link_record(row: &sqlx::postgres::PgRow) -> ShareLinkRecord {
    ShareLinkRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        role: row.get("role"),
        expires_at_epoch_ms: row.get("expires_at_epoch_ms"),
        download_limit: row.get("download_limit"),
        download_count: row.get("download_count"),
        access_code_hash: row.try_get("access_code_hash").ok().flatten(),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_by: row.get("created_by"),
    }
}

pub(crate) fn map_comment_row(row: &sqlx::postgres::PgRow) -> CommentRecord {
    let resolved: i64 = row.get("resolved");
    CommentRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        content: row.get("content"),
        anchor: row.get("anchor"),
        resolved: resolved != 0,
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn map_comment_reply_row(row: &sqlx::postgres::PgRow) -> CommentReplyRecord {
    CommentReplyRecord {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        comment_id: row.get("comment_id"),
        content: row.get("content"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        created_by: row.get("created_by"),
        updated_by: row.get("updated_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn map_node_property_row(row: &sqlx::postgres::PgRow) -> NodePropertyResponse {
    NodePropertyResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        property_key: row.get("property_key"),
        property_value: row.get("property_value"),
        visibility: row.get("visibility"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    }
}

pub(crate) fn map_label_summary_row(row: &sqlx::postgres::PgRow) -> LabelSummaryResponse {
    LabelSummaryResponse {
        id: row.get("label_id"),
        tenant_id: row.get("tenant_id"),
        label_key: row.get("label_key"),
        display_name: row.get("display_name"),
        color: row.get("color"),
        description: row.get("description"),
        lifecycle_status: row.get("label_lifecycle_status"),
        version: row.get("label_version"),
    }
}

pub(crate) fn map_node_label_row(row: &sqlx::postgres::PgRow) -> NodeLabelResponse {
    NodeLabelResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        node_id: row.get("node_id"),
        label_id: row.get("label_id"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
        label: map_label_summary_row(row),
    }
}

pub(crate) fn map_watch_channel_row(row: &sqlx::postgres::PgRow) -> DriveWatchChannelResponse {
    DriveWatchChannelResponse {
        id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        node_id: row.get("node_id"),
        resource_type: row.get("resource_type"),
        resource_id: row.get("resource_id"),
        channel_type: row.get("channel_type"),
        address: row.get("address"),
        expiration_epoch_ms: row.get("expiration_epoch_ms"),
        lifecycle_status: row.get("lifecycle_status"),
        version: row.get("version"),
    }
}

pub(crate) struct NodeCapabilitiesInput<'a> {
    pub tenant_id: String,
    pub node_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub role: String,
    pub source: String,
    pub permission_id: Option<String>,
    pub inherited: bool,
    pub inherited_from_node_id: Option<String>,
    pub node_lifecycle_status: &'a str,
}

pub(crate) fn build_node_capabilities_response(
    input: NodeCapabilitiesInput<'_>,
) -> NodeCapabilitiesResponse {
    let can_read = matches!(
        input.role.as_str(),
        "reader" | "commenter" | "writer" | "owner"
    );
    let can_comment = matches!(input.role.as_str(), "commenter" | "writer" | "owner");
    let can_write = matches!(input.role.as_str(), "writer" | "owner");
    let can_delete = input.role == "owner";
    let can_manage_permissions = input.role == "owner";
    let can_manage_versions = matches!(input.role.as_str(), "writer" | "owner");
    if input.node_lifecycle_status == "trashed" {
        return NodeCapabilitiesResponse {
            tenant_id: input.tenant_id,
            node_id: input.node_id,
            subject_type: input.subject_type,
            subject_id: input.subject_id,
            role: input.role,
            source: input.source,
            permission_id: input.permission_id,
            inherited: input.inherited,
            inherited_from_node_id: input.inherited_from_node_id,
            can_read,
            can_comment: false,
            can_write: false,
            can_download: false,
            can_copy: false,
            can_move: false,
            can_trash: false,
            can_restore: can_write,
            can_delete,
            can_share: false,
            can_manage_permissions: false,
            can_manage_versions: false,
        };
    }
    NodeCapabilitiesResponse {
        tenant_id: input.tenant_id,
        node_id: input.node_id,
        subject_type: input.subject_type,
        subject_id: input.subject_id,
        role: input.role,
        source: input.source,
        permission_id: input.permission_id,
        inherited: input.inherited,
        inherited_from_node_id: input.inherited_from_node_id,
        can_read,
        can_comment,
        can_write,
        can_download: can_read,
        can_copy: can_read,
        can_move: can_write,
        can_trash: can_write,
        can_restore: can_write,
        can_delete,
        can_share: can_write,
        can_manage_permissions,
        can_manage_versions,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_timestamp_text;

    #[test]
    fn normalizes_naive_sql_timestamp_to_rfc3339_utc() {
        assert_eq!(
            normalize_timestamp_text("2026-06-04 12:00:00".to_string()),
            "2026-06-04T12:00:00Z"
        );
    }

    #[test]
    fn normalizes_postgres_short_offset_to_rfc3339_utc() {
        assert_eq!(
            normalize_timestamp_text("2026-06-17 12:34:56.789+00".to_string()),
            "2026-06-17T12:34:56.789Z"
        );
    }

    #[test]
    fn normalizes_rfc3339_offset_to_utc() {
        assert_eq!(
            normalize_timestamp_text("2026-06-17T08:34:56-04:00".to_string()),
            "2026-06-17T12:34:56Z"
        );
    }
}
