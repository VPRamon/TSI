/// Domain models re-exported from the database layer.
///
/// This module provides a clean abstraction layer over the database models,
/// allowing the rest of the codebase to work with domain types without
/// directly depending on database-specific implementations.

pub use crate::db::models::{
    Period,
    Schedule,
    ScheduleId,
    ScheduleInfo,
    ScheduleMetadata,
    SchedulingBlock,
    SchedulingBlockId,
    TargetId,
    ConstraintsId,
    Constraints,
};
