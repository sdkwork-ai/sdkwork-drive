use crate::application::upload_content_policy::{
    evaluate_upload_content_policy, UploadContentPolicyContext, UploadContentPolicyDecision,
};
use crate::domain::uploader::{content_type_group_for, DriveUploadItem, DriveUploadPart};
use crate::ports::uploader_store::{
    CompleteDriveStoredUpload, DriveUploaderStore, NewDriveUploadItem, NewDriveUploadPart,
    NewDriveUploaderNode, NewDriveUploaderSession, NewDriveUploaderSpace,
};
use crate::{drive_share_token_hash, DriveServiceError};
use sdkwork_drive_storage_contract::{DriveObjectLocator, DriveObjectStore, PutObjectRequest};
use sdkwork_utils_rust::{
    format_numbered_filename_variant, sha256_hash, split_filename_stem_extension,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum UploaderActor {
    Anonymous { anonymous_id: String },
    User { user_id: String },
    System { operator_id: String },
}

#[derive(Debug, Clone)]
pub enum UploaderRetention {
    LongTerm,
    Temporary {
        ttl_seconds: i64,
        cleanup_action: String,
        hard_delete_after_seconds: Option<i64>,
    },
}

#[derive(Debug, Clone)]
pub enum UploaderTarget {
    AutoUploadSpace {
        parent_node_id: Option<String>,
    },
    AiGeneratedSpace {
        parent_node_id: Option<String>,
    },
    Space {
        space_id: String,
        parent_node_id: Option<String>,
        share_token: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct PrepareUploaderUploadCommand {
    pub id: String,
    pub task_id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub actor: UploaderActor,
    pub app_id: String,
    pub app_resource_type: String,
    pub app_resource_id: String,
    pub scene: Option<String>,
    pub source: Option<String>,
    pub upload_profile_code: String,
    pub file_fingerprint: String,
    pub original_file_name: String,
    pub content_type: String,
    pub content_length: i64,
    pub chunk_size_bytes: i64,
    pub target: UploaderTarget,
    pub retention: UploaderRetention,
    pub operator_id: String,
    pub now_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct MarkUploaderPartUploadedCommand {
    pub id: String,
    pub tenant_id: String,
    pub upload_item_id: String,
    pub upload_session_id: String,
    pub part_no: i64,
    pub offset_bytes: i64,
    pub size_bytes: i64,
    pub etag: String,
    pub checksum_sha256_hex: Option<String>,
    pub uploaded_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct CompleteStoredUploaderUploadCommand {
    pub tenant_id: String,
    pub upload_item_id: String,
    pub upload_session_id: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
    pub uploaded_parts_count: i64,
    pub operator_id: String,
}

#[derive(Debug, Clone)]
pub struct UploadBytesCommand {
    pub prepare: PrepareUploaderUploadCommand,
    pub body: Vec<u8>,
    pub uploaded_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct DriveUploaderService<S>
where
    S: DriveUploaderStore,
{
    store: S,
}

impl<S> DriveUploaderService<S>
where
    S: DriveUploaderStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn prepare_video_upload(
        &self,
        mut command: PrepareUploaderUploadCommand,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        command.upload_profile_code = "video".to_string();
        self.prepare_upload(command).await
    }

    pub async fn prepare_image_upload(
        &self,
        mut command: PrepareUploaderUploadCommand,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        command.upload_profile_code = "image".to_string();
        self.prepare_upload(command).await
    }

    pub async fn prepare_upload(
        &self,
        command: PrepareUploaderUploadCommand,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        let id = require_identifier(command.id, "id")?;
        let task_id = require_identifier(command.task_id, "task_id")?;
        let tenant_id = require_identifier(command.tenant_id, "tenant_id")?;
        if let Some(existing) = self
            .store
            .find_upload_item_by_task(&tenant_id, &task_id)
            .await?
        {
            return Ok(existing);
        }

        let app_id = require_identifier(command.app_id, "app_id")?;
        let app_resource_type = require_identifier(command.app_resource_type, "app_resource_type")?;
        let app_resource_id = require_identifier(command.app_resource_id, "app_resource_id")?;
        let organization_id =
            normalize_optional_identifier(command.organization_id, "organization_id")?;
        let scene = normalize_optional_scene(command.scene)?;
        let source = normalize_optional_usage_context(command.source, "source")?;
        let upload_profile_code = normalize_profile(command.upload_profile_code)?;
        let file_fingerprint = require_identifier(command.file_fingerprint, "file_fingerprint")?;
        let original_file_name = require_file_name(command.original_file_name)?;
        let content_type = normalize_content_type(command.content_type)?;
        if command.content_length < 0 {
            return Err(DriveServiceError::Validation(
                "content_length must be greater than or equal to 0".to_string(),
            ));
        }
        if command.chunk_size_bytes <= 0 {
            return Err(DriveServiceError::Validation(
                "chunk_size_bytes must be greater than 0".to_string(),
            ));
        }
        if command.now_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "now_epoch_ms must be greater than 0".to_string(),
            ));
        }
        let operator_id = require_identifier(command.operator_id, "operator_id")?;
        let (actor_type, actor_id, user_id, owner_subject_type, owner_subject_id) =
            resolve_actor(command.actor, &app_id)?;
        let retention = resolve_retention(command.retention, command.now_epoch_ms)?;
        let (auto_space_type, auto_space_display_name) =
            resolve_auto_upload_space_profile(scene.as_deref());
        let (space_id, parent_node_id) = match command.target {
            UploaderTarget::AutoUploadSpace { parent_node_id } => {
                let space_id = self
                    .resolve_or_create_upload_space(
                        &tenant_id,
                        &owner_subject_type,
                        &owner_subject_id,
                        auto_space_type,
                        auto_space_display_name,
                        &operator_id,
                    )
                    .await?;
                (
                    space_id,
                    normalize_optional_identifier(parent_node_id, "parent_node_id")?,
                )
            }
            UploaderTarget::AiGeneratedSpace { parent_node_id } => {
                let space_id = self
                    .resolve_or_create_upload_space(
                        &tenant_id,
                        &owner_subject_type,
                        &owner_subject_id,
                        "ai_generated",
                        "AI Generated",
                        &operator_id,
                    )
                    .await?;
                (
                    space_id,
                    normalize_optional_identifier(parent_node_id, "parent_node_id")?,
                )
            }
            UploaderTarget::Space {
                space_id,
                parent_node_id,
                share_token,
            } => {
                let space_id = require_identifier(space_id, "space_id")?;
                let parent_node_id =
                    normalize_optional_identifier(parent_node_id, "parent_node_id")?;
                let share_token = normalize_optional_share_token(share_token)?;
                self.ensure_target_space_upload_allowed(TargetSpaceUploadAccess {
                    tenant_id: &tenant_id,
                    space_id: &space_id,
                    parent_node_id: parent_node_id.as_deref(),
                    actor_type: &actor_type,
                    actor_id: &actor_id,
                    owner_subject_type: &owner_subject_type,
                    owner_subject_id: &owner_subject_id,
                    share_token: share_token.as_deref(),
                    now_epoch_ms: command.now_epoch_ms,
                })
                .await?;
                (space_id, parent_node_id)
            }
        };
        let node_id = format!("node-{id}");
        let mut node_name = String::new();
        for attempt in 0..5 {
            node_name = resolve_unique_upload_node_name(
                &self.store,
                &tenant_id,
                &space_id,
                parent_node_id.as_deref(),
                &original_file_name,
            )
            .await?;
            match self
                .store
                .insert_upload_node(&NewDriveUploaderNode {
                    id: node_id.clone(),
                    tenant_id: tenant_id.clone(),
                    space_id: space_id.clone(),
                    parent_node_id: parent_node_id.clone(),
                    node_name: node_name.clone(),
                    scene: scene.clone(),
                    source: source.clone(),
                    operator_id: operator_id.clone(),
                })
                .await
            {
                Ok(_) => break,
                Err(DriveServiceError::Conflict(_)) if attempt < 4 => continue,
                Err(error) => return Err(error),
            }
        }
        let (storage_provider_id, bucket) = self
            .store
            .find_default_storage_provider(&tenant_id)
            .await?
            .ok_or_else(|| {
                DriveServiceError::NotFound("active storage provider not found".to_string())
            })?;
        let upload_session_id = format!("session-{id}");
        let object_key = build_uploader_object_key_prefix(UploaderObjectKeyPrefix {
            tenant_id: &tenant_id,
            organization_id: organization_id.as_deref(),
            user_id: user_id.as_deref(),
            space_id: &space_id,
            actor_type: &actor_type,
            actor_id: &actor_id,
            scene: scene.as_deref(),
            source: source.as_deref(),
            upload_profile_code: &upload_profile_code,
            now_epoch_ms: command.now_epoch_ms,
            node_id: &node_id,
            upload_item_id: &id,
        });
        self.store
            .insert_upload_session(&NewDriveUploaderSession {
                id: upload_session_id.clone(),
                tenant_id: tenant_id.clone(),
                space_id: space_id.clone(),
                node_id: node_id.clone(),
                storage_provider_id: storage_provider_id.clone(),
                bucket: bucket.clone(),
                object_key: object_key.clone(),
                operator_id: operator_id.clone(),
                expires_at_epoch_ms: command.now_epoch_ms + 86_400_000,
            })
            .await?;

        let total_parts = total_parts(command.content_length, command.chunk_size_bytes);
        self.store
            .insert_upload_item(&NewDriveUploadItem {
                id,
                task_id,
                tenant_id,
                organization_id,
                user_id,
                actor_type,
                actor_id,
                app_id,
                app_resource_type,
                app_resource_id,
                scene,
                source,
                upload_profile_code: upload_profile_code.clone(),
                file_fingerprint,
                space_id,
                node_id,
                upload_session_id: Some(upload_session_id.clone()),
                storage_provider_id: Some(storage_provider_id),
                storage_upload_id: Some(upload_session_id),
                original_file_name: node_name.clone(),
                file_extension: file_extension(&node_name),
                content_type: content_type.clone(),
                content_type_group: profile_content_group(&upload_profile_code, &content_type),
                detected_content_type: Some(content_type),
                content_length: command.content_length,
                checksum_sha256_hex: None,
                chunk_size_bytes: command.chunk_size_bytes,
                total_parts,
                status: "prepared".to_string(),
                retention_mode: retention.mode,
                retention_expires_at_epoch_ms: retention.expires_at_epoch_ms,
                cleanup_action: retention.cleanup_action,
                hard_delete_after_epoch_ms: retention.hard_delete_after_epoch_ms,
                created_by: operator_id.clone(),
                updated_by: operator_id,
            })
            .await
    }

    pub async fn mark_part_uploaded(
        &self,
        command: MarkUploaderPartUploadedCommand,
    ) -> Result<DriveUploadPart, DriveServiceError> {
        let id = require_identifier(command.id, "id")?;
        let tenant_id = require_identifier(command.tenant_id, "tenant_id")?;
        let upload_item_id = require_identifier(command.upload_item_id, "upload_item_id")?;
        let upload_session_id = require_identifier(command.upload_session_id, "upload_session_id")?;
        if !(1..=10_000).contains(&command.part_no) {
            return Err(DriveServiceError::Validation(
                "part_no must be in range [1, 10000]".to_string(),
            ));
        }
        if command.offset_bytes < 0 {
            return Err(DriveServiceError::Validation(
                "offset_bytes must be greater than or equal to 0".to_string(),
            ));
        }
        if command.size_bytes <= 0 {
            return Err(DriveServiceError::Validation(
                "size_bytes must be greater than 0".to_string(),
            ));
        }
        let etag = require_non_empty(command.etag, "etag")?;
        if command.uploaded_at_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "uploaded_at_epoch_ms must be greater than 0".to_string(),
            ));
        }
        if let Some(checksum) = &command.checksum_sha256_hex {
            validate_sha256_checksum(checksum)?;
        }

        self.store
            .record_uploaded_part(&NewDriveUploadPart {
                id,
                tenant_id,
                upload_item_id,
                upload_session_id,
                part_no: command.part_no,
                offset_bytes: command.offset_bytes,
                size_bytes: command.size_bytes,
                etag,
                checksum_sha256_hex: command.checksum_sha256_hex,
                uploaded_at_epoch_ms: command.uploaded_at_epoch_ms,
            })
            .await
    }

    pub async fn complete_stored_upload(
        &self,
        command: CompleteStoredUploaderUploadCommand,
    ) -> Result<DriveUploadItem, DriveServiceError> {
        let tenant_id = require_identifier(command.tenant_id, "tenant_id")?;
        let upload_item_id = require_identifier(command.upload_item_id, "upload_item_id")?;
        let upload_session_id = require_identifier(command.upload_session_id, "upload_session_id")?;
        let content_type = normalize_content_type(command.content_type)?;
        if command.content_length < 0 {
            return Err(DriveServiceError::Validation(
                "content_length must be greater than or equal to 0".to_string(),
            ));
        }
        validate_sha256_checksum(&command.checksum_sha256_hex)?;
        if command.uploaded_parts_count <= 0 {
            return Err(DriveServiceError::Validation(
                "uploaded_parts_count must be greater than 0".to_string(),
            ));
        }
        let operator_id = require_identifier(command.operator_id, "operator_id")?;

        let policy_context = UploadContentPolicyContext {
            tenant_id: tenant_id.clone(),
            upload_item_id: upload_item_id.clone(),
            content_type: content_type.clone(),
            content_length: command.content_length,
            checksum_sha256_hex: command.checksum_sha256_hex.clone(),
        };
        match evaluate_upload_content_policy(&policy_context) {
            UploadContentPolicyDecision::Allow => {}
            UploadContentPolicyDecision::Block => {
                return Err(DriveServiceError::Validation(format!(
                    "upload blocked by content policy for content type {}",
                    content_type
                )));
            }
            UploadContentPolicyDecision::Quarantine => {
                self.store
                    .quarantine_blocked_upload_content(&tenant_id, &upload_item_id, &operator_id)
                    .await?;
                return Err(DriveServiceError::Validation(format!(
                    "upload quarantined by content policy for content type {}",
                    content_type
                )));
            }
        }

        self.store
            .complete_stored_upload(&CompleteDriveStoredUpload {
                tenant_id,
                upload_item_id,
                upload_session_id,
                content_type,
                content_length: command.content_length,
                checksum_sha256_hex: command.checksum_sha256_hex,
                uploaded_parts_count: command.uploaded_parts_count,
                operator_id,
            })
            .await
    }

    pub async fn upload_bytes<O>(
        &self,
        object_store: &O,
        command: UploadBytesCommand,
    ) -> Result<DriveUploadItem, DriveServiceError>
    where
        O: DriveObjectStore + ?Sized,
    {
        if command.uploaded_at_epoch_ms <= 0 {
            return Err(DriveServiceError::Validation(
                "uploaded_at_epoch_ms must be greater than 0".to_string(),
            ));
        }
        if command.prepare.content_length != command.body.len() as i64 {
            return Err(DriveServiceError::Validation(
                "prepare content_length must match body length".to_string(),
            ));
        }

        let operator_id = command.prepare.operator_id.clone();
        let prepared = self.prepare_upload(command.prepare).await?;
        let upload_session_id = prepared.upload_session_id.clone().ok_or_else(|| {
            DriveServiceError::Internal("prepared upload is missing upload_session_id".to_string())
        })?;
        let bucket = prepared.object_bucket.clone().ok_or_else(|| {
            DriveServiceError::Internal("prepared upload is missing object bucket".to_string())
        })?;
        let object_key = prepared.object_key.clone().ok_or_else(|| {
            DriveServiceError::Internal("prepared upload is missing object key".to_string())
        })?;
        let checksum_sha256_hex = sha256_checksum(&command.body);
        object_store
            .put_object(PutObjectRequest {
                locator: DriveObjectLocator { bucket, object_key },
                content_type: Some(prepared.content_type.clone()),
                metadata: upload_object_metadata(&prepared),
                body: command.body,
                checksum_sha256_hex: Some(strip_checksum_prefix(&checksum_sha256_hex).to_string()),
            })
            .await
            .map_err(|error| {
                DriveServiceError::Internal(format!("write uploaded object failed: {error}"))
            })?;

        self.mark_part_uploaded(MarkUploaderPartUploadedCommand {
            id: format!("part-{}-1", prepared.id),
            tenant_id: prepared.tenant_id.clone(),
            upload_item_id: prepared.id.clone(),
            upload_session_id: upload_session_id.clone(),
            part_no: 1,
            offset_bytes: 0,
            size_bytes: prepared.content_length,
            etag: checksum_sha256_hex.clone(),
            checksum_sha256_hex: Some(checksum_sha256_hex.clone()),
            uploaded_at_epoch_ms: command.uploaded_at_epoch_ms,
        })
        .await?;

        self.complete_stored_upload(CompleteStoredUploaderUploadCommand {
            tenant_id: prepared.tenant_id,
            upload_item_id: prepared.id,
            upload_session_id,
            content_type: prepared.content_type,
            content_length: prepared.content_length,
            checksum_sha256_hex,
            uploaded_parts_count: 1,
            operator_id,
        })
        .await
    }

    async fn resolve_or_create_upload_space(
        &self,
        tenant_id: &str,
        owner_subject_type: &str,
        owner_subject_id: &str,
        space_type: &str,
        display_name: &str,
        operator_id: &str,
    ) -> Result<String, DriveServiceError> {
        if let Some(space_id) = self
            .store
            .find_upload_space(tenant_id, owner_subject_type, owner_subject_id, space_type)
            .await?
        {
            return Ok(space_id);
        }
        let space_id = format!(
            "space-{}-{}-{}",
            space_type.replace('_', "-"),
            owner_subject_type,
            stable_identifier_suffix(owner_subject_id)
        );
        self.store
            .insert_upload_space(&NewDriveUploaderSpace {
                id: space_id,
                tenant_id: tenant_id.to_string(),
                owner_subject_type: owner_subject_type.to_string(),
                owner_subject_id: owner_subject_id.to_string(),
                space_type: space_type.to_string(),
                display_name: display_name.to_string(),
                operator_id: operator_id.to_string(),
            })
            .await
    }

    async fn ensure_target_space_upload_allowed(
        &self,
        access: TargetSpaceUploadAccess<'_>,
    ) -> Result<(), DriveServiceError> {
        let space = self
            .store
            .find_active_space(access.tenant_id, access.space_id)
            .await?
            .ok_or_else(|| DriveServiceError::NotFound("target space not found".to_string()))?;
        if space.tenant_id != access.tenant_id {
            return Err(DriveServiceError::NotFound(
                "target space not found".to_string(),
            ));
        }

        let permission_anchor_node_id = if let Some(parent_node_id) = access.parent_node_id {
            let parent = self
                .store
                .find_active_node(access.tenant_id, parent_node_id)
                .await?
                .ok_or_else(|| {
                    DriveServiceError::NotFound("target parent node not found".to_string())
                })?;
            if parent.space_id != access.space_id || parent.node_type != "folder" {
                return Err(DriveServiceError::NotFound(
                    "target parent node not found".to_string(),
                ));
            }
            parent.id
        } else {
            String::new()
        };

        let is_space_owner = space.owner_subject_type == access.owner_subject_type
            && space.owner_subject_id == access.owner_subject_id;
        if is_space_owner && access.actor_type != "anonymous" {
            return Ok(());
        }

        if !permission_anchor_node_id.is_empty()
            && self
                .store
                .has_writer_permission(
                    access.tenant_id,
                    &permission_anchor_node_id,
                    access.actor_type,
                    access.actor_id,
                )
                .await?
        {
            return Ok(());
        }

        if let Some(share_token) = access.share_token {
            if permission_anchor_node_id.is_empty() {
                return Err(DriveServiceError::PermissionDenied(
                    "share token uploads require a target parent folder".to_string(),
                ));
            }
            let token_hash = drive_share_token_hash(share_token);
            if self
                .store
                .has_writer_share_token(
                    access.tenant_id,
                    &permission_anchor_node_id,
                    &token_hash,
                    access.now_epoch_ms,
                )
                .await?
            {
                return Ok(());
            }
        }

        if access.actor_type == "anonymous" {
            return Err(DriveServiceError::PermissionDenied(
                "anonymous upload to target space requires a public writer share token".to_string(),
            ));
        }

        Err(DriveServiceError::PermissionDenied(
            "actor does not have writer permission for target space".to_string(),
        ))
    }
}

