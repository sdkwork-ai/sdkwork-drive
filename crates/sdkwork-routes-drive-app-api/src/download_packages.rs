use crate::acl;
use crate::app_context::DriveRequestContext;
use crate::archive::{
    sanitize_archive_path_segment, sanitize_relative_archive_path, unique_archive_path,
};
use crate::constants::{DOWNLOAD_PACKAGE_MAX_FILES, DOWNLOAD_PACKAGE_MAX_TOTAL_BYTES};
use crate::dto::{
    CreateDownloadPackageRequest, DownloadPackageFileItem, DownloadPackageItemResponse,
    DownloadPackageRecordView, DownloadPackageResponse, DriveNodeResponse,
    InsertDownloadPackageRecord,
};
use crate::error::{
    internal_problem, internal_sql_error, map_object_store_route_error, map_service_error,
    not_found_problem, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::hashing::{sha256_raw_hex_separated, tenant_shard_prefix};
use crate::mappers::map_node_row;
use crate::node_repository::find_node;
use crate::object_store::{
    build_s3_object_store_for_provider, find_storage_provider_by_id,
    missing_signing_provider_error, require_active_storage_provider,
    unsupported_signing_provider_error,
};
use crate::response::{success_created_command_data, success_envelope};
use crate::state::AppState;
use crate::time::{current_epoch_ms, signing_ttl_seconds};
use crate::validators::{normalize_optional_text, validate_requested_ttl_seconds};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, PresignDownloadRequest, PutObjectRequest,
    ReadObjectRangeRequest,
};
use sdkwork_drive_storage_s3::S3DriveObjectStore;
use sdkwork_drive_workspace_service::infrastructure::sql::NODE_API_SELECT_COLUMNS;
use sdkwork_drive_workspace_service::ports::storage_object_store::SignedDownloadPayload;
use sdkwork_utils_rust::SdkWorkApiResponse;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{Cursor, Write};
use tempfile::NamedTempFile;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const PACKAGE_STREAM_CHUNK_BYTES: usize = 256 * 1024;
const IN_MEMORY_PACKAGE_ARCHIVE_THRESHOLD_BYTES: i64 = 32 * 1024 * 1024;

enum BuiltPackageArchive {
    Memory(Vec<u8>),
    File(NamedTempFile),
}

impl BuiltPackageArchive {
    fn size_bytes(&self) -> i64 {
        match self {
            Self::Memory(bytes) => bytes.len() as i64,
            Self::File(file) => file
                .path()
                .metadata()
                .map(|metadata| metadata.len() as i64)
                .unwrap_or(0),
        }
    }
}

