use std::io;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{
    ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, IF_MATCH,
    IF_MODIFIED_SINCE, IF_NONE_MATCH, IF_RANGE, IF_UNMODIFIED_SINCE, LAST_MODIFIED, RANGE,
};
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use futures_util::stream;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStoreError, DriveObjectStoreErrorKind,
    HeadObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_web_core::RequireInternalApi;

use crate::dto::RetrieveDriveResourceContentQuery;
use crate::error::{
    map_object_store_error, map_service_error, precondition_failed, range_not_satisfiable,
    RouteProblem,
};
use crate::handlers::{parse_timestamp, resolve_resource};
use crate::state::InternalApiState;

struct ContentResponseSpec<'a> {
    status: StatusCode,
    content_type: &'a str,
    response_length: u64,
    full_length: u64,
    selected_range: Option<DriveByteRange>,
    etag: &'a str,
    last_modified: DateTime<Utc>,
}

pub async fn retrieve_drive_resource_content(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(node_version_id): Path<String>,
    Query(query): Query<RetrieveDriveResourceContentQuery>,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    uuid::Uuid::parse_str(&query.scope_uuid)
        .map_err(|_| crate::error::invalid_parameter("scopeUuid must be a UUID").into_response())?;
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(|| crate::error::missing_internal_principal().into_response())?;
    let resource = resolve_resource(
        &state,
        principal.tenant_id(),
        &query.scope_type,
        query.scope_uuid,
        query.relative_path,
        query.pinned_generation,
        Some(node_version_id.clone()),
    )
    .await
    .map_err(IntoResponse::into_response)?;
    if resource.node_version_id != node_version_id {
        return Err(map_service_error(
            sdkwork_drive_workspace_service::DriveServiceError::NotFound(
                "root-qualified Drive node version was not found".to_string(),
            ),
        )
        .into_response());
    }

    let content_length = u64::try_from(resource.content_length).map_err(|_| {
        internal_problem("resolved Drive resource content length is invalid").into_response()
    })?;
    let etag = format!("\"{}\"", resource.checksum_sha256_hex);
    let last_modified = parse_timestamp(&resource.last_modified).ok_or_else(|| {
        internal_problem("resolved Drive resource timestamp is invalid").into_response()
    })?;

    evaluate_preconditions(&headers, &etag, last_modified).map_err(IntoResponse::into_response)?;
    if is_not_modified(&headers, &etag, last_modified) {
        return Ok(not_modified_response(&etag, last_modified));
    }

    let requested_range = header_text(&headers, RANGE);
    let range_allowed = requested_range.is_some()
        && if_range_allows(header_text(&headers, IF_RANGE), &etag, last_modified);
    let selected_range = if range_allowed {
        match parse_single_range(requested_range.expect("range was checked"), content_length) {
            Ok(range) => Some(range),
            Err(()) => return Err(range_problem_response(content_length)),
        }
    } else {
        None
    };
    let (start, end, status) = match selected_range {
        Some(range) => (
            range.start_inclusive,
            range.end_inclusive,
            StatusCode::PARTIAL_CONTENT,
        ),
        None if content_length > 0 => (0, content_length - 1, StatusCode::OK),
        None => (0, 0, StatusCode::OK),
    };
    let response_length = if content_length == 0 {
        0
    } else {
        end - start + 1
    };

    let object_store = state
        .object_runtime
        .resolve(
            &resource.content_locator.storage_provider_id,
            resource.content_locator.storage_provider_version,
        )
        .await
        .map_err(|error| map_object_store_error(error).into_response())?;
    let locator = DriveObjectLocator {
        bucket: resource.content_locator.bucket,
        object_key: resource.content_locator.object_key,
    };
    let body = if content_length == 0 {
        let head = object_store
            .head_object(HeadObjectRequest {
                locator: locator.clone(),
            })
            .await
            .map_err(|error| map_object_store_error(error).into_response())?;
        if head.content_length != 0 {
            return Err(integrity_problem(
                "empty Drive resource metadata does not match storage",
            ));
        }
        Body::empty()
    } else {
        let (read, chunks) = object_store
            .read_object_range(ReadObjectRangeRequest {
                locator,
                range: DriveByteRange {
                    start_inclusive: start,
                    end_inclusive: end,
                },
            })
            .await
            .map_err(|error| map_object_store_error(error).into_response())?;
        if read.content_length != response_length {
            return Err(integrity_problem(
                "Drive content range length does not match committed metadata",
            ));
        }
        stream_body(chunks, response_length)
    };

    build_content_response(
        body,
        ContentResponseSpec {
            status,
            content_type: &resource.content_type,
            response_length,
            full_length: content_length,
            selected_range,
            etag: &etag,
            last_modified,
        },
    )
    .map_err(IntoResponse::into_response)
}

