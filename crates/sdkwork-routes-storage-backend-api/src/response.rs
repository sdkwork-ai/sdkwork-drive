use axum::Json;
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkApiResponse, SdkWorkPageData};
use serde::Serialize;

use crate::dto::OffsetPage;

pub(crate) type StorageListHttpResponse<T> = Json<SdkWorkApiResponse<SdkWorkPageData<T>>>;

pub(crate) fn current_trace_id() -> String {
    sdkwork_drive_http::problem_correlation::current_problem_correlation().trace_id
}

pub(crate) fn no_content() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

pub(crate) fn page_info_from_offset_token(
    page: OffsetPage,
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

pub(crate) fn success_list_page_simple<T: Serialize>(
    items: Vec<T>,
    page: OffsetPage,
    next_page_token: Option<String>,
) -> StorageListHttpResponse<T> {
    Json(SdkWorkApiResponse::success(
        SdkWorkPageData {
            items,
            page_info: page_info_from_offset_token(page, next_page_token),
        },
        current_trace_id(),
    ))
}

pub(crate) fn success_cursor_list_page<T: Serialize>(
    items: Vec<T>,
    page_size: i32,
    next_cursor: Option<String>,
) -> StorageListHttpResponse<T> {
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
