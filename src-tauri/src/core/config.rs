use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
    pub test_url: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            test_url: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AuthConfig {
    pub jwt_secret: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 8848 }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub directory: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            directory: "logs".to_string(),
        }
    }
}

pub fn runtime_config() -> &'static RuntimeConfig {
    static CONFIG: OnceLock<RuntimeConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        load_runtime_config().unwrap_or_else(|err| panic!("load runtime config failed: {err}"))
    })
}

pub fn load_runtime_config() -> Result<RuntimeConfig, String> {
    load_dotenv_files();

    let sources = vec![
        PathBuf::from("config/default.toml"),
        PathBuf::from("src-tauri/config/default.toml"),
        PathBuf::from("config/local.toml"),
        PathBuf::from("src-tauri/config/local.toml"),
    ];
    load_runtime_config_from_sources(&sources, None)
}

fn load_runtime_config_from_sources(
    sources: &[PathBuf],
    env: Option<&HashMap<String, String>>,
) -> Result<RuntimeConfig, String> {
    let mut builder = ::config::Config::builder();
    for source in sources {
        builder = builder.add_source(
            ::config::File::from(source.clone())
                .required(false)
                .format(::config::FileFormat::Toml),
        );
    }

    if env.is_none() {
        builder =
            builder.add_source(::config::Environment::with_prefix("PURE_ADMIN").separator("__"));
    }

    let mut runtime = builder
        .build()
        .map_err(|err| format!("build runtime config failed: {err}"))?
        .try_deserialize::<RuntimeConfig>()
        .map_err(|err| format!("deserialize runtime config failed: {err}"))?;

    let owned_env;
    let env_values = match env {
        Some(values) => values,
        None => {
            owned_env = collect_process_env();
            &owned_env
        }
    };
    apply_env_overrides(&mut runtime, env_values)?;
    validate_runtime_config(&runtime)?;

    Ok(runtime)
}

fn collect_process_env() -> HashMap<String, String> {
    std::env::vars().collect()
}

fn apply_env_overrides(
    runtime: &mut RuntimeConfig,
    env: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some(url) = env_lookup(env, "PURE_ADMIN_DATABASE_URL", "PURE_ADMIN_DATABASE__URL") {
        runtime.database.url = url;
    }
    if let Some(test_url) = env_lookup(
        env,
        "PURE_ADMIN_TEST_DATABASE_URL",
        "PURE_ADMIN_DATABASE__TEST_URL",
    ) {
        runtime.database.test_url = Some(test_url);
    }
    if let Some(jwt_secret) =
        env_lookup(env, "PURE_ADMIN_JWT_SECRET", "PURE_ADMIN_AUTH__JWT_SECRET")
    {
        runtime.auth.jwt_secret = jwt_secret;
    }
    if let Some(port) = env_lookup(env, "PURE_ADMIN_SERVER_PORT", "PURE_ADMIN_SERVER__PORT") {
        runtime.server.port = port
            .parse::<u16>()
            .map_err(|_| "PURE_ADMIN_SERVER_PORT must be a valid u16 integer".to_string())?;
    }
    if let Some(level) = env_lookup(env, "PURE_ADMIN_LOG_LEVEL", "PURE_ADMIN_LOGGING__LEVEL")
        .or_else(|| env_lookup(env, "PURE_ADMIN_LOGGING_LEVEL", "PURE_ADMIN_LOGGING__LEVEL"))
    {
        runtime.logging.level = level;
    }
    if let Some(directory) = env_lookup(env, "PURE_ADMIN_LOG_DIR", "PURE_ADMIN_LOGGING__DIRECTORY")
        .or_else(|| {
            env_lookup(
                env,
                "PURE_ADMIN_LOGGING_DIR",
                "PURE_ADMIN_LOGGING__DIRECTORY",
            )
        })
    {
        runtime.logging.directory = directory;
    }
    Ok(())
}