pub(crate) async fn create_download_package(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Json(payload): Json<CreateDownloadPackageRequest>,
) -> Result<
    (
        StatusCode,
        Json<SdkWorkApiResponse<DownloadPackageResponse>>,
    ),
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let requested_node_ids = normalize_download_package_node_ids(payload.node_ids)?;
    acl::ensure_node_ids_role(&state.pool, &ctx, &requested_node_ids, "reader").await?;
    let package_name = normalize_package_name(payload.package_name);
    let requested_ttl_seconds = validate_requested_ttl_seconds(
        payload.requested_ttl_seconds,
        300,
        30,
        3600,
        "requestedTtlSeconds",
    )?;
    let expires_at_epoch_ms = current_epoch_ms() + i64::from(requested_ttl_seconds) * 1000;
    let package_id = build_download_package_id(&tenant_id, &requested_node_ids);

    let package_files =
        collect_download_package_files(&state.pool, &tenant_id, &requested_node_ids).await?;
    validate_download_package_manifest(&package_files)?;
    let first_file = package_files.first().ok_or_else(|| {
        problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "nodeIds must resolve to at least one active file",
            SdkWorkResultCode::ValidationError,
        )
    })?;
    let bucket = first_file.bucket.clone();
    let provider = find_storage_provider_by_id(&state.pool, &first_file.storage_provider_id)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(missing_signing_provider_error(&bucket)))?;
    let provider = require_active_storage_provider(provider, &bucket).map_err(map_service_error)?;
    let object_store = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(unsupported_signing_provider_error(&bucket)))?;
    let archive_object_key = build_download_package_object_key(&tenant_id, &package_id);
    let total_bytes = package_files.iter().map(|item| item.content_length).sum();
    let built_archive =
        build_download_package_zip(&state.pool, &package_files, total_bytes).await?;
    let archive_size_bytes = built_archive.size_bytes();
    let requested_node_ids_json = serde_json::to_string(&requested_node_ids).map_err(|error| {
        internal_problem(format!("serialize requested node ids failed: {error}"))
    })?;
    let item_manifest_json = serde_json::to_string(&package_files)
        .map_err(|error| internal_problem(format!("serialize package manifest failed: {error}")))?;
    upload_built_package_archive(
        &object_store,
        DriveObjectLocator {
            bucket: bucket.clone(),
            object_key: archive_object_key.clone(),
        },
        BTreeMap::from([
            ("sdkwork-drive-package-id".to_string(), package_id.clone()),
            ("sdkwork-drive-tenant-id".to_string(), tenant_id.clone()),
        ]),
        built_archive,
    )
    .await
    .map_err(map_object_store_route_error)?;

    insert_download_package_record(
        &state.pool,
        InsertDownloadPackageRecord {
            id: &package_id,
            tenant_id: &tenant_id,
            package_name: &package_name,
            state: "ready",
            storage_provider_id: &provider.id,
            bucket: &bucket,
            archive_object_key: &archive_object_key,
            file_count: package_files.len() as i64,
            total_bytes,
            archive_size_bytes,
            requested_node_ids_json: &requested_node_ids_json,
            item_manifest_json: &item_manifest_json,
            expires_at_epoch_ms,
            operator_id: &operator_id,
        },
    )
    .await?;

    let signed = sign_download_package(
        &object_store,
        &bucket,
        &archive_object_key,
        expires_at_epoch_ms,
    )
    .await?;
    Ok(success_created_command_data(
        build_download_package_response(
            &state,
            DownloadPackageRecordView {
                id: package_id,
                tenant_id,
                package_name,
                state: "ready".to_string(),
                storage_provider_id: provider.id,
                bucket,
                archive_object_key,
                content_type: "application/zip".to_string(),
                file_count: package_files.len() as i64,
                total_bytes,
                archive_size_bytes,
                expires_at_epoch_ms,
                items: package_files,
            },
            signed,
        ),
    ))
}

pub(crate) async fn resolve_download_package_url(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(package_id): Path<String>,
) -> Result<Json<SdkWorkApiResponse<DownloadPackageResponse>>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let package = find_download_package_record(&state.pool, &tenant_id, &package_id).await?;
    let node_ids = package
        .items
        .iter()
        .map(|item| item.node_id.clone())
        .collect::<Vec<_>>();
    acl::ensure_node_ids_role(&state.pool, &ctx, &node_ids, "reader").await?;
    if package.state != "ready" {
        return Err(problem(
            StatusCode::CONFLICT,
            "conflict",
            "download package is not ready",
            SdkWorkResultCode::Conflict,
        ));
    }
    if package.expires_at_epoch_ms <= current_epoch_ms() {
        return Err(download_package_expired_problem());
    }
    let provider = find_storage_provider_by_id(&state.pool, &package.storage_provider_id)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(missing_signing_provider_error(&package.bucket)))?;
    let provider =
        require_active_storage_provider(provider, &package.bucket).map_err(map_service_error)?;
    let object_store = build_s3_object_store_for_provider(&provider)
        .await
        .map_err(map_service_error)?
        .ok_or_else(|| map_service_error(unsupported_signing_provider_error(&package.bucket)))?;
    let signed = sign_download_package(
        &object_store,
        &package.bucket,
        &package.archive_object_key,
        package.expires_at_epoch_ms,
    )
    .await?;
    Ok(success_envelope(build_download_package_response(
        &state, package, signed,
    )))
}

