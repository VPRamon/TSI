use pyo3::prelude::*;

crate::define_id_type!(i64, ConstraintsId);

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Constraints {
    #[serde(with = "qtty::serde_f64")]
    pub min_alt: qtty::Degrees,
    #[serde(with = "qtty::serde_f64")]
    pub max_alt: qtty::Degrees,
    #[serde(with = "qtty::serde_f64")]
    pub min_az: qtty::Degrees,
    #[serde(with = "qtty::serde_f64")]
    pub max_az: qtty::Degrees,
    pub fixed_time: Option<crate::api::Period>,
}