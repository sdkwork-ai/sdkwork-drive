use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::domain::resource_resolution::{
    DriveResourceContentLocator, DriveResourceScopeKind, ResolvedDriveResource,
};
use crate::ports::resource_resolution_store::{DriveResourceResolutionStore, ResolveDriveResource};
use crate::DriveServiceError;

#[derive(Debug, Clone)]
pub struct SqlResourceResolutionStore {
    pool: PgPool,
}

impl SqlResourceResolutionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Clone)]
struct ResolvedScope {
    space_id: String,
    root_node_id: String,
    generation: i64,
}

#[async_trait]
impl DriveResourceResolutionStore for SqlResourceResolutionStore {
    async fn resolve(
        &self,
        request: &ResolveDriveResource,
    ) -> Result<ResolvedDriveResource, DriveServiceError> {
        let scope = self.resolve_scope(request).await?;
        let node_id = self.resolve_node(request, &scope).await?;
        let row = self.resolve_version(request, &scope, &node_id).await?;
        map_resource(row, request, scope.generation)
    }
}

impl SqlResourceResolutionStore {
    async fn resolve_scope(
        &self,
        request: &ResolveDriveResource,
    ) -> Result<ResolvedScope, DriveServiceError> {
        match request.scope_kind {
            DriveResourceScopeKind::WebsiteRoot => self.resolve_website_root(request).await,
            DriveResourceScopeKind::RootScopeSubscription => {
                self.resolve_root_scope_subscription(request).await
            }
        }
    }

    async fn resolve_website_root(
        &self,
        request: &ResolveDriveResource,
    ) -> Result<ResolvedScope, DriveServiceError> {
        let row = if let Some(generation) = request.pinned_generation {
            sqlx::query(
                "SELECT root.space_id,
                        generation.root_node_id,
                        generation.generation_no
                 FROM dr_drive_website_root root
                 INNER JOIN dr_drive_website_root_generation generation
                    ON generation.tenant_id=root.tenant_id
                   AND generation.website_root_id=root.id
                 INNER JOIN dr_drive_space space_row
                    ON space_row.tenant_id=root.tenant_id
                   AND space_row.id=root.space_id
                   AND space_row.space_type='website'
                   AND space_row.lifecycle_status='active'
                 INNER JOIN dr_drive_node root_node
                    ON root_node.tenant_id=root.tenant_id
                   AND root_node.space_id=root.space_id
                   AND root_node.id=generation.root_node_id
                   AND root_node.node_type='folder'
                   AND root_node.lifecycle_status='active'
                 WHERE root.tenant_id=$1
                   AND root.uuid=$2
                   AND root.root_status='active'
                   AND generation.generation_no=$3
                   AND generation.generation_status IN ('current', 'retained')",
            )
            .bind(&request.tenant_id)
            .bind(&request.scope_uuid)
            .bind(generation)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT root.space_id, root.active_node_id AS root_node_id,
                        root.active_generation AS generation_no
                 FROM dr_drive_website_root root
                 INNER JOIN dr_drive_space space_row
                    ON space_row.tenant_id=root.tenant_id
                   AND space_row.id=root.space_id
                   AND space_row.space_type='website'
                   AND space_row.lifecycle_status='active'
                 INNER JOIN dr_drive_node root_node
                    ON root_node.tenant_id=root.tenant_id
                   AND root_node.space_id=root.space_id
                   AND root_node.id=root.active_node_id
                   AND root_node.node_type='folder'
                   AND root_node.lifecycle_status='active'
                 WHERE root.tenant_id=$1 AND root.uuid=$2 AND root.root_status='active'",
            )
            .bind(&request.tenant_id)
            .bind(&request.scope_uuid)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|error| {
            DriveServiceError::Internal(format!("resolve WebsiteRoot scope failed: {error}"))
        })?;

