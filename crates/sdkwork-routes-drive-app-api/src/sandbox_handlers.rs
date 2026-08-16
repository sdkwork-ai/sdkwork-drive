use axum::{
    extract::{rejection::JsonRejection, rejection::QueryRejection, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use sdkwork_drive_sandbox_local::LocalSandboxDirectoryProvider;
use sdkwork_drive_workspace_service::{
    application::sandbox_directory_service::{
        CreateSandboxDirectoryCommand, DriveSandboxDirectoryService, ListSandboxDirectoryCommand,
    },
    application::sandbox_file_system_service::{
        CreateSandboxFileCommand, DeleteSandboxEntryCommand, DriveSandboxFileSystemService,
        MoveSandboxEntryCommand, ReadSandboxFileCommand, SandboxMutationContext,
        UpdateSandboxFileCommand,
    },
    application::sandbox_service::DriveSandboxService,
    domain::sandbox_directory::{
        SandboxDirectoryEntry, SandboxEntryKind, MAX_SANDBOX_DIRECTORY_PAGE_SIZE,
        MAX_SANDBOX_FILE_CONTENT_BYTES,
    },
    infrastructure::sql::sandbox_mutation_operation_store::SqlSandboxMutationOperationStore,
    infrastructure::sql::sandbox_store::SqlSandboxStore,
};

use crate::{
    app_context::DriveRequestContext,
    dto::{
        CreateSandboxDirectoryRequest, CreateSandboxFileRequest, ListSandboxEntriesQuery,
        ListSandboxesQuery, MoveSandboxEntryRequest, PurgeSandboxEntryRequest,
        ReadSandboxFileQuery, SandboxCapabilitiesResponse, SandboxEntryResponse,
        SandboxFileContentResponse, SandboxMutationCommandResponse, SandboxVolumeResponse,
        UpdateSandboxFileContentRequest,
    },
    error::{
        invalid_parameter_problem, malformed_request_problem, map_service_error,
        missing_required_field_problem, payload_too_large_problem, precondition_failed_problem,
        precondition_required_problem, validation_problem, ProblemDetail,
    },
    response::{
        success_created_resource, success_cursor_list_page, success_envelope,
        success_offset_list_page, success_resource, DriveListHttpResponse,
    },
    runtime_sandbox_roots::ensure_runtime_sandbox_roots,
    sandbox_principals::token_bound_sandbox_principals,
    state::AppState,
    validators::validate_page_size_i64,
};

const MAX_SANDBOX_BASE64_CONTENT_CHARS: usize = 5_592_408;

pub(crate) async fn list_sandboxes(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    query: Result<Query<ListSandboxesQuery>, QueryRejection>,
) -> Result<
    DriveListHttpResponse<SandboxVolumeResponse>,
    (axum::http::StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let query = decode_query(query)?;
    let page = query.page.unwrap_or(1);
    if page < 1 {
        return Err(crate::error::validation_problem(
            "page must be greater than or equal to 1",
        ));
    }
    let page_size = validate_page_size_i64(query.page_size, 20, 1, 200, "page_size")?;
    let offset = (page - 1)
        .checked_mul(page_size)
        .ok_or_else(|| crate::error::validation_problem("page is too large"))?;
    ensure_runtime_sandbox_roots(&state.pool, &ctx, &state.runtime_sandbox_roots)
        .await
        .map_err(|error| {
            map_service_error(
                sdkwork_drive_workspace_service::DriveServiceError::Internal(format!(
                    "initialize runtime sandbox roots failed: {error}"
                )),
            )
        })?;
    let principals = token_bound_sandbox_principals(&ctx);
    let service = DriveSandboxService::new(SqlSandboxStore::new(state.pool.clone()));
    let (volumes, total) = service
        .list_accessible_for_principals(&tenant_id, &principals, offset, page_size)
        .await
        .map_err(map_service_error)?;
    let items = volumes
        .into_iter()
        .map(|value| {
            let supported = value.provider_kind == "local_filesystem";
            let writable =
                supported && value.lifecycle_status == "active" && value.effective_access == "full";
            SandboxVolumeResponse {
                id: value.id,
                display_name: value.display_name,
                root_entry_id: value.root_entry_id,
                effective_access: value.effective_access,
                lifecycle_status: value.lifecycle_status,
                capabilities: SandboxCapabilitiesResponse {
                    browse: supported,
                    create_directory: writable,
                    select_directory: supported,
                    read_file: supported,
                    create_file: writable,
                    write_file: writable,
                    move_entry: writable,
                    delete_entry: writable,
                },
                revision: value.version.to_string(),
            }
        })
        .collect();
    Ok(success_offset_list_page(
        items,
        page as i32,
        page_size as i32,
        total,
    ))
}

pub(crate) async fn list_sandbox_entries(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(sandbox_id): Path<String>,
    query: Result<Query<ListSandboxEntriesQuery>, QueryRejection>,
) -> Result<
    DriveListHttpResponse<SandboxEntryResponse>,
    (axum::http::StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let query = decode_query(query)?;
    let page_size = validate_page_size_i64(
        query.page_size,
        20,
        1,
        MAX_SANDBOX_DIRECTORY_PAGE_SIZE as i64,
        "page_size",
    )?;
    let principals = token_bound_sandbox_principals(&ctx);
    let service = sandbox_directory_service(&state);
    let page = service
        .list_children(ListSandboxDirectoryCommand {
            tenant_id,
            sandbox_id,
            principals,
            parent_logical_path: query.parent_path.unwrap_or_default(),
            page_size: page_size as usize,
            cursor: query.cursor,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_cursor_list_page(
        page.items.into_iter().map(map_sandbox_entry).collect(),
        page_size as i32,
        page.next_cursor,
    ))
}

pub(crate) async fn create_sandbox_directory(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(sandbox_id): Path<String>,
    headers: HeaderMap,
    request: Result<Json<CreateSandboxDirectoryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let request = decode_json(request)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let idempotency_key = required_header(&headers, "idempotency-key", "Idempotency-Key")?;
    let principals = token_bound_sandbox_principals(&ctx);
    let service = sandbox_directory_service(&state);
    let entry = service
        .create_directory(CreateSandboxDirectoryCommand {
            tenant_id,
            sandbox_id,
            principals,
            parent_logical_path: request.parent_path,
            name: request.name,
            operator_id: ctx.actor_id,
            request_id: Some(ctx.request_id),
            trace_id: Some(ctx.trace_id),
            idempotency_key,
        })
        .await
        .map_err(map_service_error)?;
    Ok(success_created_resource(map_sandbox_entry(entry)))
}

pub(crate) async fn read_sandbox_file_content(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((sandbox_id, entry_id)): Path<(String, String)>,
    query: Result<Query<ReadSandboxFileQuery>, QueryRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let query = decode_query(query)?;
    let encoding = SandboxContentEncoding::parse(query.encoding.as_deref())?;
    let content = sandbox_file_system_service(&state)
        .read_file(ReadSandboxFileCommand {
            tenant_id,
            sandbox_id,
            principals: token_bound_sandbox_principals(&ctx),
            entry_id,
            logical_path: query.logical_path,
        })
        .await
        .map_err(map_sandbox_file_error)?;
    let checksum_sha256 = sdkwork_utils_rust::sha256_hash(&content.bytes);
    let size_bytes = content.bytes.len().to_string();
    let encoded = encoding.encode(content.bytes)?;
    Ok(success_resource(SandboxFileContentResponse {
        entry: map_sandbox_entry(content.entry),
        encoding: encoding.as_str().to_string(),
        content: encoded,
        size_bytes,
        checksum_sha256,
    }))
}

pub(crate) async fn create_sandbox_file(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path(sandbox_id): Path<String>,
    headers: HeaderMap,
    request: Result<Json<CreateSandboxFileRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let request = decode_json(request)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let bytes = decode_request_content(&request.encoding, &request.content)?;
    let mutation = mutation_context(&ctx, &headers)?;
    let entry = sandbox_file_system_service(&state)
        .create_file(CreateSandboxFileCommand {
            tenant_id,
            sandbox_id,
            principals: token_bound_sandbox_principals(&ctx),
            parent_logical_path: request.parent_path,
            name: request.name,
            bytes,
            mutation,
        })
        .await
        .map_err(map_sandbox_file_error)?;
    Ok(success_created_resource(map_sandbox_entry(entry)))
}

pub(crate) async fn update_sandbox_file_content(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((sandbox_id, entry_id)): Path<(String, String)>,
    headers: HeaderMap,
    request: Result<Json<UpdateSandboxFileContentRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let request = decode_json(request)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let expected_revision = required_if_match(&headers)?;
    let bytes = decode_request_content(&request.encoding, &request.content)?;
    let mutation = mutation_context(&ctx, &headers)?;
    let entry = sandbox_file_system_service(&state)
        .update_file(UpdateSandboxFileCommand {
            tenant_id,
            sandbox_id,
            principals: token_bound_sandbox_principals(&ctx),
            entry_id,
            logical_path: request.logical_path,
            expected_revision,
            bytes,
            mutation,
        })
        .await
        .map_err(map_sandbox_file_error)?;
    Ok(success_resource(map_sandbox_entry(entry)))
}

pub(crate) async fn move_sandbox_entry(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((sandbox_id, entry_id)): Path<(String, String)>,
    headers: HeaderMap,
    request: Result<Json<MoveSandboxEntryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let request = decode_json(request)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let expected_revision = required_if_match(&headers)?;
    let mutation = mutation_context(&ctx, &headers)?;
    let entry = sandbox_file_system_service(&state)
        .move_entry(MoveSandboxEntryCommand {
            tenant_id,
            sandbox_id,
            principals: token_bound_sandbox_principals(&ctx),
            entry_id,
            logical_path: request.logical_path,
            destination_parent_logical_path: request.destination_parent_path,
            destination_name: request.destination_name,
            expected_revision,
            mutation,
        })
        .await
        .map_err(map_sandbox_file_error)?;
    Ok(success_resource(map_sandbox_entry(entry)))
}

pub(crate) async fn purge_sandbox_entry(
    State(state): State<AppState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Path((sandbox_id, entry_id)): Path<(String, String)>,
    headers: HeaderMap,
    request: Result<Json<PurgeSandboxEntryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let request = decode_json(request)?;
    let tenant_id = ctx.resolve_tenant_id()?;
    let expected_revision = required_if_match(&headers)?;
    let mutation = mutation_context(&ctx, &headers)?;
    sandbox_file_system_service(&state)
        .delete_entry(DeleteSandboxEntryCommand {
            tenant_id,
            sandbox_id,
            principals: token_bound_sandbox_principals(&ctx),
            entry_id: entry_id.clone(),
            logical_path: request.logical_path,
            expected_revision,
            recursive: request.recursive,
            mutation,
        })
        .await
        .map_err(map_sandbox_file_error)?;
    Ok(success_envelope(SandboxMutationCommandResponse {
        accepted: true,
        resource_id: entry_id,
        status: "deleted".to_string(),
    }))
}

fn sandbox_directory_service(
    state: &AppState,
) -> DriveSandboxDirectoryService<
    SqlSandboxStore,
    LocalSandboxDirectoryProvider,
    SqlSandboxMutationOperationStore,
> {
    DriveSandboxDirectoryService::new(
        SqlSandboxStore::new(state.pool.clone()),
        LocalSandboxDirectoryProvider,
        SqlSandboxMutationOperationStore::new(state.pool.clone()),
    )
}

fn sandbox_file_system_service(
    state: &AppState,
) -> DriveSandboxFileSystemService<
    SqlSandboxStore,
    LocalSandboxDirectoryProvider,
    SqlSandboxMutationOperationStore,
> {
    DriveSandboxFileSystemService::new(
        SqlSandboxStore::new(state.pool.clone()),
        LocalSandboxDirectoryProvider,
        SqlSandboxMutationOperationStore::new(state.pool.clone()),
    )
}

fn map_sandbox_entry(value: SandboxDirectoryEntry) -> SandboxEntryResponse {
    SandboxEntryResponse {
        id: value.id,
        sandbox_id: value.sandbox_id,
        parent_id: Some(value.parent_id),
        name: value.name,
        kind: match value.kind {
            SandboxEntryKind::Directory => "directory".to_string(),
            SandboxEntryKind::File => "file".to_string(),
        },
        logical_path: value.logical_path,
        revision: value.revision,
    }
}

#[derive(Debug, Clone, Copy)]
enum SandboxContentEncoding {
    Utf8,
    Base64,
}

impl SandboxContentEncoding {
    fn parse(value: Option<&str>) -> Result<Self, (axum::http::StatusCode, Json<ProblemDetail>)> {
        match value.unwrap_or("utf8") {
            "utf8" => Ok(Self::Utf8),
            "base64" => Ok(Self::Base64),
            _ => Err(validation_problem("encoding must be utf8 or base64")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Base64 => "base64",
        }
    }

    fn encode(
        self,
        bytes: Vec<u8>,
    ) -> Result<String, (axum::http::StatusCode, Json<ProblemDetail>)> {
        match self {
            Self::Utf8 => String::from_utf8(bytes).map_err(|_| {
                validation_problem("sandbox file is not valid UTF-8; request base64 encoding")
            }),
            Self::Base64 => Ok(sdkwork_utils_rust::base64_encode(&bytes)),
        }
    }
}

fn decode_request_content(
    encoding: &str,
    content: &str,
) -> Result<Vec<u8>, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let parsed_encoding = SandboxContentEncoding::parse(Some(encoding))?;
    let encoded_limit = match parsed_encoding {
        SandboxContentEncoding::Utf8 => MAX_SANDBOX_FILE_CONTENT_BYTES,
        SandboxContentEncoding::Base64 => MAX_SANDBOX_BASE64_CONTENT_CHARS,
    };
    if content.len() > encoded_limit {
        return Err(payload_too_large_problem(format!(
            "sandbox file content exceeds the {MAX_SANDBOX_FILE_CONTENT_BYTES} byte limit"
        )));
    }
    let bytes = match parsed_encoding {
        SandboxContentEncoding::Utf8 => content.as_bytes().to_vec(),
        SandboxContentEncoding::Base64 => sdkwork_utils_rust::base64_decode(content)
            .ok_or_else(|| validation_problem("content is not valid base64"))?,
    };
    if bytes.len() > MAX_SANDBOX_FILE_CONTENT_BYTES {
        return Err(payload_too_large_problem(format!(
            "sandbox file content exceeds the {MAX_SANDBOX_FILE_CONTENT_BYTES} byte limit"
        )));
    }
    Ok(bytes)
}

fn decode_json<T>(
    payload: Result<Json<T>, JsonRejection>,
) -> Result<T, (axum::http::StatusCode, Json<ProblemDetail>)> {
    payload.map(|Json(value)| value).map_err(|_| {
        malformed_request_problem("request body is malformed or contains unknown fields")
    })
}

fn decode_query<T>(
    query: Result<Query<T>, QueryRejection>,
) -> Result<T, (axum::http::StatusCode, Json<ProblemDetail>)> {
    query.map(|Query(value)| value).map_err(|_| {
        invalid_parameter_problem("query contains an invalid or unsupported parameter")
    })
}

fn mutation_context(
    ctx: &DriveRequestContext,
    headers: &HeaderMap,
) -> Result<SandboxMutationContext, (axum::http::StatusCode, Json<ProblemDetail>)> {
    Ok(SandboxMutationContext {
        operator_id: ctx.actor_id.clone(),
        request_id: Some(ctx.request_id.clone()),
        trace_id: Some(ctx.trace_id.clone()),
        idempotency_key: required_header(headers, "idempotency-key", "Idempotency-Key")?,
    })
}

fn required_if_match(
    headers: &HeaderMap,
) -> Result<String, (axum::http::StatusCode, Json<ProblemDetail>)> {
    let value = headers
        .get("if-match")
        .ok_or_else(|| precondition_required_problem("If-Match header is required"))?
        .to_str()
        .map_err(|_| validation_problem("If-Match header is invalid"))?;
    if value.starts_with("W/")
        || value.len() < 3
        || !value.starts_with('"')
        || !value.ends_with('"')
    {
        return Err(validation_problem(
            "If-Match must contain one quoted strong sandbox revision",
        ));
    }
    let revision = &value[1..value.len() - 1];
    if revision.is_empty()
        || revision.len() > 128
        || revision
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b'"')
    {
        return Err(validation_problem("If-Match revision is invalid"));
    }
    Ok(revision.to_string())
}

fn required_header(
    headers: &HeaderMap,
    wire_name: &str,
    display_name: &str,
) -> Result<String, (axum::http::StatusCode, Json<ProblemDetail>)> {
    headers
        .get(wire_name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| missing_required_field_problem(format!("{display_name} header is required")))
}

fn map_sandbox_file_error(
    error: sdkwork_drive_workspace_service::DriveServiceError,
) -> (axum::http::StatusCode, Json<ProblemDetail>) {
    match error {
        sdkwork_drive_workspace_service::DriveServiceError::Conflict(detail)
            if detail.contains("revision does not match If-Match") =>
        {
            precondition_failed_problem(detail)
        }
        sdkwork_drive_workspace_service::DriveServiceError::Validation(detail)
            if detail.contains("byte limit") =>
        {
            payload_too_large_problem(detail)
        }
        other => map_service_error(other),
    }
}