struct TargetSpaceUploadAccess<'a> {
    tenant_id: &'a str,
    space_id: &'a str,
    parent_node_id: Option<&'a str>,
    actor_type: &'a str,
    actor_id: &'a str,
    owner_subject_type: &'a str,
    owner_subject_id: &'a str,
    share_token: Option<&'a str>,
    now_epoch_ms: i64,
}

fn upload_object_metadata(item: &DriveUploadItem) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("upload_item_id".to_string(), item.id.clone());
    metadata.insert(
        "upload_session_id".to_string(),
        item.upload_session_id.clone().unwrap_or_default(),
    );
    metadata.insert("tenant_id".to_string(), item.tenant_id.clone());
    metadata.insert("space_id".to_string(), item.space_id.clone());
    metadata.insert("node_id".to_string(), item.node_id.clone());
    metadata.insert("app_id".to_string(), item.app_id.clone());
    metadata.insert(
        "app_resource_type".to_string(),
        item.app_resource_type.clone(),
    );
    metadata.insert("app_resource_id".to_string(), item.app_resource_id.clone());
    if let Some(scene) = &item.scene {
        metadata.insert("scene".to_string(), scene.clone());
    }
    if let Some(source) = &item.source {
        metadata.insert("source".to_string(), source.clone());
    }
    metadata
}