fn normalize_download_package_node_ids(
    node_ids: Vec<String>,
) -> Result<Vec<String>, (StatusCode, Json<ProblemDetail>)> {
    if node_ids.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "nodeIds must contain at least one node id",
            SdkWorkResultCode::ValidationError,
        ));
    }
    if node_ids.len() > 200 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "nodeIds must contain at most 200 node ids",
            SdkWorkResultCode::ValidationError,
        ));
    }
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(node_ids.len());
    for node_id in node_ids {
        let trimmed = node_id.trim().to_string();
        if trimmed.is_empty() {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "nodeIds must not contain empty values",
                SdkWorkResultCode::ValidationError,
            ));
        }
        if seen.insert(trimmed.clone()) {
            normalized.push(trimmed);
        }
    }
    if normalized.is_empty() {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "nodeIds must contain at least one node id",
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(normalized)
}

fn normalize_package_name(package_name: Option<String>) -> String {
    normalize_optional_text(package_name)
        .map(|value| sanitize_archive_path_segment(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "download".to_string())
}

async fn collect_download_package_files(
    pool: &PgPool,
    tenant_id: &str,
    requested_node_ids: &[String],
) -> Result<Vec<DownloadPackageFileItem>, (StatusCode, Json<ProblemDetail>)> {
    let mut files = Vec::new();
    let mut used_paths = BTreeSet::new();
    let requested_set = requested_node_ids.iter().cloned().collect::<BTreeSet<_>>();
    for node_id in requested_node_ids {
        ensure_download_package_file_capacity(files.len())?;
        let node = find_node(pool, tenant_id, node_id).await?;
        if node.lifecycle_status != "active" {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "nodeIds can only reference active files or folders",
                SdkWorkResultCode::ValidationError,
            ));
        }
        match node.node_type.as_str() {
            "file" => {
                let archive_path = unique_archive_path(
                    &sanitize_archive_path_segment(&node.node_name),
                    &mut used_paths,
                );
                let file = read_download_package_file(pool, tenant_id, &node, archive_path).await?;
                files.push(file);
                ensure_download_package_file_capacity(files.len())?;
            }
            "folder" => {
                let root_path = sanitize_archive_path_segment(&node.node_name);
                let remaining_capacity = DOWNLOAD_PACKAGE_MAX_FILES.saturating_sub(files.len());
                let descendants = list_folder_descendant_files(
                    pool,
                    tenant_id,
                    &node.id,
                    &requested_set,
                    remaining_capacity,
                )
                .await?;
                for descendant in descendants {
                    ensure_download_package_file_capacity(files.len())?;
                    let archive_path = unique_archive_path(
                        &format!(
                            "{}/{}",
                            root_path,
                            sanitize_relative_archive_path(&descendant.relative_path)
                        ),
                        &mut used_paths,
                    );
                    let file =
                        read_download_package_file(pool, tenant_id, &descendant.node, archive_path)
                            .await?;
                    files.push(file);
                }
            }
            "shortcut" => {
                return Err(problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    "shortcut nodes must be resolved before package download",
                    SdkWorkResultCode::ValidationError,
                ));
            }
            _ => {
                return Err(problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    "nodeIds can only reference active files or folders",
                    SdkWorkResultCode::ValidationError,
                ));
            }
        }
    }
    Ok(files)
}

fn ensure_download_package_file_capacity(
    current_count: usize,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if current_count > DOWNLOAD_PACKAGE_MAX_FILES {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("download package can include at most {DOWNLOAD_PACKAGE_MAX_FILES} files"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    Ok(())
}

fn validate_download_package_manifest(
    files: &[DownloadPackageFileItem],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if files.len() > DOWNLOAD_PACKAGE_MAX_FILES {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("download package can include at most {DOWNLOAD_PACKAGE_MAX_FILES} files"),
            SdkWorkResultCode::ValidationError,
        ));
    }

    let mut total_bytes = 0_i64;
    for file in files {
        if file.content_length < 0 {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                "contentLength must not be negative",
                SdkWorkResultCode::ValidationError,
            ));
        }
        total_bytes = total_bytes
            .checked_add(file.content_length)
            .ok_or_else(|| {
                problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    "download package total bytes overflow",
                    SdkWorkResultCode::ValidationError,
                )
            })?;
        if total_bytes > DOWNLOAD_PACKAGE_MAX_TOTAL_BYTES {
            return Err(problem(
                StatusCode::BAD_REQUEST,
                "validation failed",
                format!(
                    "download package total bytes must be at most {DOWNLOAD_PACKAGE_MAX_TOTAL_BYTES}"
                ),
                SdkWorkResultCode::ValidationError,
            ));
        }
    }
    Ok(())
}

