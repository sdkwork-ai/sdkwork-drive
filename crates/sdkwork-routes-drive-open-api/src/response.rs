use axum::http::StatusCode;
use axum::Json;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use serde::Serialize;

pub(crate) fn success_resource<T: Serialize>(
    item: T,
) -> Json<SdkWorkApiResponse<SdkWorkResourceData<T>>> {
    let trace_id = sdkwork_drive_http::problem_correlation::current_problem_correlation().trace_id;
    Json(SdkWorkApiResponse::success(
        SdkWorkResourceData { item },
        trace_id,
    ))
}

pub(crate) fn created_resource<T: Serialize>(
    item: T,
) -> (StatusCode, Json<SdkWorkApiResponse<SdkWorkResourceData<T>>>) {
    (StatusCode::CREATED, success_resource(item))
}