fn sha256_checksum(body: &[u8]) -> String {
    format!("sha256:{}", sha256_hash(body))
}

fn strip_checksum_prefix(value: &str) -> &str {
    value
        .strip_prefix("sha256:")
        .expect("sha256_checksum should always include prefix")
}

fn resolve_actor(
    actor: UploaderActor,
    app_id: &str,
) -> Result<(String, String, Option<String>, String, String), DriveServiceError> {
    match actor {
        UploaderActor::Anonymous { anonymous_id } => {
            let anonymous_id = require_identifier(anonymous_id, "anonymous_id")?;
            Ok((
                "anonymous".to_string(),
                anonymous_id,
                None,
                "app".to_string(),
                format!("app:{app_id}:anonymous"),
            ))
        }
        UploaderActor::User { user_id } => {
            let user_id = require_identifier(user_id, "user_id")?;
            Ok((
                "user".to_string(),
                user_id.clone(),
                Some(user_id.clone()),
                "user".to_string(),
                user_id,
            ))
        }
        UploaderActor::System { operator_id } => {
            let operator_id = require_identifier(operator_id, "operator_id")?;
            Ok((
                "system".to_string(),
                operator_id,
                None,
                "app".to_string(),
                format!("app:{app_id}:system"),
            ))
        }
    }
}

