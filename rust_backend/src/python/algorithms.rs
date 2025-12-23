use pyo3::prelude::*;

use crate::algorithms::SchedulingConflict;


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





