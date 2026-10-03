use crate::error::{map_object_store_route_error, validation_problem, ProblemDetail};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::api::pagination_cursor::{decode_offset_cursor, encode_offset_cursor};
use sdkwork_drive_storage_contract::{validate_s3_bucket_name, validate_s3_region_token};
use sdkwork_utils_rust::{DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE};

/// 路径参数形式的对象 key 解码与校验。
///
/// axum 的 `Path` 提取器已对路径段做一次 percent-decode，这里**不再手动解码**，
/// 否则字面 `%` 字符会被二次解码静默改写（如 `50%20off.txt` → `50 off.txt`）。
/// 允许单个尾斜杠（目录占位对象，如 `docs/`），拒绝前导斜杠、双斜杠与
/// 空/`.`/`..` 段；返回规范化后的 key（尾斜杠折叠为单个）。
pub(crate) fn decode_path_object_key(
    raw: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    validate_path_object_key(raw, "objectKey")
}

fn validate_path_object_key(
    value: &str,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = require_non_empty_text(value.to_string(), field_name)?;
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(validation_problem(format!(
            "{field_name} must not contain NUL bytes"
        )));
    }
    if trimmed.starts_with('/') {
        return Err(validation_problem(format!(
            "{field_name} must not start with slash"
        )));
    }
    let (effective, has_trailing_slash) = match trimmed.strip_suffix('/') {
        Some(rest) => (rest, true),
        None => (trimmed.as_str(), false),
    };
    if effective.is_empty() {
        return Err(validation_problem(format!(
            "{field_name} must not be only slashes"
        )));
    }
    for segment in effective.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(if has_trailing_slash {
        format!("{effective}/")
    } else {
        effective.to_string()
    })
}

pub(crate) fn validate_object_key(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = require_non_empty_text(value, field_name)?;
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) {
        return Err(validation_problem(format!(
            "{field_name} must not contain NUL bytes"
        )));
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') {
        return Err(validation_problem(format!(
            "{field_name} must not start or end with slash"
        )));
    }
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(trimmed)
}

pub(crate) fn validate_object_prefix(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(trimmed) = normalize_optional_text(value) else {
        return Ok(None);
    };
    if trimmed.len() > 1024 {
        return Err(validation_problem(format!(
            "{field_name} must be at most 1024 UTF-8 bytes"
        )));
    }
    if trimmed.as_bytes().contains(&0) || trimmed.starts_with('/') {
        return Err(validation_problem(format!("{field_name} is invalid")));
    }
    if trimmed.contains("//") {
        return Err(validation_problem(format!(
            "{field_name} must not contain empty path segments"
        )));
    }
    for segment in trimmed.trim_end_matches('/').split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(validation_problem(format!(
                "{field_name} must not contain empty or period-only path segments"
            )));
        }
    }
    Ok(Some(trimmed))
}

pub(crate) fn validate_object_delimiter(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(trimmed) = normalize_optional_text(value) else {
        return Ok(None);
    };
    if trimmed != "/" {
        return Err(validation_problem(format!(
            "{field_name} must be '/' when provided"
        )));
    }
    Ok(Some(trimmed))
}

/// 解析本次对象操作的目标存储桶。
///
/// 管理端的存储桶浏览器读的是厂商账号下的任意一个桶，而对象路由是按“服务商配置”
/// 组织的：显式传了 `bucket` 就以它为准（仍然使用该配置的端点、区域与凭证），
/// 没传就沿用配置里设定的桶。桶名按 S3 规则校验，避免把任意字符串拼进厂商请求。
pub(crate) fn resolve_object_bucket(
    provider_bucket: &str,
    requested: Option<String>,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let Some(bucket) = normalize_optional_text(requested) else {
        return Ok(provider_bucket.to_string());
    };
    validate_s3_bucket_name(&bucket, field_name).map_err(map_object_store_route_error)?;
    Ok(bucket)
}

