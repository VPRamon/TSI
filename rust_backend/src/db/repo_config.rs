//! Repository configuration file support.
//!
//! This module provides utilities for reading repository configuration from
//! TOML configuration files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::config::{DbAuthMethod, DbConfig};
use super::factory::RepositoryType;
use super::repository::RepositoryError;
use super::repositories::postgres::PostgresConfig;

/// Repository configuration from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub repository: RepositorySettings,
    #[serde(default)]
    pub database: DatabaseSettings,
    #[serde(default)]
    pub postgres: PostgresSettings,
    #[serde(default)]
    pub connection_pool: ConnectionPoolSettings,
}

/// Repository type settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySettings {
    #[serde(rename = "type")]
    pub repo_type: String,
}

/// Database connection settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseSettings {
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub auth_method: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

/// Postgres connection settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostgresSettings {
    #[serde(default)]
    pub database_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// Connection pool settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSettings {
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
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

impl Default for ConnectionPoolSettings {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout: default_connect_timeout(),
        }
    }
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
    /// 2. `rust_backend/` directory
    /// 3. Parent directory
    ///
    /// # Returns
    /// * `Ok(RepositoryConfig)` if found and parsed successfully
    /// * `Err(RepositoryError)` if no config file found or parse error
    pub fn from_default_location() -> Result<Self, RepositoryError> {
        let search_paths = vec![
            PathBuf::from("repository.toml"),
            PathBuf::from("rust_backend/repository.toml"),
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

    /// Convert to DbConfig if this is an Azure configuration.
    ///
    /// # Returns
    /// * `Ok(Some(DbConfig))` if Azure repository with valid settings
    /// * `Ok(None)` if not Azure repository
    /// * `Err(RepositoryError)` if Azure but invalid settings
    pub fn to_db_config(&self) -> Result<Option<DbConfig>, RepositoryError> {
        let repo_type = self.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        if repo_type != RepositoryType::Azure {
            return Ok(None);
        }

        // Validate required fields for Azure
        if self.database.server.is_empty() {
            return Err(RepositoryError::ConfigurationError(
                "Azure repository requires 'database.server' setting".to_string(),
            ));
        }

        if self.database.database.is_empty() {
            return Err(RepositoryError::ConfigurationError(
                "Azure repository requires 'database.database' setting".to_string(),
            ));
        }

        // Parse auth method
        let auth_method = match self.database.auth_method.to_lowercase().as_str() {
            "sql" | "sql_password" => DbAuthMethod::SqlPassword,
            "aad" | "aad_password" | "azure_identity" => DbAuthMethod::AadPassword,
            "" => DbAuthMethod::AadPassword, // Default
            other => {
                return Err(RepositoryError::ConfigurationError(format!(
                    "Unknown auth_method: {}. Use 'sql', 'sql_password', 'aad_password', or 'azure_identity'",
                    other
                )))
            }
        };

        // Create DbConfig
        let mut config = DbConfig {
            server: self.database.server.clone(),
            database: self.database.database.clone(),
            auth_method: auth_method.clone(),
            username: self.database.username.clone(),
            password: self.database.password.clone(),
            port: 1433,
            trust_cert: true,
            tenant_id: "common".to_string(),
            client_id: "1950a258-227b-4e31-a9cf-717495945fc2".to_string(),
            resource: "https://database.windows.net/".to_string(),
        };

        // Add credentials if SQL auth
        if matches!(auth_method, DbAuthMethod::SqlPassword) {
            if self.database.username.is_empty() {
                return Err(RepositoryError::ConfigurationError(
                    "SQL auth requires 'database.username'".to_string(),
                ));
            }
            if self.database.password.is_empty() {
                return Err(RepositoryError::ConfigurationError(
                    "SQL auth requires 'database.password'".to_string(),
                ));
            }
            config.username = self.database.username.clone();
            config.password = self.database.password.clone();
        } else {
            // For AAD auth, also need credentials
            if self.database.username.is_empty() {
                return Err(RepositoryError::ConfigurationError(
                    "AAD auth requires 'database.username'".to_string(),
                ));
            }
            if self.database.password.is_empty() {
                return Err(RepositoryError::ConfigurationError(
                    "AAD auth requires 'database.password'".to_string(),
                ));
            }
            config.username = self.database.username.clone();
            config.password = self.database.password.clone();
        }

        Ok(Some(config))
    }

    /// Convert to PostgresConfig if this is a Postgres configuration.
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
        }))
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

[database]
server = ""
database = ""
auth_method = "sql"
username = ""
password = ""

[connection_pool]
max_connections = 10
min_connections = 1
connect_timeout = 30
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.repository.repo_type, "local");
        assert_eq!(config.repository_type().unwrap(), RepositoryType::Local);
        assert!(config.to_db_config().unwrap().is_none());
    }

    #[test]
    fn test_parse_azure_config() {
        let toml = r#"
[repository]
type = "azure"

[database]
server = "myserver.database.windows.net"
database = "mydb"
auth_method = "azure_identity"
username = "user@domain.com"
password = "password123"

[connection_pool]
max_connections = 20
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.repository.repo_type, "azure");
        assert_eq!(config.repository_type().unwrap(), RepositoryType::Azure);

        let db_config = config.to_db_config().unwrap().unwrap();
        assert_eq!(db_config.server, "myserver.database.windows.net");
        assert_eq!(db_config.database, "mydb");
        assert!(matches!(db_config.auth_method, DbAuthMethod::AadPassword));
    }

    #[test]
    fn test_azure_requires_server() {
        let toml = r#"
[repository]
type = "azure"

[database]
database = "mydb"
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        let result = config.to_db_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_sql_auth_requires_credentials() {
        let toml = r#"
[repository]
type = "azure"

[database]
server = "myserver.database.windows.net"
database = "mydb"
auth_method = "sql"
username = ""
password = ""
"#;

        let config: RepositoryConfig = toml::from_str(toml).unwrap();
        let result = config.to_db_config();
        assert!(result.is_err());
    }
}