#[derive(Debug)]
struct FolderDescendantFile {
    node: DriveNodeResponse,
    relative_path: String,
}

async fn list_folder_descendant_files(
    pool: &PgPool,
    tenant_id: &str,
    root_node_id: &str,
    requested_node_ids: &BTreeSet<String>,
    max_files: usize,
) -> Result<Vec<FolderDescendantFile>, (StatusCode, Json<ProblemDetail>)> {
    if max_files == 0 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            format!("download package can include at most {DOWNLOAD_PACKAGE_MAX_FILES} files"),
            SdkWorkResultCode::ValidationError,
        ));
    }
    let mut queue = VecDeque::from([(root_node_id.to_string(), String::new(), 0_usize)]);
    let mut files = Vec::new();
    while let Some((parent_id, parent_path, depth)) = queue.pop_front() {
        if depth > 32 {
            return Err(problem(
                StatusCode::CONFLICT,
                "conflict",
                "folder hierarchy exceeds maximum package depth",
                SdkWorkResultCode::Conflict,
            ));
        }
        let mut child_offset = 0_i64;
        loop {
            let remaining = max_files.saturating_sub(files.len());
            if remaining == 0 {
                return Err(problem(
                    StatusCode::BAD_REQUEST,
                    "validation failed",
                    format!(
                        "download package can include at most {DOWNLOAD_PACKAGE_MAX_FILES} files"
                    ),
                    SdkWorkResultCode::ValidationError,
                ));
            }
            let batch_limit = remaining.min(500) as i64;
            let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
                "SELECT {NODE_API_SELECT_COLUMNS}
                 FROM dr_drive_node
                 WHERE tenant_id=$1
                   AND parent_node_id=$2
                   AND lifecycle_status='active'
                 ORDER BY node_type DESC, node_name ASC, id ASC
                 LIMIT $3 OFFSET $4",
            )))
            .bind(tenant_id)
            .bind(&parent_id)
            .bind(batch_limit)
            .bind(child_offset)
            .fetch_all(pool)
            .await
            .map_err(internal_sql_error("list folder descendants failed"))?;
            if rows.is_empty() {
                break;
            }
            let fetched = rows.len();
            for row in rows {
                if files.len() >= max_files {
                    return Err(problem(
                        StatusCode::BAD_REQUEST,
                        "validation failed",
                        format!(
                            "download package can include at most {DOWNLOAD_PACKAGE_MAX_FILES} files"
                        ),
                        SdkWorkResultCode::ValidationError,
                    ));
                }
                let node = map_node_row(&row);
                if requested_node_ids.contains(&node.id) {
                    continue;
                }
                let segment = sanitize_archive_path_segment(&node.node_name);
                let relative_path = if parent_path.is_empty() {
                    segment
                } else {
                    format!("{parent_path}/{segment}")
                };
                match node.node_type.as_str() {
                    "file" => files.push(FolderDescendantFile {
                        node,
                        relative_path,
                    }),
                    "folder" => queue.push_back((node.id.clone(), relative_path, depth + 1)),
                    _ => {}
                }
            }
            if fetched < batch_limit as usize {
                break;
            }
            child_offset += batch_limit;
        }
    }
    Ok(files)
}