/// 解析本次对象操作要用的厂商地域覆盖。
///
/// 桶清单是账号级读取，跨地域；点开某个桶时要按"这个桶所在地域"寻址，才是各厂商的
/// 标准规范。地域来自客户端（清单里那一行），所以要按厂商地域码的形状校验，避免把任意
/// 字符串拼进端点主机；厂商规范表达不出来时由 `endpoint_for_region` 拒绝并退回配置端点。
pub(crate) fn resolve_object_region(
    requested: Option<String>,
    field_name: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let Some(region) = normalize_optional_text(requested) else {
        return Ok(None);
    };
    validate_s3_region_token(&region, field_name).map_err(map_object_store_route_error)?;
    Ok(Some(region))
}

pub(crate) fn validate_page_size_u16(
    value: Option<u16>,
    default_value: u16,
    min_value: u16,
    max_value: u16,
    field_name: &str,
) -> Result<u16, (StatusCode, Json<ProblemDetail>)> {
    let page_size = value.unwrap_or(default_value);
    if page_size < min_value || page_size > max_value {
        return Err(validation_problem(format!(
            "{field_name} must be between {min_value} and {max_value}"
        )));
    }
    Ok(page_size)
}

pub(crate) fn parse_offset_page(
    page_size: Option<i64>,
    page_token: Option<String>,
) -> Result<crate::dto::OffsetPage, (StatusCode, Json<ProblemDetail>)> {
    let limit = validate_page_size_i64(
        page_size,
        i64::from(DEFAULT_LIST_PAGE_SIZE),
        1,
        i64::from(MAX_LIST_PAGE_SIZE),
        "page_size",
    )?;
    let offset = decode_offset_cursor(page_token.as_deref())
        .map_err(|_| validation_problem("cursor is invalid"))?;
    Ok(crate::dto::OffsetPage { limit, offset })
}

pub(crate) fn validate_page_size_i64(
    value: Option<i64>,
    default_value: i64,
    min_value: i64,
    max_value: i64,
    field_name: &str,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    let page_size = value.unwrap_or(default_value);
    if page_size < min_value || page_size > max_value {
        return Err(validation_problem(format!(
            "{field_name} must be between {min_value} and {max_value}"
        )));
    }
    Ok(page_size)
}

pub(crate) fn next_page_token<T>(
    items: &mut Vec<T>,
    page: crate::dto::OffsetPage,
) -> Option<String> {
    if items.len() as i64 > page.limit {
        items.pop();
        encode_offset_cursor(page.offset + page.limit)
    } else {
        None
    }
}

pub(crate) fn validate_storage_binding_lifecycle_status(
    status: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(status, "active" | "disabled" | "deleted") {
        return Ok(());
    }
    Err(validation_problem("lifecycleStatus is invalid"))
}

/// Validate the resolution step a binding list is narrowed to.
///
/// Modelled as a validator rather than a passthrough because the values are a
/// closed set that mirrors `ck_dr_drive_storage_provider_binding_scope`: a typo
/// would otherwise answer an empty page, which reads as "this tenant has no
/// space-type bindings" — the exact false statement the filter exists to avoid.
pub(crate) fn validate_storage_binding_scope(
    scope: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(scope, "tenant" | "space" | "space_type") {
        return Ok(());
    }
    Err(validation_problem("bindingScope is invalid"))
}

/// Validate a client-supplied migration item status filter.
///
/// Modelled as a validator rather than reusing the domain parser because the
/// two have different meanings: the domain parser reads a *persisted* value, so
/// an unrecognized one is a corrupt row (an internal fault), whereas an
/// unrecognized *query* value is a malformed request and must answer `400`.
pub(crate) fn validate_storage_migration_item_status(
    status: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(status, "pending" | "copied" | "failed") {
        return Ok(());
    }
    Err(validation_problem("status is invalid"))
}

