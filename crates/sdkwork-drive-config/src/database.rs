use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveRuntimeConfig {
    pub app_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseEngine {
    Postgresql,
}

impl DatabaseEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Postgresql => "postgresql",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub configured: bool,
    pub engine: String,
    #[serde(rename = "maxConnections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    engine: DatabaseEngine,
    url: String,
    max_connections: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfigError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeConfigFile {
    database: RuntimeDatabaseConfig,
}

#[derive(Debug, Deserialize)]
struct RuntimeDatabaseConfig {
    engine: Option<String>,
    url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    password_file: Option<String>,
    ssl_mode: Option<String>,
    max_connections: Option<u32>,
}

impl DatabaseConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DatabaseConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for DatabaseConfigError {}

impl DatabaseConfig {
    pub const DEFAULT_POSTGRES_MAX_CONNECTIONS: u32 = 32;
    pub const DEFAULT_MAX_CONNECTIONS: u32 = Self::DEFAULT_POSTGRES_MAX_CONNECTIONS;

    pub fn from_url(database_url: &str) -> Result<Self, DatabaseConfigError> {
        let url = database_url.trim();
        if url.is_empty() {
            return Err(DatabaseConfigError::new("database url must not be blank"));
        }
        parse_database_engine_from_url(url)?;
        Self::from_url_with_max_connections(url, Self::DEFAULT_POSTGRES_MAX_CONNECTIONS)
    }

    pub fn from_url_with_max_connections(
        database_url: &str,
        max_connections: u32,
    ) -> Result<Self, DatabaseConfigError> {
        if max_connections == 0 {
            return Err(DatabaseConfigError::new(
                "database max connections must be positive",
            ));
        }
        let url = database_url.trim();
        if url.is_empty() {
            return Err(DatabaseConfigError::new("database url must not be blank"));
        }
        let engine = parse_database_engine_from_url(url)?;
        Ok(Self {
            engine,
            url: url.to_string(),
            max_connections,
        })
    }

    pub fn from_env() -> Result<Self, DatabaseConfigError> {
        Self::from_env_pairs(std::env::vars())
    }

    pub fn from_env_and_cli_args(args: &[String]) -> Result<Self, DatabaseConfigError> {
        if let Some(database_url) = parse_database_url_from_cli_args(args) {
            parse_database_engine_from_url(&database_url)?;
            let max_connections = parse_max_connections(
                std::env::var("SDKWORK_DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .as_ref(),
                Self::DEFAULT_POSTGRES_MAX_CONNECTIONS,
            )?;
            return Self::from_url_with_max_connections(&database_url, max_connections);
        }
        Self::from_env()
    }

    pub fn from_env_pairs<I, K, V>(pairs: I) -> Result<Self, DatabaseConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let env: HashMap<String, String> = pairs
            .into_iter()
            .map(|(key, value)| (key.as_ref().to_string(), value.as_ref().to_string()))
            .collect();
        if let Some(url) = env
            .get("SDKWORK_DATABASE_URL")
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
        {
            parse_database_engine_from_url(url.trim())?;
            let max_connections = parse_max_connections(
                env.get("SDKWORK_DATABASE_MAX_CONNECTIONS"),
                Self::DEFAULT_POSTGRES_MAX_CONNECTIONS,
            )?;
            return Self::from_url_with_max_connections(url, max_connections);
        }

        if let Some(config_file) = env
            .get("SDKWORK_DRIVE_CONFIG_FILE")
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
        {
            return Self::from_runtime_toml_file(config_file.trim());
        }

        reject_removed_database_env_aliases(&env)?;

        let engine = env
            .get("SDKWORK_DATABASE_ENGINE")
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "postgresql".to_string());

        match engine.as_str() {
            "postgresql" | "postgres" => {
                let max_connections = parse_max_connections(
                    env.get("SDKWORK_DATABASE_MAX_CONNECTIONS"),
                    Self::DEFAULT_POSTGRES_MAX_CONNECTIONS,
                )?;
                let host = required_env_value(&env, "SDKWORK_DATABASE_HOST")?;
                let port = env
                    .get("SDKWORK_DATABASE_PORT")
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .unwrap_or("5432");
                let database = required_env_value(&env, "SDKWORK_DATABASE_NAME")?;
                let username = required_env_value(&env, "SDKWORK_DATABASE_USERNAME")?;
                let password = required_env_value(&env, "SDKWORK_DATABASE_PASSWORD")?;
                let ssl_mode = env
                    .get("SDKWORK_DATABASE_SSL_MODE")
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty());
                let mut url = format!(
                    "postgresql://{}:{}@{host}:{port}/{}",
                    percent_encode_userinfo(username),
                    percent_encode_userinfo(password),
                    percent_encode_postgres_database_path(database)
                );
                if let Some(ssl_mode) = ssl_mode {
                    url.push_str("?sslmode=");
                    url.push_str(&percent_encode_query_value(ssl_mode));
                }
                Self::from_url_with_max_connections(&url, max_connections)
            }
            other => Err(DatabaseConfigError::new(format!(
                "unsupported database engine {other}; expected postgresql"
            ))),
        }
    }

    pub fn engine(&self) -> DatabaseEngine {
        self.engine
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    pub fn safe_health(&self) -> DatabaseHealth {
        DatabaseHealth {
            configured: true,
            engine: self.engine.as_str().to_string(),
            max_connections: self.max_connections,
        }
    }

    pub fn from_runtime_toml(content: &str) -> Result<Self, DatabaseConfigError> {
        Self::from_runtime_toml_inner(content, None)
    }

    pub fn from_runtime_toml_file(path: impl AsRef<Path>) -> Result<Self, DatabaseConfigError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|error| {
            DatabaseConfigError::new(format!(
                "failed to read drive runtime config {}: {error}",
                path.display()
            ))
        })?;
        Self::from_runtime_toml_inner(&content, Some(path))
    }

    fn from_runtime_toml_inner(
        content: &str,
        config_path: Option<&Path>,
    ) -> Result<Self, DatabaseConfigError> {
        let runtime_config: RuntimeConfigFile = toml::from_str(content)
            .map_err(|error| DatabaseConfigError::new(format!("invalid runtime TOML: {error}")))?;
        let database = runtime_config.database;
        let declared_engine = database
            .engine
            .as_deref()
            .map(parse_database_engine_name)
            .transpose()?;
        let database_url = runtime_database_url(database, declared_engine, config_path)?;
        let engine = parse_database_engine_from_url(&database_url)?;
        if let Some(declared_engine) = declared_engine {
            if declared_engine != engine {
                return Err(DatabaseConfigError::new(format!(
                    "runtime config [database].engine {} does not match database url engine {}",
                    declared_engine.as_str(),
                    engine.as_str()
                )));
            }
        }
        let max_connections =
            runtime_config_max_connections(content, Self::DEFAULT_POSTGRES_MAX_CONNECTIONS)?;
        Self::from_url_with_max_connections(&database_url, max_connections)
    }
}