async fn read_download_package_file(
    pool: &PgPool,
    tenant_id: &str,
    node: &DriveNodeResponse,
    archive_path: String,
) -> Result<DownloadPackageFileItem, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT storage_provider_id, bucket, object_key, content_type, content_length, checksum_sha256_hex
         FROM dr_drive_storage_object
         WHERE tenant_id=$1
           AND node_id=$2
           AND lifecycle_status='active'
         ORDER BY version_no DESC
         LIMIT 1",
    )
    .bind(tenant_id)
    .bind(&node.id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error(
        "read download package storage object failed",
    ))?;
    let Some(row) = row else {
        return Err(not_found_problem(
            "storage object for package node is not found or inactive",
        ));
    };
    Ok(DownloadPackageFileItem {
        node_id: node.id.clone(),
        node_name: node.node_name.clone(),
        archive_path,
        storage_provider_id: row.get("storage_provider_id"),
        bucket: row.get("bucket"),
        object_key: row.get("object_key"),
        content_type: row.get("content_type"),
        content_length: row.get("content_length"),
        checksum_sha256_hex: row.get("checksum_sha256_hex"),
    })
}

async fn build_download_package_zip(
    pool: &PgPool,
    files: &[DownloadPackageFileItem],
    total_source_bytes: i64,
) -> Result<BuiltPackageArchive, (StatusCode, Json<ProblemDetail>)> {
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);
    let mut stores = BTreeMap::<String, S3DriveObjectStore>::new();
    if total_source_bytes > IN_MEMORY_PACKAGE_ARCHIVE_THRESHOLD_BYTES {
        let mut temp_file = NamedTempFile::new().map_err(|error| {
            internal_problem(format!("create download package temp file failed: {error}"))
        })?;
        {
            let mut writer = ZipWriter::new(std::io::BufWriter::new(temp_file.as_file_mut()));
            for file in files {
                let object_store = resolve_package_object_store(pool, file, &mut stores).await?;
                stream_file_into_zip(&mut writer, object_store, file, options).await?;
            }
            writer
                .finish()
                .map_err(|error| internal_problem(format!("finish zip archive failed: {error}")))?;
        }
        return Ok(BuiltPackageArchive::File(temp_file));
    }

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for file in files {
        let object_store = resolve_package_object_store(pool, file, &mut stores).await?;
        stream_file_into_zip(&mut writer, object_store, file, options).await?;
    }
    let cursor = writer
        .finish()
        .map_err(|error| internal_problem(format!("finish zip archive failed: {error}")))?;
    Ok(BuiltPackageArchive::Memory(cursor.into_inner()))
}

async fn resolve_package_object_store<'a>(
    pool: &PgPool,
    file: &DownloadPackageFileItem,
    stores: &'a mut BTreeMap<String, S3DriveObjectStore>,
) -> Result<&'a S3DriveObjectStore, (StatusCode, Json<ProblemDetail>)> {
    if !stores.contains_key(&file.storage_provider_id) {
        let provider = find_storage_provider_by_id(pool, &file.storage_provider_id)
            .await
            .map_err(map_service_error)?
            .ok_or_else(|| map_service_error(missing_signing_provider_error(&file.bucket)))?;
        let provider =
            require_active_storage_provider(provider, &file.bucket).map_err(map_service_error)?;
        let object_store = build_s3_object_store_for_provider(&provider)
            .await
            .map_err(map_service_error)?
            .ok_or_else(|| map_service_error(unsupported_signing_provider_error(&file.bucket)))?;
        stores.insert(file.storage_provider_id.clone(), object_store);
    }
    stores
        .get(&file.storage_provider_id)
        .ok_or_else(|| internal_problem("object store cache missing provider after insertion"))
}

