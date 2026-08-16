use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use sdkwork_drive_http::middleware::rate_limit::{
    rate_limit_config_from_env, shared_rate_limit_state, sliding_window_rate_limit,
};

pub async fn share_link_rate_limit(request: Request<Body>, next: Next) -> Response {
    sliding_window_rate_limit(
        share_link_rate_limit_state(),
        request,
        next,
        "share link rate limit exceeded",
    )
    .await
}

fn share_link_rate_limit_state(
) -> &'static sdkwork_drive_http::middleware::rate_limit::SharedRateLimitState {
    shared_rate_limit_state("drive-open-api", || {
        rate_limit_config_from_env(
            "SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_WINDOW_SECONDS",
            "SDKWORK_DRIVE_OPEN_API_RATE_LIMIT_MAX_REQUESTS",
            60,
            120,
        )
    })
}