struct ResolvedRetention {
    mode: String,
    expires_at_epoch_ms: Option<i64>,
    cleanup_action: Option<String>,
    hard_delete_after_epoch_ms: Option<i64>,
}

fn resolve_retention(
    retention: UploaderRetention,
    now_epoch_ms: i64,
) -> Result<ResolvedRetention, DriveServiceError> {
    match retention {
        UploaderRetention::LongTerm => Ok(ResolvedRetention {
            mode: "long_term".to_string(),
            expires_at_epoch_ms: None,
            cleanup_action: None,
            hard_delete_after_epoch_ms: None,
        }),
        UploaderRetention::Temporary {
            ttl_seconds,
            cleanup_action,
            hard_delete_after_seconds,
        } => {
            if ttl_seconds <= 0 {
                return Err(DriveServiceError::Validation(
                    "ttl_seconds must be greater than 0".to_string(),
                ));
            }
            let cleanup_action = require_cleanup_action(cleanup_action)?;
            let hard_delete_after_epoch_ms = hard_delete_after_seconds
                .map(|seconds| {
                    if seconds <= 0 {
                        Err(DriveServiceError::Validation(
                            "hard_delete_after_seconds must be greater than 0".to_string(),
                        ))
                    } else {
                        Ok(now_epoch_ms + seconds * 1000)
                    }
                })
                .transpose()?;
            Ok(ResolvedRetention {
                mode: "temporary".to_string(),
                expires_at_epoch_ms: Some(now_epoch_ms + ttl_seconds * 1000),
                cleanup_action: Some(cleanup_action),
                hard_delete_after_epoch_ms,
            })
        }
    }
}

