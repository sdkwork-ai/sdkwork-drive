use sdkwork_drive_object_runtime::DriveObjectStoreRuntime;
use sdkwork_drive_storage_contract::{
    DeleteObjectRequest, DriveObjectLocator, DriveObjectStoreErrorKind,
};
use sdkwork_drive_workspace_service::infrastructure::sql::website_publishing_maintenance_store::SqlWebsitePublishingMaintenanceStore;
use sdkwork_drive_workspace_service::ports::website_publishing_maintenance::{
    DriveWebsitePublishingMaintenanceStore, WebsitePublishingMaintenanceResult,
};
use sqlx::PgPool;

const OBJECT_DELETE_BATCH_SIZE: i64 = 256;
const MAINTENANCE_OPERATOR_ID: &str = "sdkwork-drive-install-worker";

pub async fn cleanup_website_publishing(
    pool: &PgPool,
    work_limit: i64,
) -> Result<WebsitePublishingMaintenanceResult, String> {
    if work_limit <= 0 {
        return Err("website publishing cleanup work_limit must be positive".to_string());
    }
    let store = SqlWebsitePublishingMaintenanceStore::new(pool.clone());
    let object_runtime = DriveObjectStoreRuntime::new(pool.clone());
    let mut result = WebsitePublishingMaintenanceResult {
        expired_syncs: store
            .expire_stale_syncs(work_limit, MAINTENANCE_OPERATOR_ID)
            .await
            .map_err(service_error)?,
        ..WebsitePublishingMaintenanceResult::default()
    };
    let mut remaining_object_budget = work_limit;
    for _ in 0..work_limit {
        let Some(candidate) = store
            .claim_next_cleanup_candidate(MAINTENANCE_OPERATOR_ID)
            .await
            .map_err(service_error)?
        else {
            break;
        };
        if candidate.delete_tree {
            if remaining_object_budget == 0 {
                break;
            }
            let batch_limit = remaining_object_budget.min(OBJECT_DELETE_BATCH_SIZE);
            let objects = store
                .list_candidate_storage_objects(&candidate, batch_limit)
                .await
                .map_err(service_error)?;
            if !objects.is_empty() {
                for object in objects {
                    let object_store = object_runtime
                        .resolve(&object.storage_provider_id, object.storage_provider_version)
                        .await
                        .map_err(|error| {
                            format!(
                                "resolve provider {} for website cleanup failed: {error}",
                                object.storage_provider_id
                            )
                        })?;
                    let delete_result = object_store
                        .delete_object(DeleteObjectRequest {
                            locator: DriveObjectLocator {
                                bucket: object.bucket,
                                object_key: object.object_key,
                            },
                        })
                        .await;
                    if let Err(error) = delete_result {
                        if error.kind != DriveObjectStoreErrorKind::NotFound {
                            return Err(format!(
                                "delete website publishing object {} failed: {error}",
                                object.id
                            ));
                        }
                    }
                    if store
                        .mark_storage_object_deleted(
                            &candidate,
                            &object.id,
                            MAINTENANCE_OPERATOR_ID,
                        )
                        .await
                        .map_err(service_error)?
                    {
                        result.deleted_objects += 1;
                    }
                    remaining_object_budget -= 1;
                }
                if remaining_object_budget == 0 {
                    break;
                }
                continue;
            }
        }
        result.deleted_nodes += store
            .complete_cleanup_candidate(&candidate, MAINTENANCE_OPERATOR_ID)
            .await
            .map_err(service_error)?;
        result.completed_candidates += 1;
    }
    sdkwork_drive_observability::metrics::record_website_publishing_cleanup(
        result.expired_syncs,
        result.completed_candidates,
        result.deleted_objects,
        result.deleted_nodes,
    );
    Ok(result)
}

fn service_error(error: sdkwork_drive_workspace_service::DriveServiceError) -> String {
    format!("{error:?}")
}
