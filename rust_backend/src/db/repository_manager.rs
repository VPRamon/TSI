//! Global repository singleton manager.
//!
//! This module manages the global repository instance used throughout the application.
//! It provides thread-safe initialization and access to the repository.

use anyhow::{Context, Result};
use std::sync::{Arc, OnceLock};

use super::factory::{RepositoryFactory, RepositoryType};
use super::{config::DbConfig, PostgresConfig};
use crate::db::repository::FullRepository;
use tokio::runtime::Runtime;

/// Global repository instance initialized once
static REPOSITORY: OnceLock<Arc<dyn FullRepository>> = OnceLock::new();

/// Initialize the global repository singleton.
///
/// By default, this creates an in-memory local repository (suitable for local development
/// and testing). No database configuration is required.
///
/// This function is idempotent - calling it multiple times is safe and will
/// simply return success if already initialized.
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::db::init_repository;
///
/// fn main() -> anyhow::Result<()> {
///     init_repository()?;
///     Ok(())
/// }
/// ```
pub fn init_repository() -> Result<()> {
    // Check if already initialized
    if REPOSITORY.get().is_some() {
        // Already initialized, this is fine - just return success
        return Ok(());
    }

    let repo_type = RepositoryType::from_env();
    let runtime = Runtime::new().context("Failed to create async runtime for repository init")?;

    let repo = runtime.block_on(async {
        match repo_type {
            RepositoryType::Azure => {
                let config = DbConfig::from_env()
                    .map_err(anyhow::Error::msg)
                    .map_err(|e| crate::db::repository::RepositoryError::ConfigurationError(e.to_string()))?;
                RepositoryFactory::create(RepositoryType::Azure, Some(&config), None).await
            }
            RepositoryType::Postgres => {
                let config = PostgresConfig::from_env()
                    .map_err(anyhow::Error::msg)
                    .map_err(|e| crate::db::repository::RepositoryError::ConfigurationError(e.to_string()))?;
                RepositoryFactory::create(RepositoryType::Postgres, None, Some(&config)).await
            }
            RepositoryType::Local => RepositoryFactory::create(RepositoryType::Local, None, None).await,
        }
    })?;

    // Try to set - if it fails (race condition), that's okay
    let _ = REPOSITORY.set(repo);

    Ok(())
}

/// Get a reference to the global repository instance.
///
/// This function is used internally by database operations and validation reporting.
/// Returns an error if the repository hasn't been initialized via `init_repository()`.
///
/// # Errors
///
/// Returns an error if the repository has not been initialized.
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::db::{init_repository, get_repository};
///
/// fn main() -> anyhow::Result<()> {
///     init_repository()?;
///     let repo = get_repository()?;
///     // Use repo...
///     Ok(())
/// }
/// ```
pub fn get_repository() -> Result<&'static Arc<dyn FullRepository>> {
    // Ensure repository is initialized lazily.
    if REPOSITORY.get().is_none() {
        // Best-effort initialize to local repository if not already done.
        // This makes repository initialization transparent to callers.
        let _ = init_repository();
    }

    REPOSITORY
        .get()
        .context("Database not initialized. Call init_repository() first.")
}
