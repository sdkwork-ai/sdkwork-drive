use axum::Json;
use sdkwork_utils_rust::{SdkWorkApiResponse, SdkWorkResourceData};
use serde::Serialize;

pub type ResourceResponse<T> = Json<SdkWorkApiResponse<SdkWorkResourceData<T>>>;

pub fn current_trace_id() -> String {
    sdkwork_drive_http::problem_correlation::current_problem_correlation().trace_id
}

pub fn resource<T: Serialize>(item: T) -> ResourceResponse<T> {
    Json(SdkWorkApiResponse::success(
        SdkWorkResourceData { item },
        current_trace_id(),
    ))
}
