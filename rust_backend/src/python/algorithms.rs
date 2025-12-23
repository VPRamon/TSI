use pyo3::prelude::*;
use serde_json::Value;

use crate::algorithms::{analysis, conflicts, SchedulingConflict};

/// Get top N observations by a given column
///
/// Args:
///     records_json: JSON string of list of records with schedule data
///     by: Column name to sort by
///     n: Number of top rows (default: 10)
///
/// Returns:
///     JSON string of list of record dictionaries with top N rows
///
/// Example:
///     >>> import json
///     >>> result_json = tsi_rust.py_get_top_observations(json.dumps(records), "priority", 5)
///     >>> top = json.loads(result_json)
// #[pyfunction] - removed, function now internal only
// #[pyo3(signature = (records_json, by, n=10))] - commented out, function now internal only
pub fn py_get_top_observations(records_json: String, by: &str, n: usize) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = analysis::get_top_observations(&records, by, n).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to get top observations: {}", e))
    })?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
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
///     records_json: JSON string of list of records with schedule data
///
/// Returns:
///     List of PySchedulingConflict objects
///
/// Example:
///     >>> import json
///     >>> conflicts = tsi_rust.py_find_conflicts(json.dumps(records))
///     >>> for c in conflicts:
///     ...     print(f"{c.scheduling_block_id}: {c.conflict_reasons}")
// #[pyfunction] - removed, function now internal only
pub fn py_find_conflicts(records_json: String) -> PyResult<Vec<PySchedulingConflict>> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let conflicts_vec = conflicts::find_conflicts(&records).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to find conflicts: {}", e))
    })?;

    Ok(conflicts_vec.into_iter().map(|c| c.into()).collect())
}