        map_scope(row).ok_or_else(|| {
            DriveServiceError::NotFound("active WebsiteRoot scope was not found".to_string())
        })
    }

    async fn resolve_root_scope_subscription(
        &self,
        request: &ResolveDriveResource,
    ) -> Result<ResolvedScope, DriveServiceError> {
        let row = sqlx::query(
            "SELECT subscription.space_id, subscription.root_node_id,
                    subscription.version AS generation_no
             FROM dr_drive_root_scope_subscription subscription
             INNER JOIN dr_drive_space space_row
                ON space_row.tenant_id=subscription.tenant_id
               AND space_row.id=subscription.space_id
               AND space_row.space_type='knowledge_base'
               AND space_row.lifecycle_status='active'
             INNER JOIN dr_drive_node root_node
                ON root_node.tenant_id=subscription.tenant_id
               AND root_node.space_id=subscription.space_id
               AND root_node.id=subscription.root_node_id
               AND root_node.space_type='knowledge_base'
               AND root_node.node_type='folder'
               AND root_node.lifecycle_status='active'
             WHERE subscription.tenant_id=$1
               AND subscription.uuid=$2
               AND subscription.consumer_kind='knowledgebase_raw'
               AND subscription.scope_status='active'",
        )
        .bind(&request.tenant_id)
        .bind(&request.scope_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            DriveServiceError::Internal(format!("resolve root scope subscription failed: {error}"))
        })?;
        let scope = map_scope(row).ok_or_else(|| {
            DriveServiceError::NotFound("active root scope subscription was not found".to_string())
        })?;
        if request
            .pinned_generation
            .is_some_and(|generation| generation != scope.generation)
        {
            return Err(DriveServiceError::Conflict(
                "root scope subscription generation is no longer current".to_string(),
            ));
        }
        Ok(scope)
    }

    async fn resolve_node(
        &self,
        request: &ResolveDriveResource,
        scope: &ResolvedScope,
    ) -> Result<String, DriveServiceError> {
        let segments = request.relative_path.split('/').collect::<Vec<_>>();
        let mut parent_node_id = scope.root_node_id.clone();
        for (index, segment) in segments.iter().enumerate() {
            let row = sqlx::query(
                "SELECT id, node_type, content_state
                 FROM dr_drive_node
                 WHERE tenant_id=$1
                   AND space_id=$2
                   AND parent_node_id=$3
                   AND node_name=$4
                   AND lifecycle_status='active'
                 LIMIT 1",
            )
            .bind(&request.tenant_id)
            .bind(&scope.space_id)
            .bind(&parent_node_id)
            .bind(segment)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("resolve Drive path segment failed: {error}"))
            })?;
            let Some(row) = row else {
                return Err(DriveServiceError::NotFound(
                    "published Drive resource was not found".to_string(),
                ));
            };
            let node_id: String = row.get("id");
            let node_type: String = row.get("node_type");
            let content_state: String = row.get("content_state");
            let is_last = index + 1 == segments.len();
            if (!is_last && node_type != "folder")
                || (is_last && (node_type != "file" || content_state != "ready"))
            {
                return Err(DriveServiceError::NotFound(
                    "published Drive resource was not found".to_string(),
                ));
            }
            parent_node_id = node_id;
        }
        Ok(parent_node_id)
    }

    async fn resolve_version(
        &self,
        request: &ResolveDriveResource,
        scope: &ResolvedScope,
        node_id: &str,
    ) -> Result<PgRow, DriveServiceError> {
        let row = if let Some(version_id) = request.pinned_node_version_id.as_deref() {
            sqlx::query(
                "SELECT node.id AS node_id,
                        version.id AS node_version_id,
                        version.version_no,
                        version.checksum_sha256_hex,
                        version.content_type,
                        version.content_length,
                        CAST(version.updated_at AS TEXT) AS last_modified,
                        object.storage_provider_id,
                        provider.version AS storage_provider_version,
                        object.bucket,
                        object.object_key
                 FROM dr_drive_node node
                 INNER JOIN dr_drive_node_version version
                    ON version.tenant_id=node.tenant_id
                   AND version.space_id=node.space_id
                   AND version.node_id=node.id
                   AND version.lifecycle_status='active'
                 INNER JOIN dr_drive_storage_object object
                    ON object.tenant_id=version.tenant_id
                   AND object.node_id=version.node_id
                   AND object.id=version.storage_object_id
                   AND object.lifecycle_status='active'
                 INNER JOIN dr_drive_storage_provider provider
                    ON provider.id=object.storage_provider_id
                   AND provider.status='active'
                 WHERE node.tenant_id=$1
                   AND node.space_id=$2
                   AND node.id=$3
                   AND node.lifecycle_status='active'
                   AND node.node_type='file'
                   AND node.content_state='ready'
                   AND version.id=$4",
            )
            .bind(&request.tenant_id)
            .bind(&scope.space_id)
            .bind(node_id)
            .bind(version_id)
            .fetch_optional(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT node.id AS node_id,
                        version.id AS node_version_id,
                        version.version_no,
                        version.checksum_sha256_hex,
                        version.content_type,
                        version.content_length,
                        CAST(version.updated_at AS TEXT) AS last_modified,
                        object.storage_provider_id,
                        provider.version AS storage_provider_version,
                        object.bucket,
                        object.object_key
                 FROM dr_drive_node node
                 INNER JOIN dr_drive_node_version version
                    ON version.tenant_id=node.tenant_id
                   AND version.space_id=node.space_id
                   AND version.node_id=node.id
                   AND version.version_no=node.head_version_no
                   AND version.lifecycle_status='active'
                 INNER JOIN dr_drive_storage_object object
                    ON object.tenant_id=version.tenant_id
                   AND object.node_id=version.node_id
                   AND object.id=version.storage_object_id
                   AND object.lifecycle_status='active'
                 INNER JOIN dr_drive_storage_provider provider
                    ON provider.id=object.storage_provider_id
                   AND provider.status='active'
                 WHERE node.tenant_id=$1
                   AND node.space_id=$2
                   AND node.id=$3
                   AND node.lifecycle_status='active'
                   AND node.node_type='file'
                   AND node.content_state='ready'",
            )
            .bind(&request.tenant_id)
            .bind(&scope.space_id)
            .bind(node_id)
            .fetch_optional(&self.pool)
            .await
        }
        .map_err(|error| {
            DriveServiceError::Internal(format!("resolve Drive resource version failed: {error}"))
        })?;
        row.ok_or_else(|| {
            DriveServiceError::NotFound("eligible Drive resource version was not found".to_string())
        })
    }
}

