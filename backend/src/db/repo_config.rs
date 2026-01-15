//! Repository configuration file support.
//!
//! This module provides utilities for reading repository configuration from
//! TOML configuration files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::factory::RepositoryType;
use super::repository::RepositoryError;
use crate::db::PostgresConfig;

/// Repository configuration from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub repository: RepositorySettings,
    #[serde(default)]
    pub postgres: PostgresSettings,
}

/// Repository type settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySettings {
    #[serde(rename = "type")]
    pub repo_type: String,
}

/// Postgres connection settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostgresSettings {
    #[serde(default)]
    pub database_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    100
}

impl RepositoryConfig {
    /// Load repository configuration from a TOML file.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Ok(RepositoryConfig)` if successful
    /// * `Err(RepositoryError)` if file cannot be read or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RepositoryError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            RepositoryError::ConfigurationError(format!("Failed to read config file: {}", e))
        })?;

        let config: RepositoryConfig = toml::from_str(&content).map_err(|e| {
            RepositoryError::ConfigurationError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// Load repository configuration from the default location.
    ///
    /// Searches for `repository.toml` in:
    /// 1. Current directory
    /// 2. `backend/` directory
    /// 3. Parent directory
    ///
    /// # Returns
    /// * `Ok(RepositoryConfig)` if found and parsed successfully
    /// * `Err(RepositoryError)` if no config file found or parse error
    pub fn from_default_location() -> Result<Self, RepositoryError> {
        let search_paths = vec![
            PathBuf::from("repository.toml"),
            PathBuf::from("backend/repository.toml"),
            PathBuf::from("../repository.toml"),
            PathBuf::from("./repository.toml"),
        ];

        for path in search_paths {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        Err(RepositoryError::ConfigurationError(
            "No repository.toml found in standard locations".to_string(),
        ))
    }

    /// Get the repository type from configuration.
    pub fn repository_type(&self) -> Result<RepositoryType, String> {
        RepositoryType::from_str(&self.repository.repo_type)
    }

    /// Convert to PostgresConfig if this is a Postgres configuration.
    #[cfg(feature = "postgres-repo")]
    pub fn to_postgres_config(&self) -> Result<Option<PostgresConfig>, RepositoryError> {
        let repo_type = self.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        if repo_type != RepositoryType::Postgres {
            return Ok(None);
        }

        if self.postgres.database_url.is_empty() {
            return Err(RepositoryError::ConfigurationError(
                "Postgres repository requires 'postgres.database_url' setting".to_string(),
            ));
        }

        Ok(Some(PostgresConfig {
            database_url: self.postgres.database_url.clone(),
            max_pool_size: self.postgres.max_connections,
            min_pool_size: self.postgres.min_connections,
            connection_timeout_sec: self.postgres.connect_timeout,
            idle_timeout_sec: self.postgres.idle_timeout,
            max_retries: self.postgres.max_retries,
            retry_delay_ms: self.postgres.retry_delay_ms,
        }))
    }

    /// Convert to PostgresConfig when the feature is disabled.
    #[cfg(not(feature = "postgres-repo"))]
    pub fn to_postgres_config(&self) -> Result<Option<PostgresConfig>, RepositoryError> {
        let repo_type = self.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        if repo_type == RepositoryType::Postgres {
            return Err(RepositoryError::ConfigurationError(
                "Postgres repository feature not enabled".to_string(),
            ));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_config() {
        let toml = r#"
[repository]
type = "local"
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.repository.repo_type, "local");
        assert_eq!(config.repository_type().unwrap(), RepositoryType::Local);
    }

    #[cfg(feature = "postgres-repo")]
    #[test]
    fn test_parse_postgres_config() {
        let toml = r#"
[repository]
type = "postgres"

[postgres]
database_url = "postgres://user:pass@host:5432/dbname"
max_connections = 20
min_connections = 2
connect_timeout = 15
idle_timeout = 300
max_retries = 5
retry_delay_ms = 250
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.repository.repo_type, "postgres");
        assert_eq!(config.repository_type().unwrap(), RepositoryType::Postgres);

        let pg_config = config.to_postgres_config().unwrap().unwrap();
        assert_eq!(
            pg_config.database_url,
            "postgres://user:pass@host:5432/dbname"
        );
        assert_eq!(pg_config.max_pool_size, 20);
        assert_eq!(pg_config.min_pool_size, 2);
        assert_eq!(pg_config.connection_timeout_sec, 15);
        assert_eq!(pg_config.idle_timeout_sec, 300);
        assert_eq!(pg_config.max_retries, 5);
        assert_eq!(pg_config.retry_delay_ms, 250);
    }

    #[cfg(feature = "postgres-repo")]
    #[test]
    fn test_postgres_requires_database_url() {
        let toml = r#"
[repository]
type = "postgres"

[postgres]
database_url = ""
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        let result = config.to_postgres_config();
        assert!(result.is_err());
    }
}