async fn stream_file_into_zip<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    object_store: &S3DriveObjectStore,
    file: &DownloadPackageFileItem,
    options: SimpleFileOptions,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if file.content_length < 0 {
        return Err(problem(
            StatusCode::BAD_REQUEST,
            "validation failed",
            "contentLength must not be negative",
            SdkWorkResultCode::ValidationError,
        ));
    }
    writer
        .start_file(&file.archive_path, options)
        .map_err(|error| internal_problem(format!("start zip file failed: {error}")))?;
    if file.content_length == 0 {
        return Ok(());
    }
    let mut offset = 0_i64;
    while offset < file.content_length {
        let chunk_end =
            (offset + PACKAGE_STREAM_CHUNK_BYTES as i64 - 1).min(file.content_length - 1);
        let (_response, mut stream) = object_store
            .read_object_range(ReadObjectRangeRequest {
                locator: DriveObjectLocator {
                    bucket: file.bucket.clone(),
                    object_key: file.object_key.clone(),
                },
                range: DriveByteRange {
                    start_inclusive: offset as u64,
                    end_inclusive: chunk_end as u64,
                },
            })
            .await
            .map_err(map_object_store_route_error)?;
        while let Some(chunk) = stream
            .next_chunk()
            .await
            .map_err(map_object_store_route_error)?
        {
            writer
                .write_all(&chunk)
                .map_err(|error| internal_problem(format!("write zip file failed: {error}")))?;
        }
        offset = chunk_end + 1;
    }
    Ok(())
}

async fn upload_built_package_archive(
    object_store: &S3DriveObjectStore,
    locator: DriveObjectLocator,
    metadata: BTreeMap<String, String>,
    archive: BuiltPackageArchive,
) -> Result<(), sdkwork_drive_storage_contract::DriveObjectStoreError> {
    match archive {
        BuiltPackageArchive::Memory(bytes) => {
            object_store
                .put_object(PutObjectRequest {
                    locator,
                    content_type: Some("application/zip".to_string()),
                    metadata,
                    body: bytes,
                    checksum_sha256_hex: None,
                })
                .await?;
        }
        BuiltPackageArchive::File(temp_file) => {
            object_store
                .put_object_from_path(
                    locator,
                    Some("application/zip".to_string()),
                    metadata,
                    temp_file.path(),
                )
                .await?;
        }
    }
    Ok(())
}

async fn insert_download_package_record(
    pool: &PgPool,
    command: InsertDownloadPackageRecord<'_>,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    sqlx::query(
        "INSERT INTO dr_drive_download_package (
            id, tenant_id, package_name, state, storage_provider_id, bucket,
            archive_object_key, content_type, file_count, total_bytes,
            archive_size_bytes, requested_node_ids_json, item_manifest_json,
            expires_at_epoch_ms, error_message, version, created_by, updated_by
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, 'application/zip', $8, $9,
            $10, $11, $12, $13, NULL, 1, $14, $14
         )
         ON CONFLICT(id) DO UPDATE SET
            package_name=excluded.package_name,
            state=excluded.state,
            storage_provider_id=excluded.storage_provider_id,
            bucket=excluded.bucket,
            archive_object_key=excluded.archive_object_key,
            content_type=excluded.content_type,
            file_count=excluded.file_count,
            total_bytes=excluded.total_bytes,
            archive_size_bytes=excluded.archive_size_bytes,
            requested_node_ids_json=excluded.requested_node_ids_json,
            item_manifest_json=excluded.item_manifest_json,
            expires_at_epoch_ms=excluded.expires_at_epoch_ms,
            error_message=NULL,
            version=dr_drive_download_package.version + 1,
            updated_by=excluded.updated_by,
            updated_at=CURRENT_TIMESTAMP",
    )
    .bind(command.id)
    .bind(command.tenant_id)
    .bind(command.package_name)
    .bind(command.state)
    .bind(command.storage_provider_id)
    .bind(command.bucket)
    .bind(command.archive_object_key)
    .bind(command.file_count)
    .bind(command.total_bytes)
    .bind(command.archive_size_bytes)
    .bind(command.requested_node_ids_json)
    .bind(command.item_manifest_json)
    .bind(command.expires_at_epoch_ms)
    .bind(command.operator_id)
    .execute(pool)
    .await
    .map_err(internal_sql_error(
        "insert dr_drive_download_package failed",
    ))?;
    Ok(())
}