fn build_content_response(
    body: Body,
    spec: ContentResponseSpec<'_>,
) -> Result<Response<Body>, RouteProblem> {
    let mut builder = Response::builder()
        .status(spec.status)
        .header(ACCEPT_RANGES, "bytes")
        .header(CACHE_CONTROL, "private, max-age=31536000, immutable")
        .header(CONTENT_TYPE, safe_header_value(spec.content_type)?)
        .header(CONTENT_LENGTH, spec.response_length.to_string())
        .header(ETAG, safe_header_value(spec.etag)?)
        .header(
            LAST_MODIFIED,
            httpdate::fmt_http_date(spec.last_modified.into()),
        )
        .header("x-content-type-options", "nosniff");
    if let Some(range) = spec.selected_range {
        builder = builder.header(
            CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                range.start_inclusive, range.end_inclusive, spec.full_length
            ),
        );
    }
    builder
        .body(body)
        .map_err(|error| internal_problem(format!("build Drive content response failed: {error}")))
}

fn not_modified_response(etag: &str, last_modified: DateTime<Utc>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(ETAG, etag)
        .header(LAST_MODIFIED, httpdate::fmt_http_date(last_modified.into()))
        .body(Body::empty())
        .expect("static not-modified response headers must be valid")
}

fn range_problem_response(content_length: u64) -> Response<Body> {
    let mut response = range_not_satisfiable(content_length).into_response();
    if let Ok(value) = HeaderValue::from_str(&format!("bytes */{content_length}")) {
        response.headers_mut().insert(CONTENT_RANGE, value);
    }
    response
}

fn integrity_problem(detail: &str) -> Response<Body> {
    map_object_store_error(DriveObjectStoreError::new(
        DriveObjectStoreErrorKind::IntegrityFailed,
        detail,
    ))
    .into_response()
}

fn internal_problem(detail: impl Into<String>) -> RouteProblem {
    map_service_error(sdkwork_drive_workspace_service::DriveServiceError::Internal(detail.into()))
}

fn safe_header_value(value: &str) -> Result<HeaderValue, RouteProblem> {
    HeaderValue::from_str(value).map_err(|_| internal_problem("Drive response header is invalid"))
}

fn stream_body(
    chunks: Box<dyn sdkwork_drive_storage_contract::DriveObjectChunkStream>,
    expected_length: u64,
) -> Body {
    let stream = stream::try_unfold(
        (chunks, expected_length),
        |(mut chunks, remaining)| async move {
            if remaining == 0 {
                return Ok(None);
            }
            match chunks.next_chunk().await {
                Ok(Some(chunk)) => {
                    let chunk_length = chunk.len() as u64;
                    if chunk_length > remaining {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Drive content stream exceeded committed range length",
                        ));
                    }
                    Ok(Some((chunk, (chunks, remaining - chunk_length))))
                }
                Ok(None) => Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Drive content stream ended before committed range length",
                )),
                Err(error) => Err(io::Error::other(format!(
                    "Drive content stream failed: {}",
                    error.code()
                ))),
            }
        },
    );
    Body::from_stream(stream)
}

fn evaluate_preconditions(
    headers: &HeaderMap,
    etag: &str,
    last_modified: DateTime<Utc>,
) -> Result<(), RouteProblem> {
    if let Some(value) = header_text(headers, IF_MATCH) {
        if !etag_list_matches(value, etag, false) {
            return Err(precondition_failed("If-Match validator did not match"));
        }
    } else if let Some(value) = header_text(headers, IF_UNMODIFIED_SINCE) {
        if let Ok(date) = httpdate::parse_http_date(value) {
            if last_modified.timestamp() > epoch_seconds(date) {
                return Err(precondition_failed(
                    "resource changed after If-Unmodified-Since",
                ));
            }
        }
    }
    Ok(())
}