fn map_scope(row: Option<PgRow>) -> Option<ResolvedScope> {
    row.map(|row| ResolvedScope {
        space_id: row.get("space_id"),
        root_node_id: row.get("root_node_id"),
        generation: row.get("generation_no"),
    })
}

fn map_resource(
    row: PgRow,
    request: &ResolveDriveResource,
    scope_generation: i64,
) -> Result<ResolvedDriveResource, DriveServiceError> {
    let content_length: i64 = row.get("content_length");
    if content_length < 0 {
        return Err(DriveServiceError::Internal(
            "resolved Drive resource has a negative content length".to_string(),
        ));
    }
    Ok(ResolvedDriveResource {
        scope_kind: request.scope_kind,
        scope_uuid: request.scope_uuid.clone(),
        scope_generation,
        relative_path: request.relative_path.clone(),
        resource_type: "FILE".to_string(),
        node_id: row.get("node_id"),
        node_version_id: row.get("node_version_id"),
        version_no: row.get("version_no"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
        content_type: row.get("content_type"),
        content_length,
        last_modified: row.get("last_modified"),
        scope_status: "ACTIVE".to_string(),
        node_status: "ACTIVE".to_string(),
        eligibility: "ELIGIBLE".to_string(),
        content_locator: DriveResourceContentLocator {
            storage_provider_id: row.get("storage_provider_id"),
            storage_provider_version: row.get("storage_provider_version"),
            bucket: row.get("bucket"),
            object_key: row.get("object_key"),
        },
    })
}
