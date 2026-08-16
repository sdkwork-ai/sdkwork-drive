use sdkwork_drive_config::DatabaseConfig;
use sdkwork_drive_http::server::{bind_addr_from_env, serve_router};
use sdkwork_drive_workspace_service::application::download_service::ensure_production_download_token_signing_configured;
use sdkwork_routes_drive_backend_api::build_router_with_database_config;

#[tokio::main]
async fn main() {
    sdkwork_drive_workspace_service::enable_process_shared_database_pool();
    sdkwork_drive_observability::init_tracing("sdkwork-routes-drive-backend-api");
    sdkwork_drive_security::ensure_drive_auth_policy_refresh_task();
    ensure_production_download_token_signing_configured()
        .expect("download token signing config invalid");
    let args: Vec<String> = std::env::args().collect();
    let database_config =
        DatabaseConfig::from_env_and_cli_args(&args).expect("resolve drive database config");
    let router = build_router_with_database_config(&database_config)
        .await
        .expect("initialize backend api router and database");
    let bind_addr = bind_addr_from_env("SDKWORK_DRIVE_BACKEND_API_BIND", "127.0.0.1:18081");
    serve_router(router, bind_addr, "sdkwork-routes-drive-backend-api")
        .await
        .expect("serve backend api");
}
