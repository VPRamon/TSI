//! Repository factory for dependency injection.
//!
//! This module provides utilities for creating and configuring repository instances
//! based on runtime configuration.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use super::repo_config::RepositoryConfig;
use super::repositories::postgres::PostgresConfig;
use super::repositories::{AzureRepository, LocalRepository, PostgresRepository};
use super::repository::{FullRepository, RepositoryError, RepositoryResult};
use super::{config::DbConfig, repositories::azure::pool};

/// Repository type configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryType {
    /// Azure SQL Server (production)
    Azure,
    /// Postgres + Diesel implementation
    Postgres,
    /// In-memory local repository
    Local,
}

impl FromStr for RepositoryType {
    type Err = String;

    /// Parse repository type from string.
    ///
    /// # Arguments
    /// * `s` - String representation ("azure", "local")
    ///
    /// # Returns
    /// * `Ok(RepositoryType)` if valid
    /// * `Err` if invalid
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "azure" => Ok(Self::Azure),
            "postgres" | "pg" => Ok(Self::Postgres),
            "local" => Ok(Self::Local),
            _ => Err(format!("Unknown repository type: {}", s)),
        }
    }
}

impl RepositoryType {
    /// Get repository type from environment variable.
    ///
    /// Reads `REPOSITORY_TYPE` environment variable. Defaults to Postgres if a
    /// database URL is present, otherwise Local.
    pub fn from_env() -> Self {
        if let Ok(val) = std::env::var("REPOSITORY_TYPE") {
            return val.parse().unwrap_or(Self::Local);
        }

        if std::env::var("DATABASE_URL").is_ok() || std::env::var("PG_DATABASE_URL").is_ok() {
            Self::Postgres
        } else {
            Self::Local
        }
    }
}

/// Repository factory for creating repository instances.
///
/// This factory provides a centralized way to create repository instances
/// with proper initialization and configuration.
///
/// # Example
/// ```no_run
/// use tsi_rust::db::{DbConfig, RepositoryFactory, RepositoryType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create Azure repository
///     let config = DbConfig::from_env()?;
///     let azure_repo = RepositoryFactory::create_azure(&config).await?;
///     
///     // Create local repository
///     let local_repo = RepositoryFactory::create_local();
///     
///     // Create from configuration
///     let repo = RepositoryFactory::create(RepositoryType::Azure, Some(&config), None).await?;
///     
///     Ok(())
/// }
/// ```
pub struct RepositoryFactory;