fn normalize_profile(value: String) -> Result<String, DriveServiceError> {
    let profile = require_identifier(value, "upload_profile_code")?;
    match profile.as_str() {
        "generic" | "video" | "image" | "audio" | "document" | "archive" | "text" | "dataset"
        | "attachment" | "avatar" | "thumbnail" => Ok(profile),
        _ => Err(DriveServiceError::Validation(
            "upload_profile_code is not supported".to_string(),
        )),
    }
}

fn profile_content_group(profile: &str, content_type: &str) -> String {
    match profile {
        "video" | "image" | "audio" | "document" | "archive" | "text" => profile.to_string(),
        _ => content_type_group_for(content_type).to_string(),
    }
}

fn require_cleanup_action(value: String) -> Result<String, DriveServiceError> {
    let value = require_non_empty(value, "cleanup_action")?;
    match value.as_str() {
        "soft_delete" | "hard_delete" => Ok(value),
        _ => Err(DriveServiceError::Validation(
            "cleanup_action must be soft_delete or hard_delete".to_string(),
        )),
    }
}

fn normalize_content_type(value: String) -> Result<String, DriveServiceError> {
    let value = require_non_empty(value, "content_type")?.to_ascii_lowercase();
    if value.len() < 3
        || value.len() > 255
        || value.matches('/').count() != 1
        || value.chars().any(char::is_whitespace)
    {
        return Err(DriveServiceError::Validation(
            "content_type must be a valid media type".to_string(),
        ));
    }
    Ok(value)
}