fn parse_database_url_from_cli_args(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--database-url" {
            return iter
                .next()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
        }
        if let Some(value) = arg.strip_prefix("--database-url=") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn parse_database_engine_from_url(url: &str) -> Result<DatabaseEngine, DatabaseConfigError> {
    if url.starts_with("postgresql://") || url.starts_with("postgres://") {
        return Ok(DatabaseEngine::Postgresql);
    }
    Err(DatabaseConfigError::new(
        "database url must be a PostgreSQL connection string",
    ))
}

fn reject_removed_database_env_aliases(
    env: &HashMap<String, String>,
) -> Result<(), DatabaseConfigError> {
    let removed = [
        ("SDKWORK_DATABASE_PROVIDER", "SDKWORK_DATABASE_ENGINE"),
        ("SDKWORK_DATABASE_SSLMODE", "SDKWORK_DATABASE_SSL_MODE"), // sdkwork-retired-database-key-rejection
    ]
    .into_iter()
    .filter_map(|(removed, replacement)| {
        env.get(removed)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(|_| format!("{removed} -> {replacement}"))
    })
    .collect::<Vec<_>>();
    if removed.is_empty() {
        return Ok(());
    }
    Err(DatabaseConfigError::new(format!(
        "removed database environment aliases are not supported: {}",
        removed.join(", ")
    )))
}

fn runtime_config_max_connections(
    content: &str,
    default_value: u32,
) -> Result<u32, DatabaseConfigError> {
    let runtime_config: RuntimeConfigFile = toml::from_str(content)
        .map_err(|error| DatabaseConfigError::new(format!("invalid runtime TOML: {error}")))?;
    match runtime_config.database.max_connections {
        Some(value) if value > 0 => Ok(value),
        Some(_) => Err(DatabaseConfigError::new(
            "runtime config [database].max_connections must be positive",
        )),
        None => Ok(default_value),
    }
}

fn parse_database_engine_name(value: &str) -> Result<DatabaseEngine, DatabaseConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "postgresql" | "postgres" => Ok(DatabaseEngine::Postgresql),
        other => Err(DatabaseConfigError::new(format!(
            "unsupported runtime config [database].engine {other}; expected postgresql"
        ))),
    }
}

