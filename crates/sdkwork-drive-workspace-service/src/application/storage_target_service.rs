//! Tenant-scoped resolution of where a new Drive object version is written.
//!
//! This is the routing rule the Drive app API used to own inline. It is a
//! service because three paths now share it: the app API's upload-session and
//! uploader prepare handlers, the in-process upload-session service, and any
//! sibling module that completes a stored version. Keeping one implementation
//! means a provider-binding fix cannot land on one path and miss another.

use sqlx::PgPool;
use sqlx::Row;

use crate::application::storage_key_service::{
    BuildStorageObjectKeyCommand, DriveStorageKeyService,
};
use crate::application::upload_session_rules::{
    default_storage_root_prefix, join_storage_root_prefix,
};
use crate::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use crate::ports::storage_provider_store::{DriveStorageProviderLookup, DriveStorageProviderStore};
use crate::DriveServiceError;

/// The object locator a completed version will occupy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveStorageTarget {
    pub provider_id: String,
    pub bucket: String,
    pub object_key: String,
}

/// Inputs that select a storage target.
#[derive(Debug, Clone, Copy)]
pub struct ResolveDriveStorageTargetCommand<'a> {
    /// Tenant the node, space and provider bindings all belong to.
    pub tenant_id: &'a str,
    /// Space whose binding chain supplies the default provider.
    pub space_id: &'a str,
    /// Provider the caller named explicitly, if any.
    pub requested_provider_id: Option<&'a str>,
    /// Bucket the caller named explicitly, if any.
    pub requested_bucket: Option<&'a str>,
    /// Node the version belongs to.
    pub node_id: &'a str,
    /// Identifier the object key embeds (session id or version row id).
    pub object_id: &'a str,
    /// Version number the object key embeds.
    pub version_no: i64,
}

/// Resolve where a new object version should be written.
///
/// Three routing inputs are honoured, in decreasing priority:
///
/// 1. `requested_provider_id` - a business that knows which provider it wants.
///    It is verified to belong to the caller's tenant and to be `active` before
///    use, so naming a provider is not a way to reach across tenants or write
///    into a retired one.
/// 2. `requested_bucket` - resolves through the tenant-scoped bucket lookup.
/// 3. Otherwise the provider bound to the space / space type / tenant.
///
/// # Errors
///
/// - `NotFound` when an explicitly named provider does not belong to the tenant.
/// - `Conflict` when the named/named-by-bucket provider is inactive or absent,
///   or when two active providers of the tenant address the same bucket.
/// - `Validation` when no binding exists and no bucket was supplied, or the
///   resolved root prefix and key cannot form a valid object key.
/// - `Internal` when the binding queries fail.
pub async fn resolve_storage_target(
    pool: &PgPool,
    command: ResolveDriveStorageTargetCommand<'_>,
) -> Result<DriveStorageTarget, DriveServiceError> {
    let default_target = if let Some(provider_id) = command
        .requested_provider_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        resolve_explicit_provider_target(pool, command.tenant_id, command.space_id, provider_id)
            .await?
    } else {
        match command
            .requested_bucket
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(bucket) => {
                let store = SqlStorageProviderStore::new(pool.clone());
                let provider = match store
                    .find_active_storage_provider_by_bucket(command.tenant_id, bucket)
                    .await?
                {
                    DriveStorageProviderLookup::Found(provider) => provider,
                    DriveStorageProviderLookup::Missing => {
                        return Err(missing_active_provider_error(bucket))
                    }
                    DriveStorageProviderLookup::Ambiguous(provider_ids) => {
                        return Err(ambiguous_provider_error(
                            command.tenant_id,
                            bucket,
                            &provider_ids,
                        ))
                    }
                };
                DefaultStorageProviderTarget {
                    provider_id: provider.id,
                    bucket: bucket.to_string(),
                    storage_root_prefix: default_storage_root_prefix(
                        command.tenant_id,
                        Some(command.space_id),
                    ),
                }
            }
            None => {
                resolve_default_provider_target(pool, command.tenant_id, command.space_id).await?
            }
        }
    };
    let standard_object_key =
        DriveStorageKeyService::build_object_key(BuildStorageObjectKeyCommand {
            tenant_id: command.tenant_id,
            space_id: command.space_id,
            node_id: command.node_id,
            version_no: command.version_no,
            object_id: command.object_id,
        })
        .map_err(DriveServiceError::Validation)?;
    let object_key =
        join_storage_root_prefix(&default_target.storage_root_prefix, &standard_object_key)?;
    Ok(DriveStorageTarget {
        provider_id: default_target.provider_id,
        bucket: default_target.bucket,
        object_key,
    })
}