/// Validate a client-supplied migration run status filter.
pub(crate) fn validate_storage_migration_status(
    status: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if matches!(
        status,
        "pending" | "running" | "succeeded" | "failed" | "cancelled"
    ) {
        return Ok(());
    }
    Err(validation_problem("status is invalid"))
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

pub(crate) fn require_non_empty_text(
    value: String,
    field_name: &str,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(validation_problem(format!("{field_name} is required")));
    }
    Ok(trimmed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StorageProviderBindingTarget {
    Tenant,
    Space(String),
    SpaceType(String),
}

pub(crate) fn resolve_storage_provider_binding_target(
    space_id: Option<String>,
    space_type: Option<String>,
) -> Result<StorageProviderBindingTarget, (StatusCode, Json<ProblemDetail>)> {
    let space_id = normalize_optional_text(space_id);
    let space_type = normalize_optional_text(space_type);
    if space_id.is_some() && space_type.is_some() {
        return Err(validation_problem(
            "spaceId and spaceType are mutually exclusive",
        ));
    }
    if let Some(space_id) = space_id {
        return Ok(StorageProviderBindingTarget::Space(space_id));
    }
    if let Some(space_type) = space_type {
        validate_storage_binding_space_type(&space_type)?;
        return Ok(StorageProviderBindingTarget::SpaceType(space_type));
    }
    Ok(StorageProviderBindingTarget::Tenant)
}

pub(crate) fn validate_storage_binding_space_type(
    space_type: &str,
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    match space_type {
        "personal" | "team" | "knowledge_base" | "ai_generated" | "git_repository"
        | "deployment" | "app_upload" | "im" | "rtc" | "notary" | "website" => Ok(()),
        _ => Err(validation_problem("spaceType is invalid")),
    }
}

pub(crate) fn default_storage_provider_binding_id(
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> String {
    match target {
        StorageProviderBindingTarget::Tenant => format!("default:tenant:{tenant_id}"),
        StorageProviderBindingTarget::Space(space_id) => {
            format!("default:space:{tenant_id}:{space_id}")
        }
        StorageProviderBindingTarget::SpaceType(space_type) => {
            format!("default:space_type:{tenant_id}:{space_type}")
        }
    }
}

pub(crate) fn storage_provider_binding_scope(
    target: &StorageProviderBindingTarget,
) -> &'static str {
    match target {
        StorageProviderBindingTarget::Tenant => "tenant",
        StorageProviderBindingTarget::Space(_) => "space",
        StorageProviderBindingTarget::SpaceType(_) => "space_type",
    }
}

pub(crate) fn storage_provider_binding_purpose(target: &StorageProviderBindingTarget) -> String {
    match target {
        StorageProviderBindingTarget::Tenant | StorageProviderBindingTarget::Space(_) => {
            "primary".to_string()
        }
        StorageProviderBindingTarget::SpaceType(space_type) => space_type.clone(),
    }
}

pub(crate) fn default_storage_root_prefix(
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> String {
    match target {
        StorageProviderBindingTarget::Tenant => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}")
        }
        StorageProviderBindingTarget::Space(space_id) => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}/spaces/{space_id}")
        }
        StorageProviderBindingTarget::SpaceType(space_type) => {
            format!("sdkwork-drive/v1/tenants/{tenant_id}/space-types/{space_type}")
        }
    }
}

pub(crate) fn normalize_storage_root_prefix(
    value: Option<String>,
    tenant_id: &str,
    target: &StorageProviderBindingTarget,
) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let prefix = normalize_optional_text(value)
        .unwrap_or_else(|| default_storage_root_prefix(tenant_id, target));
    let prefix = validate_object_prefix(Some(prefix), "storageRootPrefix")?
        .ok_or_else(|| validation_problem("storageRootPrefix is required"))?;
    if prefix.len() > 512 {
        return Err(validation_problem(
            "storageRootPrefix must be at most 512 UTF-8 bytes",
        ));
    }
    if prefix.ends_with('/') {
        return Err(validation_problem(
            "storageRootPrefix must not end with slash",
        ));
    }
    Ok(prefix)
}

/// 默认的分片授权有效期（秒）：一小时，够一批分片传完，又不至于让 URL 长期可用。
const DEFAULT_UPLOAD_PART_GRANT_SECONDS: u32 = 3600;
/// 分片授权有效期的上下界（秒）：一分钟到七天，与契约声明一致。
const MIN_UPLOAD_PART_GRANT_SECONDS: u32 = 60;
const MAX_UPLOAD_PART_GRANT_SECONDS: u32 = 604_800;

