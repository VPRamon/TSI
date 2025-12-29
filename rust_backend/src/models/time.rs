use pyo3::prelude::*;
use pyo3::types::PyTuple;

use serde::*;

#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ModifiedJulianDate(qtty::Days);

impl ModifiedJulianDate {
    /// Create a new MJD value.
    pub fn new<V: Into<qtty::Days>>(v: V) -> Self {
        Self(v.into())
    }

    /// Raw MJD value as f64.
    pub fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pymethods]
impl ModifiedJulianDate {
    #[new]
    pub fn py_new(value: f64) -> Self {
        Self::new(value)
    }

    #[getter]
    pub fn get_value(&self) -> f64 {
        self.value()
    }

    pub fn __float__(&self) -> f64 {
        self.value()
    }

    #[staticmethod]
    pub fn from_datetime(dt: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| {
            let datetime_mod = py.import("datetime")?;
            let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

            let dt_obj = dt.as_ref();
            let tzinfo = dt_obj.getattr(py, "tzinfo")?;

            let timestamp = if tzinfo.is_none(py) {
                let kwargs = pyo3::types::PyDict::new(py);
                kwargs.set_item("tzinfo", &timezone_utc)?;
                let aware = dt_obj.call_method(py, "replace", (), Some(&kwargs))?;
                aware.call_method0(py, "timestamp")?.extract::<f64>(py)?
            } else {
                dt_obj.call_method0(py, "timestamp")?.extract::<f64>(py)?
            };

            Ok(Self::new(timestamp / 86400.0 + 40587.0))
        })
    }

    pub fn to_datetime<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let secs = (self.value() - 40587.0) * 86400.0;

        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

        let dt_obj = datetime_cls.call_method1("fromtimestamp", (secs, timezone_utc))?;
        Ok(dt_obj)
    }
}

impl From<f64> for ModifiedJulianDate {
    fn from(v: f64) -> Self {
        ModifiedJulianDate::new(v)
    }
}

/// Time period in Modified Julian Date (MJD) format.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time in MJD
    pub start: ModifiedJulianDate,
    /// End time in MJD
    pub stop: ModifiedJulianDate,
}

#[pymethods]
impl Period {
    #[new]
    pub fn py_new(start: f64, stop: f64) -> Self {
        Self {
            start: crate::api::ModifiedJulianDate::new(start),
            stop: crate::api::ModifiedJulianDate::new(stop),
        }
    }

    #[staticmethod]
    pub fn from_datetime(start: Py<PyAny>, stop: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| {
            let datetime_mod = py.import("datetime")?;
            let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

            // Helper to convert a datetime object to MJD
            let to_mjd = |dt: &Py<PyAny>| -> PyResult<f64> {
                let dt_obj = dt.as_ref();
                let tzinfo = dt_obj.getattr(py, "tzinfo")?;

                let timestamp = if tzinfo.is_none(py) {
                    // Naive datetime - assume UTC
                    let kwargs = pyo3::types::PyDict::new(py);
                    kwargs.set_item("tzinfo", &timezone_utc)?;
                    let aware = dt_obj.call_method(py, "replace", (), Some(&kwargs))?;
                    aware.call_method0(py, "timestamp")?.extract::<f64>(py)?
                } else {
                    dt_obj.call_method0(py, "timestamp")?.extract::<f64>(py)?
                };

                // Convert Unix timestamp to MJD (MJD 0 = 1858-11-17 00:00:00 UTC)
                let mjd = timestamp / 86400.0 + 40587.0;
                Ok(mjd)
            };

            let start_mjd = to_mjd(&start)?;
            let stop_mjd = to_mjd(&stop)?;

            Ok(Self {
                start: crate::api::ModifiedJulianDate::new(start_mjd),
                stop: crate::api::ModifiedJulianDate::new(stop_mjd),
            })
        })
    }

    #[getter]
    pub fn start_mjd(&self) -> f64 {
        self.start.value()
    }

    #[getter]
    pub fn stop_mjd(&self) -> f64 {
        self.stop.value()
    }

    pub fn contains_mjd(&self, mjd: f64) -> bool {
        let min_mjd = self.start.value().min(self.stop.value());
        let max_mjd = self.start.value().max(self.stop.value());
        mjd >= min_mjd && mjd <= max_mjd
    }

    pub fn to_datetime<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        // Convert MJD -> seconds since UNIX epoch then use Python's datetime
        let s_secs = (self.start.value() - 40587.0) * 86400.0;
        let e_secs = (self.stop.value() - 40587.0) * 86400.0;

        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

        let s_obj = datetime_cls.call_method1("fromtimestamp", (s_secs, timezone_utc.clone()))?;
        let e_obj = datetime_cls.call_method1("fromtimestamp", (e_secs, timezone_utc))?;

        let tup = PyTuple::new(py, &[s_obj, e_obj])?;
        Ok(tup)
    }
}

impl Period {
    pub fn new(
        start: crate::api::ModifiedJulianDate,
        stop: crate::api::ModifiedJulianDate,
    ) -> Option<Self> {
        if start.value() < stop.value() {
            Some(Self { start, stop })
        } else {
            None
        }
    }

    /// Length of the interval in days.
    pub fn duration(&self) -> qtty::Days {
        qtty::Days::new(self.stop.value() - self.start.value())
    }

    /// Check if a given MJD instant lies inside this interval (inclusive start, exclusive end).
    pub fn contains(&self, t_mjd: crate::api::ModifiedJulianDate) -> bool {
        self.start.value() <= t_mjd.value() && t_mjd.value() < self.stop.value()
    }

    /// Check if this interval overlaps with another.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.value() < other.stop.value() && other.start.value() < self.stop.value()
    }
}