impl RepositoryFactory {
    /// Create a repository instance based on type.
    ///
    /// # Arguments
    /// * `repo_type` - Type of repository to create
    /// * `config` - Optional database configuration (required for Azure)
    ///
    /// # Returns
    /// * `Ok(Arc<dyn FullRepository>)` - Boxed repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn create(
        repo_type: RepositoryType,
        azure_config: Option<&DbConfig>,
        postgres_config: Option<&PostgresConfig>,
    ) -> RepositoryResult<Arc<dyn FullRepository>> {
        match repo_type {
            RepositoryType::Azure => {
                let config = azure_config.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Azure repository requires DbConfig".to_string(),
                    )
                })?;
                let azure = Self::create_azure(config).await?;
                Ok(azure as Arc<dyn FullRepository>)
            }
            RepositoryType::Postgres => {
                let config = postgres_config.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Postgres repository requires PostgresConfig".to_string(),
                    )
                })?;
                let pg = Self::create_postgres(config).await?;
                Ok(pg as Arc<dyn FullRepository>)
            }
            RepositoryType::Local => Ok(Self::create_local()),
        }
    }

    /// Create an Azure SQL Server repository.
    ///
    /// This initializes the global connection pool if not already initialized.
    ///
    /// # Arguments
    /// * `config` - Database configuration
    ///
    /// # Returns
    /// * `Ok(Arc<AzureRepository>)` - Azure repository instance
    /// * `Err(RepositoryError)` - If pool initialization fails
    pub async fn create_azure(config: &DbConfig) -> RepositoryResult<Arc<AzureRepository>> {
        // Initialize pool if not already done
        pool::init_pool(config)
            .await
            .map_err(RepositoryError::ConnectionError)?;

        Ok(Arc::new(AzureRepository::new()))
    }

    /// Create a Postgres repository.
    ///
    /// # Arguments
    /// * `config` - Postgres configuration
    ///
    /// # Returns
    /// * `Ok(Arc<PostgresRepository>)` - Postgres repository instance
    /// * `Err(RepositoryError)` - If initialization fails
    pub async fn create_postgres(
        config: &PostgresConfig,
    ) -> RepositoryResult<Arc<PostgresRepository>> {
        let repo = PostgresRepository::new(config.clone())?;
        Ok(Arc::new(repo))
    }

    /// Create an in-memory local repository.
    ///
    /// # Returns
    /// Boxed local repository instance
    pub fn create_local() -> Arc<dyn FullRepository> {
        Arc::new(LocalRepository::new())
    }

    /// Create repository from environment configuration.
    ///
    /// Reads `REPOSITORY_TYPE` environment variable to determine which
    /// repository to create. Defaults to Azure if not set.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn from_env() -> RepositoryResult<Arc<dyn FullRepository>> {
        let repo_type = RepositoryType::from_env();

        match repo_type {
            RepositoryType::Azure => {
                let config = DbConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
                let azure = Self::create_azure(&config).await?;
                Ok(azure as Arc<dyn FullRepository>)
            }
            RepositoryType::Postgres => {
                let config =
                    PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
                let pg = Self::create_postgres(&config).await?;
                Ok(pg as Arc<dyn FullRepository>)
            }
            RepositoryType::Local => Ok(Self::create_local()),
        }
    }

    /// Create repository from a TOML configuration file.
    ///
    /// # Arguments
    /// * `config_path` - Path to the repository.toml configuration file
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn from_config_file<P: AsRef<Path>>(
        config_path: P,
    ) -> RepositoryResult<Arc<dyn FullRepository>> {
        let config = RepositoryConfig::from_file(config_path)?;
        Self::from_repository_config(&config).await
    }

    /// Create repository from the default configuration file location.
    ///
    /// Searches for `repository.toml` in standard locations and creates
    /// the appropriate repository instance.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn from_default_config() -> RepositoryResult<Arc<dyn FullRepository>> {
        let config = RepositoryConfig::from_default_location()?;
        Self::from_repository_config(&config).await
    }

    /// Create repository from a RepositoryConfig instance.
    ///
    /// # Arguments
    /// * `config` - Repository configuration
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    async fn from_repository_config(
        config: &RepositoryConfig,
    ) -> RepositoryResult<Arc<dyn FullRepository>> {
        let repo_type = config.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        match repo_type {
            RepositoryType::Azure => {
                let db_config = config.to_db_config()?.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Azure repository requires database configuration".to_string(),
                    )
                })?;
                let azure = Self::create_azure(&db_config).await?;
                Ok(azure as Arc<dyn FullRepository>)
            }
            RepositoryType::Postgres => {
                let pg_config = config.to_postgres_config()?.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Postgres repository requires database configuration".to_string(),
                    )
                })?;
                let pg = Self::create_postgres(&pg_config).await?;
                Ok(pg as Arc<dyn FullRepository>)
            }
            RepositoryType::Local => Ok(Self::create_local()),
        }
    }
}

/// Builder for configuring repository creation.
///
/// This provides a fluent API for configuring and creating repository instances.
///
/// # Example
/// ```no_run
/// use tsi_rust::db::{DbConfig, RepositoryBuilder, RepositoryType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DbConfig::from_env()?;
///     
///     let repo = RepositoryBuilder::new()
///         .repository_type(RepositoryType::Azure)
///         .config(config)
///         .build()
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub struct RepositoryBuilder {
    repo_type: RepositoryType,
    azure_config: Option<DbConfig>,
    postgres_config: Option<PostgresConfig>,
}

impl RepositoryBuilder {
    /// Create a new repository builder with default settings.
    ///
    /// Defaults to Azure repository type.
    pub fn new() -> Self {
        Self {
            repo_type: RepositoryType::Azure,
            azure_config: None,
            postgres_config: None,
        }
    }

    /// Set the repository type.
    pub fn repository_type(mut self, repo_type: RepositoryType) -> Self {
        self.repo_type = repo_type;
        self
    }