/// 校验并归一化分片授权有效期。
pub(crate) fn validate_grant_lifetime(
    value: Option<u32>,
) -> Result<u32, (StatusCode, Json<ProblemDetail>)> {
    let seconds = value.unwrap_or(DEFAULT_UPLOAD_PART_GRANT_SECONDS);
    if !(MIN_UPLOAD_PART_GRANT_SECONDS..=MAX_UPLOAD_PART_GRANT_SECONDS).contains(&seconds) {
        return Err(validation_problem(format!(
            "expiresInSeconds must be between {MIN_UPLOAD_PART_GRANT_SECONDS} and {MAX_UPLOAD_PART_GRANT_SECONDS}"
        )));
    }
    Ok(seconds)
}

/// 校验厂商的 `uploadId`。
///
/// 它是**不透明**令牌：本服务不解析、只回传，所以这里只做"非空且长度合理"的检查。长度上限
/// 与契约一致（2048），避免把它当自由文本用于日志/审计之外的任何拼接。
pub(crate) fn validate_upload_id(value: &str) -> Result<String, (StatusCode, Json<ProblemDetail>)> {
    let upload_id = value.trim();
    if upload_id.is_empty() {
        return Err(validation_problem("uploadId is required"));
    }
    if upload_id.len() > 2048 {
        return Err(validation_problem("uploadId must be at most 2048 bytes"));
    }
    Ok(upload_id.to_string())
}

/// 一次请求最多签发多少个分片授权。///
/// 上限同时存在于契约（`partNumbers` maxItems）与这里：客户端要并发上传，所以按批签发，
/// 但一批无限大就等于把"签发"变成一次遍历整对象的操作。
pub(crate) const MAX_PRESIGNED_UPLOAD_PARTS: usize = 100;

/// 单一对象的分片数上限（S3/COS 一致）。
pub(crate) const MAX_UPLOAD_PARTS: u16 = 10_000;

