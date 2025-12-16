//! Repository factory for dependency injection.
//!
//! This module provides utilities for creating and configuring repository instances
//! based on runtime configuration.

use std::sync::Arc;

use super::repository::{RepositoryError, RepositoryResult, ScheduleRepository};
use super::repositories::{AzureRepository, TestRepository};
use super::{config::DbConfig, repositories::azure::pool};

/// Repository type configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryType {
    /// Azure SQL Server (production)
    Azure,
    /// In-memory test repository
    Test,
}

impl RepositoryType {
    /// Parse repository type from string.
    ///
    /// # Arguments
    /// * `s` - String representation ("azure", "test")
    ///
    /// # Returns
    /// * `Ok(RepositoryType)` if valid
    /// * `Err` if invalid
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "azure" => Ok(Self::Azure),
            "test" => Ok(Self::Test),
            _ => Err(format!("Unknown repository type: {}", s)),
        }
    }

    /// Get repository type from environment variable.
    ///
    /// Reads `REPOSITORY_TYPE` environment variable. Defaults to Azure if not set.
    pub fn from_env() -> Self {
        std::env::var("REPOSITORY_TYPE")
            .ok()
            .and_then(|s| Self::from_str(&s).ok())
            .unwrap_or(Self::Azure)
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
///     // Create test repository
///     let test_repo = RepositoryFactory::create_test();
///     
///     // Create from configuration
///     let repo = RepositoryFactory::create(RepositoryType::Azure, Some(&config)).await?;
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
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Boxed repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn create(
        repo_type: RepositoryType,
        config: Option<&DbConfig>,
    ) -> RepositoryResult<Arc<dyn ScheduleRepository>> {
        match repo_type {
            RepositoryType::Azure => {
                let config = config.ok_or_else(|| {
                    RepositoryError::ConfigurationError(
                        "Azure repository requires DbConfig".to_string(),
                    )
                })?;
                let azure = Self::create_azure(config).await?;
                Ok(azure as Arc<dyn ScheduleRepository>)
            }
            RepositoryType::Test => Ok(Self::create_test()),
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
            .map_err(|e| RepositoryError::ConnectionError(e))?;

        Ok(Arc::new(AzureRepository::new()))
    }

    /// Create an in-memory test repository.
    ///
    /// # Returns
    /// Boxed test repository instance
    pub fn create_test() -> Arc<dyn ScheduleRepository> {
        Arc::new(TestRepository::new())
    }

    /// Create repository from environment configuration.
    ///
    /// Reads `REPOSITORY_TYPE` environment variable to determine which
    /// repository to create. Defaults to Azure if not set.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Repository instance
    /// * `Err(RepositoryError)` - If creation fails
    pub async fn from_env() -> RepositoryResult<Arc<dyn ScheduleRepository>> {
        let repo_type = RepositoryType::from_env();

        match repo_type {
            RepositoryType::Azure => {
                let config = DbConfig::from_env()
                    .map_err(|e| RepositoryError::ConfigurationError(e))?;
                let azure = Self::create_azure(&config).await?;
                Ok(azure as Arc<dyn ScheduleRepository>)
            }
            RepositoryType::Test => Ok(Self::create_test()),
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
    config: Option<DbConfig>,
}

impl RepositoryBuilder {
    /// Create a new repository builder with default settings.
    ///
    /// Defaults to Azure repository type.
    pub fn new() -> Self {
        Self {
            repo_type: RepositoryType::Azure,
            config: None,
        }
    }

    /// Set the repository type.
    pub fn repository_type(mut self, repo_type: RepositoryType) -> Self {
        self.repo_type = repo_type;
        self
    }

    /// Set the database configuration.
    pub fn config(mut self, config: DbConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Load configuration from environment variables.
    pub fn from_env(mut self) -> Result<Self, RepositoryError> {
        self.repo_type = RepositoryType::from_env();

        if self.repo_type == RepositoryType::Azure {
            let config = DbConfig::from_env()
                .map_err(|e| RepositoryError::ConfigurationError(e))?;
            self.config = Some(config);
        }

        Ok(self)
    }

    /// Build the repository instance.
    ///
    /// # Returns
    /// * `Ok(Arc<dyn ScheduleRepository>)` - Configured repository
    /// * `Err(RepositoryError)` - If build fails
    pub async fn build(self) -> RepositoryResult<Arc<dyn ScheduleRepository>> {
        RepositoryFactory::create(self.repo_type, self.config.as_ref()).await
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
        assert_eq!(RepositoryType::from_str("azure").unwrap(), RepositoryType::Azure);
        assert_eq!(RepositoryType::from_str("test").unwrap(), RepositoryType::Test);
        assert_eq!(RepositoryType::from_str("Azure").unwrap(), RepositoryType::Azure);
        assert!(RepositoryType::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_create_test_repository() {
        let repo = RepositoryFactory::create_test();
        assert!(repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_builder_test_repository() {
        let repo = RepositoryBuilder::new()
            .repository_type(RepositoryType::Test)
            .build()
            .await
            .unwrap();

        assert!(repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_azure_requires_config() {
        let result = RepositoryFactory::create(RepositoryType::Azure, None).await;
        assert!(matches!(result, Err(RepositoryError::ConfigurationError(_))));
    }
}
