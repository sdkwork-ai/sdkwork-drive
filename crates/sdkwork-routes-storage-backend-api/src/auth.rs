use crate::error::{map_auth_error, problem, ProblemDetail, SdkWorkResultCode};
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Json;
use sdkwork_drive_security::{
    validate_json_context_projection, validate_uri_context, DriveAppContext,
};

const AUTH_CONTEXT_BODY_LIMIT_BYTES: usize = 1_048_576;

pub(crate) async fn drive_context_projection_guard(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ProblemDetail>)> {
    let Some(context) = request.extensions().get::<DriveAppContext>().cloned() else {
        return Ok(next.run(request).await);
    };

    validate_uri_context(request.uri(), &context).map_err(map_auth_error)?;

    if request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"))
    {
        let (parts, body) = request.into_parts();
        let bytes = to_bytes(body, AUTH_CONTEXT_BODY_LIMIT_BYTES)
            .await
            .map_err(|error| {
                problem(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "request body too large",
                    format!("request body could not be read: {error}"),
                    SdkWorkResultCode::PayloadTooLarge,
                )
            })?;
        if !bytes.is_empty() {
            let body_json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
                problem(
                    StatusCode::BAD_REQUEST,
                    "invalid request body",
                    format!("request body must be valid JSON: {error}"),
                    SdkWorkResultCode::MalformedRequest,
                )
            })?;
            validate_json_context_projection(&body_json, &context).map_err(map_auth_error)?;
        }
        request = Request::from_parts(parts, Body::from(bytes));
    }

    request.extensions_mut().insert::<DriveAppContext>(context);
    Ok(next.run(request).await)
}
