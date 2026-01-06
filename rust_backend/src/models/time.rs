use pyo3::prelude::*;

use serde::*;

#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
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