/// Resolve a default storage provider target for a tenant/space pair.
///
/// # Errors
///
/// Returns `Validation` when no binding chain resolves a provider, and
/// `Internal` when the binding query fails.
pub async fn resolve_default_provider_target(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
) -> Result<DefaultStorageProviderTarget, DriveServiceError> {
    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.space_id=$2
           AND binding.purpose='primary'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "resolve space dr_drive_storage_provider_binding failed: {error}"
        ))
    })?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }

    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         INNER JOIN dr_drive_space space ON space.tenant_id = binding.tenant_id
           AND space.id = $2
           AND space.lifecycle_status = 'active'
         WHERE binding.tenant_id=$1
           AND binding.binding_scope='space_type'
           AND binding.purpose = space.space_type
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "resolve space_type dr_drive_storage_provider_binding failed: {error}"
        ))
    })?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }

    let row = sqlx::query(
        "SELECT provider.id AS provider_id, provider.bucket, binding.storage_root_prefix
         FROM dr_drive_storage_provider_binding binding
         INNER JOIN dr_drive_storage_provider provider ON provider.id = binding.provider_id
         WHERE binding.tenant_id=$1
           AND binding.space_id IS NULL
           AND binding.purpose='primary'
           AND binding.lifecycle_status='active'
           AND provider.status='active'
         LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        DriveServiceError::Internal(format!(
            "resolve tenant dr_drive_storage_provider_binding failed: {error}"
        ))
    })?;
    if let Some(row) = row {
        return Ok(DefaultStorageProviderTarget {
            provider_id: row.get("provider_id"),
            bucket: row.get("bucket"),
            storage_root_prefix: row.get("storage_root_prefix"),
        });
    }
    Err(DriveServiceError::Validation(
        "bucket is required when no default storage provider binding exists".to_string(),
    ))
}

/// Resolve a provider the caller named explicitly on the write path.
///
/// # Errors
///
/// `NotFound` when the provider does not exist or belongs to another tenant
/// (cross-tenant existence is deliberately not confirmed), and `Conflict` when
/// the provider exists but is not active.
async fn resolve_explicit_provider_target(
    pool: &PgPool,
    tenant_id: &str,
    space_id: &str,
    provider_id: &str,
) -> Result<DefaultStorageProviderTarget, DriveServiceError> {
    let store = SqlStorageProviderStore::new(pool.clone());
    let provider = store
        .find_storage_provider(provider_id)
        .await?
        .filter(|provider| provider.tenant_id == tenant_id)
        .ok_or_else(|| DriveServiceError::NotFound("storage provider not found".to_string()))?;
    if provider.status != "active" {
        return Err(missing_active_provider_error(&provider.bucket));
    }
    Ok(DefaultStorageProviderTarget {
        provider_id: provider.id,
        bucket: provider.bucket,
        storage_root_prefix: default_storage_root_prefix(tenant_id, Some(space_id)),
    })
}

/// A resolved provider binding before the object key is built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultStorageProviderTarget {
    pub provider_id: String,
    pub bucket: String,
    pub storage_root_prefix: String,
}

/// Error for a bucket that has no active provider.
///
/// The message is the one the app API already emits, so delegating its
/// resolution here keeps every HTTP error body identical.
pub fn missing_active_provider_error(bucket: &str) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "active storage provider is required for bucket {bucket} to sign object store URLs"
    ))
}

/// Error for a `(tenant, bucket)` pair claimed by more than one active provider.
///
/// Failing loudly is the point: silently choosing one would publish objects
/// under a key that another active provider also claims, producing two copies
/// of the same logical object and a read that depends on which row sorted first.
pub fn ambiguous_provider_error(
    tenant_id: &str,
    bucket: &str,
    provider_ids: &[String],
) -> DriveServiceError {
    DriveServiceError::Conflict(format!(
        "bucket {bucket} of tenant {tenant_id} is addressed by {} active storage providers ({}); \
         disable or repoint all but one before writing to this bucket",
        provider_ids.len(),
        provider_ids.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::missing_active_provider_error;

    #[test]
    fn missing_provider_error_names_the_bucket() {
        let error = missing_active_provider_error("media-bucket");
        assert_eq!(
            error.message(),
            "active storage provider is required for bucket media-bucket to sign object store URLs"
        );
    }
}