fn require_identifier(value: String, field_name: &str) -> Result<String, DriveServiceError> {
    let value = require_non_empty(value, field_name)?;
    if value.len() > 255 || !value.chars().all(is_allowed_identifier_char) {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} contains invalid characters"
        )));
    }
    Ok(value)
}

fn normalize_optional_identifier(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, DriveServiceError> {
    value
        .map(|value| require_identifier(value, field_name))
        .transpose()
}

fn normalize_optional_scene(value: Option<String>) -> Result<Option<String>, DriveServiceError> {
    normalize_optional_usage_context(value, "scene")
}

fn resolve_auto_upload_space_profile(scene: Option<&str>) -> (&'static str, &'static str) {
    match scene {
        Some(value) if value.eq_ignore_ascii_case("rtc") => ("rtc", "RTC Records"),
        Some(value) if value.eq_ignore_ascii_case("im") => ("im", "IM"),
        Some(value)
            if value.eq_ignore_ascii_case("deployment")
                || value.eq_ignore_ascii_case("application-source") =>
        {
            ("deployment", "Deployment")
        }
        Some(value) if value.eq_ignore_ascii_case("git-repository") => {
            ("git_repository", "Git Repository")
        }
        _ => ("app_upload", "Upload"),
    }
}

fn normalize_optional_share_token(
    value: Option<String>,
) -> Result<Option<String>, DriveServiceError> {
    value
        .map(|value| {
            let value = require_non_empty(value, "share_token")?;
            if value.len() > 512 || value.contains('\0') {
                return Err(DriveServiceError::Validation(
                    "share_token contains invalid characters".to_string(),
                ));
            }
            Ok(value)
        })
        .transpose()
}

fn normalize_optional_usage_context(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, DriveServiceError> {
    value
        .map(|value| {
            let value = require_non_empty(value, field_name)?;
            if value.len() > 128 || !value.chars().all(is_allowed_identifier_char) {
                return Err(DriveServiceError::Validation(format!(
                    "{field_name} contains invalid characters"
                )));
            }
            Ok(value)
        })
        .transpose()
}

fn require_file_name(value: String) -> Result<String, DriveServiceError> {
    let value = require_non_empty(value, "original_file_name")?;
    if value.len() > 255 || value.contains('/') || value.contains('\\') || value.contains('\0') {
        return Err(DriveServiceError::Validation(
            "original_file_name must be a valid file name".to_string(),
        ));
    }
    Ok(value)
}

fn require_non_empty(value: String, field_name: &str) -> Result<String, DriveServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DriveServiceError::Validation(format!(
            "{field_name} is required"
        )));
    }
    Ok(value)
}

