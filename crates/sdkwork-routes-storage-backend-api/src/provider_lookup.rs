use crate::error::{map_service_error, ProblemDetail};
use crate::state::AdminStorageState;
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_workspace_service::application::storage_provider_service::{
    DriveStorageProviderService, GetStorageProviderCommand,
};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProvider;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::DriveServiceError;

pub(crate) async fn get_provider(
    state: &AdminStorageState,
    tenant_id: &str,
    provider_id: &str,
) -> Result<DriveStorageProvider, (StatusCode, Json<ProblemDetail>)> {
    let service =
        DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()));
    let provider = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: provider_id.to_string(),
        })
        .await
        .map_err(map_service_error)?;
    // Tenant isolation choke point: every provider-addressing handler resolves
    // its row here, so a row owned by another tenant must be indistinguishable
    // from a missing one (404, never 403 — no existence oracle).
    if provider.tenant_id != tenant_id {
        return Err(map_service_error(DriveServiceError::NotFound(format!(
            "storage provider {provider_id} not found"
        ))));
    }
    Ok(provider)
}

pub(crate) async fn get_active_provider(
    state: &AdminStorageState,
    tenant_id: &str,
    provider_id: &str,
) -> Result<DriveStorageProvider, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_provider(state, tenant_id, provider_id).await?;
    if provider.status != "active" {
        return Err(map_service_error(DriveServiceError::Conflict(
            "storage provider must be active for object store operations".to_string(),
        )));
    }
    Ok(provider)
}

/// Resolve a provider for a **read**-only operation.
///
/// `active` and `disabled` are both acceptable: `disabled` retires a provider
/// from the write path while its bytes stay online, which is exactly the state a
/// migration drains. `deleted` is rejected because the provider no longer
/// promises its objects are readable at all.
///
/// The symmetry matters — using [`get_active_provider`] here would make a
/// retired provider unmigratable, so the one operation whose purpose is to
/// rescue those bytes would be the one operation that cannot touch them.
pub(crate) async fn get_provider_for_read(
    state: &AdminStorageState,
    tenant_id: &str,
    provider_id: &str,
) -> Result<DriveStorageProvider, (StatusCode, Json<ProblemDetail>)> {
    let provider = get_provider(state, tenant_id, provider_id).await?;
    match provider.status.as_str() {
        "active" | "disabled" => Ok(provider),
        other => Err(map_service_error(DriveServiceError::Conflict(format!(
            "storage provider is {other}; only an active or disabled provider can be read from"
        )))),
    }
}
