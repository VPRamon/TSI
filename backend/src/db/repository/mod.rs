//! Repository trait definitions for database operations.
//!
//! This module provides a collection of focused repository traits that abstract
//! database operations. By splitting responsibilities across multiple traits,
//! implementations can be more focused and testable.
//!
//! # Module Organization
//!
//! - [`error`]: Error types for repository operations
//! - [`schedule`]: Core CRUD operations for schedules and blocks
//! - [`analytics`]: Pre-computed analytics and statistics  
//! - [`validation`]: Validation results storage and retrieval
//! - [`visualization`]: Specialized queries for dashboard components
//!
//! # Trait Composition
//!
//! A complete repository implementation typically implements all traits:
//!
//! ```ignore
//! impl ScheduleRepository for MyRepo { ... }
//! impl AnalyticsRepository for MyRepo { ... }
//! impl ValidationRepository for MyRepo { ... }
//! impl VisualizationRepository for MyRepo { ... }
//! ```
//!
//! # Convenience Trait Bound
//!
//! For functions that need all repository capabilities, use the [`FullRepository`] trait bound:
//!
//! ```ignore
//! async fn my_service<R: FullRepository>(repo: &R) -> Result<()> {
//!     // Can use any repository method
//!     repo.store_schedule(&schedule).await?;
//!     repo.populate_schedule_analytics(schedule_id).await?;
//!     Ok(())
//! }
//! ```

pub mod analytics;
pub mod error;
pub mod schedule;
pub mod validation;
pub mod visualization;

// Re-export error types
pub use error::{ErrorContext, RepositoryError, RepositoryResult};

// Re-export all traits
pub use analytics::AnalyticsRepository;
pub use schedule::ScheduleRepository;
pub use validation::ValidationRepository;
pub use visualization::VisualizationRepository;

/// Composite trait bound for a complete repository implementation.
///
/// This trait is automatically implemented for any type that implements
/// all four repository traits. Use this as a convenient bound when you
/// need access to all repository operations.
///
/// # Example
///
/// ```ignore
/// async fn process_schedule<R: FullRepository>(
///     repo: &R,
///     schedule: &Schedule
/// ) -> RepositoryResult<()> {
///     // Can use all repository methods
///     let metadata = repo.store_schedule(schedule).await?;
///     repo.populate_schedule_analytics(metadata.schedule_id.unwrap()).await?;
///     Ok(())
/// }
/// ```
pub trait FullRepository:
    ScheduleRepository + AnalyticsRepository + ValidationRepository + VisualizationRepository
{
}

// Blanket implementation: any type implementing all four traits automatically implements FullRepository
impl<T> FullRepository for T where
    T: ScheduleRepository + AnalyticsRepository + ValidationRepository + VisualizationRepository
{
}