fn is_allowed_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-')
}

fn validate_sha256_checksum(value: &str) -> Result<(), DriveServiceError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(DriveServiceError::Validation(
            "checksum_sha256_hex must use sha256:<64 lowercase hex>".to_string(),
        ));
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DriveServiceError::Validation(
            "checksum_sha256_hex must use sha256:<64 lowercase hex>".to_string(),
        ));
    }
    Ok(())
}

fn total_parts(content_length: i64, chunk_size_bytes: i64) -> i64 {
    let normalized_length = content_length.max(1);
    (normalized_length + chunk_size_bytes - 1) / chunk_size_bytes
}

fn file_extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.trim().to_ascii_lowercase())
        .filter(|extension| !extension.is_empty() && extension.len() <= 64)
}

fn split_upload_file_name(file_name: &str) -> (String, Option<String>) {
    split_filename_stem_extension(file_name)
}

async fn resolve_unique_upload_node_name<S: DriveUploaderStore>(
    store: &S,
    tenant_id: &str,
    space_id: &str,
    parent_node_id: Option<&str>,
    base_name: &str,
) -> Result<String, DriveServiceError> {
    if !store
        .live_node_name_exists_in_parent(tenant_id, space_id, parent_node_id, base_name)
        .await?
    {
        return Ok(base_name.to_string());
    }

    let (stem, extension) = split_upload_file_name(base_name);
    for index in 1..=9999 {
        let candidate = format_numbered_filename_variant(&stem, index, extension.as_deref());
        if !store
            .live_node_name_exists_in_parent(tenant_id, space_id, parent_node_id, &candidate)
            .await?
        {
            return Ok(candidate);
        }
    }

    Ok(format_numbered_filename_variant(
        &stem,
        9_999,
        extension.as_deref(),
    ))
}

