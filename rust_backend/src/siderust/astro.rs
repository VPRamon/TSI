use serde::{Deserialize, Serialize};

/// Minimal replacement for `siderust::astro::ModifiedJulianDate` used across the codebase.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModifiedJulianDate(f64);

impl ModifiedJulianDate {
    /// Create a new MJD value.
    pub fn new(v: f64) -> Self {
        Self(v)
    }

    /// Raw MJD value as f64.
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<f64> for ModifiedJulianDate {
    fn from(v: f64) -> Self {
        ModifiedJulianDate::new(v)
    }
}

// Note: We intentionally keep this type as a thin wrapper around `f64`.
// Python-facing conversion is handled by the surrounding API (e.g. getters).