fn runtime_database_url(
    database: RuntimeDatabaseConfig,
    declared_engine: Option<DatabaseEngine>,
    config_path: Option<&Path>,
) -> Result<String, DatabaseConfigError> {
    if let Some(url) = database
        .url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if has_structured_postgres_fields(&database) {
            return Err(DatabaseConfigError::new(
                "runtime config [database] must use either url or structured PostgreSQL fields, not both",
            ));
        }
        return Ok(url.to_string());
    }

    match declared_engine {
        Some(DatabaseEngine::Postgresql) => structured_postgres_url(database, config_path),
        None => Err(DatabaseConfigError::new(
            "runtime config [database] must declare engine and url or structured PostgreSQL fields",
        )),
    }
}

fn has_structured_postgres_fields(database: &RuntimeDatabaseConfig) -> bool {
    database.host.is_some()
        || database.port.is_some()
        || database.database.is_some()
        || database.username.is_some()
        || database.password.is_some()
        || database.password_file.is_some()
        || database.ssl_mode.is_some()
}

fn structured_postgres_url(
    database: RuntimeDatabaseConfig,
    config_path: Option<&Path>,
) -> Result<String, DatabaseConfigError> {
    let host = required_config_value("runtime config [database].host", database.host.as_deref())?;
    let port = database.port.unwrap_or(5432);
    let database_name = required_config_value(
        "runtime config [database].database",
        database.database.as_deref(),
    )?;
    let username = required_config_value(
        "runtime config [database].username",
        database.username.as_deref(),
    )?;
    let password = structured_postgres_password(
        database.password.as_deref(),
        database.password_file.as_deref(),
        config_path,
    )?;
    let mut url = format!(
        "postgresql://{}:{}@{host}:{port}/{}",
        percent_encode_userinfo(username),
        percent_encode_userinfo(&password),
        percent_encode_postgres_database_path(database_name)
    );
    if let Some(ssl_mode) = database
        .ssl_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        url.push_str("?sslmode=");
        url.push_str(&percent_encode_query_value(ssl_mode));
    }
    Ok(url)
}

fn structured_postgres_password(
    password: Option<&str>,
    password_file: Option<&str>,
    config_path: Option<&Path>,
) -> Result<String, DatabaseConfigError> {
    match (
        password.map(str::trim).filter(|value| !value.is_empty()),
        password_file
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(_), Some(_)) => Err(DatabaseConfigError::new(
            "runtime config [database] must use only one of password or password_file",
        )),
        (Some(password), None) => Ok(password.to_string()),
        (None, Some(password_file)) => {
            let path = resolve_runtime_path(password_file, config_path);
            let password = std::fs::read_to_string(&path).map_err(|error| {
                DatabaseConfigError::new(format!(
                    "failed to read runtime config [database].password_file {}: {error}",
                    path.display()
                ))
            })?;
            required_config_value(
                &format!("runtime config [database].password_file {}", path.display()),
                Some(password.as_str()),
            )
            .map(str::to_string)
        }
        (None, None) => Err(DatabaseConfigError::new(
            "runtime config [database] must provide password or password_file for PostgreSQL",
        )),
    }
}

fn resolve_runtime_path(value: &str, config_path: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        return path;
    }
    config_path
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join(path.as_path()))
        .unwrap_or(path)
}

fn required_config_value<'a>(
    label: &str,
    value: Option<&'a str>,
) -> Result<&'a str, DatabaseConfigError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| DatabaseConfigError::new(format!("{label} must be set")))
}

fn parse_max_connections(
    value: Option<&String>,
    default_value: u32,
) -> Result<u32, DatabaseConfigError> {
    match value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(raw) => {
            let parsed = raw.parse::<u32>().map_err(|_| {
                DatabaseConfigError::new(
                    "SDKWORK_DATABASE_MAX_CONNECTIONS must be a positive integer",
                )
            })?;
            if parsed == 0 {
                return Err(DatabaseConfigError::new(
                    "SDKWORK_DATABASE_MAX_CONNECTIONS must be a positive integer",
                ));
            }
            Ok(parsed)
        }
        None => Ok(default_value),
    }
}

fn required_env_value<'a>(
    env: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a str, DatabaseConfigError> {
    env.get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| DatabaseConfigError::new(format!("{key} must be set")))
}

fn percent_encode_userinfo(value: &str) -> String {
    percent_encode(value, false)
}

fn percent_encode_postgres_database_path(value: &str) -> String {
    percent_encode(value, true)
}

fn percent_encode_query_value(value: &str) -> String {
    percent_encode(value, false)
}

fn percent_encode(value: &str, keep_slash: bool) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        let allowed = matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
        ) || (keep_slash && *byte == b'/');
        if allowed {
            output.push(*byte as char);
        } else {
            output.push('%');
            output.push(hex_digit(byte >> 4));
            output.push(hex_digit(byte & 0x0f));
        }
    }
    output
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + value - 10) as char,
        _ => unreachable!("hex digit value must be lower than 16"),
    }
}