#[cfg(test)]
fn allocate_unique_upload_node_name(base_name: &str, taken_names: &[String]) -> String {
    use std::collections::HashSet;
    let taken: HashSet<&str> = taken_names.iter().map(String::as_str).collect();
    sdkwork_utils_rust::allocate_unique_display_name(base_name, |candidate| {
        taken.contains(candidate)
    })
}

fn stable_identifier_suffix(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(80)
        .collect()
}

struct UploaderObjectKeyPrefix<'a> {
    tenant_id: &'a str,
    organization_id: Option<&'a str>,
    user_id: Option<&'a str>,
    space_id: &'a str,
    actor_type: &'a str,
    actor_id: &'a str,
    scene: Option<&'a str>,
    source: Option<&'a str>,
    upload_profile_code: &'a str,
    now_epoch_ms: i64,
    node_id: &'a str,
    upload_item_id: &'a str,
}

fn build_uploader_object_key_prefix(prefix: UploaderObjectKeyPrefix<'_>) -> String {
    let date = utc_date_from_epoch_ms(prefix.now_epoch_ms).unwrap_or("1970-01-01".to_string());
    format!(
        "sdkwork-drive/uploader/tenants/{}/organizations/{}/users/{}/spaces/{}/actors/{}/{}/scene/{}/source/{}/profile/{}/dt/{}/nodes/{}/uploads/{}/content",
        path_segment(prefix.tenant_id),
        path_segment(prefix.organization_id.unwrap_or("none")),
        path_segment(prefix.user_id.unwrap_or("anonymous")),
        path_segment(prefix.space_id),
        path_segment(prefix.actor_type),
        path_segment(prefix.actor_id),
        path_segment(prefix.scene.unwrap_or("unspecified")),
        path_segment(prefix.source.unwrap_or("unspecified")),
        path_segment(prefix.upload_profile_code),
        date,
        path_segment(prefix.node_id),
        path_segment(prefix.upload_item_id),
    )
}

fn path_segment(value: &str) -> String {
    let mut segment = value
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '@' | '-') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while segment.contains("--") {
        segment = segment.replace("--", "-");
    }
    let segment = segment.trim_matches('-').to_string();
    if segment.is_empty() {
        "unspecified".to_string()
    } else {
        segment.chars().take(128).collect()
    }
}

fn utc_date_from_epoch_ms(epoch_ms: i64) -> Option<String> {
    let days = epoch_ms.checked_div(86_400_000)?;
    let (year, month, day) = civil_from_days(days)?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn civil_from_days(days_since_unix_epoch: i64) -> Option<(i32, u32, u32)> {
    let z = days_since_unix_epoch.checked_add(719_468)?;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    Some((
        i32::try_from(year).ok()?,
        u32::try_from(month).ok()?,
        u32::try_from(day).ok()?,
    ))
}

#[cfg(test)]
mod upload_node_name_tests {
    use super::allocate_unique_upload_node_name;

    #[test]
    fn keeps_original_name_when_parent_is_empty() {
        let name = allocate_unique_upload_node_name("report.txt", &[]);
        assert_eq!(name, "report.txt");
    }

    #[test]
    fn appends_numeric_suffix_when_name_is_taken() {
        let taken = vec!["report.txt".to_string()];
        let name = allocate_unique_upload_node_name("report.txt", &taken);
        assert_eq!(name, "report (1).txt");
    }

    #[test]
    fn increments_suffix_until_unique() {
        let taken = vec![
            "report.txt".to_string(),
            "report (1).txt".to_string(),
            "report (2).txt".to_string(),
        ];
        let name = allocate_unique_upload_node_name("report.txt", &taken);
        assert_eq!(name, "report (3).txt");
    }

    #[test]
    fn handles_extensionless_names() {
        let taken = vec!["README".to_string()];
        let name = allocate_unique_upload_node_name("README", &taken);
        assert_eq!(name, "README (1)");
    }
}
