//! Repository factory for dependency injection.
//!
//! This module provides utilities for creating and configuring repository instances
//! based on runtime configuration.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use super::repo_config::RepositoryConfig;
use super::repositories::LocalRepository;
#[cfg(feature = "postgres-repo")]
use super::repositories::PostgresRepository;
use super::repository::{FullRepository, RepositoryError, RepositoryResult};
use super::PostgresConfig;

/// Repository type configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryType {
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
    /// * `s` - String representation ("postgres", "local")
    ///
    /// # Returns
    /// * `Ok(RepositoryType)` if valid
    /// * `Err` if invalid
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
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
/// ```ignore
/// use tsi_rust::db::{PostgresConfig, RepositoryFactory, RepositoryType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create Postgres repository
///     let config = PostgresConfig::from_env()?;
///     let _pg_repo = RepositoryFactory::create(RepositoryType::Postgres, Some(&config)).await?;
///
///     // Create local repository
///     let local_repo = RepositoryFactory::create_local();
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
    /// * `postgres_config` - Optional database configuration (required for Postgres)
    ///
    /// # Returns
    /// * `Ok(Arc<dyn FullRepository>)` - Boxed repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn create(
        repo_type: RepositoryType,
        postgres_config: Option<&PostgresConfig>,
    ) -> RepositoryResult<Arc<dyn FullRepository>> {
        match repo_type {
            RepositoryType::Postgres => {
                #[cfg(feature = "postgres-repo")]
                {
                    let config = postgres_config.ok_or_else(|| {
                        RepositoryError::ConfigurationError(
                            "Postgres repository requires PostgresConfig".to_string(),
                        )
                    })?;
                    let pg = Self::create_postgres(config).await?;
                    Ok(pg as Arc<dyn FullRepository>)
                }
                #[cfg(not(feature = "postgres-repo"))]
                {
                    let _ = postgres_config;
                    Err(RepositoryError::ConfigurationError(
                        "Postgres repository feature not enabled".to_string(),
                    ))
                }
            }
            RepositoryType::Local => Ok(Self::create_local()),
        }
    }

    /// Create a Postgres repository.
    ///
    /// # Arguments
    /// * `config` - Postgres configuration
    ///
    /// # Returns
    /// * `Ok(Arc<PostgresRepository>)` - Postgres repository instance
    /// * `Err(RepositoryError)` - If initialization fails
    #[cfg(feature = "postgres-repo")]
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
    /// repository to create. Defaults to Postgres if a database URL is set,
    /// otherwise Local.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn from_env() -> RepositoryResult<Arc<dyn FullRepository>> {
        let repo_type = RepositoryType::from_env();

        match repo_type {
            RepositoryType::Postgres => {
                #[cfg(feature = "postgres-repo")]
                {
                    let config =
                        PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
                    let pg = Self::create_postgres(&config).await?;
                    Ok(pg as Arc<dyn FullRepository>)
                }
                #[cfg(not(feature = "postgres-repo"))]
                {
                    Err(RepositoryError::ConfigurationError(
                        "Postgres repository feature not enabled".to_string(),
                    ))
                }
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
            RepositoryType::Postgres => {
                #[cfg(feature = "postgres-repo")]
                {
                    let pg_config = config.to_postgres_config()?.ok_or_else(|| {
                        RepositoryError::ConfigurationError(
                            "Postgres repository requires database configuration".to_string(),
                        )
                    })?;
                    let pg = Self::create_postgres(&pg_config).await?;
                    Ok(pg as Arc<dyn FullRepository>)
                }
                #[cfg(not(feature = "postgres-repo"))]
                {
                    Err(RepositoryError::ConfigurationError(
                        "Postgres repository feature not enabled".to_string(),
                    ))
                }
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
/// ```ignore
/// use tsi_rust::db::{PostgresConfig, RepositoryBuilder, RepositoryType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Requires the `postgres-repo` feature.
///     let config = PostgresConfig::from_env()?;
///     
///     let repo = RepositoryBuilder::new()
///         .repository_type(RepositoryType::Postgres)
///         .postgres_config(config)
///         .build()
///         .await?;
///     
///     Ok(())
/// }
/// ```
pub struct RepositoryBuilder {
    repo_type: RepositoryType,
    #[cfg(feature = "postgres-repo")]
    postgres_config: Option<PostgresConfig>,
}

impl RepositoryBuilder {
    /// Create a new repository builder with default settings.
    ///
    /// Defaults to Postgres if configured, otherwise Local.
    pub fn new() -> Self {
        Self {
            repo_type: RepositoryType::from_env(),
            #[cfg(feature = "postgres-repo")]
            postgres_config: None,
        }
    }

    /// Set the repository type.
    pub fn repository_type(mut self, repo_type: RepositoryType) -> Self {
        self.repo_type = repo_type;
        self
    }

    /// Set the Postgres configuration.
    #[cfg(feature = "postgres-repo")]
    pub fn postgres_config(mut self, config: PostgresConfig) -> Self {
        self.postgres_config = Some(config);
        self
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Result<Self, RepositoryError> {
        self.repo_type = RepositoryType::from_env();

        if self.repo_type == RepositoryType::Postgres {
            #[cfg(feature = "postgres-repo")]
            {
                let config =
                    PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
                self.postgres_config = Some(config);
            }
            #[cfg(not(feature = "postgres-repo"))]
            {
                return Err(RepositoryError::ConfigurationError(
                    "Postgres repository feature not enabled".to_string(),
                ));
            }
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

        if self.repo_type == RepositoryType::Postgres {
            #[cfg(feature = "postgres-repo")]
            {
                let config = repo_config.to_postgres_config()?.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Postgres repository requires database configuration".to_string(),
                    )
                })?;
                self.postgres_config = Some(config);
            }
            #[cfg(not(feature = "postgres-repo"))]
            {
                return Err(RepositoryError::ConfigurationError(
                    "Postgres repository feature not enabled".to_string(),
                ));
            }
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

        if self.repo_type == RepositoryType::Postgres {
            #[cfg(feature = "postgres-repo")]
            {
                let config = repo_config.to_postgres_config()?.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Postgres repository requires database configuration".to_string(),
                    )
                })?;
                self.postgres_config = Some(config);
            }
            #[cfg(not(feature = "postgres-repo"))]
            {
                return Err(RepositoryError::ConfigurationError(
                    "Postgres repository feature not enabled".to_string(),
                ));
            }
        }

        Ok(self)
    }

    /// Build the repository instance.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn FullRepository>)` - Configured repository
    /// * `Err(RepositoryError)` - If build fails
    pub async fn build(self) -> RepositoryResult<Arc<dyn FullRepository>> {
        #[cfg(feature = "postgres-repo")]
        let pg_config = self.postgres_config.as_ref();
        #[cfg(not(feature = "postgres-repo"))]
        let pg_config = None;

        RepositoryFactory::create(self.repo_type, pg_config).await
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
            RepositoryType::from_str("local").unwrap(),
            RepositoryType::Local
        );
        assert_eq!(
            RepositoryType::from_str("postgres").unwrap(),
            RepositoryType::Postgres
        );
        assert_eq!(
            RepositoryType::from_str("Pg").unwrap(),
            RepositoryType::Postgres
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
}
