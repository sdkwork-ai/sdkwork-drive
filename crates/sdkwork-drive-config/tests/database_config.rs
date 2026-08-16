use sdkwork_drive_config::{DatabaseConfig, DatabaseEngine};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rejects_sqlite_database_urls() {
    let direct = DatabaseConfig::from_url("sqlite://target/dev/sdkwork-drive.sqlite")
        .expect_err("server runtime must reject SQLite URLs");
    assert!(direct
        .to_string()
        .contains("database url must be a PostgreSQL connection string"));

    let env = [("SDKWORK_DATABASE_URL", "sqlite::memory:")];
    let from_env = DatabaseConfig::from_env_pairs(env)
        .expect_err("server runtime environment must reject SQLite URLs");
    assert!(from_env
        .to_string()
        .contains("database url must be a PostgreSQL connection string"));
}

#[test]
fn parses_postgres_database_urls_for_server_mode() {
    let config = DatabaseConfig::from_url_with_max_connections(
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1:5432/sdkwork_ai_dev",
        12,
    )
    .expect("postgres database url should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1:5432/sdkwork_ai_dev",
        config.url()
    );
    assert_eq!(12, config.max_connections());
    assert_eq!("postgresql", config.safe_health().engine);
}

#[test]
fn rejects_empty_unsupported_urls_and_invalid_pool_sizes() {
    let blank = DatabaseConfig::from_url_with_max_connections("   ", 5).unwrap_err();
    assert!(
        blank.to_string().contains("database url must not be blank"),
        "unexpected blank error: {blank}"
    );

    let mysql = DatabaseConfig::from_url_with_max_connections(
        "mysql://sdkwork_ai_dev:secret@127.0.0.1/sdkwork_ai_dev",
        5,
    )
    .unwrap_err();
    assert!(
        mysql.to_string().contains("PostgreSQL connection string"),
        "unexpected unsupported url error: {mysql}"
    );

    let pool_size = DatabaseConfig::from_url_with_max_connections(
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1/sdkwork_ai_dev",
        0,
    )
    .unwrap_err();
    assert!(
        pool_size
            .to_string()
            .contains("max connections must be positive"),
        "unexpected max connection error: {pool_size}"
    );
}