/// 校验要签发的分片号。
///
/// 分片号由客户端给出，厂商只会在收到分片时才抱怨，所以这里先挡一遍：
/// - 空数组：契约要求至少一片，空批是客户端 bug（界面会以为签过了）；
/// - 超出 1..=10000：厂商直接拒绝，且 0 会被当成非法 partNumber；
/// - 重复：同一片签两次没有意义，还会掩盖客户端的分片编号错误；
/// - 单批超过上限：拒绝而不是截断，否则客户端会等一个永远不会到来的授权。
pub(crate) fn validate_upload_part_numbers(
    part_numbers: &[u16],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if part_numbers.is_empty() {
        return Err(validation_problem(
            "partNumbers must contain at least one part number",
        ));
    }
    if part_numbers.len() > MAX_PRESIGNED_UPLOAD_PARTS {
        return Err(validation_problem(format!(
            "partNumbers must contain at most {MAX_PRESIGNED_UPLOAD_PARTS} part numbers per request"
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for part_number in part_numbers {
        if *part_number == 0 || *part_number > MAX_UPLOAD_PARTS {
            return Err(validation_problem(format!(
                "partNumber must be between 1 and {MAX_UPLOAD_PARTS}"
            )));
        }
        if !seen.insert(*part_number) {
            return Err(validation_problem(format!(
                "partNumber {part_number} is requested twice"
            )));
        }
    }
    Ok(())
}

/// 校验完成分片上传时提交的分片清单。
///
/// 厂商按"分片号 + ETag"重组对象，列表错了会得到一句没有上下文的上报错，所以这里先验：
/// - 非空；
/// - 分片号在 1..=10000 且不重复；
/// - ETag 非空（浏览器读不到 ETag 时常常拿到空串，这是 CORS 少配 `ExposeHeaders` 的典型
///   表现，提前用明确的 400 说明，比让厂商返回 400 更好定位）；
/// - 分片号必须从 1 连续，因为厂商要求提交的就是"已上传的那一组分片"，中间缺号说明客户端
///   漏传了一片。
pub(crate) fn validate_completed_upload_parts(
    parts: &[(u16, String)],
) -> Result<(), (StatusCode, Json<ProblemDetail>)> {
    if parts.is_empty() {
        return Err(validation_problem(
            "parts must contain at least one uploaded part",
        ));
    }
    if parts.len() > usize::from(MAX_UPLOAD_PARTS) {
        return Err(validation_problem(format!(
            "parts must contain at most {MAX_UPLOAD_PARTS} uploaded parts"
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (part_number, etag) in parts {
        if *part_number == 0 || *part_number > MAX_UPLOAD_PARTS {
            return Err(validation_problem(format!(
                "partNumber must be between 1 and {MAX_UPLOAD_PARTS}"
            )));
        }
        if etag.trim().is_empty() {
            return Err(validation_problem(format!(
                "etag is required for partNumber {part_number}; browsers can only read it when the bucket CORS policy exposes the ETag header"
            )));
        }
        if etag.len() > 256 {
            return Err(validation_problem(format!(
                "etag for partNumber {part_number} is longer than 256 bytes"
            )));
        }
        if !seen.insert(*part_number) {
            return Err(validation_problem(format!(
                "partNumber {part_number} is submitted twice"
            )));
        }
    }
    // 提交顺序不要求与请求一致（客户端并发收集），但集合必须是 1..=n。
    let highest = seen.iter().next_back().copied().unwrap_or(0);
    if usize::from(highest) != seen.len() {
        return Err(validation_problem(format!(
            "parts must be numbered contiguously from 1; got {} parts up to partNumber {highest}",
            seen.len()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod pagination_cursor_tests {
    use super::{next_page_token, parse_offset_page};
    use crate::dto::OffsetPage;

    fn is_numeric_token(value: &str) -> bool {
        value.bytes().all(|byte| byte.is_ascii_digit())
    }

    #[test]
    fn offset_page_tokens_are_opaque_and_round_trip() {
        let mut items = vec![1, 2];

        let token = next_page_token(
            &mut items,
            OffsetPage {
                limit: 1,
                offset: 0,
            },
        )
        .expect("first page should expose continuation token");

        assert!(!is_numeric_token(&token));
        let parsed = parse_offset_page(Some(1), Some(token)).expect("opaque cursor should parse");
        assert_eq!(parsed.offset, 1);
        assert_eq!(items, vec![1]);
    }

    #[test]
    fn offset_page_rejects_numeric_cursor_alias() {
        let err = parse_offset_page(Some(20), Some("20".to_string()))
            .expect_err("numeric cursor is pre-launch pagination debt");

        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}

#[cfg(test)]
mod object_bucket_tests {
    use super::{resolve_object_bucket, resolve_object_region};
    use axum::http::StatusCode;

    #[test]
    fn a_missing_or_blank_override_keeps_the_configured_bucket() {
        assert_eq!(
            resolve_object_bucket("bucket-admin", None, "bucket").expect("absent override resolves"),
            "bucket-admin"
        );
        assert_eq!(
            resolve_object_bucket("bucket-admin", Some("   ".to_string()), "bucket")
                .expect("blank override resolves"),
            "bucket-admin"
        );
    }

    #[test]
    fn an_explicit_override_wins_and_is_trimmed() {
        assert_eq!(
            resolve_object_bucket("bucket-admin", Some("  image2-1253947560 ".to_string()), "bucket")
                .expect("valid override resolves"),
            "image2-1253947560"
        );
    }

    #[test]
    fn an_invalid_override_is_rejected_before_any_vendor_call() {
        for invalid in ["Drive_Bucket", "ab", "bucket_admin"] {
            let error = resolve_object_bucket("bucket-admin", Some(invalid.to_string()), "sourceBucket")
                .expect_err("invalid bucket must be rejected");
            assert_eq!(error.0, StatusCode::BAD_REQUEST);
            let body = serde_json::to_value(&error.1 .0).expect("problem detail should serialize");
            assert!(
                body["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("sourceBucket")),
                "validation detail should name the offending field: {body}"
            );
        }
    }

    /// 地域覆盖会参与端点主机的推导，所以它按厂商地域码的形状校验：缺省就是"用配置里的
    /// 地域"，非法值必须在任何厂商调用之前被拒掉。
    #[test]    fn region_override_requires_a_vendor_region_code() {
        assert_eq!(
            resolve_object_region(None, "region").expect("absent override resolves"),
            None
        );
        assert_eq!(
            resolve_object_region(Some("   ".to_string()), "region")
                .expect("blank override resolves"),
            None
        );
        assert_eq!(
            resolve_object_region(Some(" ap-beijing ".to_string()), "region")
                .expect("valid override resolves"),
            Some("ap-beijing".to_string())
        );

        for invalid in ["ap_beijing", "ap beijing", "ap-beijing/", "..", "-ap"] {
            let error = resolve_object_region(Some(invalid.to_string()), "region")
                .expect_err("invalid region must be rejected");
            assert_eq!(error.0, StatusCode::BAD_REQUEST, "{invalid} must be a 400");
            let body = serde_json::to_value(&error.1 .0).expect("problem detail should serialize");
            assert!(
                body["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("region")),
                "validation detail should name the offending field: {body}"
            );
        }
    }
}

#[cfg(test)]
mod multipart_part_tests {
    use super::{
        validate_completed_upload_parts, validate_upload_part_numbers, MAX_PRESIGNED_UPLOAD_PARTS,
    };
    use axum::http::StatusCode;

    fn detail_of(error: &(StatusCode, axum::Json<crate::error::ProblemDetail>)) -> String {
        let body = serde_json::to_value(&error.1 .0).expect("problem detail should serialize");
        body["detail"].as_str().unwrap_or_default().to_string()
    }

    #[test]
    fn presign_part_numbers_must_be_a_bounded_unique_batch() {
        validate_upload_part_numbers(&[1, 2, 3]).expect("a well-formed batch is accepted");

        let empty = validate_upload_part_numbers(&[]).expect_err("an empty batch is rejected");
        assert_eq!(empty.0, StatusCode::BAD_REQUEST);
        assert!(detail_of(&empty).contains("partNumbers"));

        let zero = validate_upload_part_numbers(&[0]).expect_err("part 0 is rejected");
        assert_eq!(zero.0, StatusCode::BAD_REQUEST);
        let too_high =
            validate_upload_part_numbers(&[10_001]).expect_err("part 10001 is rejected");
        assert_eq!(too_high.0, StatusCode::BAD_REQUEST);

        let duplicate =
            validate_upload_part_numbers(&[2, 2]).expect_err("a duplicate part is rejected");
        assert!(detail_of(&duplicate).contains("twice"));

        let oversized: Vec<u16> = (1..=(MAX_PRESIGNED_UPLOAD_PARTS as u16 + 1)).collect();
        let too_many =
            validate_upload_part_numbers(&oversized).expect_err("an oversized batch is rejected");
        assert!(detail_of(&too_many).contains("at most"));
    }

    #[test]
    fn completed_parts_must_be_contiguous_and_carry_etags() {
        validate_completed_upload_parts(&[(1, "etag-1".into()), (2, "etag-2".into())])
            .expect("a contiguous list is accepted");

        // 乱序但集合完整：客户端并发收集 ETag，顺序不该成为失败原因。
        validate_completed_upload_parts(&[(2, "etag-2".into()), (1, "etag-1".into())])
            .expect("an out-of-order but complete list is accepted");

        let empty = validate_completed_upload_parts(&[]).expect_err("an empty list is rejected");
        assert_eq!(empty.0, StatusCode::BAD_REQUEST);

        // 缺号：说明客户端漏传了一片，必须在提交前拦住。
        let gap = validate_completed_upload_parts(&[(1, "a".into()), (3, "c".into())])
            .expect_err("a gap in part numbers is rejected");
        assert!(detail_of(&gap).contains("contiguously"));

        // 浏览器读不到 ETag 时是空串：文案必须指向 CORS 而不是让厂商报 400。
        let missing_etag = validate_completed_upload_parts(&[(1, "  ".into())])
            .expect_err("a blank etag is rejected");
        assert!(detail_of(&missing_etag).contains("ETag"));

        let duplicate = validate_completed_upload_parts(&[(1, "a".into()), (1, "b".into())])
            .expect_err("a duplicate part is rejected");
        assert!(detail_of(&duplicate).contains("twice"));

        let zero = validate_completed_upload_parts(&[(0, "a".into())])
            .expect_err("part 0 is rejected");
        assert_eq!(zero.0, StatusCode::BAD_REQUEST);
    }
}
