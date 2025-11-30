use pyo3::prelude::*;
use siderust::astro::ModifiedJulianDate;

/// Convert Modified Julian Date to Python datetime (PyO3 binding)
#[pyfunction]
pub fn mjd_to_datetime(py: Python, mjd: f64) -> PyResult<Py<PyAny>> {
    use chrono::Datelike;
    use chrono::Timelike;

    let epoch = ModifiedJulianDate::new(mjd);

    // Convert to Python datetime using chrono DateTime
    let datetime_utc = epoch
        .to_utc()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid MJD value"))?;

    let datetime_module = py.import("datetime")?;
    let datetime_cls = datetime_module.getattr("datetime")?;
    let timezone_cls = datetime_module.getattr("timezone")?;
    let utc = timezone_cls.getattr("utc")?;

    let year = datetime_utc.year();
    let month = datetime_utc.month();
    let day = datetime_utc.day();
    let hour = datetime_utc.hour();
    let minute = datetime_utc.minute();
    let second = datetime_utc.second();
    let microsecond = datetime_utc.timestamp_subsec_micros();

    let py_dt = datetime_cls.call1((year, month, day, hour, minute, second, microsecond, utc))?;

    Ok(py_dt.unbind())
}

/// Convert Python datetime to Modified Julian Date (PyO3 binding)
#[pyfunction]
pub fn datetime_to_mjd(dt: &Bound<'_, PyAny>) -> PyResult<f64> {
    use chrono::prelude::*;

    // Get datetime components from Python
    let year = dt.getattr("year")?.extract::<i32>()?;
    let month = dt.getattr("month")?.extract::<u32>()?;
    let day = dt.getattr("day")?.extract::<u32>()?;
    let hour = dt.getattr("hour")?.extract::<u32>()?;
    let minute = dt.getattr("minute")?.extract::<u32>()?;
    let second = dt.getattr("second")?.extract::<u32>()?;
    let microsecond = dt.getattr("microsecond")?.extract::<u32>()?;

    // Create chrono DateTime
    let datetime_utc = Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))?
        .checked_add_signed(chrono::Duration::microseconds(microsecond as i64))
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))?;

    // Create epoch and convert to MJD
    let epoch = ModifiedJulianDate::from_utc(datetime_utc);
    Ok(epoch.value())
}
