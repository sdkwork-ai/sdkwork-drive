use sdkwork_drive_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_routes_drive_app_api::build_router_with_database_config;

#[tokio::test]
async fn app_runtime_accepts_sqlite_database_config() {
    let config = DatabaseConfig::from_url_with_max_connections("sqlite::memory:", 1)
        .expect("sqlite config should parse");

    let _router = build_router_with_database_config(&config)
        .await
        .expect("sqlite database config should build app router");
}

#[test]
fn app_runtime_recognizes_postgres_database_engine() {
    let config = DatabaseConfig::from_url_with_max_connections(
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1:65535/sdkwork_ai_dev",
        5,
    )
    .expect("postgres config should parse");

    assert_eq!(config.engine(), DatabaseEngine::Postgresql);
}