fn env_lookup(env: &HashMap<String, String>, legacy_key: &str, nested_key: &str) -> Option<String> {
    env.get(legacy_key)
        .or_else(|| env.get(nested_key))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_runtime_config(runtime: &RuntimeConfig) -> Result<(), String> {
    if runtime.database.url.trim().is_empty() {
        return Err("database.url must not be empty".to_string());
    }
    if runtime.auth.jwt_secret.trim().is_empty() {
        return Err("auth.jwt_secret must not be empty".to_string());
    }
    if runtime.server.port == 0 {
        return Err("server.port must be greater than 0".to_string());
    }
    if runtime.logging.level.trim().is_empty() {
        return Err("logging.level must not be empty".to_string());
    }
    if runtime.logging.directory.trim().is_empty() {
        return Err("logging.directory must not be empty".to_string());
    }
    Ok(())
}

fn load_dotenv_files() {
    for path in [
        PathBuf::from(".env"),
        PathBuf::from(".env.local"),
        PathBuf::from("src-tauri/.env"),
        PathBuf::from("src-tauri/.env.local"),
    ] {
        let _ = dotenvy::from_path(&path);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    use super::RuntimeConfig;

    fn load_from_files_and_env(
        base_dir: &std::path::Path,
        env: Option<&HashMap<String, String>>,
    ) -> Result<RuntimeConfig, String> {
        let default_path = base_dir.join("default.toml");
        let local_path = base_dir.join("local.toml");
        super::load_runtime_config_from_sources(&[default_path, local_path], env)
    }

    fn write_config(path: &PathBuf, content: &str) {
        fs::write(path, content).expect("write config file");
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pure_admin_config_test_{name}_{nanos}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn local_file_overrides_default_file() {
        let dir = unique_temp_dir("local_overrides_default");
        write_config(
            &dir.join("default.toml"),
            r#"
[database]
url = "postgres://default"
test_url = "postgres://default_test"

[auth]
jwt_secret = "default-secret"

[server]
port = 17000
"#,
        );
        write_config(
            &dir.join("local.toml"),
            r#"
[database]
url = "postgres://local"
"#,
        );

        let empty_env = HashMap::new();
        let config = load_from_files_and_env(&dir, Some(&empty_env)).expect("load config");
        assert_eq!(config.database.url, "postgres://local");
        assert_eq!(
            config.database.test_url.as_deref(),
            Some("postgres://default_test")
        );
        assert_eq!(config.auth.jwt_secret, "default-secret");
        assert_eq!(config.server.port, 17000);
    }

    #[test]
    fn env_overrides_file_values() {
        let dir = unique_temp_dir("env_overrides_file");
        write_config(
            &dir.join("default.toml"),
            r#"
[database]
url = "postgres://default"
test_url = "postgres://default_test"

[auth]
jwt_secret = "default-secret"

[server]
port = 17000
"#,
        );

        let mut env = HashMap::new();
        env.insert(
            "PURE_ADMIN_DATABASE_URL".to_string(),
            "postgres://env".to_string(),
        );
        env.insert(
            "PURE_ADMIN_TEST_DATABASE_URL".to_string(),
            "postgres://env_test".to_string(),
        );
        env.insert(
            "PURE_ADMIN_JWT_SECRET".to_string(),
            "env-secret".to_string(),
        );
        env.insert("PURE_ADMIN_SERVER_PORT".to_string(), "26000".to_string());

        let config = load_from_files_and_env(&dir, Some(&env)).expect("load config");
        assert_eq!(config.database.url, "postgres://env");
        assert_eq!(
            config.database.test_url.as_deref(),
            Some("postgres://env_test")
        );
        assert_eq!(config.auth.jwt_secret, "env-secret");
        assert_eq!(config.server.port, 26000);
    }

    #[test]
    fn legacy_env_works_without_config_files() {
        let dir = unique_temp_dir("legacy_env_only");
        let env = HashMap::from([
            (
                "PURE_ADMIN_DATABASE_URL".to_string(),
                "postgres://legacy_env".to_string(),
            ),
            (
                "PURE_ADMIN_JWT_SECRET".to_string(),
                "legacy-secret".to_string(),
            ),
            ("PURE_ADMIN_SERVER_PORT".to_string(), "29000".to_string()),
        ]);

        let config = load_from_files_and_env(&dir, Some(&env)).expect("load config from env only");
        assert_eq!(config.database.url, "postgres://legacy_env");
        assert_eq!(config.database.test_url, None);
        assert_eq!(config.auth.jwt_secret, "legacy-secret");
        assert_eq!(config.server.port, 29000);
    }

    #[test]
    fn logging_values_can_be_loaded_from_file() {
        let dir = unique_temp_dir("logging_values_from_file");
        write_config(
            &dir.join("default.toml"),
            r#"
[database]
url = "postgres://default"

[auth]
jwt_secret = "default-secret"

[server]
port = 17000

[logging]
level = "warn"
directory = "runtime-logs"
"#,
        );

        let empty_env = HashMap::new();
        let config = load_from_files_and_env(&dir, Some(&empty_env)).expect("load config");
        assert_eq!(config.logging.level, "warn");
        assert_eq!(config.logging.directory, "runtime-logs");
    }

    #[test]
    fn env_overrides_logging_values() {
        let dir = unique_temp_dir("env_overrides_logging_values");
        write_config(
            &dir.join("default.toml"),
            r#"
[database]
url = "postgres://default"

[auth]
jwt_secret = "default-secret"

[server]
port = 17000

[logging]
level = "info"
directory = "logs"
"#,
        );

        let env = HashMap::from([
            ("PURE_ADMIN_LOGGING_LEVEL".to_string(), "error".to_string()),
            (
                "PURE_ADMIN_LOGGING_DIR".to_string(),
                "logs-prod".to_string(),
            ),
        ]);

        let config = load_from_files_and_env(&dir, Some(&env)).expect("load config");
        assert_eq!(config.logging.level, "error");
        assert_eq!(config.logging.directory, "logs-prod");
    }
}
