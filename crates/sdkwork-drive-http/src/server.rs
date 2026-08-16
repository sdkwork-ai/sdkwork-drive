use std::net::SocketAddr;

/// Resolve a service bind address from an environment variable.
pub fn bind_addr_from_env(env_key: &str, default_addr: &str) -> SocketAddr {
    std::env::var(env_key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_addr.to_string())
        .parse::<SocketAddr>()
        .unwrap_or_else(|error| {
            panic!("invalid bind address in {env_key}: {error}");
        })
}

/// Wait for Ctrl+C (and SIGTERM on Unix) before shutting down.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler")
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

/// Serve an Axum router with graceful shutdown.
pub async fn serve_router(
    router: axum::Router,
    bind_addr: SocketAddr,
    service_name: &str,
) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!(
        target: "sdkwork.drive",
        event = "drive.http.listen",
        service = service_name,
        bind = %bind_addr,
        "listening"
    );
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
}
