/// JSON loader for raw schedule data
/// This will be implemented in Phase 1 with preprocessing logic
use anyhow::Result;
use crate::models::schedule::SchedulingBlock;

/// Load and preprocess scheduling blocks from raw JSON
/// TODO: Implement in Phase 1
pub fn load_json(_json_data: &str) -> Result<Vec<SchedulingBlock>> {
    anyhow::bail!("JSON loading not yet implemented - coming in Phase 1")
}
