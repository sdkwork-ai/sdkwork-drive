use sqlx::PgConnection;

use crate::infrastructure::sql::next_drive_runtime_id;
use crate::ports::space_store::NewDriveSpace;
use crate::DriveServiceError;

pub(super) async fn provision_default_website_root_on_connection(
    connection: &mut PgConnection,
    space: &NewDriveSpace,
) -> Result<(), DriveServiceError> {
    if space.space_type != "website" {
        return Err(DriveServiceError::Validation(
            "default WebsiteRoot requires a website space".to_string(),
        ));
    }

    let root_node_id = next_drive_runtime_id("website root node")?.to_string();
    let website_root_id = next_drive_runtime_id("WebsiteRoot")?.to_string();
    let website_root_uuid = uuid::Uuid::new_v4().to_string();
    let generation_id = next_drive_runtime_id("WebsiteRoot generation")?.to_string();

    sqlx::query(
        "INSERT INTO dr_drive_node (
            id, tenant_id, space_id, space_type, parent_node_id, node_type, node_name,
            content_state, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, 'website', NULL, 'folder', $4, 'ready', 'active', 1, $5, $5)",
    )
    .bind(&root_node_id)
    .bind(&space.tenant_id)
    .bind(&space.id)
    .bind(&space.display_name)
    .bind(&space.created_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("insert website root node failed: {error}"))
    })?;

    sqlx::query(
        "INSERT INTO dr_drive_website_root (
            id, uuid, tenant_id, space_id, root_key, display_name,
            source_root_mode, selected_folder_node_id, selector_key,
            content_mode, active_node_id, active_generation, root_status,
            last_switch_by, version, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, 'default', 'Website root',
            'space_root', NULL, 'space_root',
            'live_tree', $5, 1, 'active',
            $6, 1, $6, $6
         )",
    )
    .bind(&website_root_id)
    .bind(&website_root_uuid)
    .bind(&space.tenant_id)
    .bind(&space.id)
    .bind(&root_node_id)
    .bind(&space.created_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("insert default WebsiteRoot failed: {error}"))
    })?;

    sqlx::query(
        "INSERT INTO dr_drive_website_root_generation (
            id, tenant_id, website_root_id, generation_no, root_node_id,
            source_sync_id, manifest_sha256, file_count, total_bytes,
            generation_status, activated_by
         ) VALUES ($1, $2, $3, 1, $4, NULL, NULL, 0, 0, 'current', $5)",
    )
    .bind(&generation_id)
    .bind(&space.tenant_id)
    .bind(&website_root_id)
    .bind(&root_node_id)
    .bind(&space.created_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "insert default WebsiteRoot generation failed: {error}"
        ))
    })?;

    sqlx::query(
        "INSERT INTO dr_drive_space_website_profile (
            space_id, tenant_id, project_key, default_root_id,
            case_collision_policy, retained_generation_count, sync_policy,
            profile_status, version, created_by, updated_by
         ) VALUES ($1, $2, $3, $4, 'reject', 3, 'ordinary', 'active', 1, $5, $5)",
    )
    .bind(&space.id)
    .bind(&space.tenant_id)
    .bind(&space.id)
    .bind(&website_root_id)
    .bind(&space.created_by)
    .execute(&mut *connection)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!("insert website space profile failed: {error}"))
    })?;

    Ok(())
}