async fn find_download_package_record(
    pool: &PgPool,
    tenant_id: &str,
    package_id: &str,
) -> Result<DownloadPackageRecordView, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT id, tenant_id, package_name, state, storage_provider_id, bucket,
                archive_object_key, content_type, file_count, total_bytes,
                archive_size_bytes, item_manifest_json, expires_at_epoch_ms
         FROM dr_drive_download_package
         WHERE tenant_id=$1 AND id=$2",
    )
    .bind(tenant_id)
    .bind(package_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_sql_error("find dr_drive_download_package failed"))?;
    let Some(row) = row else {
        return Err(not_found_problem("download package not found"));
    };
    let item_manifest_json: String = row.get("item_manifest_json");
    let items = serde_json::from_str::<Vec<DownloadPackageFileItem>>(&item_manifest_json)
        .map_err(|error| internal_problem(format!("parse package manifest failed: {error}")))?;
    Ok(DownloadPackageRecordView {
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
        items,
    })
}

async fn sign_download_package(
    object_store: &S3DriveObjectStore,
    bucket: &str,
    archive_object_key: &str,
    expires_at_epoch_ms: i64,
) -> Result<SignedDownloadPayload, (StatusCode, Json<ProblemDetail>)> {
    let ttl_seconds =
        signing_ttl_seconds(expires_at_epoch_ms).map_err(|_| download_package_expired_problem())?;
    let signed = object_store
        .presign_download(PresignDownloadRequest {
            locator: DriveObjectLocator {
                bucket: bucket.to_string(),
                object_key: archive_object_key.to_string(),
            },
            expires_in_seconds: ttl_seconds,
        })
        .await
        .map_err(map_object_store_route_error)?;
    Ok(SignedDownloadPayload {
        method: signed.method,
        raw_url: signed.url,
        headers: signed.headers,
        expires_at_epoch_ms: signed.expires_at_epoch_ms,
    })
}

fn download_package_expired_problem() -> (StatusCode, Json<ProblemDetail>) {
    problem(
        StatusCode::GONE,
        "download package expired",
        "download package has expired",
        SdkWorkResultCode::Gone,
    )
}

fn build_download_package_response(
    state: &AppState,
    package: DownloadPackageRecordView,
    signed: SignedDownloadPayload,
) -> DownloadPackageResponse {
    let base = state.download_public_base_url.trim_end_matches('/');
    DownloadPackageResponse {
        id: package.id.clone(),
        tenant_id: package.tenant_id,
        package_name: package.package_name,
        state: package.state,
        storage_provider_id: package.storage_provider_id,
        bucket: package.bucket,
        archive_object_key: package.archive_object_key,
        content_type: package.content_type,
        file_count: package.file_count,
        total_bytes: package.total_bytes,
        archive_size_bytes: package.archive_size_bytes,
        expires_at_epoch_ms: signed.expires_at_epoch_ms,
        download_url: format!("{base}/download_packages/{}/download_url", package.id),
        signed_source_url: signed.raw_url,
        method: signed.method,
        items: package
            .items
            .into_iter()
            .map(|item| DownloadPackageItemResponse {
                node_id: item.node_id,
                node_name: item.node_name,
                archive_path: item.archive_path,
                bucket: item.bucket,
                object_key: item.object_key,
                content_type: item.content_type,
                content_length: item.content_length,
                checksum_sha256_hex: item.checksum_sha256_hex,
            })
            .collect(),
    }
}

fn build_download_package_id(tenant_id: &str, node_ids: &[String]) -> String {
    let epoch = current_epoch_ms().to_string();
    let mut parts: Vec<&[u8]> = vec![tenant_id.trim().as_bytes()];
    parts.extend(node_ids.iter().map(|node_id| node_id.trim().as_bytes()));
    parts.push(epoch.as_bytes());
    let digest = sha256_raw_hex_separated(&parts);
    format!("pkg-{}", &digest[..32])
}

fn build_download_package_object_key(tenant_id: &str, package_id: &str) -> String {
    let tenant_shard = tenant_shard_prefix(tenant_id);
    format!("sdkwork-drive/v1/t/{tenant_shard}/tenants/{tenant_id}/download-packages/{package_id}/archive.zip")
}