    /// Set the database configuration.
    pub fn config(mut self, config: DbConfig) -> Self {
        self.azure_config = Some(config);
        self
    }

    /// Set the Postgres configuration.
    pub fn postgres_config(mut self, config: PostgresConfig) -> Self {
        self.postgres_config = Some(config);
        self
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Result<Self, RepositoryError> {
        self.repo_type = RepositoryType::from_env();

        if self.repo_type == RepositoryType::Azure {
            let config = DbConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
            self.azure_config = Some(config);
        } else if self.repo_type == RepositoryType::Postgres {
            let config = PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
            self.postgres_config = Some(config);
        }

        Ok(self)
    }

    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    /// * `config_path` - Path to the repository.toml configuration file
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with loaded configuration
    /// * `Err(RepositoryError)` - If file cannot be read or parsed
    pub fn from_config_file<P: AsRef<Path>>(
        mut self,
        config_path: P,
    ) -> Result<Self, RepositoryError> {
        let repo_config = RepositoryConfig::from_file(config_path)?;

        self.repo_type = repo_config.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        if self.repo_type == RepositoryType::Azure {
            let config = repo_config.to_db_config()?.ok_or_else(|| {
                RepositoryError::ConfigurationError(
                    "Azure repository requires database configuration".to_string(),
                )
            })?;
            self.azure_config = Some(config);
        } else if self.repo_type == RepositoryType::Postgres {
            let config = repo_config.to_postgres_config()?.ok_or_else(|| {
                RepositoryError::ConfigurationError(
                    "Postgres repository requires database configuration".to_string(),
                )
            })?;
            self.postgres_config = Some(config);
        }

        Ok(self)
    }

    /// Load configuration from default location.
    ///
    /// Searches for `repository.toml` in standard locations.
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with loaded configuration
    /// * `Err(RepositoryError)` - If no config file found or parse error
    pub fn from_default_config(mut self) -> Result<Self, RepositoryError> {
        let repo_config = RepositoryConfig::from_default_location()?;

        self.repo_type = repo_config.repository_type().map_err(|e| {
            RepositoryError::ConfigurationError(format!("Invalid repository type: {}", e))
        })?;

        if self.repo_type == RepositoryType::Azure {
            let config = repo_config.to_db_config()?.ok_or_else(|| {
                RepositoryError::ConfigurationError(
                    "Azure repository requires database configuration".to_string(),
                )
            })?;
            self.azure_config = Some(config);
        } else if self.repo_type == RepositoryType::Postgres {
            let config = repo_config.to_postgres_config()?.ok_or_else(|| {
                RepositoryError::ConfigurationError(
                    "Postgres repository requires database configuration".to_string(),
                )
            })?;
            self.postgres_config = Some(config);
        }

        Ok(self)
    }

    /// Build the repository instance.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn FullRepository>)` - Configured repository
    /// * `Err(RepositoryError)` - If build fails
    pub async fn build(self) -> RepositoryResult<Arc<dyn FullRepository>> {
        RepositoryFactory::create(
            self.repo_type,
            self.azure_config.as_ref(),
            self.postgres_config.as_ref(),
        )
        .await
    }
}

impl Default for RepositoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_type_from_str() {
        assert_eq!(
            RepositoryType::from_str("azure").unwrap(),
            RepositoryType::Azure
        );
        assert_eq!(
            RepositoryType::from_str("local").unwrap(),
            RepositoryType::Local
        );
        assert_eq!(
            RepositoryType::from_str("postgres").unwrap(),
            RepositoryType::Postgres
        );
        assert_eq!(
            RepositoryType::from_str("Azure").unwrap(),
            RepositoryType::Azure
        );
        assert!(RepositoryType::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_create_local_repository() {
        let repo = RepositoryFactory::create_local();
        assert!(repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_builder_local_repository() {
        let repo = RepositoryBuilder::new()
            .repository_type(RepositoryType::Local)
            .build()
            .await
            .unwrap();

        assert!(repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_azure_requires_config() {
        let result = RepositoryFactory::create(RepositoryType::Azure, None, None).await;
        assert!(matches!(
            result,
            Err(RepositoryError::ConfigurationError(_))
        ));
    }
}
