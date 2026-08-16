use sdkwork_drive_http::infra::{drive_service_router_config, mount_drive_infra_routes};
use sqlx::PgPool;
use std::net::SocketAddr;

pub async fn spawn_install_worker_health_server(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind = std::env::var("SDKWORK_DRIVE_INSTALL_WORKER_HEALTH_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18084".to_string());
    let addr: SocketAddr = bind
        .parse()
        .map_err(|error| format!("invalid install worker health bind `{bind}`: {error}"))?;

    let app = mount_drive_infra_routes(
        axum::Router::new(),
        drive_service_router_config(&pool),
        Some("sdkwork-drive-install-worker"),
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(
        target: "sdkwork.drive",
        event = "drive.install_worker.health_listen",
        bind = %addr,
        "install worker health server listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}
