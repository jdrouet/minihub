//! Configuration loading — TOML file with environment variable overrides.
//!
//! Looks for `minihub.toml` in the working directory. Every field has a
//! sensible default so the file is optional. Environment variables take
//! precedence over file values.

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// HTTP server settings.
    pub server: ServerConfig,
    /// Database settings.
    pub database: DatabaseConfig,
    /// Logging settings.
    pub logging: LoggingConfig,
    /// Integration toggles.
    pub integrations: IntegrationsConfig,
}

/// HTTP listener configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Address to bind to (e.g. `0.0.0.0`).
    pub host: String,
    /// TCP port.
    pub port: u16,
}

/// `SQLite` database configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// `SQLite` connection URL or file path.
    pub url: String,
}

/// Logging configuration.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Filter directive (`RUST_LOG` syntax).
    pub filter: String,
}

/// Per-integration toggles.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct IntegrationsConfig {
    /// Enable the virtual/demo integration.
    pub virtual_enabled: bool,
}

impl Config {
    /// Load configuration from `minihub.toml` (if present) then apply
    /// environment-variable overrides.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML file exists but is malformed.
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::from_file("minihub.toml")?;
        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    fn from_file(path: &str) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).map_err(ConfigError::Parse),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(ConfigError::Io(err)),
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("MINIHUB_HOST") {
            self.server.host = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_PORT") {
            if let Ok(port) = val.parse() {
                self.server.port = port;
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_BIND") {
            if let Some((host, port)) = val.rsplit_once(':') {
                self.server.host = host.to_string();
                if let Ok(port) = port.parse() {
                    self.server.port = port;
                }
            }
        }
        if let Ok(val) = std::env::var("MINIHUB_DATABASE_URL") {
            self.database.url = val;
        }
        if let Ok(val) = std::env::var("MINIHUB_LOG") {
            self.logging.filter = val;
        }
        if let Ok(val) = std::env::var("RUST_LOG") {
            self.logging.filter = val;
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("port must be non-zero".to_string()));
        }
        Ok(())
    }

    /// Return the `host:port` bind address.
    #[must_use]
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Return the database URL in `sqlx`-compatible format.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database.url
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:minihub.db?mode=rwc".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            filter: "minihubd=info,minihub=info,tower_http=debug".to_string(),
        }
    }
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self {
            virtual_enabled: true,
        }
    }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// TOML parse failure.
    #[error("failed to parse config file")]
    Parse(#[from] toml::de::Error),
    /// File I/O failure.
    #[error("failed to read config file")]
    Io(#[from] std::io::Error),
    /// Semantic validation failure.
    #[error("invalid configuration: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_produce_sensible_defaults() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.url, "sqlite:minihub.db?mode=rwc");
        assert!(config.integrations.virtual_enabled);
    }

    #[test]
    fn should_parse_minimal_toml() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn should_parse_full_toml() {
        let toml = "
            [server]
            host = '127.0.0.1'
            port = 9090

            [database]
            url = 'sqlite:test.db'

            [logging]
            filter = 'debug'

            [integrations]
            virtual_enabled = false
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.logging.filter, "debug");
        assert!(!config.integrations.virtual_enabled);
    }

    #[test]
    fn should_return_default_when_file_not_found() {
        let config = Config::from_file("nonexistent.toml").unwrap();
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn should_reject_zero_port() {
        let mut config = Config::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn should_accept_valid_port() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_format_bind_addr() {
        let config = Config::default();
        assert_eq!(config.bind_addr(), "0.0.0.0:3000");
    }

    #[test]
    fn should_format_custom_bind_addr() {
        let mut config = Config::default();
        config.server.host = "127.0.0.1".to_string();
        config.server.port = 9090;
        assert_eq!(config.bind_addr(), "127.0.0.1:9090");
    }

    #[test]
    fn should_return_database_url() {
        let config = Config::default();
        assert_eq!(config.database_url(), "sqlite:minihub.db?mode=rwc");
    }

    #[test]
    fn should_parse_partial_toml_with_defaults() {
        let toml = "
            [server]
            port = 8080
        ";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.url, "sqlite:minihub.db?mode=rwc");
        assert!(config.integrations.virtual_enabled);
    }

    #[test]
    fn should_report_parse_error_for_invalid_toml() {
        let result: Result<Config, _> = toml::from_str("invalid {{{");
        assert!(result.is_err());
    }
}
