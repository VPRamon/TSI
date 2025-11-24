/// Application state management for in-memory data storage
use std::sync::{Arc, RwLock};

use crate::models::schedule::{DatasetMetadata, SchedulingBlock};

/// Thread-safe application state
#[derive(Clone)]
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
}

struct AppStateInner {
    /// Currently loaded dataset
    dataset: Option<Dataset>,
    /// Optional comparison dataset for the Compare page
    comparison_dataset: Option<Dataset>,
}

struct Dataset {
    blocks: Vec<SchedulingBlock>,
    metadata: DatasetMetadata,
}

impl AppState {
    /// Create a new empty application state
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                dataset: None,
                comparison_dataset: None,
            })),
        }
    }

    /// Load a new dataset (replaces current dataset)
    pub fn load_dataset(
        &self,
        blocks: Vec<SchedulingBlock>,
        filename: String,
    ) -> Result<DatasetMetadata, String> {
        let num_blocks = blocks.len();
        let num_scheduled = blocks.iter().filter(|b| b.scheduled_flag).count();
        let num_unscheduled = num_blocks - num_scheduled;

        let metadata = DatasetMetadata {
            filename,
            num_blocks,
            num_scheduled,
            num_unscheduled,
            loaded_at: chrono::Utc::now(),
        };

        let dataset = Dataset {
            blocks,
            metadata: metadata.clone(),
        };

        let mut state = self.inner.write().map_err(|e| e.to_string())?;
        state.dataset = Some(dataset);

        Ok(metadata)
    }

    /// Load a comparison dataset
    pub fn load_comparison_dataset(
        &self,
        blocks: Vec<SchedulingBlock>,
        filename: String,
    ) -> Result<DatasetMetadata, String> {
        let num_blocks = blocks.len();
        let num_scheduled = blocks.iter().filter(|b| b.scheduled_flag).count();
        let num_unscheduled = num_blocks - num_scheduled;

        let metadata = DatasetMetadata {
            filename,
            num_blocks,
            num_scheduled,
            num_unscheduled,
            loaded_at: chrono::Utc::now(),
        };

        let dataset = Dataset {
            blocks,
            metadata: metadata.clone(),
        };

        let mut state = self.inner.write().map_err(|e| e.to_string())?;
        state.comparison_dataset = Some(dataset);

        Ok(metadata)
    }

    /// Get the current dataset (returns a clone for thread safety)
    pub fn get_dataset(&self) -> Result<Option<(Vec<SchedulingBlock>, DatasetMetadata)>, String> {
        let state = self.inner.read().map_err(|e| e.to_string())?;
        
        Ok(state.dataset.as_ref().map(|ds| {
            (ds.blocks.clone(), ds.metadata.clone())
        }))
    }

    /// Get the comparison dataset
    pub fn get_comparison_dataset(&self) -> Result<Option<(Vec<SchedulingBlock>, DatasetMetadata)>, String> {
        let state = self.inner.read().map_err(|e| e.to_string())?;
        
        Ok(state.comparison_dataset.as_ref().map(|ds| {
            (ds.blocks.clone(), ds.metadata.clone())
        }))
    }

    /// Check if a dataset is loaded
    pub fn has_dataset(&self) -> bool {
        self.inner
            .read()
            .map(|state| state.dataset.is_some())
            .unwrap_or(false)
    }

    /// Clear the current dataset
    pub fn clear_dataset(&self) -> Result<(), String> {
        let mut state = self.inner.write().map_err(|e| e.to_string())?;
        state.dataset = None;
        Ok(())
    }

    /// Clear the comparison dataset
    pub fn clear_comparison_dataset(&self) -> Result<(), String> {
        let mut state = self.inner.write().map_err(|e| e.to_string())?;
        state.comparison_dataset = None;
        Ok(())
    }

    /// Get dataset metadata only (without cloning all blocks)
    pub fn get_metadata(&self) -> Result<Option<DatasetMetadata>, String> {
        let state = self.inner.read().map_err(|e| e.to_string())?;
        Ok(state.dataset.as_ref().map(|ds| ds.metadata.clone()))
    }

    /// Get reference to blocks for read-only operations (helper for analytics)
    pub fn with_dataset<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&[SchedulingBlock]) -> R,
    {
        let state = self.inner.read().map_err(|e| e.to_string())?;
        match &state.dataset {
            Some(ds) => Ok(f(&ds.blocks)),
            None => Err("No dataset loaded".to_string()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    fn create_test_block(id: &str, scheduled: bool) -> SchedulingBlock {
        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority: 8.5,
            min_observation_time_in_sec: 1200.0,
            requested_duration_sec: 1200.0,
            fixed_start_time: None,
            fixed_stop_time: None,
            target_name: None,
            target_id: None,
            dec_in_deg: 0.0,
            ra_in_deg: 0.0,
            min_azimuth_angle_in_deg: 0.0,
            max_azimuth_angle_in_deg: 360.0,
            min_elevation_angle_in_deg: 60.0,
            max_elevation_angle_in_deg: 90.0,
            scheduled_period_start: if scheduled { Some(61892.0) } else { None },
            scheduled_period_stop: if scheduled { Some(61893.0) } else { None },
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61893.0,
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 24.0,
            priority_bin: PriorityBin::MediumHigh,
            scheduled_flag: scheduled,
            requested_hours: 0.333,
            elevation_range_deg: 30.0,
        }
    }

    #[test]
    fn test_load_and_get_dataset() {
        let state = AppState::new();
        assert!(!state.has_dataset());

        let blocks = vec![
            create_test_block("block1", true),
            create_test_block("block2", false),
        ];

        let metadata = state
            .load_dataset(blocks.clone(), "test.csv".to_string())
            .unwrap();

        assert_eq!(metadata.num_blocks, 2);
        assert_eq!(metadata.num_scheduled, 1);
        assert_eq!(metadata.num_unscheduled, 1);
        assert!(state.has_dataset());

        let (loaded_blocks, loaded_meta) = state.get_dataset().unwrap().unwrap();
        assert_eq!(loaded_blocks.len(), 2);
        assert_eq!(loaded_meta.filename, "test.csv");
    }

    #[test]
    fn test_clear_dataset() {
        let state = AppState::new();
        let blocks = vec![create_test_block("block1", true)];

        state.load_dataset(blocks, "test.csv".to_string()).unwrap();
        assert!(state.has_dataset());

        state.clear_dataset().unwrap();
        assert!(!state.has_dataset());
    }

    #[test]
    fn test_comparison_dataset() {
        let state = AppState::new();
        let blocks1 = vec![create_test_block("block1", true)];
        let blocks2 = vec![create_test_block("block2", false)];

        state.load_dataset(blocks1, "dataset1.csv".to_string()).unwrap();
        state.load_comparison_dataset(blocks2, "dataset2.csv".to_string()).unwrap();

        let (_, meta1) = state.get_dataset().unwrap().unwrap();
        let (_, meta2) = state.get_comparison_dataset().unwrap().unwrap();

        assert_eq!(meta1.filename, "dataset1.csv");
        assert_eq!(meta2.filename, "dataset2.csv");
    }
}
