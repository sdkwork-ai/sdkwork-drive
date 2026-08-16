mod config;

use config::{
    load_gateway_config, resolve_config_path, resolve_gateway_config, web_framework_env_projection,
};
use sdkwork_web_bootstrap::{infra_public_path_prefixes, ComposedApiAssembly};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdkwork_database_sqlx::enable_process_shared_database_pool();
    sdkwork_drive_observability::init_tracing("sdkwork-api-drive-standalone-gateway");

    let args: Vec<String> = std::env::args().collect();
    let config_path = resolve_config_path(&args)?;
    let file_config = load_gateway_config(std::path::Path::new(&config_path))
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;
    let gateway_config = resolve_gateway_config(file_config)
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;
    for (key, value) in web_framework_env_projection(&gateway_config) {
        std::env::set_var(key, value);
    }

    let drive = sdkwork_api_drive_assembly::assemble_api_router_from_env()
        .await
        .map_err(|error| format!("failed to assemble Drive APIs: {error}"))?;
    let process_pool = sdkwork_database_sqlx::process_shared_database_pool()
        .ok_or("Drive assembly did not install the process-shared database pool")?;
    let iam = sdkwork_api_iam_assembly::assemble_app_api_contribution_with_pool(process_pool)
        .await
        .map_err(|error| format!("failed to assemble embedded IAM App API: {error}"))?;
    let composed = ComposedApiAssembly::try_compose("SDKWork Drive API", vec![iam, drive])
        .map_err(|error| format!("failed to compose Drive API profile: {error}"))?;
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    let framework = sdkwork_iam_web_adapter::build_web_framework_builder(
        resolver,
        composed.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    let app = composed.into_hosted(framework).router;

    let bind_addr: std::net::SocketAddr = gateway_config
        .bind
        .parse()
        .map_err(|error| format!("invalid bind address `{}`: {error}", gateway_config.bind))?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!(
        target: "sdkwork.drive",
        event = "drive.http.listen",
        service = %gateway_config.service_name,
        environment = %gateway_config.environment,
        bind = %bind_addr,
        "listening"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
