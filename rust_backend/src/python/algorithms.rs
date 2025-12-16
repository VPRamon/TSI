use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_polars::PyDataFrame;

use crate::algorithms::{analysis, conflicts, optimization, AnalyticsSnapshot, SchedulingConflict};

/// Python wrapper for AnalyticsSnapshot
#[pyclass]
#[derive(Clone)]
pub struct PyAnalyticsSnapshot {
    #[pyo3(get)]
    pub total_observations: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub unscheduled_count: usize,
    #[pyo3(get)]
    pub scheduling_rate: f64,
    #[pyo3(get)]
    pub mean_priority: f64,
    #[pyo3(get)]
    pub median_priority: f64,
    #[pyo3(get)]
    pub mean_priority_scheduled: f64,
    #[pyo3(get)]
    pub mean_priority_unscheduled: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub mean_requested_hours: f64,
}

#[pymethods]
impl PyAnalyticsSnapshot {
    fn __repr__(&self) -> String {
        format!(
            "AnalyticsSnapshot(total={}, scheduled={}, rate={:.2}%)",
            self.total_observations,
            self.scheduled_count,
            self.scheduling_rate * 100.0
        )
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("total_observations", self.total_observations)?;
        dict.set_item("scheduled_count", self.scheduled_count)?;
        dict.set_item("unscheduled_count", self.unscheduled_count)?;
        dict.set_item("scheduling_rate", self.scheduling_rate)?;
        dict.set_item("mean_priority", self.mean_priority)?;
        dict.set_item("median_priority", self.median_priority)?;
        dict.set_item("mean_priority_scheduled", self.mean_priority_scheduled)?;
        dict.set_item("mean_priority_unscheduled", self.mean_priority_unscheduled)?;
        dict.set_item("total_visibility_hours", self.total_visibility_hours)?;
        dict.set_item("mean_requested_hours", self.mean_requested_hours)?;
        Ok(dict.into())
    }
}

impl From<AnalyticsSnapshot> for PyAnalyticsSnapshot {
    fn from(snapshot: AnalyticsSnapshot) -> Self {
        PyAnalyticsSnapshot {
            total_observations: snapshot.total_observations,
            scheduled_count: snapshot.scheduled_count,
            unscheduled_count: snapshot.unscheduled_count,
            scheduling_rate: snapshot.scheduling_rate,
            mean_priority: snapshot.mean_priority,
            median_priority: snapshot.median_priority,
            mean_priority_scheduled: snapshot.mean_priority_scheduled,
            mean_priority_unscheduled: snapshot.mean_priority_unscheduled,
            total_visibility_hours: snapshot.total_visibility_hours,
            mean_requested_hours: snapshot.mean_requested_hours,
        }
    }
}

/// Compute dataset-level summary metrics
///
/// Args:
///     df: PyDataFrame with schedule data
///
/// Returns:
///     PyAnalyticsSnapshot with computed metrics
///
/// Example:
///     >>> metrics = tsi_rust.compute_metrics(df)
///     >>> print(f"Scheduling rate: {metrics.scheduling_rate:.1%}")
#[pyfunction]
pub fn py_compute_metrics(df: PyDataFrame) -> PyResult<PyAnalyticsSnapshot> {
    let snapshot = analysis::compute_metrics(&df.0).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to compute metrics: {}", e))
    })?;

    Ok(snapshot.into())
}

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

/// Python wrapper for OptimizationResult
#[pyclass]
#[derive(Clone)]
pub struct PyOptimizationResult {
    #[pyo3(get)]
    pub solution: Vec<usize>,
    #[pyo3(get)]
    pub objective_value: f64,
    #[pyo3(get)]
    pub iterations: usize,
    #[pyo3(get)]
    pub converged: bool,
}

#[pymethods]
impl PyOptimizationResult {
    fn __repr__(&self) -> String {
        format!(
            "OptimizationResult(selected={}, objective={:.2}, iterations={}, converged={})",
            self.solution.len(),
            self.objective_value,
            self.iterations,
            self.converged
        )
    }
}

impl From<optimization::OptimizationResult> for PyOptimizationResult {
    fn from(result: optimization::OptimizationResult) -> Self {
        PyOptimizationResult {
            solution: result.solution,
            objective_value: result.objective_value,
            iterations: result.iterations,
            converged: result.converged,
        }
    }
}

/// Run greedy scheduling optimization
///
/// Args:
///     priorities: List of priority values for each observation
///     max_iterations: Maximum number of iterations (default: 1000)
///
/// Returns:
///     PyOptimizationResult with solution indices and objective value
///
/// Example:
///     >>> priorities = df["priority"].tolist()
///     >>> result = tsi_rust.greedy_schedule(priorities)
///     >>> print(f"Selected {len(result.solution)} observations")
///     >>> print(f"Total priority: {result.objective_value}")
#[pyfunction]
#[pyo3(signature = (priorities, max_iterations=1000))]
pub fn py_greedy_schedule(
    priorities: Vec<f64>,
    max_iterations: usize,
) -> PyResult<PyOptimizationResult> {
    // Convert priorities to Observation objects
    let observations: Vec<optimization::Observation> = priorities
        .into_iter()
        .enumerate()
        .map(|(i, priority)| optimization::Observation { index: i, priority })
        .collect();

    // Run optimization with no constraints (baseline)
    let constraints: Vec<Box<dyn optimization::Constraint>> = vec![];
    let result = optimization::greedy_schedule(&observations, &constraints, max_iterations);

    Ok(result.into())
}
