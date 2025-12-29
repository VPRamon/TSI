use pyo3::prelude::*;

crate::define_id_type!(i64, SchedulingBlockId);

/// Atomic observing request (mirrors scheduling_blocks).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulingBlock {
    pub id: SchedulingBlockId,
    pub original_block_id: Option<String>,
    #[serde(with = "qtty::serde_f64")]
    pub target_ra: qtty::Degrees,
    #[serde(with = "qtty::serde_f64")]
    pub target_dec: qtty::Degrees,
    pub constraints: super::Constraints,
    pub priority: f64,
    #[serde(with = "qtty::serde_f64")]
    pub min_observation: qtty::Seconds,
    #[serde(with = "qtty::serde_f64")]
    pub requested_duration: qtty::Seconds,
    #[serde(default)]
    pub visibility_periods: Vec<super::Period>,
    pub scheduled_period: Option<super::Period>,
}