fn is_not_modified(headers: &HeaderMap, etag: &str, last_modified: DateTime<Utc>) -> bool {
    if let Some(value) = header_text(headers, IF_NONE_MATCH) {
        return etag_list_matches(value, etag, true);
    }
    header_text(headers, IF_MODIFIED_SINCE)
        .and_then(|value| httpdate::parse_http_date(value).ok())
        .is_some_and(|date| last_modified.timestamp() <= epoch_seconds(date))
}

fn if_range_allows(value: Option<&str>, etag: &str, last_modified: DateTime<Utc>) -> bool {
    let Some(value) = value else {
        return true;
    };
    let value = value.trim();
    if value.starts_with('"') || value.starts_with("W/") {
        return !value.starts_with("W/") && value == etag;
    }
    httpdate::parse_http_date(value)
        .ok()
        .is_some_and(|date| last_modified.timestamp() <= epoch_seconds(date))
}

fn etag_list_matches(value: &str, etag: &str, weak_match: bool) -> bool {
    value.split(',').any(|candidate| {
        let candidate = candidate.trim();
        if candidate == "*" {
            return true;
        }
        if weak_match {
            candidate.strip_prefix("W/").unwrap_or(candidate)
                == etag.strip_prefix("W/").unwrap_or(etag)
        } else {
            !candidate.starts_with("W/") && candidate == etag
        }
    })
}

fn parse_single_range(value: &str, content_length: u64) -> Result<DriveByteRange, ()> {
    if content_length == 0 || value.contains(',') {
        return Err(());
    }
    let raw = value.strip_prefix("bytes=").ok_or(())?;
    let (start_raw, end_raw) = raw.split_once('-').ok_or(())?;
    if start_raw.is_empty() {
        let suffix_length = end_raw.parse::<u64>().map_err(|_| ())?;
        if suffix_length == 0 {
            return Err(());
        }
        let selected_length = suffix_length.min(content_length);
        return Ok(DriveByteRange {
            start_inclusive: content_length - selected_length,
            end_inclusive: content_length - 1,
        });
    }
    let start = start_raw.parse::<u64>().map_err(|_| ())?;
    if start >= content_length {
        return Err(());
    }
    let end = if end_raw.is_empty() {
        content_length - 1
    } else {
        end_raw
            .parse::<u64>()
            .map_err(|_| ())?
            .min(content_length - 1)
    };
    if end < start {
        return Err(());
    }
    Ok(DriveByteRange {
        start_inclusive: start,
        end_inclusive: end,
    })
}

fn header_text(headers: &HeaderMap, name: axum::http::header::HeaderName) -> Option<&str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn epoch_seconds(value: std::time::SystemTime) -> i64 {
    value
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(i64::MIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_range_parser_supports_closed_open_and_suffix_ranges() {
        assert_eq!(
            parse_single_range("bytes=2-5", 10).expect("closed range"),
            DriveByteRange {
                start_inclusive: 2,
                end_inclusive: 5
            }
        );
        assert_eq!(
            parse_single_range("bytes=7-", 10).expect("open range"),
            DriveByteRange {
                start_inclusive: 7,
                end_inclusive: 9
            }
        );
        assert_eq!(
            parse_single_range("bytes=-4", 10).expect("suffix range"),
            DriveByteRange {
                start_inclusive: 6,
                end_inclusive: 9
            }
        );
        assert!(parse_single_range("bytes=10-", 10).is_err());
        assert!(parse_single_range("bytes=0-1,4-5", 10).is_err());
        assert!(parse_single_range("bytes=-0", 10).is_err());
    }

    #[test]
    fn etag_matching_observes_strong_and_weak_rules() {
        let etag = "\"sha256:abc\"";
        assert!(etag_list_matches(etag, etag, false));
        assert!(!etag_list_matches("W/\"sha256:abc\"", etag, false));
        assert!(etag_list_matches("W/\"sha256:abc\"", etag, true));
        assert!(etag_list_matches("*", etag, false));
    }
}