#[test]
fn resolves_postgres_from_structured_environment() {
    let env = [
        ("SDKWORK_DATABASE_ENGINE", "postgresql"),
        ("SDKWORK_DATABASE_HOST", "127.0.0.1"),
        ("SDKWORK_DATABASE_PORT", "15432"),
        ("SDKWORK_DATABASE_NAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_USERNAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_PASSWORD", "drive_pass"),
        ("SDKWORK_DATABASE_SSL_MODE", "disable"),
        ("SDKWORK_DATABASE_MAX_CONNECTIONS", "10"),
    ];

    let config = DatabaseConfig::from_env_pairs(env).expect("postgres env should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(10, config.max_connections());
    assert_eq!(
        "postgresql://sdkwork_ai_dev:drive_pass@127.0.0.1:15432/sdkwork_ai_dev?sslmode=disable",
        config.url()
    );
}

#[test]
fn rejects_sqlite_from_structured_environment() {
    let env = [
        ("SDKWORK_DATABASE_ENGINE", "sqlite"),
        ("SDKWORK_DATABASE_FILE", "target/dev/sdkwork-drive.sqlite"),
    ];

    let error = DatabaseConfig::from_env_pairs(env)
        .expect_err("server runtime must reject the SQLite engine");
    assert!(error
        .to_string()
        .contains("unsupported database engine sqlite; expected postgresql"));
}

#[test]
fn structured_postgres_environment_url_encodes_connection_parts() {
    let env = [
        ("SDKWORK_DATABASE_ENGINE", "postgresql"),
        ("SDKWORK_DATABASE_HOST", "db.internal"),
        ("SDKWORK_DATABASE_PORT", "5432"),
        ("SDKWORK_DATABASE_NAME", "sdkwork drive/dev"),
        ("SDKWORK_DATABASE_USERNAME", "sdkworkprod@2026++"),
        ("SDKWORK_DATABASE_PASSWORD", "pa@ss+word/with space"),
        ("SDKWORK_DATABASE_SSL_MODE", "require"),
    ];

    let config = DatabaseConfig::from_env_pairs(env).expect("postgres env should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(
        "postgresql://sdkworkprod%402026%2B%2B:pa%40ss%2Bword%2Fwith%20space@db.internal:5432/sdkwork%20drive/dev?sslmode=require",
        config.url()
    );
}

#[test]
fn cli_database_url_overrides_structured_environment() {
    let args = vec![
        "sdkwork-routes-drive-app-api".to_string(),
        "--database-url".to_string(),
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1:5432/sdkwork_ai_dev".to_string(),
    ];
    let config =
        DatabaseConfig::from_env_and_cli_args(&args).expect("cli database url should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(
        "postgresql://sdkwork_ai_dev:secret@127.0.0.1:5432/sdkwork_ai_dev",
        config.url()
    );
    assert_eq!(
        DatabaseConfig::DEFAULT_POSTGRES_MAX_CONNECTIONS,
        config.max_connections()
    );
}

#[test]
fn explicit_database_url_overrides_structured_environment() {
    let env = [
        (
            "SDKWORK_DATABASE_URL",
            "postgresql://override:secret@db.internal:5432/sdkwork_ai_dev",
        ),
        ("SDKWORK_DATABASE_ENGINE", "postgresql"),
        ("SDKWORK_DATABASE_HOST", "127.0.0.1"),
        ("SDKWORK_DATABASE_PORT", "15432"),
        ("SDKWORK_DATABASE_NAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_USERNAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_PASSWORD", "drive_pass"),
        ("SDKWORK_DATABASE_MAX_CONNECTIONS", "3"),
    ];

    let config = DatabaseConfig::from_env_pairs(env).expect("explicit url should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(
        "postgresql://override:secret@db.internal:5432/sdkwork_ai_dev",
        config.url()
    );
    assert_eq!(3, config.max_connections());
}

#[test]
fn rejects_removed_database_environment_aliases() {
    let env = [
        ("SDKWORK_DATABASE_PROVIDER", "postgresql"),
        ("SDKWORK_DATABASE_SSLMODE", "disable"),
        ("SDKWORK_DATABASE_HOST", "127.0.0.1"),
        ("SDKWORK_DATABASE_NAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_USERNAME", "sdkwork_ai_dev"),
        ("SDKWORK_DATABASE_PASSWORD", "drive_pass"),
    ];

    let error = DatabaseConfig::from_env_pairs(env).unwrap_err();

    assert!(
        error.to_string().contains("SDKWORK_DATABASE_PROVIDER")
            && error.to_string().contains("SDKWORK_DATABASE_SSLMODE")
            && error.to_string().contains("SDKWORK_DATABASE_ENGINE")
            && error.to_string().contains("SDKWORK_DATABASE_SSL_MODE"),
        "removed aliases should be rejected with standard replacements, got: {error}"
    );
}

#[test]
fn rejects_sqlite_runtime_toml() {
    let error = DatabaseConfig::from_runtime_toml(
        r#"
        [database]
        engine = "sqlite"
        url = "sqlite://target/dev/sdkwork-drive.sqlite"
        "#,
    )
    .expect_err("server runtime TOML must reject SQLite");

    assert!(error
        .to_string()
        .contains("unsupported runtime config [database].engine sqlite; expected postgresql"));
}

#[test]
fn parses_structured_postgres_runtime_toml_with_password_file() {
    let temp_dir = unique_temp_dir("postgres-password-file");
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    let secret_path = temp_dir.join("database.secret");
    std::fs::write(&secret_path, "pa@ss+word/with space\n").expect("secret should be written");
    let config_path = temp_dir.join("drive.database.toml");
    std::fs::write(
        &config_path,
        r#"
        [database]
        engine = "postgresql"
        host = "db.internal"
        port = 5432
        database = "sdkwork drive/dev"
        username = "sdkworkprod@2026++"
        password_file = "./database.secret"
        ssl_mode = "require"
        max_connections = 16
        "#,
    )
    .expect("toml config should be written");

    let config =
        DatabaseConfig::from_runtime_toml_file(&config_path).expect("postgres toml should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(16, config.max_connections());
    assert_eq!(
        "postgresql://sdkworkprod%402026%2B%2B:pa%40ss%2Bword%2Fwith%20space@db.internal:5432/sdkwork%20drive/dev?sslmode=require",
        config.url()
    );
}

#[test]
fn env_url_override_wins_over_config_file() {
    let temp_dir = unique_temp_dir("env-override");
    std::fs::create_dir_all(&temp_dir).expect("temp dir should be created");
    let config_path = temp_dir.join("drive.database.toml");
    std::fs::write(
        &config_path,
        r#"
        [database]
        engine = "postgresql"
        host = "db.internal"
        database = "sdkwork_ai_dev"
        username = "sdkwork_ai_dev"
        password = "drive_pass"
        "#,
    )
    .expect("toml config should be written");

    let config_path_string = config_path.to_string_lossy().to_string();
    let env = [
        ("SDKWORK_DRIVE_CONFIG_FILE", config_path_string.as_str()),
        (
            "SDKWORK_DATABASE_URL",
            "postgresql://override:secret@db.internal:5432/sdkwork_ai_dev",
        ),
    ];
    let config = DatabaseConfig::from_env_pairs(env).expect("env override should parse");

    assert_eq!(DatabaseEngine::Postgresql, config.engine());
    assert_eq!(
        "postgresql://override:secret@db.internal:5432/sdkwork_ai_dev",
        config.url()
    );
    assert_eq!(
        DatabaseConfig::DEFAULT_POSTGRES_MAX_CONNECTIONS,
        config.max_connections()
    );
}

#[test]
fn safe_health_never_exposes_connection_string_material() {
    let config = DatabaseConfig::from_url_with_max_connections(
        "postgresql://sdkwork_ai_dev:secret@db.internal:5432/sdkwork_ai_dev?sslmode=require",
        7,
    )
    .expect("postgres database url should parse");

    let health = serde_json::to_string(&config.safe_health()).expect("health should serialize");
    assert!(health.contains("\"configured\":true"));
    assert!(health.contains("\"engine\":\"postgresql\""));
    assert!(health.contains("\"maxConnections\":7"));
    assert!(!health.contains("secret"));
    assert!(!health.contains("db.internal"));
    assert!(!health.contains("sdkwork_ai_dev"));
    assert!(!health.contains("sslmode"));
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("sdkwork-drive-config-{label}-{nanos}"))
}
