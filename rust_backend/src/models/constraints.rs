crate::define_id_type!(i64, ConstraintsId);

/// Internal constraints with quantity types for calculations.
/// For Python-facing code, use `crate::api::Constraints` instead.
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
