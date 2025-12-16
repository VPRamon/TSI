use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

use crate::algorithms::{analysis, conflicts, SchedulingConflict};

/// Get top N observations by a given column
///
/// Args:
///     df: PyDataFrame with schedule data
///     by: Column name to sort by
///     n: Number of top rows (default: 10)
///
/// Returns:
///     PyDataFrame with top N rows
///
/// Example:
///     >>> top = tsi_rust.get_top_observations(df, "priority", 5)
///     >>> print(top.to_pandas())
#[pyfunction]
#[pyo3(signature = (df, by, n=10))]
pub fn py_get_top_observations(df: PyDataFrame, by: &str, n: usize) -> PyResult<PyDataFrame> {
    let top_df = analysis::get_top_observations(&df.0, by, n).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to get top observations: {}", e))
    })?;

    Ok(PyDataFrame(top_df))
}

/// Python wrapper for SchedulingConflict
#[pyclass]
#[derive(Clone)]
pub struct PySchedulingConflict {
    #[pyo3(get)]
    pub scheduling_block_id: String,
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub scheduled_start: String,
    #[pyo3(get)]
    pub scheduled_stop: String,
    #[pyo3(get)]
    pub conflict_reasons: String,
}

#[pymethods]
impl PySchedulingConflict {
    fn __repr__(&self) -> String {
        format!(
            "Conflict(id={}, priority={:.1}, reasons={})",
            self.scheduling_block_id, self.priority, self.conflict_reasons
        )
    }
}

impl From<SchedulingConflict> for PySchedulingConflict {
    fn from(conflict: SchedulingConflict) -> Self {
        PySchedulingConflict {
            scheduling_block_id: conflict.scheduling_block_id,
            priority: conflict.priority,
            scheduled_start: conflict.scheduled_start,
            scheduled_stop: conflict.scheduled_stop,
            conflict_reasons: conflict.conflict_reasons,
        }
    }
}

/// Find scheduling conflicts in the schedule
///
/// Args:
///     df: PyDataFrame with schedule data
///
/// Returns:
///     List of PySchedulingConflict objects
///
/// Example:
///     >>> conflicts = tsi_rust.find_conflicts(df)
///     >>> for c in conflicts:
///     ...     print(f"{c.scheduling_block_id}: {c.conflict_reasons}")
#[pyfunction]
pub fn py_find_conflicts(df: PyDataFrame) -> PyResult<Vec<PySchedulingConflict>> {
    let conflicts_vec = conflicts::find_conflicts(&df.0).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to find conflicts: {}", e))
    })?;

    Ok(conflicts_vec.into_iter().map(|c| c.into()).collect())
}
