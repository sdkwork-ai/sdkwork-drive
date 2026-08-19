use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use gitserver_http::{router as gitserver_router, AuthConfig, ServicePolicy, SharedState};
use sdkwork_drive_git_server::{validate_git_request_auth, DriveGitRepoRegistry, GitAuthContext};
use sdkwork_drive_security::DriveAuthPolicyHandle;

use crate::state::GitState;

pub async fn git_auth_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let policy_handle = DriveAuthPolicyHandle::shared_from_env();
    let auth_context = policy_handle
        .read(|policy| validate_git_request_auth(request.headers(), policy))
        .map_err(auth_error_response)?;
    request.extensions_mut().insert(auth_context.clone());
    Ok(GitAuthContext::scope(auth_context, async move { next.run(request).await }).await)
}

fn auth_error_response(error: sdkwork_drive_security::DriveAuthError) -> Response {
    (
        StatusCode::from_u16(error.status).unwrap_or(StatusCode::UNAUTHORIZED),
        error.detail,
    )
        .into_response()
}

fn build_gitserver_router_with_pool(pool: sqlx::PgPool) -> axum::Router {
    let registry = Arc::new(DriveGitRepoRegistry::new(pool));
    let policy = ServicePolicy {
        upload_pack: true,
        upload_pack_v2: true,
        receive_pack: true,
    };
    let shared_state = SharedState::with_registry(registry, AuthConfig::default(), policy);
    gitserver_router(shared_state)
}

pub fn build_git_business_router(state: GitState) -> axum::Router {
    let git_router = build_gitserver_router_with_pool(state.pool);
    axum::Router::new()
        .nest("/git", git_router)
        .route_layer(axum::middleware::from_fn(git_auth_middleware))
        .layer(axum::middleware::from_fn(
            sdkwork_drive_http::metrics::record_request_metrics,
        ))
}

pub async fn gateway_mount_business(pool: sqlx::PgPool) -> axum::Router {
    build_git_business_router(GitState::new(pool))
}
