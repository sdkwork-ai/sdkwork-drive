use axum::Json;
use sdkwork_utils_rust::{
    PageInfo, PageMode, SdkWorkApiResponse, SdkWorkPageData, SdkWorkResourceData,
};
use serde::Serialize;

use crate::dto::{is_false_bool, DriveNodeResponse, PageRequest};

/// Standard list payload with optional drive-specific ACL scan metadata.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DriveListPageData<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
    #[serde(skip_serializing_if = "is_false_bool")]
    pub incomplete_page: bool,
}

pub(crate) type DriveNodeListHttpResponse =
    Json<SdkWorkApiResponse<DriveListPageData<DriveNodeResponse>>>;

pub(crate) type DriveListHttpResponse<T> = Json<SdkWorkApiResponse<SdkWorkPageData<T>>>;

pub(crate) fn current_trace_id() -> String {
    sdkwork_drive_http::problem_correlation::current_problem_correlation().trace_id
}

pub(crate) fn no_content() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

/// Build cursor-mode `PageInfo` for numeric cursor offset continuation.
pub(crate) fn page_info_from_offset_token(
    page: PageRequest,
    next_page_token: Option<String>,
) -> PageInfo {
    PageInfo {
        mode: PageMode::Cursor,
        page: None,
        page_size: Some(page.limit as i32),
        total_items: None,
        total_pages: None,
        next_cursor: next_page_token.clone(),
        has_more: Some(next_page_token.is_some()),
    }
}

pub(crate) fn success_list_page<T: Serialize>(
    items: Vec<T>,
    page: PageRequest,
    next_page_token: Option<String>,
    incomplete_page: bool,
) -> Json<SdkWorkApiResponse<DriveListPageData<T>>> {
    Json(SdkWorkApiResponse::success(
        DriveListPageData {
            items,
            page_info: page_info_from_offset_token(page, next_page_token),
            incomplete_page,
        },
        current_trace_id(),
    ))
}

pub(crate) fn success_list_page_simple<T: Serialize>(
    items: Vec<T>,
    page: PageRequest,
    next_page_token: Option<String>,
) -> Json<SdkWorkApiResponse<SdkWorkPageData<T>>> {
    Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: page_info_from_offset_token(page, next_page_token),
        },
        current_trace_id(),
    ))
}

pub(crate) fn success_offset_list_page<T: Serialize>(
    items: Vec<T>,
    page: i32,
    page_size: i32,
    total_items: i64,
) -> Json<SdkWorkApiResponse<SdkWorkPageData<T>>> {
    let total_pages = if total_items == 0 {
        0
    } else {
        (total_items + i64::from(page_size) - 1) / i64::from(page_size)
    };
    Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(page),
                page_size: Some(page_size),
                total_items: Some(total_items.to_string()),
                total_pages: Some(total_pages as i32),
                next_cursor: None,
                has_more: Some(i64::from(page) < total_pages),
            },
        },
        current_trace_id(),
    ))
}

/// Full list payload when every item is returned in a single response (no continuation).
pub(crate) fn success_full_list<T: Serialize>(
    items: Vec<T>,
) -> Json<SdkWorkApiResponse<SdkWorkPageData<T>>> {
    let total_items = items.len() as i64;
    let page_size = usize::max(items.len(), 1) as i32;
    Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(1),
                page_size: Some(page_size),
                total_items: Some(total_items.to_string()),
                total_pages: Some(1),
                next_cursor: None,
                has_more: Some(false),
            },
        },
        current_trace_id(),
    ))
}

pub(crate) fn success_created_command_data<T: Serialize>(
    data: T,
) -> (axum::http::StatusCode, Json<SdkWorkApiResponse<T>>) {
    (
        axum::http::StatusCode::CREATED,
        Json(SdkWorkApiResponse::success(data, current_trace_id())),
    )
}

pub(crate) fn success_resource<T: Serialize>(
    item: T,
) -> Json<SdkWorkApiResponse<SdkWorkResourceData<T>>> {
    Json(SdkWorkApiResponse::success(
        SdkWorkResourceData { item },
        current_trace_id(),
    ))
}

pub(crate) fn success_created_resource<T: Serialize>(
    item: T,
) -> (
    axum::http::StatusCode,
    Json<SdkWorkApiResponse<SdkWorkResourceData<T>>>,
) {
    (
        axum::http::StatusCode::CREATED,
        Json(SdkWorkApiResponse::success(
            SdkWorkResourceData { item },
            current_trace_id(),
        )),
    )
}

pub(crate) fn success_envelope<T: Serialize>(data: T) -> Json<SdkWorkApiResponse<T>> {
    Json(SdkWorkApiResponse::success(data, current_trace_id()))
}

pub(crate) fn success_created_envelope<T: Serialize>(
    data: T,
) -> (axum::http::StatusCode, Json<SdkWorkApiResponse<T>>) {
    success_created_command_data(data)
}

pub(crate) fn success_cursor_list_page<T: Serialize>(
    items: Vec<T>,
    page_size: i32,
    next_cursor: Option<String>,
) -> Json<SdkWorkApiResponse<SdkWorkPageData<T>>> {
    Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Cursor,
                page: None,
                page_size: Some(page_size),
                total_items: None,
                total_pages: None,
                next_cursor: next_cursor.clone(),
                has_more: Some(next_cursor.is_some()),
            },
        },
        current_trace_id(),
    ))
}
